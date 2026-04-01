use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use regex::Regex;
use crate::engine::auth::AuthManager;

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

    pub async fn grab_page(&self, url: &str) -> Result<Vec<GrabbedAsset>, Box<dyn std::error::Error + Send + Sync>> {
        let mut rb = self.client.get(url);
        
        // Inject auth if available
        let (auth, cookies) = self.auth_manager.get_headers_for_url(url).await;
        if let Some(a) = auth { rb = rb.header("Authorization", a); }
        if let Some(c) = cookies { rb = rb.header(reqwest::header::COOKIE, c); }

        let html = rb.send().await?.text().await?;
        let links = self.extract_links(&html, url);
        
        let mut assets = Vec::new();
        let mut futures = Vec::new();

        for link in links {
            let client = self.client.clone();
            let auth_manager = self.auth_manager.clone();
            futures.push(tokio::spawn(async move {
                Self::resolve_asset(&client, &auth_manager, link).await
            }));
        }

        for handle in futures {
            if let Ok(Ok(Some(asset))) = handle.await {
                assets.push(asset);
            }
        }

        Ok(assets)
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
    ) -> Result<Option<GrabbedAsset>, Box<dyn std::error::Error + Send + Sync>> {
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

        // We only care about actual files, not pages
        if mime.contains("text/html") {
            return Ok(None);
        }

        Ok(Some(GrabbedAsset {
            url,
            filename,
            mime,
            size,
            category,
        }))
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
