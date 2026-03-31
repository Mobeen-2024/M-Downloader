use rusqlite::{params, Connection};
use std::path::PathBuf;
use chrono::{Utc, Duration as ChronoDuration};
use std::sync::Arc;
use tokio::sync::Mutex as AsyncMutex;

/// Manages persistent data usage tracking to enforce user quotas.
/// Uses SQLite to store a historical log of downloaded bytes.
pub struct UsageTracker {
    db_path: PathBuf,
    // Use an async mutex for DB operations to avoid blocking workers.
    conn: Arc<AsyncMutex<Connection>>,
}

impl UsageTracker {
    pub fn new(app_dir: PathBuf) -> Result<Self, String> {
        let db_path = app_dir.join("usage.db");
        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Could not open usage database: {}", e))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS data_usage (
                timestamp INTEGER NOT NULL,
                bytes INTEGER NOT NULL
            )",
            [],
        ).map_err(|e| format!("Failed to create usage table: {}", e))?;

        // Index for faster rolling window queries.
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_timestamp ON data_usage (timestamp)",
            [],
        ).map_err(|e| format!("Failed to create index: {}", e))?;

        Ok(Self {
            db_path,
            conn: Arc::new(AsyncMutex::new(conn)),
        })
    }

    /// Logs `n` bytes was successfully downloaded and written to disk.
    pub async fn log_bytes(&self, n: u64) {
        let conn = self.conn.lock().await;
        let ts = Utc::now().timestamp();
        let _ = conn.execute(
            "INSERT INTO data_usage (timestamp, bytes) VALUES (?, ?)",
            params![ts, n as i64],
        );
    }

    /// Calculates total data usage (in MB) within the last `hours`.
    pub async fn get_usage_mb(&self, hours: i64) -> f64 {
        let conn = self.conn.lock().await;
        let threshold = (Utc::now() - ChronoDuration::hours(hours)).timestamp();
        
        let result: Result<i64, _> = conn.query_row(
            "SELECT COALESCE(SUM(bytes), 0) FROM data_usage WHERE timestamp > ?",
            params![threshold],
            |row| row.get(0),
        );

        match result {
            Ok(bytes) => (bytes as f64) / (1024.0 * 1024.0),
            Err(_) => 0.0,
        }
    }

    /// Prunes old records to prevent database bloat.
    /// Recommended to keep 30 days of data for historical reporting.
    pub async fn prune_old_data(&self) {
        let conn = self.conn.lock().await;
        let thirty_days_ago = (Utc::now() - ChronoDuration::days(30)).timestamp();
        let _ = conn.execute(
            "DELETE FROM data_usage WHERE timestamp < ?",
            params![thirty_days_ago],
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_quota_tracking() {
        let dir = tempdir().unwrap();
        let tracker = UsageTracker::new(dir.path().to_path_buf()).unwrap();
        
        tracker.log_bytes(1024 * 1024).await; // 1MB
        tracker.log_bytes(1024 * 1024).await; // 1MB
        
        let usage = tracker.get_usage_mb(1).await;
        assert_eq!(usage, 2.0);
    }
}
