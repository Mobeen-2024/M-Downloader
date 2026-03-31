use crate::engine::segmenter::DownloadState;
use crate::engine::connection::download_segment;
use log;
use crate::engine::persistence;
use crate::types::{DownloadProgressEvent, DownloadStatus};
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::{Emitter, Window};
use tokio_util::sync::CancellationToken;
use reqwest::Client;

pub struct DownloadManager {
    pub url: String,
    pub file_path: String,
    pub client: Client,
    pub state: Arc<Mutex<DownloadState>>,
    pub cancel_token: CancellationToken,
    pub max_workers: usize,
    pub shaper: Option<Arc<crate::engine::shaper::TokenBucket>>,
}

impl DownloadManager {
    /// Creates a fresh DownloadManager for a new download.
    pub fn new(
        id: String,
        url: String,
        file_path: String,
        total_size: u64,
        client: Client,
        cancel_token: CancellationToken,
        max_workers: usize,
    ) -> Self {
        let state = DownloadState::new(id, url.clone(), file_path.clone(), total_size);
        Self {
            url,
            file_path,
            client,
            state: Arc::new(Mutex::new(state)),
            cancel_token,
            max_workers,
            shaper: None, // No limit by default
        }
    }

    /// Creates a DownloadManager from a recovered .mdown sidecar state (resume path).
    pub fn from_state(
        state: DownloadState,
        client: Client,
        cancel_token: CancellationToken,
        max_workers: usize,
    ) -> Self {
        let limit = state.speed_limit_bps;
        Self {
            url: state.url.clone(),
            file_path: state.file_path.clone(),
            client,
            state: Arc::new(Mutex::new(state)),
            cancel_token,
            max_workers,
            shaper: limit.map(|bps| Arc::new(crate::engine::shaper::TokenBucket::new(bps))),
        }
    }

    /// Runs the download to completion or until cancelled.
    ///
    /// ## Worker Scheduling
    /// Each worker executes a tight loop:
    /// 1. **Fast path** — `claim_next_segment()`: Takes the first Pending segment.
    /// 2. **Slow path** — `split_and_claim()`: Halves the largest Active segment and
    ///    claims the new tail (IDM "in-half division" rule).
    /// 3. **Exit** — If neither returns a segment, all work is done and the worker exits.
    ///
    /// Because the initial state has one Pending segment, all `max_workers` workers
    /// immediately get work on their first iteration: worker 0 claims the Pending segment,
    /// workers 1-N each call split_and_claim() to steal increasingly smaller tails.
    pub async fn start(&self, window: Option<tauri::WebviewWindow>, app_state: Arc<crate::engine::state::AppState>) {
        // ── Pre-flight Quota Check ──────────────────────────────────────────
        // Check if the user has exceeded their rolling 24-hour quota (1GB default).
        let current_usage_mb = app_state.quota_tracker.get_usage_mb(24).await;
        if current_usage_mb > 1024.0 { // 1GB quota hardcoded for demo, usually configurable
            log::warn!("Daily quota exceeded ({} MB). Download paused.", current_usage_mb);
            return;
        }

        let start_time = std::time::Instant::now();
        let downloaded_at_start = {
            let s = self.state.lock().await;
            s.total_downloaded()
        };

        let mut workers = Vec::new();

        for _worker_id in 0..self.max_workers {
            let url = self.url.clone();
            let file_path = self.file_path.clone();
            let client = self.client.clone();
            let state = self.state.clone();
            let cancel_token = self.cancel_token.clone();
            let window_clone = window.clone();
            let shaper = self.shaper.clone();
            let quota_tracker = app_state.quota_tracker.clone();

            let worker = tokio::spawn(async move {
                loop {
                    if cancel_token.is_cancelled() {
                        break;
                    }

                    // Two-phase segment acquisition: fast path first, slow path second.
                    let (segment_idx, is_split, total_size) = {
                        let mut s = state.lock().await;
                        let ts = s.total_size;
                        if let Some(idx) = s.claim_next_segment() {
                            (Some(idx), false, ts)
                        } else if let Some(idx) = s.split_and_claim() {
                            let seg = &s.segments[idx];
                            log::info!("Mathematical re-balance: Split S[k] into S[k] and S[new] at mid {}", seg.start);
                            (Some(idx), true, ts)
                        } else {
                            (None, false, ts)
                        }
                    };

                    match segment_idx {
                        Some(idx) => {
                            let result = download_segment(
                                client.clone(),
                                url.clone(),
                                idx,
                                file_path.clone(),
                                state.clone(),
                                cancel_token.clone(),
                                total_size,
                                shaper.clone(),
                                quota_tracker.clone(),
                            ).await;

                            // On error, mark the segment for retry.
                            if let Err(e) = result {
                                let mut s = state.lock().await;
                                s.mark_failed(idx);
                                if e.to_string().starts_with("AUTH_REQUIRED") {
                                    if let Some(ref win) = window_clone {
                                        let event = DownloadProgressEvent {
                                            id: s.id.clone(),
                                            downloaded: s.total_downloaded(),
                                            total: s.total_size,
                                            speed_bps: 0.0,
                                            status: DownloadStatus::RefreshNeeded,
                                            segments: s.segments.clone(),
                                            last_error_code: Some(403),
                                        };
                                        let _ = win.emit("download-progress", event);
                                    }
                                    break; // Stop this worker.
                                }
                            }

                            // Emit a live progress event after each segment finishes if a window is present.
                            let s = state.lock().await;
                            let current_dl = s.total_downloaded();
                            let elapsed = start_time.elapsed().as_secs_f64();
                            let speed = if elapsed > 1.0 {
                                (current_dl.saturating_sub(downloaded_at_start)) as f64 / elapsed
                            } else {
                                0.0
                            };

                            if let Some(ref win) = window_clone {
                                let event = DownloadProgressEvent {
                                    id: s.id.clone(),
                                    downloaded: current_dl,
                                    total: s.total_size,
                                    speed_bps: speed,
                                    status: DownloadStatus::Downloading,
                                    segments: s.segments.clone(),
                                };
                                let _ = win.emit("download-progress", event);
                            }

                            // Persist after each completed segment for crash recovery.
                            let _ = persistence::save_state(&s).await;
                        }
                        None => {
                            // No work left — this worker exits.
                            break;
                        }
                    }
                }
            });

            workers.push(worker);
        }

        // Wait for all workers to finish or be cancelled.
        for worker in workers {
            let _ = worker.await;
        }

        // Final status event.
        let s_lock = self.state.lock().await;
        let is_done = s_lock.is_complete();

        // Persist the final state if we're not done (e.g. paused or error).
        if !is_done {
            let _ = persistence::save_state(&s_lock).await;
        }

        // Clean up the sidecar file if the download completed successfully.
        if is_done {
            let _ = persistence::delete_state(&s_lock.file_path).await;
        }

        if let Some(win) = window {
            let event = DownloadProgressEvent {
                id: s_lock.id.clone(),
                downloaded: s_lock.total_downloaded(),
                total: s_lock.total_size,
                speed_bps: 0.0,
                status: if is_done {
                    DownloadStatus::Completed
                } else {
                    DownloadStatus::Paused
                },
                segments: s_lock.segments.clone(),
            };
            let _ = win.emit("download-progress", event);
        }
    }
}
