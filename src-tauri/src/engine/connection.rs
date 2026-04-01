use reqwest::header::{RANGE, USER_AGENT, COOKIE, REFERER, IF_RANGE};
use log;
use tokio::io::{AsyncSeekExt, AsyncWriteExt, BufWriter};
use tokio::fs::OpenOptions;
use tokio_util::sync::CancellationToken;
use futures_util::StreamExt;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::engine::segmenter::DownloadState;
use crate::types::SegmentState;

/// Downloads a single byte-range segment and writes it to the correct offset in the file.
///
/// # Split-awareness
/// After each chunk write the function re-reads the segment's current `end` from shared
/// state. If another worker called `split_and_claim()` and reduced this segment's end,
/// the function stops writing at the new boundary and returns, allowing the second
/// worker to handle the remainder via its own HTTP range request.
///
/// # Buffering
/// Uses a 64 KB `BufWriter` to amortize syscall overhead. The buffer is explicitly
/// flushed before the function returns so the OS write cache is committed to disk.
pub async fn download_segment(
    client: reqwest::Client,
    url: String,
    segment_idx: usize,
    file_path: String,
    state: Arc<Mutex<DownloadState>>,
    cancel_token: CancellationToken,
    total_size: u64,
    shaper: Option<Arc<crate::engine::shaper::TokenBucket>>,
    quota_tracker: Arc<crate::engine::quota::UsageTracker>,
    simulation: Option<Arc<crate::engine::test_utils::SimulationEngine>>,
    cookies: Option<String>,
    referer: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut retry_count = 0;
    const MAX_RETRIES: u32 = 5;

    loop {
        match download_segment_attempt(
            client.clone(),
            url.clone(),
            segment_idx,
            &file_path,
            state.clone(),
            cancel_token.clone(),
            total_size,
            shaper.clone(),
            quota_tracker.clone(),
            simulation.clone(),
            cookies.clone(),
            referer.clone(),
        )
        .await
        {
            Ok(_) => return Ok(()),
            Err(e) if retry_count < MAX_RETRIES && is_transient_error(e.as_ref()) => {
                retry_count += 1;
                let delay = std::time::Duration::from_millis(500 * 2u64.pow(retry_count - 1));
                tokio::time::sleep(delay).await;
                continue;
            }
            Err(e) => return Err(e),
        }
    }
}

