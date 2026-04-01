use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::Mutex;
use std::path::PathBuf;
use base64::Engine;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SiteCredential {
    pub domain: String,
    pub username: Option<String>,
    #[serde(skip_serializing, skip_deserializing)]
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
                // We use a temporary struct for migration to catch legacy plaintexts
                #[derive(Deserialize)]
                struct LegacyCred {
                    domain: String,
                    username: Option<String>,
                    password: Option<String>,
                    cookies: Option<String>,
                }

                if let Ok(data) = serde_json::from_str::<Vec<LegacyCred>>(&content) {
                    let mut needs_migration_save = false;
                    for site in data {
                        let mut final_site = SiteCredential {
                            domain: site.domain.clone(),
                            username: site.username.clone(),
                            password: None,
                            cookies: site.cookies.clone(),
                        };

                        // Migration: If password was found in JSON, move it to keyring
                        if let Some(plain_pw) = site.password {
                            log::info!("[Security] Migrating plaintext password for: {}", site.domain);
                            let entry = keyring::Entry::new("com.mobeen.mdownloader.sites", &format!("{}|{}", site.domain, site.username.as_deref().unwrap_or("guest"))).unwrap();
                            let _ = entry.set_password(&plain_pw);
                            needs_migration_save = true;
                        }

                        sites.insert(site.domain.clone(), final_site);
                    }

                    if needs_migration_save {
                        let sanitized: Vec<SiteCredential> = sites.values().cloned().collect();
                        if let Ok(sc) = serde_json::to_string_pretty(&sanitized) {
                            let _ = std::fs::write(&auth_path, sc);
                        }
                    }
                }
            }
        }

        Self {
            sites: Mutex::new(sites),
            auth_path,
        }
    }

    pub async fn add_site(&self, mut cred: SiteCredential) {
        if let Some(pw) = cred.password.take() {
            log::info!("[Security] Storing secure credential for: {}", cred.domain);
            let entry = keyring::Entry::new("com.mobeen.mdownloader.sites", &format!("{}|{}", cred.domain, cred.username.as_deref().unwrap_or("guest"))).unwrap();
            let _ = entry.set_password(&pw);
        }

        let mut sites = self.sites.lock().await;
        sites.insert(cred.domain.clone(), cred);
        self.save_internal(&sites).await;
    }

    pub async fn remove_site(&self, domain: &str) {
        let mut sites = self.sites.lock().await;
        if let Some(cred) = sites.remove(domain) {
            let entry = keyring::Entry::new("com.mobeen.mdownloader.sites", &format!("{}|{}", cred.domain, cred.username.as_deref().unwrap_or("guest"))).unwrap();
            let _ = entry.delete_password();
        }
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
            let entry = keyring::Entry::new("com.mobeen.mdownloader.sites", &format!("{}|{}", cred.domain, cred.username.as_deref().unwrap_or("guest"))).unwrap();
            let keyring_pw = entry.get_password().ok();

            let auth_header = if let (Some(u), Some(p)) = (&cred.username, &keyring_pw) {
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
