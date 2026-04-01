use std::time::Duration;
use tauri::{AppHandle, Manager, Emitter};
use crate::engine::sniffing::{SNIFFER, handle::SniffedUrl};

pub async fn start_sniffer_orchestrator(app_handle: AppHandle) {
    log::info!("[Sniffer] Starting global orchestration loop");
    
    let mut interval = tokio::time::interval(Duration::from_millis(500));
    
    loop {
        interval.tick().await;
        
        let events = {
            let sniffer_lock = SNIFFER.lock();
            if let Some(sniffer) = &*sniffer_lock {
                match sniffer.poll_driver_events() {
                    Ok(evs) => evs,
                    Err(e) => {
                        log::error!("[Sniffer] Poll error: {}", e);
                        vec![]
                    }
                }
            } else {
                vec![]
            }
        };

        for event in events {
            log::info!("[Sniffer] Intercepted URL: {} from {}", event.url, event.process_name);
            
            // Emit to frontend for the "Live Sniffer Log"
            let _ = app_handle.emit("sniffer-hit", event.clone());
            
            // Logic to trigger MediaHUD or Auto-Download
            // For now, we just notify the frontend.
        }
    }
}

/// Simulated injection for testing the bridge without a real driver.
pub fn simulate_sniff_event(url: String, process: String) -> SniffedUrl {
    SniffedUrl {
        url,
        process_id: 1234,
        process_name: process,
        content_type: "video/mp4".to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    }
}
