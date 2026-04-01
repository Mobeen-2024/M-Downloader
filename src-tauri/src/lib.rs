pub mod types;
pub mod engine;
pub mod commands;

extern crate uuid;
extern crate serde_json;
extern crate lazy_static;

use tauri::Manager;

use crate::engine::state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().expect("Failed to get app data dir");
            std::fs::create_dir_all(&app_data_dir).expect("Failed to create app data dir");
            
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
            let app_state = AppState::new(app_data_dir, tx);
            let state_arc = std::sync::Arc::new(app_state);
            app.manage(state_arc.clone());

            // ── detached Orchestration Listener ────────────────────────────────
            // Breaking the E0391 cycle by offloading the StartJob trigger
            let app_handle = app.handle().clone();
            let state_for_listener = state_arc.clone();
            tauri::async_runtime::spawn(async move {
                while let Some(job_id) = rx.recv().await {
                    log::info!("[Orchestrator] Detached job trigger received for: {}", job_id);
                    let _ = crate::commands::download::resume_download_internal(
                        job_id, 
                        app_handle.get_webview_window("main"), 
                        state_for_listener.clone()
                    ).await;
                }
            });

            // Start the Sniffer Orchestrator
            let app_handle_for_sniffer = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                crate::engine::sniffing::orchestrator::start_sniffer_orchestrator(app_handle_for_sniffer).await;
            });

            crate::engine::bridge::setup_ipc_bridge(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            crate::commands::download::start_download,
            crate::commands::download::pause_download,
            crate::commands::download::cancel_download,
            crate::commands::download::resume_download,
            crate::commands::download::set_speed_limit,
            crate::commands::download::update_download_settings,
            crate::commands::reliability::update_download_url,
            crate::commands::reliability::start_refresh_mode,
            crate::commands::reliability::cancel_refresh_mode,
            crate::commands::reliability::set_ignore_ssl,
            crate::commands::reliability::set_network_condition,
            crate::commands::system::check_dependencies,
            crate::commands::system::toggle_system_sniffing,
            crate::commands::system::get_sniffer_status,
            crate::commands::system::simulate_sniffer_event,
            crate::commands::queue::add_to_queue,
            crate::commands::queue::start_queue_scheduler,
            crate::commands::queue::stop_queue_scheduler,
            crate::commands::queue::set_parallel_job_limit,
            crate::commands::auth::add_site_credential,
            crate::commands::auth::remove_site_credential,
            crate::commands::auth::get_all_site_credentials,
            crate::commands::grabber::start_grabber_crawl,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
