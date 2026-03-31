use rusqlite::{Connection, Result as SqlResult, params};
use chrono::Utc;
use std::path::PathBuf;
use crate::models::{ScanResult, SensitiveType, WhitelistEntry, ScanStats};

pub struct Database {
    conn: Connection,
}

impl Database {
    /// Initialize database connection
    pub fn new() -> SqlResult<Self> {
        let db_path = Self::get_db_path();
        std::fs::create_dir_all(db_path.parent().unwrap()).ok();
        
        let conn = Connection::open(&db_path)?;
        conn.execute_batch("PRAGMA journal_mode = WAL;")?;
        
        let db = Database { conn };
        db.init_schema()?;
        Ok(db)
    }
    
    /// Get database file path based on platform
    fn get_db_path() -> PathBuf {
        let config_dir = if cfg!(target_os = "windows") {
            dirs::config_dir()
                .map(|p| p.join("SensitiveScanner"))
                .unwrap_or_else(|| PathBuf::from("./config"))
        } else if cfg!(target_os = "macos") {
            dirs::home_dir()
                .map(|p| p.join("Library/Application Support/SensitiveScanner"))
                .unwrap_or_else(|| PathBuf::from("./config"))
        } else {
            dirs::config_dir()
                .map(|p| p.join("sensitive-scanner"))
                .unwrap_or_else(|| PathBuf::from("./config"))
        };
        
        config_dir.join("results.db")
    }
    
    /// Initialize database schema
    fn init_schema(&self) -> SqlResult<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS scan_results (
                id TEXT PRIMARY KEY,
                file_path TEXT NOT NULL,
                sheet_name TEXT,
                row INTEGER NOT NULL,
                column INTEGER NOT NULL,
                sensitive_type TEXT NOT NULL,
                content TEXT NOT NULL,
                masked_content TEXT NOT NULL,
                found_at DATETIME NOT NULL
            );
            
            CREATE TABLE IF NOT EXISTS scan_history (
                id TEXT PRIMARY KEY,
                scan_paths TEXT NOT NULL,
                config TEXT NOT NULL,
                stats TEXT NOT NULL,
                created_at DATETIME NOT NULL,
                completed_at DATETIME
            );
            
            CREATE TABLE IF NOT EXISTS whitelist (
                id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                sensitive_type TEXT NOT NULL,
                description TEXT,
                created_at DATETIME NOT NULL
            );
            
