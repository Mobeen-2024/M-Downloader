use tauri::{State, Window, Manager};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use crate::engine::state::{AppState, DownloadHandle};
use crate::engine::manager::DownloadManager;
use crate::engine::persistence;
use crate::types::DownloadStatus;

/// The default number of concurrent connections per download.
/// Matches IDM's default of 8 connections.
const DEFAULT_WORKERS: usize = 8;

#[tauri::command]
pub async fn start_download(
    url: String,
    window: tauri::WebviewWindow,
    state: State<'_, Arc<AppState>>,
) -> Result<String, String> {
    start_download_internal(url, window, &state).await
}

/// Internal shared logic for starting a download.
/// Used by both the Tauri command and the Browser Integration pipe listener.
pub async fn start_download_internal(
    url: String,
    window: tauri::WebviewWindow,
    state: &AppState,
) -> Result<String, String> {
    let id = uuid::Uuid::new_v4().to_string();

    let filename = url.split('/').last().unwrap_or("download.bin");
    let file_path = format!("downloads/{}", filename);

    // Ensure the downloads directory exists.
    tokio::fs::create_dir_all("downloads")
        .await
        .map_err(|e| e.to_string())?;

    // ── Step 1: Server capability negotiation ─────────────────────────────
    let res = state
        .client
        .head(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let total_size = res
        .headers()
        .get(reqwest::header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .ok_or("Server did not return Content-Length — cannot pre-allocate")?;

    let supports_ranges = res
        .headers()
        .get("accept-ranges")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_ascii_lowercase() == "bytes")
        .unwrap_or(false);

    let etag = res
        .headers()
        .get(reqwest::header::ETAG)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let last_modified = res
        .headers()
        .get(reqwest::header::LAST_MODIFIED)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let num_workers = if supports_ranges { DEFAULT_WORKERS } else { 1 };

    // ── Step 2: Sparse File Pre-allocation (Native Win32 fast_io) ──────────
    // This activates FSCTL_SET_SPARSE and attempts the VDL bypass.
    let _file = crate::engine::fast_io::prepare_file_allocation(std::path::Path::new(&file_path), total_size)
        .map_err(|e| e.to_string())?;

    // ── Step 3: Start workers ─────────────────────────────────────────────
    let cancel_token = CancellationToken::new();
    let mut manager = DownloadManager::new(
        id.clone(),
        url.clone(),
        file_path.clone(),
        total_size,
        state.client.clone(),
        cancel_token.clone(),
        num_workers,
    );

    // Save integrity tokens
    {
        let mut s = manager.state.lock().await;
        s.etag = etag;
        s.last_modified = last_modified;
    }

    // Register the download handle in shared state.
    {
        let mut d_map = state.downloads.lock().await;
        d_map.insert(
            id.clone(),
            DownloadHandle {
                cancel_token: cancel_token.clone(),
                state: manager.state.clone(),
                status: DownloadStatus::Downloading,
                url,
                file_path,
                max_workers: num_workers,
            },
        );
    }

    // Spawn the download orchestration in a background task.
    let app_state_arc = state.0.clone();
    tokio::spawn(async move {
        let _ = manager.start(Some(window), app_state_arc).await;
    });

    Ok(id)
}

#[tauri::command]
pub async fn set_speed_limit(
    id: String,
    limit: Option<u64>,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let d_map = state.downloads.lock().await;
    if let Some(handle) = d_map.get(&id) {
        let mut s = handle.state.lock().await;
        s.speed_limit_bps = limit;
    }
    Ok(())
}

#[tauri::command]
pub async fn pause_download(id: String, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let mut d_map = state.downloads.lock().await;
    if let Some(handle) = d_map.get_mut(&id) {
        handle.cancel_token.cancel();
        handle.status = DownloadStatus::Paused;
    }
    Ok(())
}

#[tauri::command]
pub async fn cancel_download(id: String, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let mut d_map = state.downloads.lock().await;
    if let Some(handle) = d_map.get_mut(&id) {
        handle.cancel_token.cancel();
        // Clean up the sidecar file on explicit cancellation.
        persistence::delete_state(&handle.file_path).await;
    }
    d_map.remove(&id);
    Ok(())
}

#[tauri::command]
pub async fn resume_download(
    id: String,
    window: tauri::WebviewWindow,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let (url, file_path, max_workers) = {
        let mut d_map = state.downloads.lock().await;
        let handle = d_map.get_mut(&id).ok_or("Download not found")?;
        if handle.status != DownloadStatus::Paused && handle.status != DownloadStatus::Error {
            return Err("Download is not in a resumable state".to_string());
        }
        (handle.url.clone(), handle.file_path.clone(), handle.max_workers)
    };

    // ── Resume path: try to recover persisted segment state ───────────────
    let cancel_token = CancellationToken::new();

    let manager = if let Some(saved_state) = persistence::load_state(&file_path).await {
        // ── Step 1: Integrity Check ───────────────────────────────────────
        let res = state
            .client
            .head(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let current_etag = res
            .headers()
            .get(reqwest::header::ETAG)
            .and_then(|v| v.to_str().ok());

        let current_lm = res
            .headers()
            .get(reqwest::header::LAST_MODIFIED)
            .and_then(|v| v.to_str().ok());

        // If the ETag exists and has changed, the file was modified.
        if let (Some(saved_etag), Some(now_etag)) = (&saved_state.etag, current_etag) {
            if saved_etag != now_etag {
                return Err("Remote file changed (ETag mismatch). Re-download required.".to_string());
            }
        } else if let (Some(saved_lm), Some(now_lm)) = (&saved_state.last_modified, current_lm) {
            // Fallback to Last-Modified comparison.
            if saved_lm != now_lm {
                return Err("Remote file changed (Last-Modified mismatch). Re-download required.".to_string());
            }
        }

        // Byte-level recovery: restore exact segment positions from the .mdown sidecar.
        DownloadManager::from_state(saved_state, state.client.clone(), cancel_token.clone(), max_workers)
    } else {
        // No sidecar found (first resume or sidecar deleted): re-negotiate with server.
        let res = state
            .client
            .head(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let total_size = res
            .headers()
            .get(reqwest::header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .ok_or("Could not determine file size")?;

        DownloadManager::new(
            id.clone(),
            url.clone(),
            file_path.clone(),
            total_size,
            state.client.clone(),
            cancel_token.clone(),
            max_workers,
        )
    };

    // Update the existing handle with the fresh cancellation token.
    {
        let mut d_map = state.downloads.lock().await;
        if let Some(handle) = d_map.get_mut(&id) {
            handle.cancel_token = cancel_token.clone();
            handle.state = manager.state.clone();
            handle.status = DownloadStatus::Downloading;
        }
    }

    let app_state_arc = state.0.clone();
    tokio::spawn(async move {
        let _ = manager.start(Some(window), app_state_arc).await;
    });

    Ok(())
}
