use std::path::Path;
use std::fs;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use walkdir::WalkDir;
use calamine::{Reader, open_workbook, Xlsx, Xls};
use crate::models::{ScanResult, SensitiveType, ScanConfig};
use crate::patterns::{detect_phone_number, detect_id_card, detect_name, detect_address, mask_content};
use crate::db::Database;
use uuid::Uuid;
use chrono::Utc;

#[allow(dead_code)]
pub struct Scanner {
    config: ScanConfig,
    db: Arc<Database>,
    is_running: Arc<AtomicBool>,
    is_paused: Arc<AtomicBool>,
    files_scanned: Arc<AtomicU64>,
    results_found: Arc<AtomicU64>,
}

#[allow(dead_code)]
impl Scanner {
    pub fn new(config: ScanConfig, db: Arc<Database>) -> Self {
        Scanner {
            config,
            db,
            is_running: Arc::new(AtomicBool::new(false)),
            is_paused: Arc::new(AtomicBool::new(false)),
            files_scanned: Arc::new(AtomicU64::new(0)),
            results_found: Arc::new(AtomicU64::new(0)),
        }
    }
    
    /// Start scanning
    pub async fn start_scan(&self) -> Result<(), String> {
        self.is_running.store(true, Ordering::SeqCst);
        self.is_paused.store(false, Ordering::SeqCst);
        
        for scan_path in &self.config.scan_paths {
            self.scan_directory(scan_path).await?;
        }
        
        self.is_running.store(false, Ordering::SeqCst);
        Ok(())
    }
    
    /// Pause scanning
    pub fn pause_scan(&self) {
        self.is_paused.store(true, Ordering::SeqCst);
    }
    
    /// Resume scanning
    pub fn resume_scan(&self) {
        self.is_paused.store(false, Ordering::SeqCst);
    }
    
    /// Stop scanning
    pub fn stop_scan(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }
    
    /// Scan a directory recursively
    async fn scan_directory(&self, dir_path: &str) -> Result<(), String> {
        let path = Path::new(dir_path);
        if !path.exists() {
            return Err(format!("Path does not exist: {}", dir_path));
        }
        
        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            // Check if we should stop or pause
            if !self.is_running.load(Ordering::SeqCst) {
                break;
            }
            
            while self.is_paused.load(Ordering::SeqCst) {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
            
            let file_path = entry.path();
            
            // Skip if it's a directory
            if file_path.is_dir() {
                continue;
            }
            
            // Check if path should be excluded
            if self.should_exclude_path(file_path) {
                continue;
            }
            
            // Check file size
            if let Ok(metadata) = fs::metadata(file_path) {
                if metadata.len() > self.config.max_file_size {
                    continue;
                }
            }
            
            // Check file extension
            let extension = file_path
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| format!(".{}", s.to_lowercase()))
                .unwrap_or_default();
            
            if !self.config.file_types.contains(&extension) {
                continue;
            }
            
            // Scan the file
            let _ = self.scan_file(file_path).await;
            
            self.files_scanned.fetch_add(1, Ordering::SeqCst);
        }
        
