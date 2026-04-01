use tauri::State;
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
    cookies: Option<String>,
    referer: Option<String>,
) -> Result<String, String> {
    start_download_internal(url, Some(window), state.inner().clone(), cookies, referer, false).await
}

pub async fn start_download_internal(
    url: String,
    window: Option<tauri::WebviewWindow>,
    app_state: Arc<AppState>,
    cookies: Option<String>,
    referer: Option<String>,
    should_queue: bool,
) -> Result<String, String> {
    let id = uuid::Uuid::new_v4().to_string();
    
    // ── Auth Fallback ────────────────────────────────────────────────────
    let (cookies, referer) = if cookies.is_none() || referer.is_none() {
        let (a, c) = app_state.auth_manager.get_headers_for_url(&url).await;
        (c.or(cookies), a.or(referer)) // Use 'a' (auth/referer) from get_headers_for_url
    } else {
        (cookies, referer)
    };

    let filename = url.split('/').last().unwrap_or("download.bin");
    let file_path = format!("downloads/{}", filename);
 
    // ── Step 0: Media Stream Acquisition (HLS/DASH/Direct) ────────────────
    let is_manifest = url.contains(".m3u8") || url.contains(".mpd");
    let is_youtube = url.contains("youtube.com") || url.contains("googlevideo.com");

    if is_manifest || is_youtube {
        log::info!("[Download] Media Job Identified: {}", url);
        
        let mut base_js_url = None;
        if is_youtube {
            log::info!("[Download] YouTube media detected. Commencing deobfuscation discovery...");
            // Extract player JS for signature solving
            base_js_url = Some("https://www.youtube.com/s/player/3f3f3f3f/player_ias.vflset/en_US/base.js");
        }

        // If it's a direct googlevideo hit, we don't need to parse a manifest, 
        // but we DO need to solve the parameters before workers start.
        if is_youtube && !is_manifest {
            log::info!("[Download] Processing direct YouTube stream address...");
            // The workers in manager.rs will handle the deobfuscation per segment
        }

        let stream = if url.contains(".m3u8") {
            crate::engine::media::MediaStream::from_hls(&app_state.client, &url).await
                .map_err(|e| format!("HLS Parse Error: {}", e))?
        } else {
            crate::engine::media::MediaStream::from_dash(
                &app_state.client, 
                &url, 
                Some(&app_state.deobfuscator),
                base_js_url.as_deref()
            ).await
                .map_err(|e| format!("DASH Parse Error: {}", e))?
        };

        let metadata = crate::types::MediaJobMetadata {
            tracks: stream.tracks,
            master_url: url.clone(),
        };

        let cancel_token = CancellationToken::new();
        let manager = DownloadManager::new_stream(
            id.clone(),
            url.clone(),
            file_path.clone(),
            metadata,
            app_state.client.clone(),
            cancel_token.clone(),
            DEFAULT_WORKERS,
        );

        if should_queue {
            app_state.queue_manager.add_job(id.clone()).await;
            // Mark as Queued in persistence/state later in start_manager_orchestration logic
        }

        return start_manager_orchestration(id, window, app_state, manager, cancel_token, should_queue).await;
    }

    // Ensure the downloads directory exists.
    tokio::fs::create_dir_all("downloads")
        .await
        .map_err(|e| e.to_string())?;

    // ── Step 1: Server capability negotiation ─────────────────────────────
    let mut rb = app_state.client.head(&url);
    if let Some(ref c) = cookies { rb = rb.header(reqwest::header::COOKIE, c); }
    if let Some(ref r) = referer { rb = rb.header(reqwest::header::REFERER, r); }
    
    let res = rb.send()
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
    let _file = crate::engine::fast_io::prepare_file_allocation(std::path::Path::new(&file_path), total_size)
        .map_err(|e| e.to_string())?;

    // ── Step 3: Start workers ─────────────────────────────────────────────
    let cancel_token = CancellationToken::new();
    let mut manager = DownloadManager::new(
        id.clone(),
        url.clone(),
        file_path.clone(),
        total_size,
        app_state.client.clone(),
        cancel_token.clone(),
        num_workers,
    );
    manager.cookies = cookies;
    manager.referer = referer;

    // Save integrity tokens
    {
        let mut s = manager.state.lock().await;
        s.etag = etag;
        s.last_modified = last_modified;
    }

    if should_queue {
        app_state.queue_manager.add_job(id.clone()).await;
    }

    start_manager_orchestration(id, window, app_state, manager, cancel_token, should_queue).await
}

/// Helper to register a manager and spawn the orchestration task.
async fn start_manager_orchestration(
    id: String,
    window: Option<tauri::WebviewWindow>,
    app_state: Arc<AppState>,
    manager: DownloadManager,
    cancel_token: CancellationToken,
    should_queue: bool,
) -> Result<String, String> {
    let url = manager.url.clone();
    let file_path = manager.file_path.clone();
    let max_workers = manager.max_workers;
    let state_clone = manager.state.clone();

    // Register the download handle in shared state.
    {
        let mut d_map = app_state.downloads.lock().await;
        d_map.insert(
            id.clone(),
            DownloadHandle {
                cancel_token: cancel_token.clone(),
                state: state_clone,
                status: if should_queue { DownloadStatus::Queued } else { DownloadStatus::Downloading },
                url,
                file_path,
                max_workers,
            },
        );
    }

    // Spawn the download orchestration in a background task.
    // Spawn the download orchestration in a background task if NOT queued.
    // If queued, the QueueManager heartbeat will call resume_download_internal later.
    if !should_queue {
        let app_state_arc = app_state.clone();
        tokio::spawn(async move {
            let _ = manager.start(window, app_state_arc).await;
        });
    }

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
        if let Some(l) = limit {
            state.global_shaper.set_rate(l);
        } else {
            state.global_shaper.set_rate(1024 * 1024 * 1024); // Reset to 1 Gbps
        }
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
    resume_download_internal(id, Some(window), state.inner().clone()).await
}

pub async fn resume_download_internal(
    id: String,
    window: Option<tauri::WebviewWindow>,
    state: Arc<AppState>,
) -> Result<(), String> {
    let (url, file_path, max_workers) = {
        let d_map = state.downloads.lock().await;
        let handle = d_map.get(&id).ok_or("Download not found")?;
        if handle.status != DownloadStatus::Paused && handle.status != DownloadStatus::Error && handle.status != DownloadStatus::Queued {
            return Err("Download is not in a resumable or queued state".to_string());
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

    let app_state_arc = state.clone();
    tokio::spawn(async move {
        let _ = manager.start(window, app_state_arc).await;
    });

    Ok(())
}

#[tauri::command]
pub async fn update_download_settings(
    state: tauri::State<'_, Arc<AppState>>,
    enable_speed_limit: bool,
    max_speed_kbps: u64,
) -> Result<(), String> {
    let downloads = state.downloads.lock().await;
    let speed_limit = if enable_speed_limit {
        max_speed_kbps * 1024
    } else {
        1024 * 1024 * 1024 // 1 Gbps (unlimited)
    };

    state.global_shaper.set_rate(speed_limit);

    for handle in downloads.values() {
        let mut s = handle.state.lock().await;
        s.speed_limit_bps = if enable_speed_limit { Some(speed_limit) } else { None };
    }
    
    Ok(())
}
