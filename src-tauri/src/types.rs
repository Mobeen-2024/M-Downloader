use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum DownloadStatus {
    Downloading,
    Paused,
    Completed,
    Error,
    Queued,
    RefreshNeeded,
    Converting,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum JobType {
    /// Standard single-file download via HTTP Range requests.
    Monolithic,
    /// Multi-segment media stream (HLS/DASH) with individual fetch tasks.
    Stream,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MediaTrack {
    pub name: String,
    pub segments: Vec<String>,
    pub mime_type: Option<String>,
    pub size: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MediaResolution {
    pub label: String,
    pub video_url: String,
    pub audio_url: Option<String>,
    pub bandwidth: u64,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MediaInterceptionEvent {
    pub id: Option<String>,
    pub url: String,
    pub filename: String,
    pub mime: Option<String>,
    pub resolutions: Vec<MediaResolution>,
    pub cookies: Option<String>,
    pub referer: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MediaJobMetadata {
    pub tracks: Vec<MediaTrack>,
    pub master_url: String,
}

/// Fine-grained state for each byte-range segment.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum SegmentState {
    /// Not yet started — any idle worker can claim this.
    Pending,
    /// Currently being downloaded by one worker.
    Active,
    /// Byte range fully written and flushed to disk.
    Completed,
    /// Failed due to network error; eligible for retry.
    Failed,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SegmentInfo {
    /// Absolute byte offset in the file where this segment starts.
    pub start: u64,
    /// Current end of this segment (can be reduced when a new worker steals the tail).
    pub end: u64,
    /// Bytes written to disk so far within [start, end].
    pub downloaded: u64,
    pub state: SegmentState,
    pub retry_count: u8,
    pub last_latency_ms: u64,
}

impl SegmentInfo {
    /// Bytes still needed to complete this segment.
    pub fn remaining(&self) -> u64 {
        (self.end.saturating_sub(self.start) + 1).saturating_sub(self.downloaded)
    }
    /// Current write cursor: where the next byte will be written.
    pub fn write_head(&self) -> u64 {
        self.start + self.downloaded
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EngineEventStats {
    pub total_splits: u64,
    pub total_retries: u64,
    pub http_version: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DownloadMetrics {
    pub io_efficiency: f64,
    pub active_workers: usize,
    pub avg_latency_ms: u64,
    pub engine_stats: Option<EngineEventStats>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DownloadProgressEvent {
    pub id: String,
    pub downloaded: u64,
    pub total: u64,
    pub speed_bps: f64,
    pub status: DownloadStatus,
    pub segments: Vec<SegmentInfo>,
    pub last_error_code: Option<u16>,
    pub metrics: Option<DownloadMetrics>,
}
