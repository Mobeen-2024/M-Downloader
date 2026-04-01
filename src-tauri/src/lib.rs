pub mod types;
pub mod engine;
pub mod commands;

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
            
            let app_state = AppState::new(app_data_dir);
            app.manage(std::sync::Arc::new(app_state));

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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
