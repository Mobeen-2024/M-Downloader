use crate::engine::state::AppState;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use serde::Deserialize;
use tauri::{AppHandle, Manager, Emitter};
use tokio::io::AsyncReadExt;
use tokio::net::windows::named_pipe::ServerOptions;

/// Enriched download request payload.
/// Accepts both minimal requests (manual pipe writes) and full payloads
/// from the native_host browser bridge.
#[derive(Deserialize, Debug)]
pub struct DownloadRequest {
    #[serde(default)]
    pub id: Option<String>,
    pub url: String,
    pub filename: Option<String>,
    #[serde(default)]
    pub mime: Option<String>,
    #[serde(default)]
    pub filesize: Option<u64>,
    #[serde(default)]
    pub referrer: Option<String>,
    #[serde(default)]
    pub cookies: Option<String>,
    /// "browser_capture" when sent from the extension, absent otherwise.
    #[serde(default)]
    pub source: Option<String>,
}

/// Spawns a persistent Named Pipe server loop that listens for incoming
/// download requests from the `native_host.exe` bridge (or any other
/// local IPC client).
///
/// The pipe is re-created after each client disconnects, allowing
/// unlimited sequential connections over the application's lifetime.
pub fn setup_ipc_bridge(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let pipe_name = r"\\.\pipe\mdownloader_bridge";
        let mut is_first = true;

        loop {
            // Build the pipe server.
            // `first_pipe_instance(true)` is only used on the FIRST iteration
            // to claim ownership. Subsequent iterations use `false` so that
            // the pipe name can be re-created after the previous instance
            // was consumed.
            let server = match ServerOptions::new()
                .first_pipe_instance(is_first)
                .create(pipe_name)
            {
                Ok(s) => s,
                Err(e) => {
                    log::error!("[Bridge] Failed to create named pipe: {}", e);
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    continue;
                }
            };

            is_first = false;

            log::info!("[Bridge] Waiting for client connection on {}", pipe_name);

            // Wait for a client (native_host.exe) to connect.
            if let Err(e) = server.connect().await {
                log::warn!("[Bridge] Pipe connect failed: {}", e);
                continue;
            }

            let state_arc = app.state::<Arc<AppState>>();
            state_arc.bridge_connected.store(true, Ordering::SeqCst);
            let _ = app.emit("bridge-status-changed", true);

            log::info!("[Bridge] Client connected — reading payload...");

            let mut server = server;
            let mut buffer = vec![0u8; 8192];
            let app_handle = app.clone();

            // Read the full JSON payload from the pipe.
            match server.read(&mut buffer).await {
                Ok(n) if n > 0 => {
                    let request_str = String::from_utf8_lossy(&buffer[..n]);
                    log::debug!("[Bridge] Raw payload ({} bytes): {}", n, &request_str);

                    match serde_json::from_str::<DownloadRequest>(&request_str) {
                        Ok(req) => {
                            let source_label = req.source.as_deref().unwrap_or("manual");
                            log::info!(
                                "[Bridge] Download request [{}]: url={} filename={:?}",
                                source_label,
                                req.url,
                                req.filename
                            );

                            let window = match app_handle.get_webview_window("main") {
                                Some(w) => w,
                                None => {
                                    log::error!("[Bridge] Main window not available");
                                    continue;
                                }
                            };

                            let state_arc = app_handle.state::<std::sync::Arc<AppState>>();

                            // ── Refresh Address Mode (403 Recovery) ──────────
                            let mut refresh_id_lock = state_arc.refresh_task_id.lock().await;
                            if let Some(target_id) = refresh_id_lock.take() {
                                log::warn!("[Bridge] REFRESH CAPTURE HIT: Updating task {} with new URL", target_id);
                                match crate::commands::reliability::refresh_task_logic(
                                    target_id.clone(),
                                    req.url.clone(), // Clone to allow further use of req below
                                    state_arc.inner().clone()
                                ).await {
                                    Ok(_) => {
                                        log::info!("[Bridge] Task {} address refreshed successfully.", target_id);
                                        let _ = app_handle.emit("url-refreshed", serde_json::json!({ "id": target_id }));
                                        continue; // Finished refresh — exit read loop
                                    }
                                    Err(e) => log::error!("[Bridge] Automated refresh failed: {}", e),
                                }
                            }
                            drop(refresh_id_lock);

                            // ── Intelligent Media Sniffing ──────────────────
                            let is_media = req.url.contains(".m3u8") || req.url.contains(".mpd") || req.url.contains("googlevideo.com");
                            
                            if is_media {
                                log::info!("[Bridge] Media stream intercepted: {}", req.url);
                                
                                let app_for_analysis = app_handle.clone();
                                let req_for_analysis = req; // consumes req
                                
                                tauri::async_runtime::spawn(async move {
                                    // 1. Notify UI that analysis is starting
                                    let _ = app_for_analysis.emit("media-analyzing", serde_json::json!({
                                        "url": req_for_analysis.url
                                    }));

                                    let client = reqwest::Client::new();
                                    
                                    // 2. Extract real resolutions (DASH/HLS)
                                    let resolutions = if req_for_analysis.url.contains(".mpd") || req_for_analysis.url.contains("googlevideo.com") {
                                        crate::engine::media::MediaStream::extract_resolutions(&client, &req_for_analysis.url).await
                                            .unwrap_or_default()
                                    } else {
                                        // Fallback for simple HLS or direct links
                                        vec![crate::types::MediaResolution {
                                            label: "Original (HD)".to_string(),
                                            video_url: req_for_analysis.url.clone(),
                                            audio_url: None,
                                            bandwidth: 0,
                                            width: None,
                                            height: None,
                                        }]
                                    };

                                    // 3. Emit final interception data
                                    let _ = app_for_analysis.emit("media-intercepted", crate::types::MediaInterceptionEvent {
                                        id: req_for_analysis.id,
                                        url: req_for_analysis.url,
                                        filename: req_for_analysis.filename.unwrap_or_else(|| "Detected Media".to_string()),
                                        mime: req_for_analysis.mime,
                                        resolutions,
                                        cookies: req_for_analysis.cookies,
                                        referer: req_for_analysis.referrer,
                                    });
                                });
                                continue; 
                            }

                            match crate::commands::download::start_download_internal(
                                req.url,
                                Some(window),
                                state_arc.inner().clone(),
                                req.cookies,
                                req.referrer,
                                true, // Default browser capture to Queued
                            ).await {
                                Ok(download_id) => {
                                    log::info!(
                                        "[Bridge] Download started successfully: id={}",
                                        download_id
                                    );
                                }
                                Err(e) => {
                                    log::error!(
                                        "[Bridge] Failed to start download: {}",
                                        e
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            log::error!(
                                "[Bridge] Failed to parse download request: {}",
                                e
                            );
                        }
                    }
                }
                Ok(_) => {
                    log::debug!("[Bridge] Client connected but sent no data");
                }
                Err(e) => {
                    log::warn!("[Bridge] Pipe read error: {}", e);
                }
            }

            // Client disconnected or read finished
            state_arc.bridge_connected.store(false, Ordering::SeqCst);
            let _ = app.emit("bridge-status-changed", false);
        }
    });
}
