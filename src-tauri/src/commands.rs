use serde_json::json;
use crate::models::{ScanConfig, SensitiveType, WhitelistEntry};
use crate::db::Database;
use uuid::Uuid;
use chrono::Utc;

/// Start a new scan task
#[tauri::command]
pub async fn start_scan(
    scan_paths: Vec<String>,
    exclude_paths: Vec<String>,
    max_file_size: u64,
    sensitive_types: Vec<String>,
) -> Result<String, String> {
    let config = ScanConfig {
        scan_paths,
        exclude_paths,
        max_file_size,
        file_types: vec![
            ".xlsx".to_string(),
            ".xls".to_string(),
            ".csv".to_string(),
            ".txt".to_string(),
        ],
        sensitive_types: sensitive_types
            .iter()
            .filter_map(|t| match t.as_str() {
                "phonenumber" => Some(SensitiveType::PhoneNumber),
                "idcard" => Some(SensitiveType::IdCard),
                "name" => Some(SensitiveType::Name),
                "address" => Some(SensitiveType::Address),
                _ => None,
            })
            .collect(),
        min_records_threshold: 50,
        thread_count: num_cpus::get().saturating_sub(1).max(1),
    };
    
    // TODO: Implement actual scanning logic
    Ok(json!({
        "task_id": Uuid::new_v4().to_string(),
        "status": "started"
    }).to_string())
}

/// Pause current scan
#[tauri::command]
pub async fn pause_scan() -> Result<String, String> {
    Ok(json!({
        "status": "paused"
    }).to_string())
}

/// Resume paused scan
#[tauri::command]
pub async fn resume_scan() -> Result<String, String> {
    Ok(json!({
        "status": "resumed"
    }).to_string())
}

/// Stop current scan
#[tauri::command]
pub async fn stop_scan() -> Result<String, String> {
    Ok(json!({
        "status": "stopped"
    }).to_string())
}

/// Get scan results
#[tauri::command]
pub async fn get_scan_results(
    limit: Option<i64>,
    offset: Option<i64>,
    file_path_filter: Option<String>,
    sensitive_type_filter: Option<String>,
) -> Result<String, String> {
    let db = Database::new().map_err(|e| e.to_string())?;
    
    let results = db.get_scan_results(
        limit,
        offset,
        file_path_filter.as_deref(),
        sensitive_type_filter.as_deref(),
    ).map_err(|e| e.to_string())?;
    
    Ok(serde_json::to_string(&results).unwrap_or_default())
}

/// Export scan results
#[tauri::command]
pub async fn export_results(
    format: String,
    file_path: String,
) -> Result<String, String> {
    // TODO: Implement export logic
    Ok(json!({
        "status": "exported",
        "file_path": file_path
    }).to_string())
}

/// Get scan history
#[tauri::command]
pub async fn get_history(limit: Option<i64>) -> Result<String, String> {
    let db = Database::new().map_err(|e| e.to_string())?;
    
    let history = db.get_scan_history(limit).map_err(|e| e.to_string())?;
    
    Ok(serde_json::to_string(&history).unwrap_or_default())
}

/// Delete scan history
#[tauri::command]
pub async fn delete_history(history_id: String) -> Result<String, String> {
    let db = Database::new().map_err(|e| e.to_string())?;
    
    db.delete_scan_history(&history_id).map_err(|e| e.to_string())?;
    
    Ok(json!({
        "status": "deleted"
    }).to_string())
}

/// Add whitelist entry
#[tauri::command]
pub async fn add_whitelist(
    content: String,
    sensitive_type: String,
    description: Option<String>,
) -> Result<String, String> {
    let db = Database::new().map_err(|e| e.to_string())?;
    
    let sensitive_type = match sensitive_type.as_str() {
        "phonenumber" => SensitiveType::PhoneNumber,
        "idcard" => SensitiveType::IdCard,
        "name" => SensitiveType::Name,
        "address" => SensitiveType::Address,
        _ => return Err("Invalid sensitive type".to_string()),
    };
    
    let entry = WhitelistEntry {
        id: Uuid::new_v4().to_string(),
        content,
        sensitive_type,
        description,
        created_at: Utc::now(),
    };
    
    db.add_whitelist(&entry).map_err(|e| e.to_string())?;
    
    Ok(serde_json::to_string(&entry).unwrap_or_default())
}

/// Get whitelist
#[tauri::command]
pub async fn get_whitelist() -> Result<String, String> {
    let db = Database::new().map_err(|e| e.to_string())?;
    
    let whitelist = db.get_whitelist().map_err(|e| e.to_string())?;
    
    Ok(serde_json::to_string(&whitelist).unwrap_or_default())
}

/// Delete whitelist entry
#[tauri::command]
pub async fn delete_whitelist(entry_id: String) -> Result<String, String> {
    let db = Database::new().map_err(|e| e.to_string())?;
    
    db.delete_whitelist(&entry_id).map_err(|e| e.to_string())?;
    
    Ok(json!({
        "status": "deleted"
    }).to_string())
}

/// Get scan statistics
#[tauri::command]
pub async fn get_scan_stats() -> Result<String, String> {
    let db = Database::new().map_err(|e| e.to_string())?;
    
    let total_results = db.count_scan_results().map_err(|e| e.to_string())?;
    
    Ok(json!({
        "total_results": total_results,
        "timestamp": Utc::now().to_rfc3339()
    }).to_string())
}
