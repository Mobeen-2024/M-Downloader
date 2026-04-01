use tauri::State;
use std::sync::Arc;
use crate::engine::state::AppState;
use crate::engine::grabber::{SiteGrabber, GrabbedAsset};

#[tauri::command]
pub async fn start_grabber_crawl(
    url: String,
    depth: u32,
    strict_domain: bool,
    state: State<'_, Arc<AppState>>
) -> Result<Vec<GrabbedAsset>, String> {
    let grabber = SiteGrabber::new(
        state.client.clone(),
        state.auth_manager.clone()
    );

    log::info!("[Grabber] Starting crawl for: {} (Depth: {}, Strict: {})", url, depth, strict_domain);
    match grabber.grab_page(&url, depth, strict_domain).await {
        Ok(assets) => {
            log::info!("[Grabber] Crawl complete. Found {} assets.", assets.len());
            Ok(assets)
        },
        Err(e) => Err(format!("Grabber Error: {}", e))
    }
}
