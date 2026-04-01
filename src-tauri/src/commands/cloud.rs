use tauri::State;
use std::sync::Arc;
use crate::engine::state::AppState;
use crate::engine::cloud::CloudConfig;

#[tauri::command]
pub async fn update_cloud_config(config: CloudConfig, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let mut c = state.cloud_manager.config.lock().await;
    *c = config;
    Ok(())
}

#[tauri::command]
pub async fn get_cloud_config(state: State<'_, Arc<AppState>>) -> Result<CloudConfig, String> {
    let c = state.cloud_manager.config.lock().await;
    Ok(c.clone())
}
