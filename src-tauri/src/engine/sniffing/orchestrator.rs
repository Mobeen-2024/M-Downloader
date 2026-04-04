use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use crate::engine::sniffing::{SNIFFER, handle::SniffedUrl};
// Removed MediaStream as it is used via full path
use crate::types::MediaInterceptionEvent;
use lazy_static::lazy_static;

// Global cache to prevent duplicate HUD triggers within a window.
lazy_static! {
    static ref RECENT_INTERCEPTS: std::sync::Mutex<std::collections::HashMap<String, u64>> = std::sync::Mutex::new(std::collections::HashMap::new());
}

fn is_media_extension(url: &str) -> bool {
    let url_lower = url.to_lowercase();
    // Industrial grade signatures for media streams and fragments
    url_lower.contains(".mp4") || 
    url_lower.contains(".mkv") || 
    url_lower.contains(".m3u8") || 
    url_lower.contains(".mpd") ||
    url_lower.contains(".ts") ||
    url_lower.contains(".f4m") ||
    url_lower.contains(".flv") ||
    url_lower.contains(".webm") ||
    url_lower.contains("googlevideo.com") ||
    url_lower.contains("/frag/") ||
    url_lower.contains("/segment/") ||
    url_lower.contains("/get_video?")
}

pub async fn start_sniffer_orchestrator(app_handle: AppHandle) {
    log::info!("[Sniffer] Starting global orchestration loop");
    
    let mut interval = tokio::time::interval(Duration::from_millis(500));
    
    // Pre-fetch shared infrastructure from state
    let state = app_handle.state::<std::sync::Arc<crate::engine::state::AppState>>();
    let deobfuscator = state.deobfuscator.clone();

    loop {
        interval.tick().await;
        
        let events = {
            let sniffer_lock = SNIFFER.lock();
            if let Some(sniffer) = &*sniffer_lock {
                match sniffer.poll_driver_events() {
                    Ok(evs) => evs,
                    Err(e) => {
                        log::error!("[Sniffer] Driver synchronization fault: {}", e);
                        vec![]
                    }
                }
            } else {
                vec![]
            }
        };

        for event in events {
            if !is_media_extension(&event.url) {
                continue;
            }

            // 2. Threshold check: Prevent duplicate triggers within 10 seconds
            {
                let mut cache = RECENT_INTERCEPTS.lock().unwrap();
                let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
                if let Some(last_seen) = cache.get(&event.url) {
                    if now - last_seen < 10 {
                        continue; 
                    }
                }
                cache.insert(event.url.clone(), now);
                if cache.len() > 100 {
                    cache.retain(|_, v| now - *v < 60);
                }
            }

            log::info!("[Sniffer] Industrial Intercept: {} (Source: {})", event.url, event.process_name);
            
            let _ = app_handle.emit("sniffer-hit", event.clone());
            
            let app_for_analysis = app_handle.clone();
            let url_for_analysis = event.url.clone();
            let deobfuscator_for_analysis = deobfuscator.clone();
            
            tauri::async_runtime::spawn(async move {
                let _ = app_for_analysis.emit("media-analyzing", serde_json::json!({
                    "url": url_for_analysis
                }));

                let client = reqwest::Client::new();
                
                // Extract real resolutions using the shared deobfuscator
                let resolutions = if url_for_analysis.contains(".mpd") || url_for_analysis.contains("googlevideo.com") {
                    let base_js_url = if url_for_analysis.contains("youtube.com") || url_for_analysis.contains("googlevideo.com") {
                        Some("https://www.youtube.com/s/player/6f5d506d/player_ias.vflset/en_US/base.js")
                    } else {
                        None
                    };

                    crate::engine::media::MediaStream::extract_resolutions(
                        &client, 
                        &url_for_analysis, 
                        Some(&deobfuscator_for_analysis),
                        base_js_url
                    ).await.unwrap_or_default()
                } else {
                    vec![crate::types::MediaResolution {
                        label: "Direct Stream (Original)".to_string(),
                        video_url: url_for_analysis.clone(),
                        audio_url: None,
                        bandwidth: 0,
                        width: None,
                        height: None,
                    }]
                };

                let _ = app_for_analysis.emit("media-intercepted", MediaInterceptionEvent {
                    id: Some(uuid::Uuid::new_v4().to_string()),
                    url: url_for_analysis.clone(),
                    filename: url_for_analysis.split('/').last().unwrap_or("Detected Media").to_string(),
                    mime: Some(event.content_type),
                    resolutions,
                    cookies: None, 
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
        process_id: 1,
        process_name: process,
        content_type: "video/application/octet-stream".to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    }
}
