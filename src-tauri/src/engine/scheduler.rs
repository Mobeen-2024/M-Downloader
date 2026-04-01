use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::engine::state::AppState;
use crate::types::DownloadStatus;

pub struct QueueManager {
    queue: Mutex<VecDeque<String>>, // Ordered list of Job IDs
    max_parallel: Mutex<usize>,
    is_active: Mutex<bool>,
}

impl QueueManager {
    pub fn new() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            max_parallel: Mutex::new(2), // Default to 2 parallel downloads
            is_active: Mutex::new(false),
        }
    }

    pub async fn add_job(&self, id: String) {
        let mut q = self.queue.lock().await;
        if !q.contains(&id) {
            q.push_back(id);
        }
    }

    pub async fn remove_job(&self, id: &str) {
        let mut q = self.queue.lock().await;
        q.retain(|x| x != id);
    }

    pub async fn set_parallel_limit(&self, limit: usize) {
        let mut p = self.max_parallel.lock().await;
        *p = limit;
    }

    pub async fn start_queue(&self, app_state: Arc<AppState>) {
        let mut active = self.is_active.lock().await;
        *active = true;
        drop(active);
        self.tick(app_state).await;
    }

    pub async fn stop_queue(&self) {
        let mut active = self.is_active.lock().await;
        *active = false;
    }

    /// The heartbeat of the scheduler.
    /// Checks if we have available slots and starts the next job in the queue.
    pub async fn tick(&self, app_state: Arc<AppState>) {
        let active = self.is_active.lock().await;
        if !*active { return; }
        drop(active);

        let max = *self.max_parallel.lock().await;
        
        let currently_running = {
            let downloads = app_state.downloads.lock().await;
            downloads.values().filter(|d| d.status == crate::types::DownloadStatus::Downloading).count()
        };

        if currently_running < max {
            let mut q = self.queue.lock().await;
            if let Some(next_id) = q.pop_front() {
                log::info!("[Scheduler] Starting next job in queue: {}", next_id);
                let app_state_arc = app_state.clone();
                tokio::spawn(async move {
                    if let Err(e) = crate::commands::download::resume_download_internal(next_id, None, app_state_arc).await {
                        log::error!("[Scheduler] Failed to auto-start job: {}", e);
                    }
                });
            }
        }
    }
}
