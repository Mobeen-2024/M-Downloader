use std::time::Duration;
use std::collections::HashMap;
use tauri::{AppHandle, Manager, Emitter};
use crate::engine::sniffing::{SNIFFER, handle::SniffedUrl};
use crate::engine::media::MediaStream;
use crate::types::MediaInterceptionEvent;
use lazy_static::lazy_static;

/// Global cache to prevent duplicate HUD triggers within a window.
lazy_static! {
    static ref RECENT_INTERCEPTS: std::sync::Mutex<std::collections::HashMap<String, u64>> = std::sync::Mutex::new(std::collections::HashMap::new());
}

fn is_media_extension(url: &str) -> bool {
    let url_lower = url.to_lowercase();
    url_lower.contains(".mp4") || 
    url_lower.contains(".mkv") || 
    url_lower.contains(".m3u8") || 
    url_lower.contains(".mpd") ||
    url_lower.contains("googlevideo.com")
}

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
            // 1. Check if this is a media URL based on extensions/patterns
            if !is_media_extension(&event.url) {
                continue;
            }

            // 2. Threshold check: Prevent duplicate triggers within 10 seconds
            {
                let mut cache = RECENT_INTERCEPTS.lock().unwrap();
                let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
                if let Some(last_seen) = cache.get(&event.url) {
                    if now - last_seen < 10 {
                        continue; // Skip noise
                    }
                }
                cache.insert(event.url.clone(), now);
                // Prune old entries
                if cache.len() > 100 {
                    cache.retain(|_, v| now - *v < 60);
                }
            }

            log::info!("[Sniffer] High-Value Intercept: {} (Process: {})", event.url, event.process_name);
            
            // 3. Emit to frontend for the "Live Sniffer Log"
            let _ = app_handle.emit("sniffer-hit", event.clone());
            
            // 4. Active Interception: Trigger Media HUD Analysis
            let app_for_analysis = app_handle.clone();
            let url_for_analysis = event.url.clone();
            
            tauri::async_runtime::spawn(async move {
                // Notify UI that analysis is starting
                let _ = app_for_analysis.emit("media-analyzing", serde_json::json!({
                    "url": url_for_analysis
                }));

                let client = reqwest::Client::new();
                
                // Extract real resolutions
                let resolutions = if url_for_analysis.contains(".mpd") || url_for_analysis.contains("googlevideo.com") {
                    MediaStream::extract_resolutions(&client, &url_for_analysis).await
                        .unwrap_or_default()
                } else {
                    vec![crate::types::MediaResolution {
                        label: "Original (HD)".to_string(),
                        video_url: url_for_analysis.clone(),
                        audio_url: None,
                        bandwidth: 0,
                        width: None,
                        height: None,
                    }]
                };

                // Emit final interception data for HUD
                let _ = app_for_analysis.emit("media-intercepted", MediaInterceptionEvent {
                    id: Some(uuid::Uuid::new_v4().to_string()),
                    url: url_for_analysis.clone(),
                    filename: url_for_analysis.split('/').last().unwrap_or("Detected Media").to_string(),
                    mime: Some(event.content_type),
                    resolutions,
                    cookies: None, // WFP doesn't have easy cookie access, Bridge is better for that
                    referer: None,
                });
            });
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
