use crate::engine::state::AppState;
use crate::commands::download::start_download_internal;
use serde::Deserialize;
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::io::AsyncReadExt;
use tokio::net::windows::named_pipe::ServerOptions;

#[derive(Deserialize, Debug)]
pub struct DownloadRequest {
    pub url: String,
    pub filename: Option<String>,
}

pub fn setup_ipc_bridge(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let pipe_name = r"\\.\pipe\mdownloader_bridge";
        
        loop {
            let server = match ServerOptions::new()
                .first_pipe_instance(true)
                .create(pipe_name) 
            {
                Ok(s) => s,
                Err(e) => {
                    log::error!("Failed to create named pipe: {}", e);
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    continue;
                }
            };

            // Wait for a client to connect.
            if server.connect().await.is_err() {
                continue;
            }

            let mut server = server;
            let mut buffer = vec![0u8; 4096];
            let app_handle = app.clone();

            match server.read(&mut buffer).await {
                Ok(n) if n > 0 => {
                    let request_str = String::from_utf8_lossy(&buffer[..n]);
                    if let Ok(req) = serde_json::from_str::<DownloadRequest>(&request_str) {
                        let window = match app_handle.get_webview_window("main") {
                            Some(w) => w,
                            None => continue,
                        };

                        log::info!("Received download request via bridge: {:?}", req);
                        
                        let state = app_handle.state::<AppState>();
                        // Trigger the download internal logic.
                        let _ = start_download_internal(req.url, window, state.inner()).await;
                    }
                }
                _ => {}
            }
        }
    });
}
