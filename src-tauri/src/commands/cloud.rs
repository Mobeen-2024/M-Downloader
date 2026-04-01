use tauri::State;
use std::sync::Arc;
use crate::engine::state::AppState;
use crate::engine::cloud::CloudConfig;

#[tauri::command]
pub async fn update_cloud_config(mut config: CloudConfig, api_key: Option<String>, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    if let Some(key) = api_key {
        if !key.is_empty() {
            log::info!("[Security] Updating secure cloud API key for: {}", config.provider);
            let entry = keyring::Entry::new("com.mobeen.mdownloader.cloud", &config.provider).unwrap();
            let _ = entry.set_password(&key);
        }
    }
    
    let mut c = state.cloud_manager.config.lock().await;
    *c = config;
    Ok(())
}

#[tauri::command]
pub async fn get_cloud_config(state: State<'_, Arc<AppState>>) -> Result<CloudConfig, String> {
    let c = state.cloud_manager.config.lock().await;
    Ok(c.clone())
}
