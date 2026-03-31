use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum DownloadStatus {
    Downloading,
    Paused,
    Completed,
    Error,
    Queued,
    RefreshNeeded,
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

#[derive(Debug, Serialize, Clone)]
pub struct DownloadProgressEvent {
    pub id: String,
    pub downloaded: u64,
    pub total: u64,
    pub speed_bps: f64,
    pub status: DownloadStatus,
    pub segments: Vec<SegmentInfo>,
    pub last_error_code: Option<u16>,
}
