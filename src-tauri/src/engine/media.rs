use std::path::PathBuf;
use m3u8_rs::Playlist;
use reqwest::Client;
use async_recursion::async_recursion;
use roxmltree::Document;
use crate::types::{MediaTrack, MediaResolution};
use crate::engine::deobfuscator::YouTubeDeobfuscator;

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
                        size: None, // HLS total size is unknown until segmented
                    }],
                    master_url: url.to_string() 
                })
            }
            Err(e) => Err(format!("Failed to parse HLS playlist: {:?}", e)),
        }
    }

    /// Merges downloaded .ts or .m4s segments into a single .mp4 file via FFmpeg.
    pub fn merge_with_ffmpeg(
        app: &tauri::AppHandle,
        segment_files: &[PathBuf], 
        output_file: &PathBuf
    ) -> Result<(), String> {
        let concat_content = segment_files.iter()
            .map(|f| format!("file '{}'", f.to_str().unwrap().replace("\\", "/")))
            .collect::<Vec<_>>()
            .join("\n");
        
        let concat_file = output_file.with_extension("txt");
        std::fs::write(&concat_file, concat_content).map_err(|e| e.to_string())?;

        let mut cmd = crate::engine::provisioner::get_ffmpeg_command(app)?;
        let status = cmd
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
    pub fn mux_dash_streams(
        app: &tauri::AppHandle,
        video_file: &PathBuf, 
        audio_file: &PathBuf, 
        output_file: &PathBuf
    ) -> Result<(), String> {
        let mut cmd = crate::engine::provisioner::get_ffmpeg_command(app)?;
        let status = cmd
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

    /// Parses a DASH .mpd manifest using lightweight roxmltree and returns all tracks.
    pub async fn from_dash(
        client: &Client, 
        url: &str, 
        deobfuscator: Option<&YouTubeDeobfuscator>,
        base_js_url: Option<&str>
    ) -> Result<Self, String> {
        let res = client.get(url).send().await.map_err(|e| e.to_string())?;
        let text = res.text().await.map_err(|e| e.to_string())?;
        
        let doc = Document::parse(&text).map_err(|e| format!("MPD XML parse error: {}", e))?;
        
        let mut tracks = Vec::new();
        
        for period in doc.descendants().filter(|n| n.has_tag_name("Period")) {
            for adaptation in period.descendants().filter(|n| n.has_tag_name("AdaptationSet")) {
                let content_type = adaptation.attribute("contentType").unwrap_or("");
                let mime_type = adaptation.attribute("mimeType").unwrap_or("");
                
                let is_video = content_type == "video" || mime_type.contains("video");
                let is_audio = content_type == "audio" || mime_type.contains("audio");
                
                if !is_video && !is_audio {
                    continue;
                }

                for rep in adaptation.descendants().filter(|n| n.has_tag_name("Representation")) {
                    let bandwidth = rep.attribute("bandwidth").and_then(|b| b.parse::<u64>().ok()).unwrap_or(0);
                    let _width = rep.attribute("width").and_then(|w| w.parse::<u32>().ok());
                    let _height = rep.attribute("height").and_then(|h| h.parse::<u32>().ok());
                    
                    let mut segments = Vec::new();
                    if let Some(st) = rep.descendants().find(|n| n.has_tag_name("SegmentTemplate")) {
                        if let Some(media_tmpl) = st.attribute("media") {
                            if let Some(init_tmpl) = st.attribute("initialization") {
                                segments.push(resolve_url(url, init_tmpl));
                            }

                            // Just peek at the first few segments to verify URLs for HUD
                            for i in 1..3 { 
                                let mut segment_url = resolve_url(url, &media_tmpl.replace("$Number$", &i.to_string()));

                                // ── YouTube Deobfuscation Integration ──────────────────────
                                if let (Some(deob), Some(b_js)) = (deobfuscator, base_js_url) {
                                    if segment_url.contains("youtube.com") || segment_url.contains("googlevideo.com") {
                                        if let Some(n_param) = extract_query_param(&segment_url, "n") {
                                            if let Ok(sn) = deob.solve_n(&n_param, b_js).await {
                                                segment_url = replace_query_param(&segment_url, "n", &sn);
                                            }
                                        }
                                        if let Some(s_param) = extract_query_param(&segment_url, "sig") {
                                            if let Ok(ss) = deob.solve_signature(&s_param, b_js).await {
                                                segment_url = replace_query_param(&segment_url, "sig", &ss);
                                            }
                                        }
                                    }
                                }
                                segments.push(segment_url);
                            }
                        }
                    } else if let Some(base_url_node) = rep.descendants().find(|n| n.has_tag_name("BaseURL")) {
                        // Single file DASH (typical for some streams)
                        let mut base_url = resolve_url(url, base_url_node.text().unwrap_or(""));
                        
                        if let (Some(deob), Some(b_js)) = (deobfuscator, base_js_url) {
                            if base_url.contains("youtube.com") || base_url.contains("googlevideo.com") {
                                if let Some(n_param) = extract_query_param(&base_url, "n") {
                                    if let Ok(sn) = deob.solve_n(&n_param, b_js).await {
                                        base_url = replace_query_param(&base_url, "n", &sn);
                                    }
                                }
                                if let Some(s_param) = extract_query_param(&base_url, "sig") {
                                    if let Ok(ss) = deob.solve_signature(&s_param, b_js).await {
                                        base_url = replace_query_param(&base_url, "sig", &ss);
                                    }
                                }
                            }
                        }
                        segments.push(base_url);
                    }
                    
                    if !segments.is_empty() {
                        tracks.push(MediaTrack {
                            name: format!("{} - {}bps", if is_video { "Video" } else { "Audio" }, bandwidth),
                            segments,
                            mime_type: Some(mime_type.to_string()),
                            size: None, // DASH manifests usually only provide bitrate
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

    /// Extracts all available resolutions and bitrates from a DASH manifest for the UI HUD.
    pub async fn extract_resolutions(
        client: &Client,
        url: &str,
        deobfuscator: Option<&YouTubeDeobfuscator>,
        base_js_url: Option<&str>
    ) -> Result<Vec<MediaResolution>, String> {
        let res = client.get(url).send().await.map_err(|e| e.to_string())?;
        let text = res.text().await.map_err(|e| e.to_string())?;
        
        let doc = Document::parse(&text).map_err(|e| format!("MPD parse error: {}", e))?;
        let mut resolutions = Vec::new();

        // 1. Collect Audio Tracks first (to attach to video resolutions if needed)
        let mut audio_tracks = Vec::new();
        for adaptation in doc.descendants().filter(|n| n.has_tag_name("AdaptationSet")) {
            let mime = adaptation.attribute("mimeType").unwrap_or("");
            if mime.contains("audio") || adaptation.attribute("contentType") == Some("audio") {
                for rep in adaptation.descendants().filter(|n| n.has_tag_name("Representation")) {
                    if let Some(base_url_node) = rep.descendants().find(|n| n.has_tag_name("BaseURL")) {
                        let mut audio_url = resolve_url(url, base_url_node.text().unwrap_or(""));
                        
                        // Deobfuscate audio URL
                        if let (Some(deob), Some(b_js)) = (deobfuscator, base_js_url) {
                            if audio_url.contains("youtube.com") || audio_url.contains("googlevideo.com") {
                                if let Some(n_param) = extract_query_param(&audio_url, "n") {
                                    if let Ok(sn) = deob.solve_n(&n_param, b_js).await {
                                        audio_url = replace_query_param(&audio_url, "n", &sn);
                                    }
                                }
                                if let Some(s_param) = extract_query_param(&audio_url, "sig") {
                                    if let Ok(ss) = deob.solve_signature(&s_param, b_js).await {
                                        audio_url = replace_query_param(&audio_url, "sig", &ss);
                                    }
                                }
                            }
                        }
                        audio_tracks.push(audio_url);
                    }
                }
            }
        }

        // 2. Collect Video Representations
        for adaptation in doc.descendants().filter(|n| n.has_tag_name("AdaptationSet")) {
            let mime = adaptation.attribute("mimeType").unwrap_or("");
            if mime.contains("video") || adaptation.attribute("contentType") == Some("video") {
                for rep in adaptation.descendants().filter(|n| n.has_tag_name("Representation")) {
                    let bandwidth = rep.attribute("bandwidth").and_then(|b| b.parse::<u64>().ok()).unwrap_or(0);
                    let width = rep.attribute("width").and_then(|w| w.parse::<u32>().ok());
                    let height = rep.attribute("height").and_then(|h| h.parse::<u32>().ok());
                    
                    let mut video_url = if let Some(base) = rep.descendants().find(|n| n.has_tag_name("BaseURL")) {
                        resolve_url(url, base.text().unwrap_or(""))
                    } else {
                        url.to_string() // Fallback to master if template based
                    };

                    // Deobfuscate video URL
                    if let (Some(deob), Some(b_js)) = (deobfuscator, base_js_url) {
                        if video_url.contains("youtube.com") || video_url.contains("googlevideo.com") {
                            if let Some(n_param) = extract_query_param(&video_url, "n") {
                                if let Ok(sn) = deob.solve_n(&n_param, b_js).await {
                                    video_url = replace_query_param(&video_url, "n", &sn);
                                }
                            }
                            if let Some(s_param) = extract_query_param(&video_url, "sig") {
                                if let Ok(ss) = deob.solve_signature(&s_param, b_js).await {
                                    video_url = replace_query_param(&video_url, "sig", &ss);
                                }
                            }
                        }
                    }

                    let label = format!("{}p ({} Mbps)", height.unwrap_or(0), (bandwidth as f64 / 1_000_000.0).round());
                    
                    resolutions.push(MediaResolution {
                        label,
                        video_url,
                        audio_url: audio_tracks.first().cloned(), // Attach first available audio track
                        bandwidth,
                        width,
                        height,
                    });
                }
            }
        }

        resolutions.sort_by(|a, b| b.bandwidth.cmp(&a.bandwidth));
        Ok(resolutions)
    }

    /// Parses a YouTube HTML page and returns all streams (both progressive and adaptive) 
    /// from the embeded ytInitialPlayerResponse payload.
    pub async fn from_youtube_html(
        html: &str, 
        deobfuscator: Option<&YouTubeDeobfuscator>,
        base_js_url: Option<&str>
    ) -> Result<Self, String> {
        let json_str = extract_json_from_html(html, "ytInitialPlayerResponse")
            .ok_or_else(|| "ytInitialPlayerResponse not found in HTML".to_string())?;

        let v: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| format!("JSON Parse Error: {}", e))?;

        let mut tracks = Vec::new();

        if let Some(streaming_data) = v.get("streamingData") {
            // Progressive formats
            if let Some(formats) = streaming_data.get("formats").and_then(|f| f.as_array()) {
                for format in formats {
                    if let Some(track) = Self::parse_youtube_format(format, deobfuscator, base_js_url).await {
                        tracks.push(track);
                    }
                }
            }
            // Adaptive formats (DAB/DASH)
            if let Some(formats) = streaming_data.get("adaptiveFormats").and_then(|f| f.as_array()) {
                for format in formats {
                    if let Some(track) = Self::parse_youtube_format(format, deobfuscator, base_js_url).await {
                        tracks.push(track);
                    }
                }
            }
        }

        if tracks.is_empty() {
            return Err("No playable streams found in YouTube page".into());
        }

        Ok(Self { tracks, master_url: "youtube_video".into() })
    }

    async fn parse_youtube_format(
        format: &serde_json::Value,
        deobfuscator: Option<&YouTubeDeobfuscator>,
        base_js_url: Option<&str>
    ) -> Option<MediaTrack> {
        let mime_type = format.get("mimeType").and_then(|s| s.as_str()).unwrap_or("").to_string();
        let itag = format.get("itag").and_then(|i| i.as_u64()).unwrap_or(0);
        let quality_label = format.get("qualityLabel").and_then(|s| s.as_str()).unwrap_or("");
        let bitrate = format.get("bitrate").and_then(|b| b.as_u64()).unwrap_or(0);
        
        let mut name = format!("itag {} - {}", itag, mime_type.split(';').next().unwrap_or(&mime_type));
        if !quality_label.is_empty() {
            name = format!("{} ({} bps)", quality_label, bitrate);
        } else {
            name = format!("{} ({} bps)", name, bitrate);
        }

        let mut url = format.get("url").and_then(|s| s.as_str()).unwrap_or("").to_string();

        if url.is_empty() {
            if let Some(cipher) = format.get("signatureCipher").and_then(|s| s.as_str()) {
                url = Self::solve_youtube_cipher(cipher, deobfuscator, base_js_url).await;
            }
        }

        if url.is_empty() { return None; }

        // Also solve 'n' param if present in the final url
        if let (Some(deob), Some(b_js)) = (deobfuscator, base_js_url) {
            if let Some(n_param) = extract_query_param(&url, "n") {
                if let Ok(sn) = deob.solve_n(&n_param, b_js).await {
                    url = replace_query_param(&url, "n", &sn);
                }
            }
        }

        let size = format.get("contentLength").and_then(|s| s.as_str()).and_then(|s| s.parse::<u64>().ok());
        
        Some(MediaTrack {
            name,
            segments: vec![url],
            mime_type: Some(mime_type),
            size,
        })
    }

    async fn solve_youtube_cipher(
        cipher: &str,
        deobfuscator: Option<&YouTubeDeobfuscator>,
        base_js_url: Option<&str>
    ) -> String {
        let parsed = url::form_urlencoded::parse(cipher.as_bytes());
        let mut s_val = String::new();
        let mut url_val = String::new();
        let mut sp_val = "sig".to_string();

        for (k, v) in parsed {
            if k == "s" { s_val = v.into_owned(); }
            else if k == "url" { url_val = v.into_owned(); }
            else if k == "sp" { sp_val = v.into_owned(); }
        }

        let mut final_url = url_val;

        if let (Some(deob), Some(b_js)) = (deobfuscator, base_js_url) {
            if let Ok(sig) = deob.solve_signature(&s_val, b_js).await {
                let encoded_sig = url::form_urlencoded::byte_serialize(sig.as_bytes()).collect::<String>();
                final_url = format!("{}&{}={}", final_url, sp_val, encoded_sig);
            }
        }
        
        final_url
    }
}

/// Helper to extract query parameter from URL
fn extract_query_param(url: &str, param: &str) -> Option<String> {
    let parts: Vec<&str> = url.split('?').collect();
    if parts.len() < 2 { return None; }
    for pair in parts[1].split('&') {
        let kv: Vec<&str> = pair.split('=').collect();
        if kv.len() == 2 && kv[0] == param {
            return Some(kv[1].to_string());
        }
    }
    None
}

/// Helper to replace query parameter in URL
fn replace_query_param(url: &str, param: &str, new_value: &str) -> String {
    let parts: Vec<&str> = url.split('?').collect();
    if parts.len() < 2 { return url.to_string(); }
    let mut new_query = Vec::new();
    for pair in parts[1].split('&') {
        let kv: Vec<&str> = pair.split('=').collect();
        if kv.len() == 2 && kv[0] == param {
            new_query.push(format!("{}={}", param, new_value));
        } else {
            new_query.push(pair.to_string());
        }
    }
    format!("{}?{}", parts[0], new_query.join("&"))
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

/// Robustly extracts a JSON object from HTML by matching curly brackets after a pattern.
fn extract_json_from_html(html: &str, pattern: &str) -> Option<String> {
    let start_idx = html.find(pattern)?;
    let subst = &html[start_idx..];
    
    // Find the first opening bracket
    let obj_start = subst.find('{')?;
    let mut count = 0;
    let mut in_string = false;
    let mut escape = false;

    for (i, c) in subst[obj_start..].char_indices() {
        if escape {
            escape = false;
            continue;
        }

        match c {
            '\\' => escape = true,
            '"' => in_string = !in_string,
            '{' if !in_string => count += 1,
            '}' if !in_string => {
                count -= 1;
                if count == 0 {
                    return Some(subst[obj_start..obj_start + i + 1].to_string());
                }
            }
            _ => {}
        }
    }

    None
}
