use tauri::{AppHandle, command};
use crate::engine::provisioner::{check_ffmpeg_availability, DependencyStatus};

#[command]
pub async fn check_dependencies(app: AppHandle) -> Result<DependencyStatus, String> {
    Ok(check_ffmpeg_availability(&app))
}
