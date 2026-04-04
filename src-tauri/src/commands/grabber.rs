use tauri::State;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use crate::engine::state::AppState;
use crate::engine::grabber::{SiteGrabber, DiscoveryScope};

#[tauri::command]
pub async fn start_grabber_crawl(
    url: String,
    max_depth: u32,
    scope: DiscoveryScope,
    external_link_depth: u32,
    use_headless: bool,
    state: State<'_, Arc<AppState>>
) -> Result<(), String> {
    
    // Cancel any existing crawl
    {
        let mut token_lock = state.grabber_cancel_token.lock().await;
        if let Some(token) = token_lock.as_ref() {
            token.cancel();
        }
        *token_lock = Some(CancellationToken::new());
    }

    let cancel_token = {
        let token_lock = state.grabber_cancel_token.lock().await;
        token_lock.as_ref().unwrap().clone()
    };

    let grabber = SiteGrabber::new(
        state.client.clone(),
        state.inner().clone()
    );

    log::info!("[Grabber] Starting Industrial sweep for: {} (Max Depth: {}, Scope: {:?}, Wildcard: {})", url, max_depth, scope, external_link_depth);
    
    // Run in background task to avoid blocking the command thread
    let state_inner = state.inner().clone();
    tokio::spawn(async move {
        if let Err(e) = grabber.grab_industrial(&url, max_depth, scope, external_link_depth, use_headless, cancel_token).await {
            eprintln!("[Grabber] Sweep failed: {}", e);
        }
        // Clear token when done
        let mut token_lock = state_inner.grabber_cancel_token.lock().await;
        *token_lock = None;
    });

    Ok(())
}

#[tauri::command]
pub async fn stop_grabber_crawl(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let mut token_lock = state.grabber_cancel_token.lock().await;
    if let Some(token) = token_lock.take() {
        token.cancel();
        log::info!("[Grabber] Halt command issued. Sweep aborting...");
    }
    Ok(())
}
