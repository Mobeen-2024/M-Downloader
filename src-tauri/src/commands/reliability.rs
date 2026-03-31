use crate::engine::state::AppState;
use tauri::{State, command};
use std::sync::Arc;
use crate::engine::persistence;

#[command]
pub async fn update_download_url(
    id: String,
    new_url: String,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    log::info!("[Reliability] Initiating Manual URL Refresh for task: {}", id);
    refresh_task_logic(id, new_url, state.inner().clone()).await
}

/// Shared logic to hot-swap a task's URL and perform an atomic persistence checkpoint.
/// Can be called from IPC commands or the browser bridge.
pub async fn refresh_task_logic(
    id: String,
    new_url: String,
    state: Arc<AppState>
) -> Result<(), String> {
    let downloads = state.downloads.lock().await;
    
    if let Some(handle) = downloads.get(&id) {
        // 1. Update the inner DownloadState (Thread-safe)
        let mut download_state = handle.state.lock().await;
        
        log::info!("[Reliability] Task {} hot-swap: {} -> {}", id, download_state.url, new_url);
        download_state.update_url(new_url);
        
        // 2. Perform an atomic checkpoint to the .mdown sidecar
        persistence::save_state(&download_state).await
            .map_err(|e| format!("Atomic checkpoint failure: {}", e))?;
            
        Ok(())
    } else {
        log::error!("[Reliability] URL Update failed: Task {} not found", id);
        Err(format!("Task with ID {} not found", id))
    }
}

#[command]
pub async fn start_refresh_mode(
    id: String,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    log::warn!("[Reliability] ENTERING REFRESH CAPTURE MODE for task: {}", id);
    let mut refresh_id = state.refresh_task_id.lock().await;
    *refresh_id = Some(id);
    Ok(())
}

#[command]
pub async fn cancel_refresh_mode(
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    log::info!("[Reliability] Refresh capture mode cancelled.");
    let mut refresh_id = state.refresh_task_id.lock().await;
    *refresh_id = None;
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
