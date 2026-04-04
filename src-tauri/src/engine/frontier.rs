use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex as AsyncMutex;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FrontierItem {
    pub url: String,
    pub depth: u32,
    pub status: String, // 'pending', 'crawling', 'completed', 'failed'
    pub discovered_at: u64,
}

pub struct DiscoveryFrontier {
    _db_path: PathBuf,
    conn: Arc<AsyncMutex<Connection>>,
}

impl DiscoveryFrontier {
    pub fn new(app_dir: PathBuf) -> Result<Self, String> {
        let db_path = app_dir.join("frontier.db");
        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Could not open frontier database: {}", e))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS discovery_frontier (
                url TEXT PRIMARY KEY,
                depth INTEGER NOT NULL,
                status TEXT NOT NULL,
                discovered_at INTEGER NOT NULL
            )",
            [],
        ).map_err(|e| format!("Failed to create frontier table: {}", e))?;

        // Index for depth-first or breadth-first retrieval
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_status_depth ON discovery_frontier (status, depth)",
            [],
        ).map_err(|e| format!("Failed to create index: {}", e))?;

        Ok(Self {
            _db_path: db_path,
            conn: Arc::new(AsyncMutex::new(conn)),
        })
    }

    pub async fn add_url(&self, url: &str, depth: u32) -> Result<(), String> {
        let conn = self.conn.lock().await;
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        let _ = conn.execute(
            "INSERT OR IGNORE INTO discovery_frontier (url, depth, status, discovered_at) VALUES (?, ?, 'pending', ?)",
            params![url, depth, ts],
        );
        Ok(())
    }

    pub async fn mark_status(&self, url: &str, status: &str) -> Result<(), String> {
        let conn = self.conn.lock().await;
        conn.execute(
            "UPDATE discovery_frontier SET status = ? WHERE url = ?",
            params![status, url],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn get_pending(&self, max_count: usize) -> Result<Vec<FrontierItem>, String> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT url, depth, status, discovered_at FROM discovery_frontier WHERE status = 'pending' ORDER BY depth ASC, discovered_at ASC LIMIT ?"
        ).map_err(|e| e.to_string())?;

        let items = stmt.query_map(params![max_count as i64], |row| {
            Ok(FrontierItem {
                url: row.get(0)?,
                depth: row.get(1)?,
                status: row.get(2)?,
                discovered_at: row.get(3)?,
            })
        }).map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

        Ok(items)
    }

    pub async fn clear_all(&self) -> Result<(), String> {
        let conn = self.conn.lock().await;
        conn.execute("DELETE FROM discovery_frontier", []).map_err(|e| e.to_string())?;
        Ok(())
    }
}
