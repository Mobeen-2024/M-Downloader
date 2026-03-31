use crate::engine::state::AppState;
use crate::commands::download::start_download_internal;
use serde::Deserialize;
use tauri::{AppHandle, Manager};
use tokio::io::AsyncReadExt;
use tokio::net::windows::named_pipe::ServerOptions;

/// Enriched download request payload.
/// Accepts both minimal requests (manual pipe writes) and full payloads
/// from the native_host browser bridge.
#[derive(Deserialize, Debug)]
pub struct DownloadRequest {
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

                            let state = app_handle.state::<std::sync::Arc<AppState>>();

                            // ── Media Stream Sniffing ───────────────────────
                            let is_media = req.url.contains(".m3u8") || req.url.contains(".mpd");
                            if is_media {
                                log::info!("[Bridge] Media stream detected: {}", req.url);
                                // Future: invoke MediaManager::from_hls
                            }

                            match start_download_internal(req.url, window, state.inner().clone()).await {
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
        }
    });
}
