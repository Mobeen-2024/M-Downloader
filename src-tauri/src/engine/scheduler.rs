use std::collections::VecDeque;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::Mutex;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use serde_json;
use log;

pub struct QueueManager {
    pub queue: Mutex<VecDeque<String>>, // Ordered list of Job IDs
    pub max_parallel: Mutex<usize>,
    pub is_active: Arc<AtomicBool>,
    pub orchestration_tx: UnboundedSender<String>,
    pub app_data_dir: std::path::PathBuf,
}

impl QueueManager {
    pub fn new(app_data_dir: std::path::PathBuf, orchestration_tx: UnboundedSender<String>) -> Self {
        let mut qm = Self {
            queue: Mutex::new(VecDeque::new()),
            max_parallel: Mutex::new(2), // Default to 2 parallel downloads
            is_active: Arc::new(AtomicBool::new(false)),
            orchestration_tx,
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
            self.save_to_disk(&*q);
        }
    }

    pub async fn remove_job(&self, id: &str) {
        let mut q = self.queue.lock().await;
        q.retain(|x| x != id);
        self.save_to_disk(&*q);
    }

    pub async fn move_job_up(&self, id: String) {
        let mut q = self.queue.lock().await;
        if let Some(pos) = q.iter().position(|x| x == &id) {
            if pos > 0 {
                q.swap(pos, pos - 1);
                self.save_to_disk(&*q);
            }
        }
    }

    pub async fn move_job_down(&self, id: String) {
        let mut q = self.queue.lock().await;
        if let Some(pos) = q.iter().position(|x| x == &id) {
            if pos < q.len() - 1 {
                q.swap(pos, pos + 1);
                self.save_to_disk(&*q);
            }
        }
    }

    pub async fn set_parallel_limit(&self, limit: usize) {
        let mut p = self.max_parallel.lock().await;
        *p = limit;
    }

    pub async fn tick(&self, currently_running: usize) {
        let max = *self.max_parallel.lock().await;
        
        let mut available_slots = max.saturating_sub(currently_running);
        
        if available_slots > 0 {
            let mut q = self.queue.lock().await;
            let mut modified = false;
            
            while available_slots > 0 {
                if let Some(next_id) = q.pop_front() {
                    log::info!("[Scheduler] Queue Slot Available: Transitioning {} to Active", next_id);
                    let _ = self.orchestration_tx.send(next_id);
                    available_slots -= 1;
                    modified = true;
                } else {
                    break;
                }
            }

            if modified {
                self.save_to_disk(&*q);
            }
        }
    }

    fn save_to_disk(&self, queue: &VecDeque<String>) {
        let path = self.app_data_dir.join("queue.json");
        if let Ok(json) = serde_json::to_string(queue) {
            // Detach disk I/O from the blocking queue mutex
            tokio::spawn(async move {
                if let Err(e) = tokio::fs::write(path, json).await {
                    log::error!("[Scheduler] Failed to persist queue state: {}", e);
                }
            });
        }
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
