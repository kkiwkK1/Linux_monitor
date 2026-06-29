use super::{HistoryRecord, TimeRange};
use rusqlite::{Connection, params};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

/// SQLite-backed history store with WAL mode and auto-cleanup
pub struct HistoryStore {
    conn: Mutex<Connection>,
    retention_days: u32,
}

impl HistoryStore {
    /// Open or create the history database
    pub fn open() -> anyhow::Result<Self> {
        let db_path = Self::db_path();
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)?;

        // Enable WAL mode for better concurrent performance
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;

        // Create tables
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS snapshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                cpu_percent REAL DEFAULT 0,
                mem_percent REAL DEFAULT 0,
                mem_used_kb INTEGER DEFAULT 0,
                mem_total_kb INTEGER DEFAULT 0,
                net_rx_speed INTEGER DEFAULT 0,
                net_tx_speed INTEGER DEFAULT 0,
                net_iface TEXT DEFAULT '',
                disk_used_percent REAL DEFAULT 0,
                disk_mount TEXT DEFAULT '',
                gpu_temp REAL DEFAULT 0,
                cpu_temp REAL DEFAULT 0
            );
            CREATE INDEX IF NOT EXISTS idx_snapshots_ts ON snapshots(timestamp);
            ",
        )?;

        // Set secure permissions
        Self::set_secure_permissions(&db_path)?;

        let store = Self {
            conn: Mutex::new(conn),
            retention_days: 7,
        };

        // Cleanup old data on startup
        store.cleanup_old_data()?;

        log::info!("History store opened at {:?}", db_path);
        Ok(store)
    }

    /// Insert a new snapshot record
    pub fn insert(&self, record: &HistoryRecord) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO snapshots (timestamp, cpu_percent, mem_percent, mem_used_kb, mem_total_kb,
             net_rx_speed, net_tx_speed, net_iface, disk_used_percent, disk_mount, gpu_temp, cpu_temp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                record.timestamp as i64,
                record.cpu_percent,
                record.mem_percent,
                record.mem_used_kb,
                record.mem_total_kb,
                record.net_rx_speed,
                record.net_tx_speed,
                record.net_iface,
                record.disk_used_percent,
                record.disk_mount,
                record.gpu_temp,
                record.cpu_temp,
            ],
        )?;
        Ok(())
    }

    /// Query records for a time range
    pub fn query(&self, range: TimeRange) -> anyhow::Result<Vec<HistoryRecord>> {
        let conn = self.conn.lock().unwrap();

        let cutoff = match range {
            TimeRange::Last5Min => Some(current_timestamp() - 300),
            TimeRange::Last30Min => Some(current_timestamp() - 1800),
            TimeRange::Last1Hour => Some(current_timestamp() - 3600),
            TimeRange::Last6Hours => Some(current_timestamp() - 21600),
            TimeRange::Last24Hours => Some(current_timestamp() - 86400),
            TimeRange::All => None,
        };

        let query = if let Some(ts) = cutoff {
            "SELECT timestamp, cpu_percent, mem_percent, mem_used_kb, mem_total_kb,
                    net_rx_speed, net_tx_speed, net_iface, disk_used_percent, disk_mount,
                    gpu_temp, cpu_temp
             FROM snapshots WHERE timestamp >= ?1 ORDER BY timestamp ASC"
        } else {
            "SELECT timestamp, cpu_percent, mem_percent, mem_used_kb, mem_total_kb,
                    net_rx_speed, net_tx_speed, net_iface, disk_used_percent, disk_mount,
                    gpu_temp, cpu_temp
             FROM snapshots ORDER BY timestamp ASC"
        };

        let mut stmt = conn.prepare(query)?;

        let rows = if let Some(ts) = cutoff {
            stmt.query_map(params![ts as i64], |row| {
                Ok(HistoryRecord {
                    timestamp: row.get::<_, i64>(0)? as u64,
                    cpu_percent: row.get(1)?,
                    mem_percent: row.get(2)?,
                    mem_used_kb: row.get(3)?,
                    mem_total_kb: row.get(4)?,
                    net_rx_speed: row.get::<_, i64>(5)? as u64,
                    net_tx_speed: row.get::<_, i64>(6)? as u64,
                    net_iface: row.get(7)?,
                    disk_used_percent: row.get(8)?,
                    disk_mount: row.get(9)?,
                    gpu_temp: row.get(10)?,
                    cpu_temp: row.get(11)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect::<Vec<_>>()
        } else {
            stmt.query_map([], |row| {
                Ok(HistoryRecord {
                    timestamp: row.get::<_, i64>(0)? as u64,
                    cpu_percent: row.get(1)?,
                    mem_percent: row.get(2)?,
                    mem_used_kb: row.get(3)?,
                    mem_total_kb: row.get(4)?,
                    net_rx_speed: row.get::<_, i64>(5)? as u64,
                    net_tx_speed: row.get::<_, i64>(6)? as u64,
                    net_iface: row.get(7)?,
                    disk_used_percent: row.get(8)?,
                    disk_mount: row.get(9)?,
                    gpu_temp: row.get(10)?,
                    cpu_temp: row.get(11)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect::<Vec<_>>()
        };

        Ok(rows)
    }

    /// Export data to CSV
    pub fn export_csv(&self, path: &str, range: TimeRange) -> anyhow::Result<usize> {
        let records = self.query(range)?;
        let mut content = String::from("timestamp,cpu%,mem%,mem_used_kb,mem_total_kb,net_rx,net_tx,net_iface,disk%,disk_mount,gpu_temp,cpu_temp\n");

        for r in &records {
            content.push_str(&format!(
                "{},{:.1},{:.1},{},{},{},{},{},{:.1},{},{:.1},{:.1}\n",
                r.timestamp,
                r.cpu_percent,
                r.mem_percent,
                r.mem_used_kb,
                r.mem_total_kb,
                r.net_rx_speed,
                r.net_tx_speed,
                r.net_iface,
                r.disk_used_percent,
                r.disk_mount,
                r.gpu_temp,
                r.cpu_temp,
            ));
        }

        std::fs::write(path, &content)?;
        let count = records.len();
        log::info!("Exported {} records to {}", count, path);
        Ok(count)
    }

    /// Get the total record count and database size
    pub fn stats(&self) -> anyhow::Result<(u64, u64)> {
        let conn = self.conn.lock().unwrap();
        let count: u64 = conn
            .query_row("SELECT COUNT(*) FROM snapshots", [], |r| r.get(0))?;

        let db_path = Self::db_path();
        let size = std::fs::metadata(&db_path)
            .map(|m| m.len())
            .unwrap_or(0);

        Ok((count, size))
    }

    /// Remove data older than retention period
    fn cleanup_old_data(&self) -> anyhow::Result<()> {
        let cutoff = current_timestamp() - (self.retention_days as u64 * 86400);
        let conn = self.conn.lock().unwrap();
        let deleted = conn.execute(
            "DELETE FROM snapshots WHERE timestamp < ?1",
            params![cutoff as i64],
        )?;
        if deleted > 0 {
            log::info!("Cleaned up {} old records (>{} days)", deleted, self.retention_days);
        }
        Ok(())
    }

    /// Limit database to a maximum row count
    pub fn enforce_row_limit(&self, max_rows: u64) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        let count: u64 = conn.query_row("SELECT COUNT(*) FROM snapshots", [], |r| r.get(0))?;
        if count > max_rows {
            let to_delete = count - max_rows;
            conn.execute(
                "DELETE FROM snapshots WHERE id IN (SELECT id FROM snapshots ORDER BY timestamp ASC LIMIT ?1)",
                params![to_delete],
            )?;
            log::info!("Enforced row limit: removed {} oldest records", to_delete);
        }
        Ok(())
    }

    fn db_path() -> PathBuf {
        let base = std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
                PathBuf::from(home).join(".local/share")
            });
        base.join("linux-monitor").join("history.db")
    }

    fn set_secure_permissions(path: &PathBuf) -> anyhow::Result<()> {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(path)?.permissions();
        perms.set_mode(0o600);
        std::fs::set_permissions(path, perms)?;
        Ok(())
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
