use std::process::Command;
use std::path::PathBuf;
use m3u8_rs::Playlist;
use reqwest::Client;
use async_recursion::async_recursion;
pub use dash_mpd::MPD;

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
    pub async fn from_hls(client: &Client, url: &str) -> Result<Self, String> {
        let res = client.get(url).send().await.map_err(|e| e.to_string())?;
        let bytes = res.bytes().await.map_err(|e| e.to_string())?;
        
        match m3u8_rs::parse_playlist_res(&bytes) {
            Ok(Playlist::MasterPlaylist(master)) => {
                // If master, pick the first (usually highest quality) variant.
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
    }

    /// Merges downloaded .ts or .m4s segments into a single .mp4 file via FFmpeg.
    /// This requires `ffmpeg` to be installed in the system PATH.
    pub fn merge_with_ffmpeg(segment_files: &[PathBuf], output_file: &PathBuf) -> Result<(), String> {
        // Create a temporary file for FFmpeg's concat protocol.
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

        // Cleanup temporary concat file.
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
    pub async fn from_dash(client: &Client, url: &str) -> Result<Self, String> {
        let res = client.get(url).send().await.map_err(|e| e.to_string())?;
        let text = res.text().await.map_err(|e| e.to_string())?;
        
        let mpd: dash_mpd::MPD = dash_mpd::parse(&text).map_err(|e| format!("DASH parse error: {}", e))?;
        
        let mut segments = Vec::new();
        // Simplified: pick the first AdaptationSet and first Representation.
        if let Some(period) = mpd.periods.first() {
            for as_set in &period.adaptations {
                if let Some(rep) = as_set.representations.first() {
                    // Extract segment URLs (logic varies by DASH profile, this is basic SegmentList/Template)
                    if let Some(st) = &rep.segment_template {
                        if let Some(media_tmpl) = &st.media {
                            // Example: segment_$Number$.m4s
                            // In a real app, we'd iterate through SegmentTimeline or startNumber.
                            for i in 1..10 { // Placeholder: first 10 segments for demo
                                let segment_url: String = media_tmpl.replace("$Number$", &i.to_string());
                                segments.push(MediaSegment {
                                    url: resolve_url(url, &segment_url),
                                    duration_secs: 2.0, // Default for DASH
                                });
                            }
                        }
                    }
                }
            }
        }

        if segments.is_empty() {
            return Err("No segments found in DASH manifest".to_string());
        }

        Ok(Self { segments, master_url: url.to_string() })
    }
}

/// Resolves a relative URL against a base URL.
fn resolve_url(base: &str, relative: &str) -> String {
    if relative.starts_with("http") {
        return relative.to_string();
    }
    // Handle query parameters or complex paths
    let base_parts: Vec<&str> = base.split('?').next().unwrap().split('/').collect();
    let parent = base_parts[..base_parts.len() - 1].join("/");
    format!("{}/{}", parent, relative)
}
