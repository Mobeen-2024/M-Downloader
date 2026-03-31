use std::process::Command;
use std::path::PathBuf;
// use m3u8_rs::Playlist;
use reqwest::Client;
use async_recursion::async_recursion;
// pub use dash_mpd::MPD;

pub struct MediaSegment {
    pub url: String,
    pub duration_secs: f64,
}

pub struct MediaStream {
    pub segments: Vec<MediaSegment>,
    pub master_url: String,
}

impl MediaStream {
    /// Parses an HLS .m3u8 playlist and returns all segment URLs.
    #[async_recursion]
    pub async fn from_hls(_client: &Client, _url: &str) -> Result<Self, String> {
        Err("HLS (.m3u8) parsing is currently disabled. This feature requires the 'protoc' compiler to be installed during build time.".to_string())
        /*
        let res = client.get(url).send().await.map_err(|e| e.to_string())?;
        let bytes = res.bytes().await.map_err(|e| e.to_string())?;
        
        match m3u8_rs::parse_playlist_res(&bytes) {
            Ok(Playlist::MasterPlaylist(master)) => {
                let first_variant = master.variants.first()
                    .ok_or("Master playlist contains no variants")?;
                let variant_url = resolve_url(url, &first_variant.uri);
                Self::from_hls(client, &variant_url).await
            }
            Ok(Playlist::MediaPlaylist(media)) => {
                let mut segments = Vec::new();
                for seg in media.segments {
                    segments.push(MediaSegment {
                        url: resolve_url(url, &seg.uri),
                        duration_secs: seg.duration as f64,
                    });
                }
                Ok(Self { segments, master_url: url.to_string() })
            }
            Err(e) => Err(format!("Failed to parse HLS playlist: {:?}", e)),
        }
        */
    }

    /// Merges downloaded .ts or .m4s segments into a single .mp4 file via FFmpeg.
    pub fn merge_with_ffmpeg(segment_files: &[PathBuf], output_file: &PathBuf) -> Result<(), String> {
        let concat_content = segment_files.iter()
            .map(|f| format!("file '{}'", f.to_str().unwrap().replace("\\", "/")))
            .collect::<Vec<_>>()
            .join("\n");
        
        let concat_file = output_file.with_extension("txt");
        std::fs::write(&concat_file, concat_content).map_err(|e| e.to_string())?;

        let status = Command::new("ffmpeg")
            .arg("-f").arg("concat")
            .arg("-safe").arg("0")
            .arg("-i").arg(&concat_file)
            .arg("-c").arg("copy")
            .arg("-y")
            .arg(output_file)
            .status()
            .map_err(|e| format!("FFmpeg execution failed: {}", e))?;

        let _ = std::fs::remove_file(concat_file);

        if status.success() {
            Ok(())
        } else {
            Err("FFmpeg failed to merge segments".to_string())
        }
    }

    /// Checks if FFmpeg is installed and accessible in the system PATH.
    pub fn is_ffmpeg_available() -> bool {
        Command::new("ffmpeg")
            .arg("-version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Parses a DASH .mpd manifest and returns all segment URLs.
    pub async fn from_dash(_client: &Client, _url: &str) -> Result<Self, String> {
        Err("DASH (.mpd) parsing is currently disabled. This feature requires the 'protoc' compiler to be installed during build time.".to_string())
    }
}

/// Resolves a relative URL against a base URL.
#[allow(dead_code)]
fn resolve_url(base: &str, relative: &str) -> String {
    if relative.starts_with("http") {
        return relative.to_string();
    }
    let base_parts: Vec<&str> = base.split('?').next().unwrap().split('/').collect();
    let parent = base_parts[..base_parts.len() - 1].join("/");
    format!("{}/{}", parent, relative)
}
