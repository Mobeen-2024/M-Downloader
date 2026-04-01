use serde::{Serialize, Deserialize};
use std::sync::Arc;
use regex::Regex;
use std::time::Duration;
use std::collections::{HashSet, VecDeque};
use crate::engine::auth::AuthManager;

pub enum ResolveResult {
    Asset(GrabbedAsset),
    Html,
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
    auth_manager: Arc<AuthManager>,
}

impl SiteGrabber {
    pub fn new(client: reqwest::Client, auth_manager: Arc<AuthManager>) -> Self {
        Self {
            client,
            auth_manager,
        }
    }

    pub async fn grab_page(&self, start_url: &str, max_depth: u32, strict_domain: bool) -> Result<Vec<GrabbedAsset>, Box<dyn std::error::Error + Send + Sync>> {
        let mut assets = Vec::new();
        let mut visited_urls = HashSet::new();
        
        let mut queue: VecDeque<(String, u32)> = VecDeque::new();
        queue.push_back((start_url.to_string(), 0));
        
        let base_host = url::Url::parse(start_url).ok().and_then(|u| u.host_str().map(|s| s.to_string())).unwrap_or_default();
        let mut pages_crawled = 0;
        let max_pages = 50; // Hard cap to prevent runaway memory/rate-limit blocking

        while let Some((current_url, depth)) = queue.pop_front() {
            if visited_urls.contains(&current_url) { continue; }
            visited_urls.insert(current_url.clone());
            
            log::info!("[Grabber] Crawling depth {} - {}", depth, current_url);
            
            let mut rb = self.client.get(&current_url);
            let (auth, cookies) = self.auth_manager.get_headers_for_url(&current_url).await;
            if let Some(a) = auth { rb = rb.header("Authorization", a); }
            if let Some(c) = cookies { rb = rb.header(reqwest::header::COOKIE, c); }

            let res = match rb.send().await {
                Ok(r) => r,
                Err(_) => continue,
            };
            
            let mime = res.headers()
                .get(reqwest::header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_string();

            if !mime.contains("text/html") {
                let size = res.headers()
                    .get(reqwest::header::CONTENT_LENGTH)
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(0);
                let filename = current_url.split('/').last().unwrap_or("unknown").to_string();
                let category = Self::get_category(&mime, &filename);
                
                assets.push(GrabbedAsset {
                    url: current_url,
                    filename,
                    mime,
                    size,
                    category,
                });
                continue;
            }

            pages_crawled += 1;
            if let Ok(html) = res.text().await {
                let links = self.extract_links(&html, &current_url);
                let mut futures = Vec::new();

                for link in links {
                    if visited_urls.contains(&link) { continue; }
                    
                    let client = self.client.clone();
                    let auth_manager = self.auth_manager.clone();
                    let link_clone = link.clone();
                    
                    futures.push(tokio::spawn(async move {
                        (link_clone.clone(), Self::resolve_asset(&client, &auth_manager, link_clone).await)
                    }));
                }

                for handle in futures {
                    if let Ok((link_url, Ok(Some(res_type)))) = handle.await {
                        match res_type {
                            ResolveResult::Asset(asset) => {
                                if !assets.iter().any(|a| a.url == asset.url) {
                                    assets.push(asset);
                                }
                            },
                            ResolveResult::Html => {
                                if depth < max_depth && pages_crawled < max_pages {
                                    let is_same_domain = url::Url::parse(&link_url)
                                        .ok()
                                        .and_then(|u| u.host_str().map(|h| h.to_string()))
                                        .map_or(false, |h| h == base_host || h.ends_with(&format!(".{}", base_host)));
                                    
                                    if !strict_domain || is_same_domain {
                                        queue.push_back((link_url, depth + 1));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if pages_crawled >= max_pages {
                log::warn!("[Grabber] Max page limit ({}) reached, aborting further web crawl.", max_pages);
                break;
            }
        }

        Ok(assets)
    }

    /// Performs a lightweight HEAD request to verify file availability and basic metadata.
    pub async fn probe_sniffed_url(client: &reqwest::Client, url: &str) -> Result<Option<GrabbedAsset>, Box<dyn std::error::Error + Send + Sync>> {
        let res = client.head(url)
            .header(reqwest::header::USER_AGENT, "Mdownloader/2.0")
            .timeout(Duration::from_secs(5))
            .send()
            .await?;

        if !res.status().is_success() { return Ok(None); }

        let mime = res.headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();

        let size = res.headers()
            .get(reqwest::header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        let filename = url.split('/').last().unwrap_or("unknown").to_string();
        let category = Self::get_category(&mime, &filename);

        Ok(Some(GrabbedAsset {
            url: url.to_string(),
            filename,
            mime,
            size,
            category,
        }))
    }

    fn extract_links(&self, html: &str, base_url: &str) -> Vec<String> {
        let re = Regex::new(r#"href=["'](https?://[^"']+|/[^"']+|[^"']+)["']"#).unwrap();
        let mut links = Vec::new();
        let base = url::Url::parse(base_url).ok();

        for cap in re.captures_iter(html) {
            let found = &cap[1];
            if let Some(base) = &base {
                if let Ok(abs) = base.join(found) {
                    links.push(abs.to_string());
                }
            } else if found.starts_with("http") {
                links.push(found.to_string());
            }
        }
        links
    }

    async fn resolve_asset(
        client: &reqwest::Client, 
        auth_manager: &Arc<AuthManager>, 
        url: String
    ) -> Result<Option<ResolveResult>, Box<dyn std::error::Error + Send + Sync>> {
        let mut rb = client.head(&url);
        
        let (auth, cookies) = auth_manager.get_headers_for_url(&url).await;
        if let Some(a) = auth { rb = rb.header("Authorization", a); }
        if let Some(c) = cookies { rb = rb.header(reqwest::header::COOKIE, c); }

        let res = rb.send().await?;
        if !res.status().is_success() { return Ok(None); }

        let mime = res.headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();

        let size = res.headers()
            .get(reqwest::header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        let filename = url.split('/').last().unwrap_or("unknown").to_string();
        let category = Self::get_category(&mime, &filename);

        // Intercept HTML pages
        if mime.contains("text/html") {
            return Ok(Some(ResolveResult::Html));
        }

        Ok(Some(ResolveResult::Asset(GrabbedAsset {
            url,
            filename,
            mime,
            size,
            category,
        })))
    }

    fn get_category(mime: &str, filename: &str) -> String {
        let ext = filename.split('.').last().unwrap_or("").to_lowercase();
        if mime.contains("video") || ["mp4", "mkv", "avi"].contains(&ext.as_str()) { return "Video".to_string(); }
        if mime.contains("image") || ["jpg", "png", "gif"].contains(&ext.as_str()) { return "Images".to_string(); }
        if mime.contains("audio") || ["mp3", "wav", "flac"].contains(&ext.as_str()) { return "Music".to_string(); }
        if ["exe", "msi", "dmg"].contains(&ext.as_str()) { return "Programs".to_string(); }
        if ["zip", "rar", "7z", "iso"].contains(&ext.as_str()) { return "Compressed".to_string(); }
        "General".to_string()
    }
}
