use crate::engine::segmenter::DownloadState;
use crate::types::SegmentState;
use std::path::Path;

/// Returns the path to the sidecar metadata file for a given download.
/// Uses a dot-prefix to hide the file on Unix-like systems.
fn meta_path(file_path: &str) -> String {
    let path = Path::new(file_path);
    if let (Some(parent), Some(file_name)) = (path.parent(), path.file_name()) {
        if let Some(name_str) = file_name.to_str() {
            let hidden_name = format!(".{}.mdown", name_str);
            return parent.join(hidden_name).to_string_lossy().into_owned();
        }
    }
    format!("{}.mdown", file_path)
}

/// Persists the current DownloadState to a JSON sidecar file alongside the partial download.
/// Called after each segment completes so we lose at most one segment of progress on crash.
pub async fn save_state(state: &DownloadState) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let path = meta_path(&state.file_path);
    let json = serde_json::to_string_pretty(state)?;
    tokio::fs::write(&path, json).await?;
    Ok(())
}

/// Loads a DownloadState from the sidecar file. Returns None if no sidecar exists.
/// On resume, Active segments are reset to Pending so workers can re-claim them.
pub async fn load_state(file_path: &str) -> Option<DownloadState> {
    let path = meta_path(file_path);
    let json = tokio::fs::read_to_string(&path).await.ok()?;
    let mut state: DownloadState = serde_json::from_str(&json).ok()?;

    // Any segment that was Active when the app crashed must be re-downloaded.
    // Reset them to Pending so workers pick them up fresh.
    for seg in &mut state.segments {
        if seg.state == SegmentState::Active {
            seg.state = SegmentState::Pending;
        }
    }

    Some(state)
}

/// Removes the sidecar file once a download completes successfully.
pub async fn delete_state(file_path: &str) {
    let path = meta_path(file_path);
    let _ = tokio::fs::remove_file(&path).await;
}
