use std::path::Path;
use std::fs;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc;
use tokio::sync::Semaphore;
use walkdir::WalkDir;
use calamine::{Reader, open_workbook, Xlsx, Xls};
use rayon::prelude::*;
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
    
    /// Start scanning with optimized concurrency
    pub async fn start_scan(&self) -> Result<(), String> {
        self.is_running.store(true, Ordering::SeqCst);
        self.is_paused.store(false, Ordering::SeqCst);
        
        // Use semaphore to limit concurrent file scanning
        let semaphore = Arc::new(Semaphore::new(self.config.thread_count));
        
        // Collect all files first
        let mut files_to_scan = Vec::new();
        
        for scan_path in &self.config.scan_paths {
            let path = Path::new(scan_path);
            if !path.exists() {
                return Err(format!("Path does not exist: {}", scan_path));
            }
            
            // Collect files
            for entry in WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if !self.is_running.load(Ordering::SeqCst) {
                    break;
                }
                
                let file_path = entry.path();
                
                // Skip directories
                if file_path.is_dir() {
                    continue;
                }
                
                // Check exclusion
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
                
                if self.config.file_types.contains(&extension) {
                    files_to_scan.push(file_path.to_path_buf());
                }
            }
        }
        
        // Process files concurrently
        let mut handles = Vec::new();
        
        for file_path in files_to_scan {
            if !self.is_running.load(Ordering::SeqCst) {
                break;
            }
            
            // Wait for pause
            while self.is_paused.load(Ordering::SeqCst) {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
            
            let semaphore = semaphore.clone();
            let is_running = self.is_running.clone();
            let files_scanned = self.files_scanned.clone();
            let file_path_clone = file_path.clone();
            
            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire().await;
                
                if !is_running.load(Ordering::SeqCst) {
                    return;
                }
                
                // Use spawn_blocking for I/O operations
                let result = tokio::task::spawn_blocking(move || {
                    Self::scan_file_blocking(&file_path_clone)
                }).await;
                
                if result.is_ok() {
                    files_scanned.fetch_add(1, Ordering::SeqCst);
                }
            });
            
            handles.push(handle);
        }
        
        // Wait for all tasks to complete
        for handle in handles {
            let _ = handle.await;
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
    
    /// Scan a single file (blocking version for spawn_blocking)
    fn scan_file_blocking(file_path: &Path) -> Result<(), String> {
        let extension = file_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        match extension.as_str() {
            "xlsx" | "xls" => Self::scan_excel_file_blocking(file_path),
            "csv" | "txt" => Self::scan_text_file_blocking(file_path),
            _ => Ok(()),
        }
    }
    
    /// Scan Excel file (blocking version)
    fn scan_excel_file_blocking(file_path: &Path) -> Result<(), String> {
        let file_path_str = file_path.to_string_lossy().to_string();
        
        let results = match open_workbook::<Xlsx<_>, _>(file_path) {
            Ok(mut workbook) => {
                let mut results = Vec::new();
                
                for sheet_name in workbook.sheet_names() {
                    if let Ok(range) = workbook.worksheet_range(&sheet_name) {
                        // Use rayon for parallel cell processing
                        let sheet_results: Vec<Option<ScanResult>> = range.rows()
                            .enumerate()
                            .par_bridge()
                            .flat_map(|(row_idx, row)| {
                                row.iter().enumerate().par_bridge().filter_map(move |(col_idx, cell)| {
                                    let cell_value = cell.to_string();
                                    if !cell_value.is_empty() {
                                        Self::check_content(&file_path_str, &sheet_name, row_idx as u32 + 1, col_idx as u32 + 1, &cell_value)
                                    } else {
                                        None
                                    }
                                }).collect::<Vec<_>>()
                            }).collect();
                        
                        results.extend(sheet_results.into_iter().flatten());
                    }
                }
                
                results
            }
            Err(_) => {
                // Try as XLS format
                match open_workbook::<Xls<_>, _>(file_path) {
                    Ok(mut workbook) => {
                        let mut results = Vec::new();
                        
                        for sheet_name in workbook.sheet_names() {
                            if let Ok(range) = workbook.worksheet_range(&sheet_name) {
                                let sheet_results: Vec<Option<ScanResult>> = range.rows()
                                    .enumerate()
                                    .par_bridge()
                                    .flat_map(|(row_idx, row)| {
                                        row.iter().enumerate().par_bridge().filter_map(move |(col_idx, cell)| {
                                            let cell_value = cell.to_string();
                                            if !cell_value.is_empty() {
                                                Self::check_content(&file_path_str, &sheet_name, row_idx as u32 + 1, col_idx as u32 + 1, &cell_value)
                                            } else {
                                                None
                                            }
                                        }).collect::<Vec<_>>()
                                    }).collect();
                                
                                results.extend(sheet_results.into_iter().flatten());
                            }
                        }
                        
                        results
                    }
                    Err(e) => return Err(format!("Failed to open file: {}", e)),
                }
            }
        };
        
        Ok(())
    }
    
    /// Scan text file (blocking version with streaming)
    fn scan_text_file_blocking(file_path: &Path) -> Result<(), String> {
        let file_path_str = file_path.to_string_lossy().to_string();
        
        // Read file in streaming mode for large files
        use std::io::BufRead;
        use std::io::BufReader;
        
        let file = fs::File::open(file_path)
            .map_err(|e| format!("Failed to open file: {}", e))?;
        
        let reader = BufReader::new(file);
        
        // Determine encoding
        let mut bytes = Vec::new();
        reader.take(1024).read_to_end(&mut bytes)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        let is_utf8 = String::from_utf8(&bytes).is_ok();
        
        // Reset and read line by line
        let reader = BufReader::new(fs::File::open(file_path)?);
        
        for (line_idx, line_result) in reader.lines().enumerate() {
            let line = line_result.map_err(|e| format!("Failed to read line: {}", e))?;
            
            // Split by comma for CSV, or process whole line for TXT
            for (col_idx, cell) in line.split(',').enumerate() {
                let cell_trimmed = cell.trim();
                if !cell_trimmed.is_empty() {
                    let content = if is_utf8 {
                        cell_trimmed.to_string()
                    } else {
                        // Try GB18030 encoding
                        let bytes = cell_trimmed.as_bytes();
                        let (decoded, _, _) = encoding_rs::GB18030.decode(bytes);
                        decoded.to_string()
                    };
                    
                    let _ = Self::check_content(
                        &file_path_str,
                        None,
                        line_idx as u32 + 1,
                        col_idx as u32 + 1,
                        &content
                    );
                }
            }
        }
        
        Ok(())
    }
    
    /// Check content for sensitive information (blocking version)
    fn check_content(
        file_path: &str,
        sheet_name: &str,
        row: u32,
        column: u32,
        content: &str,
    ) -> Option<ScanResult> {
        // Check all sensitive types
        for sensitive_type in &[SensitiveType::PhoneNumber, SensitiveType::IdCard, SensitiveType::Name, SensitiveType::Address] {
            let detected = match sensitive_type {
                SensitiveType::PhoneNumber => detect_phone_number(content),
                SensitiveType::IdCard => detect_id_card(content),
                SensitiveType::Name => detect_name(content),
                SensitiveType::Address => detect_address(content),
            };
            
            if let Some(detected_content) = detected {
                return Some(ScanResult {
                    id: Uuid::new_v4().to_string(),
                    file_path: file_path.to_string(),
                    sheet_name: Some(sheet_name.to_string()),
                    row,
                    column,
                    sensitive_type: *sensitive_type,
                    content: detected_content.clone(),
                    masked_content: mask_content(&detected_content, *sensitive_type),
                    found_at: Utc::now(),
                });
            }
        }
        
        None
    }
    
    /// Get current statistics
    pub fn get_stats(&self) -> (u64, u64) {
        (
            self.files_scanned.load(Ordering::SeqCst),
            self.results_found.load(Ordering::SeqCst),
        )
    }
}
