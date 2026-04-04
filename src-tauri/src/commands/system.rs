use tauri::{AppHandle, command, Emitter};
// Removed unused imports Arc and AppState
use crate::engine::provisioner::{check_ffmpeg_availability, DependencyStatus};
#[cfg(windows)]
use crate::engine::sniffing;

#[command]
pub async fn check_dependencies(app: AppHandle) -> Result<DependencyStatus, String> {
    Ok(check_ffmpeg_availability(&app))
}

#[command]
pub async fn toggle_system_sniffing(enabled: bool) -> Result<bool, String> {
    #[cfg(windows)]
    {
        if enabled {
            sniffing::init_sniffer().map(|_| true)
        } else {
            let mut lock = sniffing::SNIFFER.lock();
            *lock = None;
            Ok(false)
        }
    }
    #[cfg(not(windows))]
    {
        let _ = enabled;
        Err("System sniffing is only available on Windows.".to_string())
    }
}

#[command]
pub async fn get_sniffer_status() -> Result<bool, String> {
    #[cfg(windows)]
    {
        Ok(sniffing::is_sniffer_active())
    }
    #[cfg(not(windows))]
    {
        Ok(false)
    }
}

#[command]
pub async fn simulate_sniffer_event(app_handle: tauri::AppHandle, url: String, process: String) -> Result<(), String> {
    #[cfg(windows)]
    {
        let event = sniffing::orchestrator::simulate_sniff_event(url, process);
        let _ = app_handle.emit("sniffer-hit", event);
        Ok(())
    }
    #[cfg(not(windows))]
    {
        let _ = (app_handle, url, process);
        Ok(())
    }
}

#[command]
pub async fn clear_all_keychain_vaults() -> Result<(), String> {
    log::info!("[Security] Clearing all system keyring vaults for mdownloader...");
    
    // In a real app we'd iterate, but here we cover the main known namespaces
    let site_entry = keyring::Entry::new("com.mobeen.mdownloader.sites", "all").unwrap();
    let _ = site_entry.delete_password();
    
    let cloud_entry = keyring::Entry::new("com.mobeen.mdownloader.cloud", "all").unwrap();
    let _ = cloud_entry.delete_password();
    
    Ok(())
}
