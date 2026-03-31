use serde::{Serialize, Deserialize};
use crate::types::{SegmentInfo, SegmentState};

/// Minimum remaining bytes in a segment before we consider splitting it.
/// Splitting smaller segments wastes HTTP round-trips.
const MIN_SPLIT_BYTES: u64 = 2 * 1024 * 1024; // 2 MB

/// The persistent state of a single download, serialized to a .mdown sidecar file.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DownloadState {
    pub id: String,
    pub url: String,
    pub file_path: String,
    pub segments: Vec<SegmentInfo>,
    pub total_size: u64,
    pub speed_limit_bps: Option<u64>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub is_fallback: bool,
    pub source_page_url: Option<String>,
    pub ignore_ssl: bool,
}

impl DownloadState {
    /// Creates a fresh DownloadState with a single Pending segment covering the full file.
    /// The Pending (not Active) state is critical — it allows all workers to claim work
    /// via `claim_next_segment()` immediately on start, preventing worker starvation.
    pub fn new(id: String, url: String, file_path: String, total_size: u64) -> Self {
        Self {
            id,
            url,
            file_path,
            segments: vec![SegmentInfo {
                start: 0,
                end: total_size.saturating_sub(1),
                downloaded: 0,
                state: SegmentState::Pending,
                retry_count: 0,
            }],
            total_size,
            speed_limit_bps: None,
            etag: None,
            last_modified: None,
            is_fallback: false,
            source_page_url: None,
            ignore_ssl: false,
        }
    }

    /// Sum of bytes written to disk across all segments.
    pub fn total_downloaded(&self) -> u64 {
        self.segments.iter().map(|s| s.downloaded).sum()
    }

    /// Returns true when every segment is in Completed state.
    pub fn is_complete(&self) -> bool {
        !self.segments.is_empty()
            && self.segments.iter().all(|s| s.state == SegmentState::Completed)
    }

    // ─── Scheduling API ──────────────────────────────────────────────────────

    /// **Fast path.** Claims the first available Pending segment.
    ///
    /// Marks it Active and returns its index. Returns None if no Pending
    /// segments remain (all are Active, Completed, or Failed).
    pub fn claim_next_segment(&mut self) -> Option<usize> {
        for (i, seg) in self.segments.iter_mut().enumerate() {
            if seg.state == SegmentState::Pending {
                seg.state = SegmentState::Active;
                return Some(i);
            }
        }
        None
    }

    /// **Slow path.** Finds the largest Active segment, splits it in half,
    /// and immediately returns the index of the new (second-half) segment.
    ///
    /// The original segment keeps [start, mid] and continues downloading.
    /// The new segment covers [mid+1, original_end] and is immediately Active.
    ///
    /// This implements the IDM "in-half division" rule: when a worker has no
    /// Pending work to claim, it steals the tail of whichever connection still
    /// has the most bytes remaining, starting a parallel stream for that range.
    ///
    /// Returns None if no Active segment is large enough to split.
    pub fn split_and_claim(&mut self) -> Option<usize> {
        let mut largest_remaining = MIN_SPLIT_BYTES;
        let mut largest_idx = None;

        for (i, seg) in self.segments.iter().enumerate() {
            if seg.state == SegmentState::Active {
                // Only split the bytes ahead of the write cursor, not already-written bytes.
                let remaining = seg.end.saturating_sub(seg.write_head());
                if remaining > largest_remaining {
                    largest_remaining = remaining;
                    largest_idx = Some(i);
                }
            }
        }

        if let Some(idx) = largest_idx {
            let seg = &self.segments[idx];
            let head = seg.write_head();
            // Split point is halfway between write cursor and segment end.
            let mid = head + (seg.end - head) / 2;

            if mid >= seg.end {
                return None; // Not enough room to create a meaningful new segment
            }

            let new_seg = SegmentInfo {
                start: mid + 1,
                end: seg.end,
                downloaded: 0,
                state: SegmentState::Active, // Immediately Active — claimed by this caller
                retry_count: 0,
            };

            // Truncate the original segment so its worker stops at `mid`.
            self.segments[idx].end = mid;
            self.segments.push(new_seg);
            Some(self.segments.len() - 1)
        } else {
            None
        }
    }

    /// Marks a segment as Failed and bumps its retry counter.
    /// The segment is reset to Pending so it can be claimed again.
    pub fn mark_failed(&mut self, idx: usize) {
        if let Some(seg) = self.segments.get_mut(idx) {
            seg.retry_count += 1;
            if seg.retry_count < 5 {
                // Reset to Pending so a worker can retry it.
                seg.state = SegmentState::Pending;
                seg.downloaded = 0; // Restart this segment from scratch.
            } else {
                seg.state = SegmentState::Failed; // Permanently failed.
            }
        }
    }
}
