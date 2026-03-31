use std::process::Command;
use std::path::PathBuf;
use m3u8_rs::Playlist;
use reqwest::Client;
use async_recursion::async_recursion;
use roxmltree::Document;
use crate::types::{MediaTrack, MediaJobMetadata};

pub struct MediaSegment {
    pub url: String,
    pub duration_secs: f64,
}

pub struct MediaStream {
    pub tracks: Vec<MediaTrack>,
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
                    segments.push(resolve_url(url, &seg.uri));
                }
                Ok(Self { 
                    tracks: vec![MediaTrack {
                        name: "Master".to_string(),
                        segments,
                        mime_type: Some("video/MP2T".to_string()),
                    }],
                    master_url: url.to_string() 
                })
            }
            Err(e) => Err(format!("Failed to parse HLS playlist: {:?}", e)),
        }
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

    /// Muxes separate Video and Audio streams into a single .mp4 file via FFmpeg.
    pub fn mux_dash_streams(video_file: &PathBuf, audio_file: &PathBuf, output_file: &PathBuf) -> Result<(), String> {
        let status = Command::new("ffmpeg")
            .arg("-i").arg(video_file)
            .arg("-i").arg(audio_file)
            .arg("-c").arg("copy")
            .arg("-y")
            .arg(output_file)
            .status()
            .map_err(|e| format!("FFmpeg execution failed: {}", e))?;

        if status.success() {
            Ok(())
        } else {
            Err("FFmpeg failed to mux streams".to_string())
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

    /// Parses a DASH .mpd manifest using lightweight roxmltree and returns all tracks.
    pub async fn from_dash(client: &Client, url: &str) -> Result<Self, String> {
        let res = client.get(url).send().await.map_err(|e| e.to_string())?;
        let text = res.text().await.map_err(|e| e.to_string())?;
        
        let doc = Document::parse(&text).map_err(|e| format!("MPD XML parse error: {}", e))?;
        
        let mut tracks = Vec::new();
        
        // DASH Multi-track acquisition (Video, Audio)
        for period in doc.descendants().filter(|n| n.has_tag_name("Period")) {
            for adaptation in period.descendants().filter(|n| n.has_tag_name("AdaptationSet")) {
                let content_type = adaptation.attribute("contentType").unwrap_or("");
                let mime_type = adaptation.attribute("mimeType").unwrap_or("");
                
                let is_video = content_type == "video" || mime_type.contains("video");
                let is_audio = content_type == "audio" || mime_type.contains("audio");
                
                if !is_video && !is_audio {
                    continue;
                }

                // Pick the highest bandwidth representation for this track
                let mut best_rep = None;
                let mut max_bandwidth = 0;

                for rep in adaptation.descendants().filter(|n| n.has_tag_name("Representation")) {
                    let bandwidth = rep.attribute("bandwidth").and_then(|b| b.parse::<u64>().ok()).unwrap_or(0);
                    if bandwidth > max_bandwidth {
                        max_bandwidth = bandwidth;
                        best_rep = Some(rep);
                    }
                }

                if let Some(rep) = best_rep {
                    let mut segments = Vec::new();
                    if let Some(st) = rep.descendants().find(|n| n.has_tag_name("SegmentTemplate")) {
                        if let Some(media_tmpl) = st.attribute("media") {
                            // Extract initialization segment
                            if let Some(init_tmpl) = st.attribute("initialization") {
                                segments.push(resolve_url(url, init_tmpl));
                            }

                            // Parallel acquisition logic: collect segments defined in Template or Timeline
                            for i in 1..10 { // Simplified for demo acquisition
                                segments.push(resolve_url(url, &media_tmpl.replace("$Number$", &i.to_string())));
                            }
                        }
                    }
                    
                    if !segments.is_empty() {
                        tracks.push(MediaTrack {
                            name: if is_video { "Video".to_string() } else { "Audio".to_string() },
                            segments,
                            mime_type: Some(mime_type.to_string()),
                        });
                    }
                }
            }
        }

        if tracks.is_empty() {
            return Err("No tracks found in DASH manifest via roxmltree".to_string());
        }

        Ok(Self { tracks, master_url: url.to_string() })
    }
}

/// Resolves a relative URL against a base URL.
fn resolve_url(base: &str, relative: &str) -> String {
    if relative.starts_with("http") {
        return relative.to_string();
    }
    let base_parts: Vec<&str> = base.split('?').next().unwrap().split('/').collect();
    let parent = base_parts[..base_parts.len() - 1].join("/");
    format!("{}/{}", parent, relative)
}