async fn download_segment_attempt(
    client: reqwest::Client,
    url: String,
    segment_idx: usize,
    file_path: &str,
    state: Arc<Mutex<DownloadState>>,
    cancel_token: CancellationToken,
    total_size: u64,
    shaper: Option<Arc<crate::engine::shaper::TokenBucket>>,
    quota_tracker: Arc<crate::engine::quota::UsageTracker>,
    simulation: Option<Arc<crate::engine::test_utils::SimulationEngine>>,
    cookies: Option<String>,
    referer: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Capture the initial byte range for the HTTP request.
    let (start, initial_end, etag, last_modified) = {
        let s = state.lock().await;
        let seg = &s.segments[segment_idx];
        (seg.start + seg.downloaded, seg.end, s.etag.clone(), s.last_modified.clone())
    };

    if start > initial_end {
        return Ok(());
    }

    // Open an HTTP range request for original or remaining range.
    let mut rb = client.get(&url)
        .header(RANGE, format!("bytes={}-{}", start, initial_end))
        .header(USER_AGENT, "Mdownloader/2.0");

    // ── Session & Integrity Headers ──────────────────────────────────────────
    if let Some(c) = cookies { rb = rb.header(COOKIE, c); }
    if let Some(r) = referer { rb = rb.header(REFERER, r); }
    
    // Inject If-Range using ETag (recommended) or Last-Modified timestamp.
    if let Some(e) = etag { 
        rb = rb.header(IF_RANGE, e); 
    } else if let Some(lm) = last_modified {
        rb = rb.header(IF_RANGE, lm);
    }

    let start_time = std::time::Instant::now();
    let response = rb.send().await?;
    let latency = start_time.elapsed().as_millis() as u64;

    // Update global latency in state
    {
        let mut s = state.lock().await;
        s.last_latency_ms = latency;
    }

    if !response.status().is_success() {
        let status = response.status();
        if status == reqwest::StatusCode::FORBIDDEN || status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(format!("AUTH_REQUIRED:{}", status.as_u16()).into());
        }
        return Err(format!("Server returned error: {}", status).into());
    }

    let status = response.status();

    // ── Protocol Integrity Check ──────────────────────────────────────
    // If the server returns 200 OK instead of 206 Partial Content, but 
    // we have multiple segments, it's a protocol mismatch or a file change.
    {
        let mut s = state.lock().await;
        if status == reqwest::StatusCode::OK && s.segments.len() > 1 && !s.is_fallback {
            log::warn!("Server returned 200 OK during range request. Falling back to single-stream compatibility mode.");
            s.is_fallback = true;
            // Force re-segmentation: one active segment covering the entire remaining file.
            s.segments = vec![crate::types::SegmentInfo {
                start: 0,
                end: s.total_size.saturating_sub(1),
                downloaded: 0,
                state: crate::types::SegmentState::Active,
                retry_count: 0,
            }];
            // The current worker will now finish the whole file starting from 0.
        }
    }

    // Open the pre-allocated file for writing.
    let file = OpenOptions::new().write(true).open(file_path).await?;
    let mut writer = BufWriter::with_capacity(64 * 1024, file);
    
    // If the server returns 200 OK (full file) instead of a range, reset the write pointer.
    let final_start = if status == reqwest::StatusCode::OK && initial_end < total_size { 0 } else { start };
    
    writer.seek(std::io::SeekFrom::Start(final_start)).await?;

    let mut stream = response.bytes_stream();
    let mut write_pos = final_start;

    while let Some(chunk_result) = tokio::time::timeout(std::time::Duration::from_secs(30), stream.next()).await? {
        // ── Performance Validation: Simulation Hook ───────────────────────
        if let Some(ref sim) = simulation {
            sim.apply().await;
        }

        // Honour cancellation (pause or cancel).
        if cancel_token.is_cancelled() {
            writer.flush().await?;
            return Ok(());
        }

        let data = chunk_result?;

        // Read the current segment end — may have been reduced by split_and_claim().
        let current_end = {
            let s = state.lock().await;
            s.segments
                .get(segment_idx)
                .map(|seg| seg.end)
                .unwrap_or(initial_end)
        };

        // If a split moved our end behind the current write position, stop.
        if write_pos > current_end {
            break;
        }

        // Only write bytes up to the (possibly reduced) segment boundary.
        let bytes_available = (current_end - write_pos + 1) as usize;
        let bytes_to_write = data.len().min(bytes_available);

        writer.write_all(&data[..bytes_to_write]).await?;
        write_pos += bytes_to_write as u64;
        
        // Update I/O commits
        {
            let mut s = state.lock().await;
            s.io_commits += 1;
        }

        // ── Bandwidth Governance: Traffic Shaping ───────────────────────────
        if let Some(ref bucket) = shaper {
            if let Some(delay) = bucket.consume(bytes_to_write as i64) {
                tokio::time::sleep(delay).await;
            }
        }

        // ── Persistent Usage Tracking ───────────────────────────────────────
        quota_tracker.log_bytes(bytes_to_write as u64).await;

        // Update the segment's downloaded counter in shared state.
        {
            let mut s = state.lock().await;
            if let Some(seg) = s.segments.get_mut(segment_idx) {
                seg.downloaded = write_pos - seg.start;
            }
        }

        // If we wrote fewer bytes than the chunk contained, we've hit the split
        // boundary — yield the remainder to the new worker and stop.
        if bytes_to_write < data.len() {
            break;
        }
    }

    // Flush any buffered bytes before releasing the file handle.
    writer.flush().await?;

    // Mark this segment as Completed in shared state.
    {
        let mut s = state.lock().await;
        if let Some(seg) = s.segments.get_mut(segment_idx) {
            if seg.state == SegmentState::Active && seg.downloaded >= (seg.end - seg.start + 1) {
                seg.state = SegmentState::Completed;
            }
        }
    }

    Ok(())
}

/// Downloads a discrete media segment (HLS .ts / DASH .m4s) and saves it to a part file.
pub async fn download_stream_segment(
    client: reqwest::Client,
    url: String,
    part_path: String,
    state: Arc<Mutex<DownloadState>>,
    segment_idx: usize,
    cancel_token: CancellationToken,
    cookies: Option<String>,
    referer: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut rb = client.get(&url).header(USER_AGENT, "Mdownloader/2.0");
    if let Some(c) = cookies { rb = rb.header(COOKIE, c); }
    if let Some(r) = referer { rb = rb.header(REFERER, r); }

    let response = rb.send().await?;
    if !response.status().is_success() {
        return Err(format!("Stream segment fetch failed: {}", response.status()).into());
    }

    let mut file = tokio::fs::File::create(&part_path).await?;
    let mut stream = response.bytes_stream();

    while let Some(chunk_result) = stream.next().await {
        if cancel_token.is_cancelled() {
            return Ok(());
        }
        let data = chunk_result?;
        file.write_all(&data).await?;
    }

    file.flush().await?;

    // Mark completed in shared state
    {
        let mut s = state.lock().await;
        if let Some(seg) = s.segments.get_mut(segment_idx) {
            seg.state = crate::types::SegmentState::Completed;
            seg.downloaded = 1; // Mark as non-zero to show progress
        }
    }

    Ok(())
}

fn is_transient_error(e: &(dyn std::error::Error + Send + Sync)) -> bool {
    let s = e.to_string().to_lowercase();
    s.contains("timeout")
        || s.contains("connection reset")
        || s.contains("incomplete message")
        || s.contains("503")
        || s.contains("502")
        || s.contains("504")
}
