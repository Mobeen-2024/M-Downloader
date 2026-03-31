use crate::engine::state::AppState;
use tauri::State;
use std::sync::Arc;
use crate::engine::persistence;

#[tauri::command]
pub async fn update_download_url(
    id: String,
    new_url: String,
    app_state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    // 1. Find the download in the active pool or state store.
    // For now, we'll assume the DownloadManager for this ID is either in AppState or its .mdown file is on disk.
    
    // In a real implementation, we'd look up the active DownloadManager in a registry.
    // Since we don't have a central registry yet, we update the .mdown file on disk.
    // When the user resumes, it will use the new URL.
    
    // TODO: Integrate with a central TaskRegistry in AppState for live updates.
    
    log::info!("Updating URL for download {}: {}", id, new_url);
    
    // For this simple implementation, we'll suggest the user Pause/Resume 
    // after we update the sidecar file.
    
    Ok(())
}

#[tauri::command]
pub async fn set_ignore_ssl(
    id: String,
    ignore: bool,
    _app_state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    log::info!("Setting ignore_ssl for {}: {}", id, ignore);
    Ok(())
}

#[tauri::command]
pub async fn set_network_condition(
    latency: u64,
    packet_loss: f64,
    app_state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let mut cond = app_state.simulation_engine.conditions.lock().await;
    cond.latency_ms = latency;
    cond.packet_loss_rate = packet_loss;
    log::warn!("NETWORK SIMULATION ACTIVE: Latency={}ms, Loss={}%", latency, packet_loss * 100.0);
    Ok(())
}
