use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::Mutex;
use std::path::PathBuf;
use base64::Engine;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SiteCredential {
    pub domain: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub cookies: Option<String>,
}

pub struct AuthManager {
    sites: Mutex<HashMap<String, SiteCredential>>,
    auth_path: PathBuf,
}

impl AuthManager {
    pub fn new(app_data_dir: PathBuf) -> Self {
        let auth_path = app_data_dir.join("auth.json");
        let mut sites = HashMap::new();

        if auth_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&auth_path) {
                if let Ok(data) = serde_json::from_str::<Vec<SiteCredential>>(&content) {
                    for site in data {
                        sites.insert(site.domain.clone(), site);
                    }
                }
            }
        }

        Self {
            sites: Mutex::new(sites),
            auth_path,
        }
    }

    pub async fn add_site(&self, cred: SiteCredential) {
        let mut sites = self.sites.lock().await;
        sites.insert(cred.domain.clone(), cred);
        self.save_internal(&sites).await;
    }

    pub async fn remove_site(&self, domain: &str) {
        let mut sites = self.sites.lock().await;
        sites.remove(domain);
        self.save_internal(&sites).await;
    }

    pub async fn get_all_sites(&self) -> Vec<SiteCredential> {
        let sites = self.sites.lock().await;
        sites.values().cloned().collect()
    }

    pub async fn get_headers_for_url(&self, url: &str) -> (Option<String>, Option<String>) {
        let host = url::Url::parse(url)
            .ok()
            .and_then(|u| u.host_str().map(|h| h.to_string()))
            .unwrap_or_default();

        let sites = self.sites.lock().await;
        if let Some(cred) = sites.get(&host) {
            let auth_header = if let (Some(u), Some(p)) = (&cred.username, &cred.password) {
                let auth_bytes = format!("{}:{}", u, p);
                Some(format!("Basic {}", base64::engine::general_purpose::STANDARD.encode(auth_bytes)))
            } else {
                None
            };
            (auth_header, cred.cookies.clone())
        } else {
            (None, None)
        }
    }

    async fn save_internal(&self, sites: &HashMap<String, SiteCredential>) {
        let data: Vec<SiteCredential> = sites.values().cloned().collect();
        if let Ok(content) = serde_json::to_string_pretty(&data) {
            let _ = tokio::fs::write(&self.auth_path, content).await;
        }
    }
}
