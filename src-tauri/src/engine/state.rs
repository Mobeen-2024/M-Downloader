use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use reqwest::Client;
use crate::types::DownloadStatus;

use crate::engine::quota::UsageTracker;
use crate::engine::scheduler::QueueManager;
use crate::engine::auth::AuthManager;
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
    pub deobfuscator: Arc<crate::engine::deobfuscator::YouTubeDeobfuscator>,
    pub queue_manager: Arc<QueueManager>,
    pub auth_manager: Arc<AuthManager>,
    pub refresh_task_id: Arc<Mutex<Option<String>>>,
    pub bridge_connected: Arc<AtomicBool>,
    pub orchestration_tx: tokio::sync::mpsc::UnboundedSender<String>,
    pub global_shaper: Arc<crate::engine::shaper::TokenBucket>,
}

impl AppState {
    pub fn new(app_data_dir: PathBuf, orchestration_tx: tokio::sync::mpsc::UnboundedSender<String>) -> Self {
        let client = Client::builder()
            .user_agent("Mdownloader/2.0")
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
            queue_manager: Arc::new(QueueManager::new()),
            auth_manager: Arc::new(AuthManager::new(app_data_dir)),
            refresh_task_id: Arc::new(Mutex::new(None)),
            bridge_connected: Arc::new(AtomicBool::new(false)),
            orchestration_tx,
            global_shaper: Arc::new(crate::engine::shaper::TokenBucket::new(1024 * 1024 * 1024)), // Default 1 Gbps (unlimited)
        }
    }
}
