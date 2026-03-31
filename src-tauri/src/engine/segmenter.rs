use crate::types::{SegmentInfo, SegmentState, JobType, MediaJobMetadata};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadState {
    pub id: String,
    pub job_type: JobType,
    pub url: String,
    pub file_path: String,
    pub total_size: u64,
    pub segments: Vec<SegmentInfo>,
    pub stream_metadata: Option<MediaJobMetadata>,
    pub is_fallback: bool,
    pub speed_limit_bps: Option<u64>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

impl DownloadState {
    pub fn new(id: String, url: String, file_path: String, total_size: u64) -> Self {
        // Initial state has 1 segment covering the entire file for monolithic jobs.
        let seed_segment = SegmentInfo {
            start: 0,
            end: total_size.saturating_sub(1),
            downloaded: 0,
            state: SegmentState::Pending,
            retry_count: 0,
        };

        Self {
            id,
            job_type: JobType::Monolithic,
            url,
            file_path,
            total_size,
            segments: vec![seed_segment],
            stream_metadata: None,
            is_fallback: false,
            speed_limit_bps: None,
            etag: None,
            last_modified: None,
        }
    }

    pub fn new_stream(id: String, url: String, file_path: String, metadata: MediaJobMetadata) -> Self {
        // Multi-segment jobs have N segments initialized as Pending.
        let segments = metadata.segments.iter().map(|_| SegmentInfo {
            start: 0, // Not used primarily for stream segments
            end: 0,
            downloaded: 0,
            state: SegmentState::Pending,
            retry_count: 0,
        }).collect();

        Self {
            id,
            job_type: JobType::Stream,
            url,
            file_path,
            total_size: 0, // Calculated dynamically during merge
            segments,
            stream_metadata: Some(metadata),
            is_fallback: false,
            speed_limit_bps: None,
            etag: None,
            last_modified: None,
        }
    }

    pub fn total_downloaded(&self) -> u64 {
        self.segments.iter().map(|s| s.downloaded).sum()
    }

    pub fn is_complete(&self) -> bool {
        self.segments.iter().all(|s| s.state == SegmentState::Completed)
    }

    /// Fast-path: take the first Pending segment.
    pub fn claim_next_segment(&mut self) -> Option<usize> {
        for (idx, seg) in self.segments.iter_mut().enumerate() {
            if seg.state == SegmentState::Pending || seg.state == SegmentState::Failed {
                seg.state = SegmentState::Active;
                return Some(idx);
            }
        }
        None
    }

    /// Slow-path: The "In-Half Division Rule"
    /// Finds the largest remaining active segment and splits it in half.
    pub fn split_and_claim(&mut self) -> Option<usize> {
        if self.is_fallback {
            return None; // Cannot split if range requests are not supported
        }

        let min_split_bytes = 2 * 1024 * 1024; // 2MB threshold
        let mut best_idx: Option<usize> = None;
        let mut max_remaining: u64 = 0;

        for (idx, seg) in self.segments.iter().enumerate() {
            if seg.state == SegmentState::Active {
                let current_pos = seg.start + seg.downloaded;
                if seg.end > current_pos {
                    let remaining = seg.end - current_pos;
                    if remaining > max_remaining && remaining > min_split_bytes {
                        max_remaining = remaining;
                        best_idx = Some(idx);
                    }
                }
            }
        }

        if let Some(idx) = best_idx {
            let seg = &self.segments[idx];
            let current_pos = seg.start + seg.downloaded;
            let remaining = seg.end - current_pos;
            
            // Midpoint split point
            let split_point = current_pos + (remaining / 2);

            // Create new segment for the tail
            let old_end = seg.end;
            let new_seg = SegmentInfo {
                start: split_point + 1,
                end: old_end,
                downloaded: 0,
                state: SegmentState::Active,
                retry_count: 0,
            };

            // Update original segment to end at split point
            self.segments[idx].end = split_point;
            self.segments.push(new_seg);
            
            return Some(self.segments.len() - 1);
        }

        None
    }

    pub fn mark_failed(&mut self, idx: usize) {
        if let Some(seg) = self.segments.get_mut(idx) {
            seg.state = SegmentState::Failed;
            seg.retry_count += 1;
        }
    }

    pub fn mark_completed(&mut self, idx: usize) {
        if let Some(seg) = self.segments.get_mut(idx) {
            seg.state = SegmentState::Completed;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_half_division_rule() {
        // Create 100MB download with 1 worker
        let mut state = DownloadState::new("test".to_string(), "url".to_string(), "path".to_string(), 100 * 1024 * 1024);
        
        // Initial state: 1 pending segment [0, 99,999,999]
        assert_eq!(state.segments.len(), 1);
        
        // Claim the first segment
        state.claim_next_segment().expect("Should claim segment");
        state.segments[0].downloaded = 10 * 1024 * 1024; // 10MB downloaded
        
        // Split the segment
        // Pos: 10MB, End: 100MB, Remaining: 90MB. 
        // Midpoint: 10MB + (90MB / 2) = 55MB.
        let new_idx = state.split_and_claim().expect("Should split segment");
        
        // Original segment should now end at the split point
        // Remaining: 94,371,839. Midpoint: 10,485,760 + 47,185,919 = 57,671,679
        assert_eq!(state.segments[0].end, 57671679);
        
        // New segment should start at Original end + 1
        assert_eq!(state.segments[new_idx].start, state.segments[0].end + 1);
        assert_eq!(state.segments[new_idx].end, 100 * 1024 * 1024 - 1);
        
        println!("Split math verified: S[0] end: {}, S[new] start: {}", state.segments[0].end, state.segments[new_idx].start);
    }
}
