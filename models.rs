use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Sensitive information types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum SensitiveType {
    PhoneNumber,
    IdCard,
    Name,
    Address,
}

impl std::fmt::Display for SensitiveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SensitiveType::PhoneNumber => write!(f, "Phone Number"),
            SensitiveType::IdCard => write!(f, "ID Card"),
            SensitiveType::Name => write!(f, "Name"),
            SensitiveType::Address => write!(f, "Address"),
        }
    }
}

/// Scan result record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub id: String,
    pub file_path: String,
    pub sheet_name: Option<String>,
    pub row: u32,
    pub column: u32,
    pub sensitive_type: SensitiveType,
    pub content: String,
    pub masked_content: String,
    pub found_at: DateTime<Utc>,
}

/// Scan task configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    pub scan_paths: Vec<String>,
    pub exclude_paths: Vec<String>,
    pub max_file_size: u64, // in bytes
    pub file_types: Vec<String>,
    pub sensitive_types: Vec<SensitiveType>,
    pub min_records_threshold: u32,
    pub thread_count: usize,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            scan_paths: vec![],
            exclude_paths: vec![],
            max_file_size: 100 * 1024 * 1024, // 100MB default
            file_types: vec![
                ".xlsx".to_string(),
                ".xls".to_string(),
                ".csv".to_string(),
                ".txt".to_string(),
            ],
            sensitive_types: vec![
                SensitiveType::PhoneNumber,
                SensitiveType::IdCard,
                SensitiveType::Name,
                SensitiveType::Address,
            ],
            min_records_threshold: 50,
            thread_count: num_cpus::get().saturating_sub(1).max(1),
        }
    }
}

/// Scan task state
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ScanState {
    Idle,
    Running,
    Paused,
    Stopped,
    Completed,
}

/// Scan statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanStats {
    pub total_files_scanned: u64,
    pub total_results_found: u64,
    pub scan_duration_secs: u64,
    pub scan_speed: f64, // files per second
    pub results_by_type: std::collections::HashMap<String, u64>,
}

/// Scan history record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanHistory {
    pub id: String,
    pub scan_paths: Vec<String>,
    pub config: ScanConfig,
    pub stats: ScanStats,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Whitelist entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistEntry {
    pub id: String,
    pub content: String,
    pub sensitive_type: SensitiveType,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Export format options
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Excel,
    Pdf,
    Csv,
}

/// Scan progress event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgress {
    pub current_file: String,
    pub files_scanned: u64,
    pub results_found: u64,
    pub progress_percentage: f64,
    pub elapsed_seconds: u64,
    pub estimated_remaining_seconds: u64,
    pub scan_speed: f64,
}
