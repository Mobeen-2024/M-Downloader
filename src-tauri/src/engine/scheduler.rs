use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::engine::state::AppState;

pub struct QueueManager {
    pub queue: Mutex<VecDeque<String>>, // Ordered list of Job IDs
    pub max_parallel: Mutex<usize>,
    pub is_active: Mutex<bool>,
    pub app_data_dir: std::path::PathBuf,
}

impl QueueManager {
    pub fn new(app_data_dir: std::path::PathBuf) -> Self {
        let mut qm = Self {
            queue: Mutex::new(VecDeque::new()),
            max_parallel: Mutex::new(2), // Default to 2 parallel downloads
            is_active: Mutex::new(false),
            app_data_dir,
        };
        // Sync with existing disk state if available
        let _ = qm.load_from_disk();
        qm
    }

    pub async fn add_job(&self, id: String) {
        let mut q = self.queue.lock().await;
        if !q.contains(&id) {
            q.push_back(id);
            let _ = self.save_to_disk(&*q);
        }
    }

    pub async fn remove_job(&self, id: &str) {
        let mut q = self.queue.lock().await;
        q.retain(|x| x != id);
        let _ = self.save_to_disk(&*q);
    }

    pub async fn move_job_up(&self, id: String) {
        let mut q = self.queue.lock().await;
        if let Some(pos) = q.iter().position(|x| x == &id) {
            if pos > 0 {
                q.swap(pos, pos - 1);
                let _ = self.save_to_disk(&*q);
            }
        }
    }

    pub async fn move_job_down(&self, id: String) {
        let mut q = self.queue.lock().await;
        if let Some(pos) = q.iter().position(|x| x == &id) {
            if pos < q.len() - 1 {
                q.swap(pos, pos + 1);
                let _ = self.save_to_disk(&*q);
            }
        }
    }

    pub async fn set_parallel_limit(&self, limit: usize) {
        let mut p = self.max_parallel.lock().await;
        *p = limit;
    }

    pub async fn start_queue(&self, app_state: Arc<AppState>, self_arc: Arc<QueueManager>) {
        let mut active = self.is_active.lock().await;
        if *active { return; }
        *active = true;
        drop(active);
        
        // Start independent heartbeat loop to drive the queue
        tokio::spawn(async move {
            loop {
                {
                    let active = self_arc.is_active.lock().await;
                    if !*active { break; }
                }
                self_arc.tick(app_state.clone()).await;
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        });
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
                log::info!("[Scheduler] Queue Slot Available: Transitioning {} to Active", next_id);
                // Update persistent state to "Queued" -> "Downloading" isn't strictly needed here 
                // but the orchestrator will handle the resume logic.
                let _ = app_state.orchestration_tx.send(next_id);
                let _ = self.save_to_disk(&*q);
            }
        }
    }

    fn save_to_disk(&self, queue: &VecDeque<String>) -> std::io::Result<()> {
        let path = self.app_data_dir.join("queue.json");
        let json = serde_json::to_string(queue)?;
        std::fs::write(path, json)
    }

    fn load_from_disk(&mut self) -> std::io::Result<()> {
        let path = self.app_data_dir.join("queue.json");
        if path.exists() {
            let json = std::fs::read_to_string(path)?;
            if let Ok(q) = serde_json::from_str::<VecDeque<String>>(&json) {
                *self.queue.get_mut() = q;
            }
        }
        Ok(())
    }
}
