use serde::{Serialize, Deserialize};
use std::sync::Arc;
use regex::Regex;
use std::time::Duration;
use crate::engine::state::AppState;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use tokio::io::AsyncWriteExt;
use url::Url;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DiscoveryScope {
    /// Only discover assets within the current folder/prefix (e.g. example.com/dir/*).
    Folder,
    /// Discover assets on the same host (e.g. www.example.com).
    SameDomain,
    /// Discover assets across all subdomains of the main organizational domain (e.g. *.example.com).
    CrossSubdomain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrabbedAsset {
    pub url: String,
    pub filename: String,
    pub mime: String,
    pub size: u64,
    pub category: String,
}

pub struct SiteGrabber {
    client: reqwest::Client,
    state: Arc<AppState>,
}

impl SiteGrabber {
    pub fn new(client: reqwest::Client, state: Arc<AppState>) -> Self {
        Self { client, state }
    }

    /// Primary entry point for industrial discovery.
    /// Orchestrates both Rust-based high-concurrency crawling and 
    /// Chrome-assisted headless traversal for SPAs.
    pub async fn grab_industrial(
        &self, 
        start_url: &str, 
        max_depth: u32, 
        scope: DiscoveryScope,
        external_link_depth: u32,
        use_headless: bool,
        cancel_token: CancellationToken
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        
        // 1. Initial Seeding of the Persistent Frontier
        self.state.discovery_frontier.clear_all().await?;
        self.state.discovery_frontier.add_url(start_url, 0).await?;

        if use_headless {
            #[cfg(windows)]
            {
                return self.headless_grab(start_url).await;
            }
            #[cfg(not(windows))]
            {
                return Err("Headless crawling is only supported on Windows.".into());
            }
        }

        let base_url = Url::parse(start_url)?;
        let start_url_prefix = if scope == DiscoveryScope::Folder {
            let mut s = start_url.to_string();
            if !s.ends_with('/') && !s.split('/').last().unwrap_or("").contains('.') {
                s.push('/');
            }
            Some(s)
        } else {
            None
        };
        let main_domain = self.extract_main_domain(&base_url);

        // 2. High-Concurrency Discovery Loop (Rust-based)
        let mut join_set: JoinSet<(String, Result<(), Box<dyn std::error::Error + Send + Sync>>)> = JoinSet::new();
        let concurrency_limit = 32;
        let mut active_count = 0;
        let current_urls: Arc<std::sync::Mutex<Vec<String>>> = Arc::new(std::sync::Mutex::new(Vec::new()));
        
        // Telemetry Pulse
        let mut stats_interval = tokio::time::interval(Duration::from_millis(500));

        loop {
            if cancel_token.is_cancelled() {
                join_set.shutdown().await;
                break;
            }

            // Fill the JoinSet up to the concurrency limit
            while active_count < concurrency_limit {
                if cancel_token.is_cancelled() { break; }

                let pending = self.state.discovery_frontier.get_pending(concurrency_limit - active_count).await?;
                if pending.is_empty() && active_count == 0 {
                    return Ok(()); // Discovery complete
                }
                
                if pending.is_empty() { break; }

                for item in pending {
                    let client = self.client.clone();
                    let state = self.state.clone();
                    let url_str = item.url.clone();
                    let depth = item.depth;
                    let main_domain_owned = main_domain.clone();
                    let prefix_owned = start_url_prefix.clone();
                    let token = cancel_token.clone();
                    let current_urls_clone = current_urls.clone();

                    state.discovery_frontier.mark_status(&url_str, "crawling").await?;

                    {
                        let mut urls = current_urls_clone.lock().unwrap();
                        urls.push(url_str.clone());
                    }

                    active_count += 1;
                    join_set.spawn(async move {
                        if token.is_cancelled() { return (url_str, Ok(())); }
                        let result = Self::process_url(&client, &state, &url_str, depth, max_depth, scope, external_link_depth, &main_domain_owned, prefix_owned.as_deref(), token).await;
                        
                        {
                            let mut urls = current_urls_clone.lock().unwrap();
                            if let Some(pos) = urls.iter().position(|r| r == &url_str) {
                                urls.remove(pos);
                            }
                        }
                        (url_str, result)
                    });
                }
            }

            // Await a discovery task result OR a telemetry pulse
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    join_set.shutdown().await;
                    break;
                }
                _ = stats_interval.tick() => {
                    let urls = current_urls.lock().unwrap().clone();
                    let current_url = urls.first().cloned().unwrap_or_default();
                    
                    let (auth, cookies) = if !current_url.is_empty() {
                        self.state.auth_manager.get_headers_for_url(&current_url).await
                    } else {
                        (None, None)
                    };

                    let _ = self.state.emit("grabber-telemetry", serde_json::json!({
                        "active_workers": active_count,
                        "is_crawling": true,
                        "auth_active": auth.is_some() || cookies.is_some(),
                        "current_url": current_url,
                        "all_urls": urls,
                    }));
                }
                res = join_set.join_next() => {
                    if let Some(res) = res {
                        active_count -= 1;
                        match res {
                            Ok((url, Ok(_))) => {
                                let _ = self.state.discovery_frontier.mark_status(&url, "completed").await;
                            },
                            Ok((url, Err(e))) => {
                                eprintln!("[Grabber] Discovery failed for {}: {}", url, e);
                                let _ = self.state.discovery_frontier.mark_status(&url, "failed").await;
                            },
                            Err(e) => eprintln!("[Grabber] JoinSet panic: {}", e),
                        }
                    } else if active_count == 0 {
                        break;
                    }
                }
            }
        }

        let _ = self.state.emit("grabber-telemetry", serde_json::json!({
            "active_workers": 0,
            "is_crawling": false,
            "current_url": "",
            "all_urls": [],
        }));

        Ok(())
    }

    async fn process_url(
        client: &reqwest::Client,
        state: &Arc<AppState>,
        url_str: &str,
        depth: u32,
        max_depth: u32,
        scope: DiscoveryScope,
        external_link_depth: u32,
        main_domain: &str,
        prefix: Option<&str>,
        cancel_token: CancellationToken
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if cancel_token.is_cancelled() { return Ok(()); }

        let mut rb = client.get(url_str).timeout(Duration::from_secs(10));
        let (auth, cookies) = state.auth_manager.get_headers_for_url(url_str).await;
        
        let auth_active = auth.is_some() || cookies.is_some();
        if auth_active {
            eprintln!("[Grabber] Authenticated request for: {}", url_str);
        }

        if let Some(a) = auth { rb = rb.header("Authorization", a); }
        if let Some(c) = cookies { rb = rb.header(reqwest::header::COOKIE, c); }

        let res = rb.send().await?;
        let mime = res.headers().get("content-type").and_then(|v| v.to_str().ok()).unwrap_or("");

        if !mime.contains("text/html") {
            let filename = url_str.split('/').last().unwrap_or("unknown").to_string();
            let size = res.headers().get("content-length").and_then(|v| v.to_str().ok().and_then(|s| s.parse::<u64>().ok())).unwrap_or(0);
            let category = Self::get_category(mime, &filename);

            let asset = GrabbedAsset {
                url: url_str.to_string(),
                filename,
                mime: mime.to_string(),
                size,
                category,
            };

            let _ = state.emit("grabber-asset-found", asset);
            return Ok(());
        }

        if depth < max_depth && !cancel_token.is_cancelled() {
            let html = res.text().await?;
            let mut extracted = Vec::new();
            let base = Url::parse(url_str).ok();
            
            let re = Regex::new(r#"href=["'](https?://[^"']+|/[^"']+|[^"']+)["']"#).unwrap();
            for cap in re.captures_iter(&html) {
                if cancel_token.is_cancelled() { break; }
                let found = &cap[1];
                if let Some(base_url) = &base {
                    if let Ok(abs) = base_url.join(found) {
                        let link_str = abs.to_string();
                        
                        let in_scope = match scope {
                            DiscoveryScope::Folder => {
                                prefix.map_or(false, |p| link_str.starts_with(p))
                            },
                            DiscoveryScope::SameDomain => {
                                abs.host_str() == base_url.host_str()
                            },
                            DiscoveryScope::CrossSubdomain => {
                                Self::extract_main_domain_static(&abs) == main_domain
                            },
                        };

                        if in_scope {
                            extracted.push(link_str);
                        } else if external_link_depth > 0 {
                            // Wildcard mode: follow external links but don't recurse them further
                            extracted.push(link_str);
                        }
                    }
                }
            }

            for link in extracted {
                if cancel_token.is_cancelled() { break; }
                state.discovery_frontier.add_url(&link, depth + 1).await?;
            }
        }

        Ok(())
    }

    /// Triggers the Chrome Bridge to navigate and extract links from an SPA.
    #[cfg(windows)]
    async fn headless_grab(&self, url: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut bridge_writer_lock = self.state.bridge_writer.lock().await;
        if let Some(writer) = bridge_writer_lock.as_mut() {
            let cmd = serde_json::json!({
                "type": "CRAWL_INSTRUCTION",
                "url": url
            });
            let payload = serde_json::to_string(&cmd)? + "\n";
            writer.write_all(payload.as_bytes()).await?;
            Ok(())
        } else {
            Err("Chrome Bridge disconnected. Could not trigger headless grab.".into())
        }
    }

    fn extract_main_domain(&self, url: &Url) -> String {
        Self::extract_main_domain_static(url)
    }

    fn extract_main_domain_static(url: &Url) -> String {
        let host = url.host_str().unwrap_or("");
        let parts: Vec<&str> = host.split('.').collect();
        if parts.len() >= 2 {
            // Very simple organizational domain logic (e.g. example.com from media.example.com)
            format!("{}.{}", parts[parts.len()-2], parts[parts.len()-1])
        } else {
            host.to_string()
        }
    }

    pub fn get_category(mime: &str, filename: &str) -> String {
        let ext = filename.split('.').last().unwrap_or("").to_lowercase();
        
        // Industrial Buckets (UI Requirements)
        if mime.contains("video") || ["mp4", "mkv", "avi", "ts", "m3u8"].contains(&ext.as_str()) { return "Video".to_string(); }
        if ["zip", "rar", "7z", "iso", "tar", "gz"].contains(&ext.as_str()) { return "Compressed".to_string(); }
        if ["exe", "msi", "dmg", "pkg", "deb", "rpm"].contains(&ext.as_str()) { return "Programs".to_string(); }
        
        // General Media/Docs
        if mime.contains("image") || ["jpg", "png", "gif", "webp"].contains(&ext.as_str()) { return "Images".to_string(); }
        if mime.contains("audio") || ["mp3", "wav", "flac", "m4a"].contains(&ext.as_str()) { return "Music".to_string(); }
        if ["pdf", "doc", "docx", "xls", "xlsx", "dwg"].contains(&ext.as_str()) { return "Documents".to_string(); }
        
        "General".to_string()
    }
}