            CREATE INDEX IF NOT EXISTS idx_scan_results_file ON scan_results(file_path);
            CREATE INDEX IF NOT EXISTS idx_scan_results_type ON scan_results(sensitive_type);
            CREATE INDEX IF NOT EXISTS idx_scan_results_found_at ON scan_results(found_at);
            CREATE INDEX IF NOT EXISTS idx_scan_history_created_at ON scan_history(created_at);
            "
        )?;
        Ok(())
    }
    
    /// Insert scan result
    pub fn insert_scan_result(&self, result: &ScanResult) -> SqlResult<()> {
        self.conn.execute(
            "INSERT INTO scan_results 
             (id, file_path, sheet_name, row, column, sensitive_type, content, masked_content, found_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                result.id,
                result.file_path,
                result.sheet_name,
                result.row,
                result.column,
                format!("{:?}", result.sensitive_type),
                result.content,
                result.masked_content,
                result.found_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }
    
    /// Get scan results with optional filtering
    pub fn get_scan_results(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
        file_path_filter: Option<&str>,
        sensitive_type_filter: Option<&str>,
    ) -> SqlResult<Vec<ScanResult>> {
        let limit = limit.unwrap_or(1000);
        let offset = offset.unwrap_or(0);
        
        let mut query = "SELECT id, file_path, sheet_name, row, column, sensitive_type, content, masked_content, found_at FROM scan_results WHERE 1=1".to_string();
        
        if let Some(file_path) = file_path_filter {
            query.push_str(&format!(" AND file_path LIKE '%{}%'", file_path.replace("'", "''")));
        }
        
        if let Some(sensitive_type) = sensitive_type_filter {
            query.push_str(&format!(" AND sensitive_type = '{}'", sensitive_type));
        }
        
        query.push_str(&format!(" ORDER BY found_at DESC LIMIT {} OFFSET {}", limit, offset));
        
        let mut stmt = self.conn.prepare(&query)?;
        let results = stmt.query_map([], |row| {
            Ok(ScanResult {
                id: row.get(0)?,
                file_path: row.get(1)?,
                sheet_name: row.get(2)?,
                row: row.get(3)?,
                column: row.get(4)?,
                sensitive_type: parse_sensitive_type(&row.get::<_, String>(5)?),
                content: row.get(6)?,
                masked_content: row.get(7)?,
                found_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?;
        
        let mut vec = Vec::new();
        for result in results {
            vec.push(result?);
        }
        Ok(vec)
    }
    
    /// Count scan results
    pub fn count_scan_results(&self) -> SqlResult<u64> {
        let mut stmt = self.conn.prepare("SELECT COUNT(*) FROM scan_results")?;
        stmt.query_row([], |row| row.get(0))
    }
    
    /// Delete scan results
    #[allow(dead_code)]
    pub fn delete_scan_results(&self) -> SqlResult<()> {
        self.conn.execute("DELETE FROM scan_results", [])?;
        Ok(())
    }

    /// Insert scan history
    #[allow(dead_code)]
    pub fn insert_scan_history(&self, history: &crate::models::ScanHistory) -> SqlResult<()> {
        let config_json = serde_json::to_string(&history.config)
            .unwrap_or_default();
        let stats_json = serde_json::to_string(&history.stats)
            .unwrap_or_default();
        let paths_json = serde_json::to_string(&history.scan_paths)
            .unwrap_or_default();
        
        self.conn.execute(
            "INSERT INTO scan_history (id, scan_paths, config, stats, created_at, completed_at)
             VALUES (?, ?, ?, ?, ?, ?)",
            params![
                history.id,
                paths_json,
                config_json,
                stats_json,
                history.created_at.to_rfc3339(),
                history.completed_at.map(|dt| dt.to_rfc3339()),
            ],
        )?;
        Ok(())
    }
    
    /// Get scan history
    pub fn get_scan_history(&self, limit: Option<i64>) -> SqlResult<Vec<crate::models::ScanHistory>> {
        let limit = limit.unwrap_or(100);
        let mut stmt = self.conn.prepare(
            "SELECT id, scan_paths, config, stats, created_at, completed_at FROM scan_history 
             ORDER BY created_at DESC LIMIT ?"
        )?;
        
        let results = stmt.query_map(params![limit], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<String>>(5)?,
            ))
        })?;
        
        let mut vec = Vec::new();
        for result in results {
            if let Ok((id, paths_json, config_json, stats_json, created_at, completed_at)) = result {
                let history = crate::models::ScanHistory {
                    id,
                    scan_paths: serde_json::from_str(&paths_json).unwrap_or_default(),
                    config: serde_json::from_str(&config_json).unwrap_or_default(),
                    stats: serde_json::from_str(&stats_json).unwrap_or_default(),
                    created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    completed_at: completed_at.and_then(|dt_str| {
                        chrono::DateTime::parse_from_rfc3339(&dt_str)
                            .ok()
                            .map(|dt| dt.with_timezone(&Utc))
                    }),
                };
                vec.push(history);
            }
        }
        Ok(vec)
    }
    
    /// Delete scan history
    pub fn delete_scan_history(&self, history_id: &str) -> SqlResult<()> {
        self.conn.execute(
            "DELETE FROM scan_history WHERE id = ?",
            params![history_id],
        )?;
        Ok(())
    }
    
    /// Add whitelist entry
    pub fn add_whitelist(&self, entry: &WhitelistEntry) -> SqlResult<()> {
        self.conn.execute(
            "INSERT INTO whitelist (id, content, sensitive_type, description, created_at)
             VALUES (?, ?, ?, ?, ?)",
            params![
                entry.id,
                entry.content,
                format!("{:?}", entry.sensitive_type),
                entry.description,
                entry.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }
    
    /// Get whitelist
    pub fn get_whitelist(&self) -> SqlResult<Vec<WhitelistEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content, sensitive_type, description, created_at FROM whitelist 
             ORDER BY created_at DESC"
        )?;
        
        let results = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
            ))
        })?;
        
        let mut vec = Vec::new();
        for result in results {
            if let Ok((id, content, sensitive_type, description, created_at)) = result {
                let entry = WhitelistEntry {
                    id,
                    content,
                    sensitive_type: parse_sensitive_type(&sensitive_type),
                    description,
                    created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                };
                vec.push(entry);
            }
        }
        Ok(vec)
    }
    
    /// Delete whitelist entry
    pub fn delete_whitelist(&self, entry_id: &str) -> SqlResult<()> {
        self.conn.execute(
            "DELETE FROM whitelist WHERE id = ?",
            params![entry_id],
        )?;
        Ok(())
    }
    
    /// Clear old data
    #[allow(dead_code)]
    pub fn cleanup_old_data(&self, days: i64) -> SqlResult<()> {
        let cutoff_date = Utc::now() - chrono::Duration::days(days);
        self.conn.execute(
            "DELETE FROM scan_results WHERE found_at < ?",
            params![cutoff_date.to_rfc3339()],
        )?;
        Ok(())
    }
}

fn parse_sensitive_type(s: &str) -> SensitiveType {
    match s {
        "PhoneNumber" => SensitiveType::PhoneNumber,
        "IdCard" => SensitiveType::IdCard,
        "Name" => SensitiveType::Name,
        "Address" => SensitiveType::Address,
        _ => SensitiveType::PhoneNumber,
    }
}

impl Default for ScanStats {
    fn default() -> Self {
        Self {
            total_files_scanned: 0,
            total_results_found: 0,
            scan_duration_secs: 0,
            scan_speed: 0.0,
            results_by_type: std::collections::HashMap::new(),
        }
    }
}