        Ok(())
    }
    
    /// Check if a path should be excluded
    fn should_exclude_path(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy().to_lowercase();
        
        // System paths to exclude
        let system_excludes = if cfg!(target_os = "windows") {
            vec![
                "\\windows\\",
                "\\program files\\",
                "\\program files (x86)\\",
                "\\programdata\\",
                "\\$recycle.bin\\",
            ]
        } else {
            vec![
                "/usr/",
                "/bin/",
                "/lib/",
                "/sbin/",
                "/sys/",
                "/proc/",
                "/dev/",
                "/var/cache/",
            ]
        };
        
        for exclude in system_excludes {
            if path_str.contains(exclude) {
                return true;
            }
        }
        
        // User-defined excludes
        for exclude_path in &self.config.exclude_paths {
            if path_str.contains(&exclude_path.to_lowercase()) {
                return true;
            }
        }
        
        false
    }
    
    /// Scan a single file
    async fn scan_file(&self, file_path: &Path) -> Result<(), String> {
        let extension = file_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        match extension.as_str() {
            "xlsx" | "xls" => self.scan_excel_file(file_path).await,
            "csv" | "txt" => self.scan_text_file(file_path).await,
            _ => Ok(()),
        }
    }
    
    /// Scan Excel file
    async fn scan_excel_file(&self, file_path: &Path) -> Result<(), String> {
        let file_path_str = file_path.to_string_lossy().to_string();
        
        match open_workbook::<Xlsx<_>, _>(file_path) {
            Ok(mut workbook) => {
                for sheet_name in workbook.sheet_names() {
                    if !self.is_running.load(Ordering::SeqCst) {
                        break;
                    }
                    
                    if let Ok(range) = workbook.worksheet_range(&sheet_name) {
                        for (row_idx, row) in range.rows().enumerate() {
                            for (col_idx, cell) in row.iter().enumerate() {
                                let cell_value = cell.to_string();
                                
                                if !cell_value.is_empty() {
                                    self.check_and_store_result(
                                        &file_path_str,
                                        Some(sheet_name.clone()),
                                        row_idx as u32 + 1,
                                        col_idx as u32 + 1,
                                        &cell_value,
                                    ).await;
                                }
                            }
                        }
                    }
                }
                Ok(())
            }
            Err(_) => {
                // Try as XLS format
                match open_workbook::<Xls<_>, _>(file_path) {
                    Ok(mut workbook) => {
                        for sheet_name in workbook.sheet_names() {
                            if !self.is_running.load(Ordering::SeqCst) {
                                break;
                            }
                            
                            if let Ok(range) = workbook.worksheet_range(&sheet_name) {
                                for (row_idx, row) in range.rows().enumerate() {
                                    for (col_idx, cell) in row.iter().enumerate() {
                                        let cell_value = cell.to_string();
                                        
                                        if !cell_value.is_empty() {
                                            self.check_and_store_result(
                                                &file_path_str,
                                                Some(sheet_name.clone()),
                                                row_idx as u32 + 1,
                                                col_idx as u32 + 1,
                                                &cell_value,
                                            ).await;
                                        }
                                    }
                                }
                            }
                        }
                        Ok(())
                    }
                    Err(e) => Err(format!("Failed to open file: {}", e)),
                }
            }
        }
    }
    
    /// Scan text file (CSV/TXT)
    async fn scan_text_file(&self, file_path: &Path) -> Result<(), String> {
        let file_path_str = file_path.to_string_lossy().to_string();
        
        match fs::read(file_path) {
            Ok(bytes) => {
                let content = if let Ok(s) = String::from_utf8(bytes.clone()) {
                    s
                } else {
                    // Try to decode with different encodings
                    let (decoded, _, _) = encoding_rs::GB18030.decode(&bytes);
                    decoded.to_string()
                };
                
                for (line_idx, line) in content.lines().enumerate() {
                    if !self.is_running.load(Ordering::SeqCst) {
                        break;
                    }
                    
                    for (col_idx, cell) in line.split(',').enumerate() {
                        if !cell.trim().is_empty() {
                            self.check_and_store_result(
                                &file_path_str,
                                None,
                                line_idx as u32 + 1,
                                col_idx as u32 + 1,
                                cell.trim(),
                            ).await;
                        }
                    }
                }
                Ok(())
            }
            Err(e) => Err(format!("Failed to read file: {}", e)),
        }
    }
    
    /// Check cell content and store result if sensitive info is found
    async fn check_and_store_result(
        &self,
        file_path: &str,
        sheet_name: Option<String>,
        row: u32,
        column: u32,
        content: &str,
    ) {
        for sensitive_type in &self.config.sensitive_types {
            let detected = match sensitive_type {
                SensitiveType::PhoneNumber => detect_phone_number(content),
                SensitiveType::IdCard => detect_id_card(content),
                SensitiveType::Name => detect_name(content),
                SensitiveType::Address => detect_address(content),
            };
            
            if let Some(detected_content) = detected {
                let result = ScanResult {
                    id: Uuid::new_v4().to_string(),
                    file_path: file_path.to_string(),
                    sheet_name: sheet_name.clone(),
                    row,
                    column,
                    sensitive_type: *sensitive_type,
                    content: detected_content.clone(),
                    masked_content: mask_content(&detected_content, *sensitive_type),
                    found_at: Utc::now(),
                };
                
                if self.db.insert_scan_result(&result).is_ok() {
                    self.results_found.fetch_add(1, Ordering::SeqCst);
                }
            }
        }
    }
    
    /// Get current statistics
    pub fn get_stats(&self) -> (u64, u64) {
        (
            self.files_scanned.load(Ordering::SeqCst),
            self.results_found.load(Ordering::SeqCst),
        )
    }
}
