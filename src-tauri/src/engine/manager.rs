use crate::engine::segmenter::DownloadState;
use crate::engine::connection::download_segment;
use log;
use crate::engine::persistence;
use crate::types::{DownloadProgressEvent, DownloadStatus};
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::{Emitter, Manager};
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
    pub cookies: Option<String>,
    pub referer: Option<String>,
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
            url: url.clone(),
            file_path: file_path.clone(),
            client,
            state: Arc::new(Mutex::new(state)),
            cancel_token,
            max_workers,
            shaper: None, // No limit by default
            cookies: None, // Will be set post-init or via options
            referer: None,
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
            cookies: None, // Metadata is usually transient and not persisted in sidecars
            referer: None,
        }
    }

    /// Creates a DownloadManager for a multi-track stream (DASH/HLS).
    pub fn new_stream(
        id: String,
        url: String,
        file_path: String,
        metadata: crate::types::MediaJobMetadata,
        client: Client,
        cancel_token: CancellationToken,
        max_workers: usize,
    ) -> Self {
        let state = DownloadState::new_stream(id, url.clone(), file_path.clone(), metadata);
        Self {
            url,
            file_path,
            client,
            state: Arc::new(Mutex::new(state)),
            cancel_token,
            max_workers,
            shaper: None,
            cookies: None,
            referer: None,
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

        // ── Advanced File Pre-allocation ───────────────────────────────────
        let total_size = {
            let s = self.state.lock().await;
            s.total_size
        };

        if total_size > 0 {
            log::info!("[Manager] Pre-allocating {} bytes for: {}", total_size, self.file_path);
            if let Ok(file) = std::fs::File::create(&self.file_path) {
                let _ = file.set_len(total_size);
                
                #[cfg(windows)]
                {
                    use std::os::windows::io::AsRawHandle;
                    let handle = file.as_raw_handle();
                    // Mark as sparse to optimize disk usage
                    unsafe {
                        let mut bytes_returned = 0;
                        let _ = windows_sys::Win32::System::IO::DeviceIoControl(
                            handle as _,
                            windows_sys::Win32::System::Ioctl::FSCTL_SET_SPARSE,
                            std::ptr::null_mut(),
                            0,
                            std::ptr::null_mut(),
                            0,
                            &mut bytes_returned,
                            std::ptr::null_mut(),
                        );
                    }
                }
            }
        }

        let mut workers = Vec::new();

        for _worker_id in 0..self.max_workers {
            let app_state = app_state.clone();
            let url = self.url.clone();
            let file_path = self.file_path.clone();
            let client = self.client.clone();
            let state = self.state.clone();
            let cancel_token = self.cancel_token.clone();
            let window_clone = window.clone();
            let shaper = self.shaper.clone();
            let quota_tracker = app_state.quota_tracker.clone();

            let cookies = self.cookies.clone();
            let referer = self.referer.clone();

            let worker = tokio::spawn(async move {
                loop {
                    if cancel_token.is_cancelled() {
                        break;
                    }

                    // Two-phase segment acquisition: 
                    // - Monolithic: fast path (CLAIM) -> slow path (SPLIT)
                    // - Stream: sequential path (CLAIM next TS segment)
                    let (segment_idx, job_type, total_size, stream_url) = {
                        let mut s = state.lock().await;
                        let ts = s.total_size;
                        let jt = s.job_type;
                        
                        if let Some(idx) = s.claim_next_segment() {
                            let url = if jt == crate::types::JobType::Stream {
                                // Multi-track mapping: Map flat idx to track -> segment
                                let mut current_idx = 0;
                                let mut final_url = None;
                                for track in &s.stream_metadata.as_ref().unwrap().tracks {
                                    if idx < current_idx + track.segments.len() {
                                        final_url = Some(track.segments[idx - current_idx].clone());
                                        break;
                                    }
                                    current_idx += track.segments.len();
                                }
                                final_url
                            } else {
                                None
                            };
                            (Some(idx), jt, ts, url)
                        } else if jt == crate::types::JobType::Monolithic {
                            // Only split for monolithic range-based downloads
                            if let Some(idx) = s.split_and_claim() {
                                let seg = &s.segments[idx];
                                log::info!("Mathematical re-balance: Split S[k] into S[k] and S[new] at mid {}", seg.start);
                                (Some(idx), jt, ts, None)
                            } else {
                                (None, jt, ts, None)
                            }
                        } else {
                            (None, jt, ts, None)
                        }
                    };

                    match segment_idx {
                        Some(idx) => {
                            let result = if job_type == crate::types::JobType::Monolithic {
                                download_segment(
                                    client.clone(),
                                    url.clone(),
                                    idx,
                                    file_path.clone(),
                                    state.clone(),
                                    cancel_token.clone(),
                                    total_size,
                                    app_state.auth_manager.clone(),
                                    shaper.clone(),
                                    quota_tracker.clone(),
                                    Some(Arc::new(app_state.simulation_engine.clone())),
                                    cookies.clone(),
                                    referer.clone(),
                                ).await
                            } else {
                                // For HLS/DASH, we download the discrete segment URLs
                                let segment_url = stream_url.unwrap_or_default();
                                let segment_path = format!("{}.part_{}", file_path, idx);
                                crate::engine::connection::download_stream_segment(
                                    client.clone(),
                                    segment_url,
                                    segment_path,
                                    state.clone(),
                                    idx,
                                    cancel_token.clone(),
                                    app_state.auth_manager.clone(),
                                    cookies.clone(),
                                    referer.clone(),
                                ).await
                            };

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
                                            metrics: None,
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
                            };                            if let Some(ref win) = window_clone {
                                let metrics = {
                                    let s_inner = state.lock().await;
                                    crate::types::DownloadMetrics {
                                        io_efficiency: (s_inner.disk_write_success_count as f64 / s_inner.network_read_count.max(1) as f64),
                                        active_workers: s_inner.segments.iter().filter(|seg| seg.state == crate::types::SegmentState::Active).count(),
                                        avg_latency_ms: s_inner.last_latency_ms,
                                        engine_stats: Some(crate::types::EngineEventStats {
                                            total_splits: s_inner.total_splits,
                                            total_retries: s_inner.total_retries,
                                            http_version: s_inner.http_version.clone(),
                                        }),
                                    }
                                };
                                let event = DownloadProgressEvent {
                                    id: s.id.clone(),
                                    downloaded: current_dl,
                                    total: s.total_size,
                                    speed_bps: speed,
                                    status: DownloadStatus::Downloading,
                                    segments: s.segments.clone(),
                                    last_error_code: None,
                                    metrics: Some(metrics),
                                };
                                let _ = win.emit("download-progress", event);
                            }

                            // Persist after each completed segment for crash recovery.
                            let _ = persistence::save_state(&s).await;
                            
                            // ── Scheduler Pulse ────────────────────────────
                            // (Heartbeat loop in scheduler handles this now)
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

        // ── Stream Post-Processing (FFmpeg Merge & Mux) ────────────────────
        if is_done && s_lock.job_type == crate::types::JobType::Stream {
            log::info!("[Manager] Media segments complete. Initiating Multi-track construction...");
            
            if let Some(win) = &window {
                let _ = win.emit("download-msg", "Building high-definition media...");
            }

            let metadata = s_lock.stream_metadata.as_ref().unwrap().clone();
            let mut track_files = Vec::new();
            let mut current_idx = 0;

            for track in &metadata.tracks {
                let mut segment_files = Vec::new();
                for i in 0..track.segments.len() {
                    segment_files.push(std::path::PathBuf::from(format!("{}.part_{}", self.file_path, current_idx + i)));
                }

                let track_output = std::path::PathBuf::from(format!("{}.track_{}", self.file_path, track.name.to_lowercase()));
                
                if let Some(win) = &window {
                    let app_handle = win.app_handle();
                    match crate::engine::media::MediaStream::merge_with_ffmpeg(&app_handle, &segment_files, &track_output) {
                        Ok(_) => {
                            log::info!("[Manager] Track '{}' merged: {:?}", track.name, track_output);
                            track_files.push((track.name.clone(), track_output));
                            // Clean up segment parts
                            for p in segment_files { let _ = std::fs::remove_file(p); }
                        }
                        Err(e) => log::error!("[Manager] Track '{}' merge failed: {}", track.name, e),
                    }
                }

                current_idx += track.segments.len();
            }

            // Final Muxing for Multi-track jobs (DASH separate Audio/Video)
            if let Some(win) = &window {
                let app_handle = win.app_handle();
                
                if track_files.len() >= 2 {
                    let video = track_files.iter().find(|(n, _)| n == "Video").map(|(_, p)| p);
                    let audio = track_files.iter().find(|(n, _)| n == "Audio").map(|(_, p)| p);

                    if let (Some(v), Some(a)) = (video, audio) {
                        let final_mp4 = std::path::PathBuf::from(&self.file_path).with_extension("mp4");
                        match crate::engine::media::MediaStream::mux_dash_streams(&app_handle, v, a, &final_mp4) {
                            Ok(_) => {
                                log::info!("[Manager] DASH muxing successful: {:?}", final_mp4);
                                for (_, p) in track_files { let _ = std::fs::remove_file(p); }
                            }
                            Err(e) => log::error!("[Manager] DASH muxing failed: {}", e),
                        }
                    }
                } else if let Some((_, single_file)) = track_files.first() {
                    // Single track (HLS) — just rename the merged stream to final
                    let final_mp4 = std::path::PathBuf::from(&self.file_path).with_extension("mp4");
                    let _ = std::fs::rename(single_file, final_mp4);
                }
            }
        }

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
                last_error_code: None,
                metrics: None,
            };
            let _ = win.emit("download-progress", event);
        }
    }
}
