use crate::models::{ScanConfig, SensitiveType};
use crate::db::Database;
use crate::scanner::Scanner;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use chrono::Utc;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;
use tauri::Emitter;
use std::fs::File;
use std::io::Write;

// 全局扫描器实例
static SCANNER: Mutex<Option<Arc<Scanner>>> = Mutex::new(None);

/// 使用系统对话框选择文件夹
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
        None => Err("未选择任何文件夹".to_string()),
    }
}

/// 启动一个新的扫描任务
#[tauri::command]
pub async fn start_scan(
    app: tauri::AppHandle,
    db: tauri::State<'_, Arc<Database>>,
    scan_paths: Vec<String>,
    exclude_paths: Vec<String>,
    max_file_size: u64,
    sensitive_types: Vec<String>,
) -> Result<String, String> {
    // 从数据库中清除先前的结果
    let _ = db.delete_scan_results();

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

    // 设置应用程序句柄以进行事件发射
    scanner.set_app_handle(app.clone());

    // 将扫描器实例全局存储
    let scanner_arc = Arc::new(scanner);
    {
        let mut guard = SCANNER.lock().unwrap();
        *guard = Some(scanner_arc.clone());
    }

    // 在后台启动扫描
    let scanner_clone = scanner_arc.clone();
    tokio::spawn(async move {
        if let Err(e) = scanner_clone.start_scan().await {
            eprintln!("扫描产生错误: {}", e);
        }

        // 触发完成事件
        let _ = app.emit("scan-complete", serde_json::json!({"status": "completed"}));
    });

    Ok(serde_json::json!({
        "task_id": Uuid::new_v4().to_string(),
        "status": "started"
    }).to_string())
}

/// 暂停当前进行中的扫描
#[tauri::command]
pub async fn pause_scan() -> Result<String, String> {
    let guard = SCANNER.lock().unwrap();
    if let Some(scanner) = guard.as_ref() {
        scanner.pause_scan();
        Ok(serde_json::json!({
            "status": "paused"
        }).to_string())
    } else {
        Err("当前未进行活动扫描任务".to_string())
    }
}

/// 恢复已暂停的扫描
#[tauri::command]
pub async fn resume_scan() -> Result<String, String> {
    let guard = SCANNER.lock().unwrap();
    if let Some(scanner) = guard.as_ref() {
        scanner.resume_scan();
        Ok(serde_json::json!({
            "status": "resumed"
        }).to_string())
    } else {
        Err("当前未进行活动扫描任务".to_string())
    }
}

/// 停止当前进行中的扫描
#[tauri::command]
pub async fn stop_scan() -> Result<String, String> {
    let guard = SCANNER.lock().unwrap();
    if let Some(scanner) = guard.as_ref() {
        scanner.stop_scan();
        Ok(serde_json::json!({
            "status": "stopped"
        }).to_string())
    } else {
        Err("当前未进行活动扫描任务".to_string())
    }
}

/// 获取按文件归总的扫描结果记录
#[tauri::command]
pub async fn get_aggregated_results(
    db: tauri::State<'_, Arc<Database>>,
    threshold: Option<u32>,
) -> Result<String, String> {
    let threshold = threshold.unwrap_or(50);
    let results = db.get_aggregated_results(threshold)
        .map_err(|e: rusqlite::Error| e.to_string())?;
    
    serde_json::to_string(&results).map_err(|e| e.to_string())
}

/// 清除当前库中所有的扫描记录结果
#[tauri::command]
pub async fn clear_results(db: tauri::State<'_, Arc<Database>>) -> Result<String, String> {
    db.delete_scan_results().map_err(|e: rusqlite::Error| e.to_string())?;
    Ok(serde_json::json!({ "status": "cleared" }).to_string())
}

/// 利用系统默认程序打开文件
#[tauri::command]
pub async fn open_file(app: tauri::AppHandle, path: String) -> Result<(), String> {
    app.opener()
       .open_path(path, Option::<String>::None)
       .map_err(|e| e.to_string())
}

/// 从磁盘和数据库中物理删除对应文件和它相关的记录
#[tauri::command]
pub async fn delete_file(
    db: tauri::State<'_, Arc<Database>>,
    path: String
) -> Result<String, String> {
    // 1. 从磁盘中删除
    if std::path::Path::new(&path).exists() {
        std::fs::remove_file(&path).map_err(|e| format!("从磁盘删除文件失败: {}", e))?;
    }

    // 2. 从数据库中删除相关记录表
    db.delete_results_by_file(&path).map_err(|e: rusqlite::Error| format!("从底层数据库清理该条记录失败: {}", e))?;

    Ok(serde_json::json!({ "status": "deleted" }).to_string())
}

/// 导出按要求配置格式的扫描结果汇总表
#[tauri::command]
pub async fn export_results(
    db: tauri::State<'_, Arc<Database>>,
    format: String,
    file_path: String,
) -> Result<String, String> {
    let results = db.get_aggregated_results(0)
        .map_err(|e: rusqlite::Error| e.to_string())?;

    if format == "csv" || format == "xlsx" {
        let mut file = File::create(&file_path).map_err(|e| e.to_string())?;
        
        // 写入 UTF-8 BOM 使 Excel 下避免乱码情形
        let _ = file.write_all(&[0xEF, 0xBB, 0xBF]);

        // 写入导出用的表头 Header 列名字段
        let _ = writeln!(file, "文件名,文件路径,文件大小,文件类型,涉敏类型");
        
        for r in results {
            let file_size_f: f64 = r.file_size as f64;
            let kb = file_size_f / 1024.0;
            let size_str = if kb < 1024.0 {
                format!("{:.0} KB", kb.max(1.0))
            } else {
                let mb = kb / 1024.0;
                format!("{:.2} MB", mb)
            };

            let translated_types = r.sensitive_types
                .replace("PhoneNumber", "手机号码")
                .replace("IdCard", "身份证号")
                .replace("Name", "姓名")
                .replace("Address", "地址");

            let _ = writeln!(
                file,
                "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"",
                r.file_name.replace("\"", "\"\""),
                r.file_path.replace("\"", "\"\""),
                size_str,
                r.file_type.replace("\"", "\"\""),
                translated_types.replace("\"", "\"\"")
            );
        }

        Ok(serde_json::json!({
            "status": "exported",
            "file_path": file_path
        }).to_string())
    } else {
        Err("不受系统支持的文件转换格式".to_string())
    }
}

/// 获取全局扫描统计数
#[tauri::command]
pub async fn get_scan_stats(db: tauri::State<'_, Arc<Database>>) -> Result<String, String> {
    let total_results = db.count_scan_results().map_err(|e: rusqlite::Error| e.to_string())?;
    
    Ok(serde_json::json!({
        "total_results": total_results,
        "timestamp": Utc::now().to_rfc3339()
    }).to_string())
}
