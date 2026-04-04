use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use reqwest::Client;
// Removed tauri::Emitter as it was unused
use crate::types::DownloadStatus;

use crate::engine::quota::UsageTracker;
use crate::engine::scheduler::QueueManager;
use std::path::PathBuf;

pub struct DownloadHandle {
    pub cancel_token: CancellationToken,
    pub state: Arc<Mutex<crate::engine::segmenter::DownloadState>>,
    pub status: DownloadStatus,
    pub url: String,
    pub file_path: String,
    pub max_workers: usize,
}

pub struct AppState {
    pub client: Client,
    pub downloads: Arc<Mutex<HashMap<String, DownloadHandle>>>,
    pub quota_tracker: Arc<UsageTracker>,
    pub simulation_engine: crate::engine::test_utils::SimulationEngine,
    pub queue_manager: Arc<QueueManager>,
    pub deobfuscator: Arc<crate::engine::deobfuscator::YouTubeDeobfuscator>,
    pub auth_manager: Arc<crate::engine::auth::SiteAuthManager>,
    pub refresh_task_id: Arc<Mutex<Option<String>>>,
    pub bridge_connected: Arc<AtomicBool>,
    pub orchestration_tx: tokio::sync::mpsc::UnboundedSender<String>,
    pub global_shaper: Arc<crate::engine::shaper::TokenBucket>,
    pub cloud_manager: Arc<crate::engine::cloud::CloudManager>,
    pub is_mobile: bool,
    #[cfg(windows)]
    pub bridge_writer: Arc<Mutex<Option<tokio::io::WriteHalf<tokio::net::windows::named_pipe::NamedPipeServer>>>>,
    pub discovery_frontier: Arc<crate::engine::frontier::DiscoveryFrontier>,
    pub grabber_cancel_token: Arc<Mutex<Option<CancellationToken>>>,
    pub app_handle: Mutex<Option<tauri::AppHandle>>,
}

impl AppState {
    pub fn new(app_data_dir: PathBuf, orchestration_tx: tokio::sync::mpsc::UnboundedSender<String>) -> Self {
        let is_mobile = cfg!(any(target_os = "android", target_os = "ios"));
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(reqwest::header::ACCEPT, "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8".parse().unwrap());
        headers.insert(reqwest::header::ACCEPT_LANGUAGE, "en-US,en;q=0.9".parse().unwrap());
        headers.insert("sec-ch-ua", "\"Google Chrome\";v=\"123\", \"Not:A-Brand\";v=\"8\", \"Chromium\";v=\"123\"".parse().unwrap());
        headers.insert("sec-ch-ua-mobile", "?0".parse().unwrap());
        headers.insert("sec-ch-ua-platform", "\"Windows\"".parse().unwrap());
        headers.insert("sec-fetch-dest", "document".parse().unwrap());
        headers.insert("sec-fetch-mode", "navigate".parse().unwrap());
        headers.insert("sec-fetch-site", "none".parse().unwrap());
        headers.insert("sec-fetch-user", "?1".parse().unwrap());
        headers.insert("upgrade-insecure-requests", "1".parse().unwrap());

        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36")
            .default_headers(headers)
            .redirect(reqwest::redirect::Policy::limited(10))
            .pool_max_idle_per_host(32)
            .tcp_keepalive(Duration::from_secs(30))
            .read_timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create reqwest client");

        let quota_tracker = UsageTracker::new(app_data_dir.clone()).expect("Failed to initialize usage tracker");

        let deobfuscator = Arc::new(crate::engine::deobfuscator::YouTubeDeobfuscator::new().expect("Failed to initialize YouTube Deobfuscator"));

        Self {
            client,
            downloads: Arc::new(Mutex::new(HashMap::new())),
            quota_tracker: Arc::new(quota_tracker),
            simulation_engine: crate::engine::test_utils::SimulationEngine::new(),
            deobfuscator,
            queue_manager: Arc::new(QueueManager::new(app_data_dir.clone(), orchestration_tx.clone())),
            auth_manager: Arc::new(crate::engine::auth::SiteAuthManager::new(app_data_dir.clone())),
            refresh_task_id: Arc::new(Mutex::new(None)),
            bridge_connected: Arc::new(AtomicBool::new(false)),
            orchestration_tx: orchestration_tx.clone(),
            global_shaper: Arc::new(crate::engine::shaper::TokenBucket::new(1024 * 1024 * 1024)), // Default 1 Gbps (unlimited)
            cloud_manager: Arc::new(crate::engine::cloud::CloudManager::new()),
            is_mobile,
            #[cfg(windows)]
            bridge_writer: Arc::new(Mutex::new(None)),
            discovery_frontier: Arc::new(crate::engine::frontier::DiscoveryFrontier::new(app_data_dir).expect("Failed to init frontier")),
            grabber_cancel_token: Arc::new(Mutex::new(None)),
            app_handle: Mutex::new(None),
        }
    }

    pub fn set_app_handle(&self, handle: tauri::AppHandle) {
        let mut handle_lock = self.app_handle.blocking_lock();
        *handle_lock = Some(handle);
    }

    pub fn emit<S: serde::Serialize + Clone>(&self, event: &str, payload: S) {
        if let Some(handle) = self.app_handle.blocking_lock().as_ref() {
            let _ = tauri::Emitter::emit(handle, event, payload);
        }
    }
}
