use std::path::PathBuf;
use std::process::Command;
use tauri::AppHandle;
use tauri::Manager;

#[derive(Debug, Clone, serde::Serialize)]
pub struct DependencyStatus {
    pub ffmpeg_found: bool,
    pub ffprobe_found: bool,
    pub ffmpeg_path: Option<String>,
}

pub fn get_bundle_bin_dir(app: &AppHandle) -> Option<PathBuf> {
    app.path().resource_dir().ok().map(|p| p.join("bin"))
}

pub fn check_ffmpeg_availability(app: &AppHandle) -> DependencyStatus {
    let mut ffmpeg_path = None;
    let mut ffmpeg_found = false;
    let mut ffprobe_found = false;

    // 1. Check if it's already in the system PATH
    if Command::new("ffmpeg").arg("-version").output().is_ok() {
        ffmpeg_found = true;
        ffmpeg_path = Some("ffmpeg".to_string());
    }
    
    if Command::new("ffprobe").arg("-version").output().is_ok() {
        ffprobe_found = true;
    }

    // 2. If not found in PATH, check the localized 'bin' resource directory
    if !ffmpeg_found || !ffprobe_found {
        if let Some(bin_dir) = get_bundle_bin_dir(app) {
            let local_ffmpeg = bin_dir.join("ffmpeg.exe");
            let local_ffprobe = bin_dir.join("ffprobe.exe");

            if local_ffmpeg.exists() && Command::new(&local_ffmpeg).arg("-version").output().is_ok() {
                ffmpeg_found = true;
                ffmpeg_path = Some(local_ffmpeg.to_string_lossy().to_string());
            }

            if local_ffprobe.exists() && Command::new(&local_ffprobe).arg("-version").output().is_ok() {
                ffprobe_found = true;
            }
        }
    }

    DependencyStatus {
        ffmpeg_found,
        ffprobe_found,
        ffmpeg_path,
    }
}

/// Helper for the engine to execute FFmpeg commands with the resolved path.
pub fn get_ffmpeg_command(app: &AppHandle) -> Result<Command, String> {
    let status = check_ffmpeg_availability(app);
    if let Some(path) = status.ffmpeg_path {
        Ok(Command::new(path))
    } else {
        Err("FFmpeg binary not detected in PATH or resource directory".to_string())
    }
}
