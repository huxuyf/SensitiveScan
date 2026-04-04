use serde_json::json;
use crate::models::{ScanConfig, SensitiveType, WhitelistEntry};
use crate::db::Database;
use crate::scanner::Scanner;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use chrono::Utc;
use tauri_plugin_dialog::DialogExt;
use tauri::Emitter;
use std::fs::File;
use std::io::Write;

// Global scanner instance
static SCANNER: Mutex<Option<Arc<Scanner>>> = Mutex::new(None);

/// Select a folder using system dialog
#[tauri::command]
pub async fn select_folder(app: tauri::AppHandle) -> Result<String, String> {
    use tokio::sync::oneshot;

    let (tx, rx) = oneshot::channel();

    app.dialog()
        .file()
        .pick_folder(move |result| {
            let _ = tx.send(result);
        });

    let folder_path = rx.await.map_err(|e| e.to_string())?;

    match folder_path {
        Some(path) => Ok(path.to_string()),
        None => Err("No folder selected".to_string()),
    }
}

/// Start a new scan task
#[tauri::command]
pub async fn start_scan(
    app: tauri::AppHandle,
    db: tauri::State<'_, Arc<Database>>,
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

    let mut scanner = Scanner::new(config, db.inner().clone());

    // Set app handle for event emission
    scanner.set_app_handle(app.clone());

    // Store scanner instance globally
    let scanner_arc = Arc::new(scanner);
    {
        let mut guard = SCANNER.lock().unwrap();
        *guard = Some(scanner_arc.clone());
    }

    // Start scanning in background
    let scanner_clone = scanner_arc.clone();
    tokio::spawn(async move {
        if let Err(e) = scanner_clone.start_scan().await {
            eprintln!("Scan error: {}", e);
        }

        // Emit completion event
        if let Err(e) = app.emit("scan-complete", json!({"status": "completed"})) {
            eprintln!("Failed to emit scan-complete event: {}", e);
        }
    });

    Ok(json!({
        "task_id": Uuid::new_v4().to_string(),
        "status": "started"
    }).to_string())
}

/// Pause current scan
#[tauri::command]
pub async fn pause_scan() -> Result<String, String> {
    let guard = SCANNER.lock().unwrap();
    if let Some(scanner) = guard.as_ref() {
        scanner.pause_scan();
        Ok(json!({
            "status": "paused"
        }).to_string())
    } else {
        Err("No active scan".to_string())
    }
}

/// Resume paused scan
#[tauri::command]
pub async fn resume_scan() -> Result<String, String> {
    let guard = SCANNER.lock().unwrap();
    if let Some(scanner) = guard.as_ref() {
        scanner.resume_scan();
        Ok(json!({
            "status": "resumed"
        }).to_string())
    } else {
        Err("No active scan".to_string())
    }
}

/// Stop current scan
#[tauri::command]
pub async fn stop_scan() -> Result<String, String> {
    let guard = SCANNER.lock().unwrap();
    if let Some(scanner) = guard.as_ref() {
        scanner.stop_scan();
        Ok(json!({
            "status": "stopped"
        }).to_string())
    } else {
        Err("No active scan".to_string())
    }
}

/// Get scan results
#[tauri::command]
pub async fn get_scan_results(
    db: tauri::State<'_, Arc<Database>>,
    limit: Option<i64>,
    offset: Option<i64>,
    file_path_filter: Option<String>,
    sensitive_type_filter: Option<String>,
) -> Result<String, String> {
    let results = db.get_scan_results(
        limit,
        offset,
        file_path_filter.as_deref(),
        sensitive_type_filter.as_deref(),
    ).map_err(|e| e.to_string())?;
    
    serde_json::to_string(&results).map_err(|e| e.to_string())
}

/// Export scan results
#[tauri::command]
pub async fn export_results(
    db: tauri::State<'_, Arc<Database>>,
    format: String,
    file_path: String,
) -> Result<String, String> {
    let results = db.get_scan_results(Some(10000), None, None, None)
        .map_err(|e| e.to_string())?;

    if format == "csv" {
        let mut file = File::create(&file_path).map_err(|e| e.to_string())?;
        
        // Write CSV header
        writeln!(file, "ID,File Path,Sheet,Row,Column,Type,Content,Found At")
            .map_err(|e| e.to_string())?;
        
        for r in results {
            writeln!(
                file,
                "\"{}\",\"{}\",\"{}\",{},{},\"{:?}\",\"{}\",\"{}\"",
                r.id,
                r.file_path.replace("\"", "\"\""),
                r.sheet_name.unwrap_or_default().replace("\"", "\"\""),
                r.row,
                r.column,
                r.sensitive_type,
                r.content.replace("\"", "\"\""),
                r.found_at.to_rfc3339()
            ).map_err(|e| e.to_string())?;
        }

        Ok(json!({
            "status": "exported",
            "file_path": file_path
        }).to_string())
    } else {
        Err("Unsupported format".to_string())
    }
}

/// Get scan history
#[tauri::command]
pub async fn get_history(
    db: tauri::State<'_, Arc<Database>>,
    limit: Option<i64>
) -> Result<String, String> {
    let history = db.get_scan_history(limit).map_err(|e| e.to_string())?;
    serde_json::to_string(&history).map_err(|e| e.to_string())
}

/// Delete scan history
#[tauri::command]
pub async fn delete_history(
    db: tauri::State<'_, Arc<Database>>,
    history_id: String
) -> Result<String, String> {
    db.delete_scan_history(&history_id).map_err(|e| e.to_string())?;
    
    Ok(json!({
        "status": "deleted"
    }).to_string())
}

/// Add whitelist entry
#[tauri::command]
pub async fn add_whitelist(
    db: tauri::State<'_, Arc<Database>>,
    content: String,
    sensitive_type: String,
    description: Option<String>,
) -> Result<String, String> {
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
    
    serde_json::to_string(&entry).map_err(|e| e.to_string())
}

/// Get whitelist
#[tauri::command]
pub async fn get_whitelist(db: tauri::State<'_, Arc<Database>>) -> Result<String, String> {
    let whitelist = db.get_whitelist().map_err(|e| e.to_string())?;
    serde_json::to_string(&whitelist).map_err(|e| e.to_string())
}

/// Delete whitelist entry
#[tauri::command]
pub async fn delete_whitelist(
    db: tauri::State<'_, Arc<Database>>,
    entry_id: String
) -> Result<String, String> {
    db.delete_whitelist(&entry_id).map_err(|e| e.to_string())?;
    
    Ok(json!({
        "status": "deleted"
    }).to_string())
}

/// Get scan statistics
#[tauri::command]
pub async fn get_scan_stats(db: tauri::State<'_, Arc<Database>>) -> Result<String, String> {
    let total_results = db.count_scan_results().map_err(|e| e.to_string())?;
    
    Ok(json!({
        "total_results": total_results,
        "timestamp": Utc::now().to_rfc3339()
    }).to_string())
}
