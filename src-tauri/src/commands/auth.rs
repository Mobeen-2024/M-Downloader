use tauri::State;
use std::sync::Arc;
use crate::engine::state::AppState;
use crate::engine::auth::SiteCredential;

#[tauri::command]
pub async fn add_site_credential(
    credential: SiteCredential, 
    state: State<'_, Arc<AppState>>
) -> Result<(), String> {
    state.auth_manager.add_site(credential).await;
    Ok(())
}

#[tauri::command]
pub async fn remove_site_credential(
    domain: String, 
    state: State<'_, Arc<AppState>>
) -> Result<(), String> {
    state.auth_manager.remove_site(&domain).await;
    Ok(())
}

#[tauri::command]
pub async fn get_all_site_credentials(
    state: State<'_, Arc<AppState>>
) -> Result<Vec<SiteCredential>, String> {
    Ok(state.auth_manager.get_all_sites().await)
}
