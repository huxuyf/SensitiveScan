use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 敏感信息类型
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
            SensitiveType::PhoneNumber => write!(f, "手机号码"),
            SensitiveType::IdCard => write!(f, "身份证号"),
            SensitiveType::Name => write!(f, "姓名"),
            SensitiveType::Address => write!(f, "地址"),
        }
    }
}

/// 扫描结果记录
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

/// 扫描任务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    pub scan_paths: Vec<String>,
    pub exclude_paths: Vec<String>,
    pub max_file_size: u64, // 采用字节为单位
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
            max_file_size: 100 * 1024 * 1024, // 默认 100MB
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

/// 扫描任务状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
pub enum ScanState {
    Idle,
    Running,
    Paused,
    Stopped,
    Completed,
}

/// 扫描统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanStats {
    pub total_files_scanned: u64,
    pub total_results_found: u64,
    pub scan_duration_secs: u64,
    pub scan_speed: f64, // 每秒扫描文件数
    pub results_by_type: std::collections::HashMap<String, u64>,
}

/// 导出格式选项
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
pub enum ExportFormat {
    Excel,
    Pdf,
    Csv,
}

/// 扫描进度事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ScanProgress {
    pub current_file: String,
    pub files_scanned: u64,
    pub results_found: u64,
    pub progress_percentage: f64,
    pub elapsed_seconds: u64,
    pub estimated_remaining_seconds: u64,
    pub scan_speed: f64,
}

/// 按文件汇总的扫描结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedResult {
    pub file_path: String,
    pub file_name: String,
    pub file_size: u64,
    pub file_type: String,
    pub sensitive_types: String, // 例如, "PhoneNumber+Name"
    pub count: u32,
}
