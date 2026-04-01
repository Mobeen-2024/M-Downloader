use tauri::State;
use std::sync::Arc;
use crate::engine::state::AppState;

#[tauri::command]
pub async fn add_to_queue(id: String, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    state.queue_manager.add_job(id).await;
    Ok(())
}

#[tauri::command]
pub async fn start_queue_scheduler(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    state.queue_manager.is_active.store(true, std::sync::atomic::Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub async fn stop_queue_scheduler(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    state.queue_manager.is_active.store(false, std::sync::atomic::Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub async fn set_parallel_job_limit(limit: usize, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    state.queue_manager.set_parallel_limit(limit).await;
    Ok(())
}

#[tauri::command]
pub async fn move_queue_item_up(id: String, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    state.queue_manager.move_job_up(id).await;
    Ok(())
}

#[tauri::command]
pub async fn move_queue_item_down(id: String, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    state.queue_manager.move_job_down(id).await;
    Ok(())
}
