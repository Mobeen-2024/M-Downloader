use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use reqwest::Client;
use crate::types::DownloadStatus;

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
}

impl AppState {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("Mdownloader/2.0")
            // Follow up to 10 redirects (handles CDN hops common in large file downloads)
            .redirect(reqwest::redirect::Policy::limited(10))
            // Per-host connection pool: allow up to 32 idle keepalive sockets
            .pool_max_idle_per_host(32)
            // TCP keepalive prevents idle connections from being silently dropped by NAT
            .tcp_keepalive(Duration::from_secs(30))
            // Per-request read timeout — prevents stalled connections from blocking workers
            .read_timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create reqwest client");

        Self {
            client,
            downloads: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
