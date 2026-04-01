use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::collections::HashMap;
use tokio::sync::{Semaphore, oneshot};
use walkdir::WalkDir;
use calamine::{Reader, open_workbook, Xlsx, Xls};
use rayon::prelude::*;
use crate::models::{ScanResult, SensitiveType, ScanConfig};
use crate::patterns::{detect_phone_number, detect_id_card, detect_name, detect_address, mask_content};
use crate::db::Database;
use crate::error::{AppError, AppResult};
use crate::logger::Logger;
use uuid::Uuid;
use chrono::Utc;
use serde::{Serialize, Deserialize};

/// Scan progress snapshot for resume
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanSnapshot {
    pub id: String,
    pub config: ScanConfig,
    pub scanned_files: Vec<String>,
    pub pending_files: Vec<String>,
    pub results_count: u64,
    pub created_at: chrono::DateTime<Utc>,
    pub last_updated: chrono::DateTime<Utc>,
}

/// Scan state for graceful shutdown
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanState {
    Idle,
    Running,
    Paused,
    Stopping,
    Stopped,
    Completed,
    Failed,
}

impl ScanState {
    pub fn is_active(&self) -> bool {
        matches!(self, ScanState::Running | ScanState::Paused | ScanState::Stopping)
    }
}

/// Enhanced scanner with graceful shutdown and resume
pub struct EnhancedScanner {
    config: ScanConfig,
    db: Arc<Database>,
    logger: Arc<Logger>,
    
    // State management
    state: Arc<AtomicU64>, // Using AtomicU64 for ScanState
    is_paused: Arc<AtomicBool>,
    is_stopping: Arc<AtomicBool>,
    
    // Progress tracking
    files_scanned: Arc<AtomicU64>,
    results_found: Arc<AtomicU64>,
    current_file: Arc<tokio::sync::Mutex<Option<String>>>,
    
    // Resume support
    scan_snapshot: Arc<tokio::sync::Mutex<Option<ScanSnapshot>>>,
    
    // Task management
    scan_tasks: Arc<tokio::sync::Mutex<Vec<tokio::task::JoinHandle<()>>>>,
    shutdown_signal: Arc<tokio::sync::Mutex<Option<oneshot::Sender<()>>>>,
}

impl EnhancedScanner {
    pub fn new(config: ScanConfig, db: Arc<Database>, logger: Arc<Logger>) -> Self {
        EnhancedScanner {
            config,
            db,
            logger,
            state: Arc::new(AtomicU64::new(ScanState::Idle as u64)),
            is_paused: Arc::new(AtomicBool::new(false)),
            is_stopping: Arc::new(AtomicBool::new(false)),
            files_scanned: Arc::new(AtomicU64::new(0)),
            results_found: Arc::new(AtomicU64::new(0)),
            current_file: Arc::new(tokio::sync::Mutex::new(None)),
            scan_snapshot: Arc::new(tokio::sync::Mutex::new(None)),
            scan_tasks: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            shutdown_signal: Arc::new(tokio::sync::Mutex::new(None)),
        }
    }
    
    /// Get current scan state
    pub fn get_state(&self) -> ScanState {
        unsafe { std::mem::transmute(self.state.load(Ordering::SeqCst)) }
    }
    
    /// Set scan state
    fn set_state(&self, state: ScanState) {
        self.state.store(state as u64, Ordering::SeqCst);
        self.logger.info(
            "SCANNER_STATE",
            &format!("State changed to {:?}", state),
            None,
        );
    }
    
    /// Start scanning with resume support
    pub async fn start_scan(&self) -> AppResult<()> {
        // Check if already running
        if self.get_state().is_active() {
            return Err(AppError::Scan("Scan is already in progress".to_string()));
        }
        
        self.set_state(ScanState::Running);
        self.is_paused.store(false, Ordering::SeqCst);
        self.is_stopping.store(false, Ordering::SeqCst);
        
        self.logger.info(
            "SCAN_START",
            &format!("Starting scan with {} paths", self.config.scan_paths.len()),
            None,
        );
        
        // Collect files to scan
        let files_to_scan = self.collect_files().await?;
        
        if files_to_scan.is_empty() {
            self.logger.warning("SCAN_START", "No files found to scan", None);
            self.set_state(ScanState::Completed);
            return Ok(());
        }
        
        // Create snapshot for resume
        let snapshot = ScanSnapshot {
            id: Uuid::new_v4().to_string(),
            config: self.config.clone(),
            scanned_files: Vec::new(),
            pending_files: files_to_scan.iter().map(|p| p.to_string_lossy().to_string()).collect(),
            results_count: 0,
            created_at: Utc::now(),
            last_updated: Utc::now(),
        };
        
        *self.scan_snapshot.lock().await = Some(snapshot);
        
        // Start scanning with graceful shutdown
        self.scan_files(files_to_scan).await?;
        
        Ok(())
    }
    
    /// Resume from snapshot
    pub async fn resume_scan(&self, snapshot: ScanSnapshot) -> AppResult<()> {
        // Check if already running
        if self.get_state().is_active() {
            return Err(AppError::Scan("Scan is already in progress".to_string()));
        }
        
        self.set_state(ScanState::Running);
        self.is_paused.store(false, Ordering::SeqCst);
        self.is_stopping.store(false, Ordering::SeqCst);
        
        self.logger.info(
            "SCAN_RESUME",
            &format!("Resuming scan from snapshot {}", snapshot.id),
            Some(&format!("Pending files: {}", snapshot.pending_files.len())),
        );
        
        *self.scan_snapshot.lock().await = Some(snapshot.clone());
        
        let pending_files: Vec<PathBuf> = snapshot.pending_files
            .iter()
            .map(|s| PathBuf::from(s))
            .collect();
        
        if pending_files.is_empty() {
            self.set_state(ScanState::Completed);
            return Ok(());
        }
        
        self.scan_files(pending_files).await?;
        
        Ok(())
    }
    
    /// Collect files to scan
    async fn collect_files(&self) -> AppResult<Vec<PathBuf>> {
        let mut files_to_scan = Vec::new();
        
        for scan_path in &self.config.scan_paths {
            let path = Path::new(scan_path);
            if !path.exists() {
                self.logger.error(
                    "SCAN_COLLECT",
                    &format!("Path does not exist: {}", scan_path),
                    None,
                );
                continue;
            }
            
            self.logger.debug(
                "SCAN_COLLECT",
                &format!("Collecting files from: {}", scan_path),
                None,
            );
            
            // Collect files
            for entry in WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if self.is_stopping.load(Ordering::SeqCst) {
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
                if let Ok(metadata) = std::fs::metadata(file_path) {
                    if metadata.len() > self.config.max_file_size {
                        self.logger.debug(
                            "SCAN_COLLECT",
                            &format!("Skipping large file: {}", file_path.display()),
                            Some(&format!("Size: {} bytes", metadata.len())),
                        );
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
        
        self.logger.info(
            "SCAN_COLLECT",
            &format!("Collected {} files to scan", files_to_scan.len()),
            None,
        );
        
        Ok(files_to_scan)
    }
    
    /// Scan files with graceful shutdown
    async fn scan_files(&self, files_to_scan: Vec<PathBuf>) -> AppResult<()> {
        let semaphore = Arc::new(Semaphore::new(self.config.thread_count));
        let mut handles = Vec::new();
        
        for file_path in files_to_scan {
            // Check for graceful shutdown
            if self.is_stopping.load(Ordering::SeqCst) {
                self.logger.info("SCAN_STOP", "Graceful shutdown initiated", None);
                break;
            }
            
            // Wait for pause
            while self.is_paused.load(Ordering::SeqCst) {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                
                if self.is_stopping.load(Ordering::SeqCst) {
                    break;
                }
            }
            
            if self.is_stopping.load(Ordering::SeqCst) {
                break;
            }
            
            // Update current file
            *self.current_file.lock().await = Some(file_path.to_string_lossy().to_string());
            
            let semaphore = semaphore.clone();
            let is_stopping = self.is_stopping.clone();
            let file_path_clone = file_path.clone();
            let files_scanned = self.files_scanned.clone();
            let results_found = self.results_found.clone();
            let logger = self.logger.clone();
            
            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire().await;
                
                if is_stopping.load(Ordering::SeqCst) {
                    return;
                }
                
                // Use spawn_blocking for I/O operations
                let result = tokio::task::spawn_blocking(move || {
                    Self::scan_file_blocking(&file_path_clone, &logger)
                }).await;
                
                match result {
                    Ok(Ok(count)) => {
                        files_scanned.fetch_add(1, Ordering::SeqCst);
                        results_found.fetch_add(count, Ordering::SeqCst);
                    }
                    Ok(Err(e)) => {
                        logger.error("SCAN_FILE", &format!("Failed to scan file: {}", file_path_clone.display()), Some(&e.to_string()));
                    }
                    Err(e) => {
                        logger.error("SCAN_FILE", &format!("Task cancelled: {}", file_path_clone.display()), Some(&e.to_string()));
                    }
                }
            });
            
            handles.push(handle);
        }
        
        // Wait for all tasks to complete or be cancelled
        for handle in handles {
            match tokio::time::timeout(
                tokio::time::Duration::from_secs(30),
                handle
            ).await {
                Ok(Ok(_)) => {},
                Ok(Err(e)) => {
                    self.logger.error("SCAN_TASK", "Task failed", Some(&e.to_string()));
                }
                Err(_) => {
                    self.logger.warning("SCAN_TASK", "Task timeout, continuing shutdown", None);
                }
            }
        }
        
        // Update final state
        if self.is_stopping.load(Ordering::SeqCst) {
            self.set_state(ScanState::Stopped);
            self.logger.info("SCAN_STOP", "Scan stopped gracefully", None);
        } else {
            self.set_state(ScanState::Completed);
            self.logger.info("SCAN_COMPLETE", "Scan completed successfully", None);
        }
        
        Ok(())
    }
    
    /// Pause scanning
    pub fn pause_scan(&self) {
        if self.get_state() == ScanState::Running {
            self.is_paused.store(true, Ordering::SeqCst);
            self.set_state(ScanState::Paused);
            self.logger.info("SCAN_PAUSE", "Scan paused", None);
        }
    }
    
    /// Resume scanning
    pub fn resume_scan_current(&self) {
        if self.get_state() == ScanState::Paused {
            self.is_paused.store(false, Ordering::SeqCst);
            self.set_state(ScanState::Running);
            self.logger.info("SCAN_RESUME", "Scan resumed", None);
        }
    }
    
    /// Stop scanning gracefully
    pub async fn stop_scan(&self) -> AppResult<()> {
        if !self.get_state().is_active() {
            return Err(AppError::Scan("No active scan to stop".to_string()));
        }
        
        self.logger.info("SCAN_STOP_REQUEST", "Initiating graceful shutdown", None);
        self.set_state(ScanState::Stopping);
        self.is_stopping.store(true, Ordering::SeqCst);
        
        // Save current state for resume
        if let Some(snapshot) = self.scan_snapshot.lock().await.as_ref() {
            let updated_snapshot = ScanSnapshot {
                pending_files: Vec::new(), // Will be updated by scan_files
                results_count: self.results_found.load(Ordering::SeqCst),
                last_updated: Utc::now(),
                ..snapshot.clone()
            };
            
            // TODO: Save snapshot to database
            self.logger.info(
                "SNAPSHOT_SAVE",
                "Saving scan snapshot for resume",
                Some(&format!("ID: {}", updated_snapshot.id)),
            );
        }
        
        // Cancel all tasks
        let mut tasks = self.scan_tasks.lock().await;
        for task in tasks.drain(..) {
            task.abort();
        }
        
        Ok(())
    }
    
    /// Get current statistics
    pub fn get_stats(&self) -> (u64, u64, Option<String>) {
        (
            self.files_scanned.load(Ordering::SeqCst),
            self.results_found.load(Ordering::SeqCst),
            self.current_file.blocking_lock().clone(),
        )
    }
    
    /// Get current snapshot
    pub async fn get_snapshot(&self) -> Option<ScanSnapshot> {
        self.scan_snapshot.lock().await.clone()
    }
    
    /// Check if a path should be excluded
    fn should_exclude_path(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy().to_lowercase();
        
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
        
        for exclude_path in &self.config.exclude_paths {
            if path_str.contains(&exclude_path.to_lowercase()) {
                return true;
            }
        }
        
        false
    }
    
    /// Scan a single file (blocking version)
    fn scan_file_blocking(file_path: &Path, logger: &Logger) -> AppResult<u64> {
        let extension = file_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        logger.debug(
            "SCAN_FILE_START",
            &format!("Scanning file: {}", file_path.display()),
            Some(&format!("Type: {}", extension)),
        );
        
        let result = match extension.as_str() {
            "xlsx" | "xls" => Self::scan_excel_file_blocking(file_path, logger),
            "csv" | "txt" => Self::scan_text_file_blocking(file_path, logger),
            _ => Ok(0),
        };
        
        match &result {
            Ok(count) => {
                logger.debug(
                    "SCAN_FILE_COMPLETE",
                    &format!("Completed: {}", file_path.display()),
                    Some(&format!("Found: {} results", count)),
                );
            }
            Err(e) => {
                logger.error(
                    "SCAN_FILE_ERROR",
                    &format!("Failed: {}", file_path.display()),
                    Some(&e.to_string()),
                );
            }
        }
        
        result
    }
    
    /// Scan Excel file (blocking version)
    fn scan_excel_file_blocking(file_path: &Path, logger: &Logger) -> AppResult<u64> {
        let file_path_str = file_path.to_string_lossy().to_string();
        let mut total_count = 0u64;
        
        match open_workbook::<Xlsx<_>, _>(file_path) {
            Ok(mut workbook) => {
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
                        
                        total_count += sheet_results.len() as u64;
                    }
                }
            }
            Err(_) => {
                // Try as XLS format
                match open_workbook::<Xls<_>, _>(file_path) {
                    Ok(mut workbook) => {
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
                                
                                total_count += sheet_results.len() as u64;
                            }
                        }
                    }
                    Err(e) => {
                        logger.error("EXCEL_PARSE", "Failed to parse Excel file", Some(&e.to_string()));
                        return Err(AppError::Excel(e.to_string()));
                    }
                }
            }
        }
        
        Ok(total_count)
    }
    
    /// Scan text file (blocking version)
    fn scan_text_file_blocking(file_path: &Path, logger: &Logger) -> AppResult<u64> {
        let file_path_str = file_path.to_string_lossy().to_string();
        let mut total_count = 0u64;
        
        use std::io::BufRead;
        
        let file = std::fs::File::open(file_path)?;
        let reader = std::io::BufReader::new(file);
        
        for (line_idx, line_result) in reader.lines().enumerate() {
            let line = line_result?;
            
            for (col_idx, cell) in line.split(',').enumerate() {
                let cell_trimmed = cell.trim();
                if !cell_trimmed.is_empty() {
                    if let Some(_) = Self::check_content(
                        &file_path_str,
                        None,
                        line_idx as u32 + 1,
                        col_idx as u32 + 1,
                        cell_trimmed
                    ) {
                        total_count += 1;
                    }
                }
            }
        }
        
        Ok(total_count)
    }
    
    /// Check content for sensitive information (blocking version)
    fn check_content(
        file_path: &str,
        sheet_name: &str,
        row: u32,
        column: u32,
        content: &str,
    ) -> Option<ScanResult> {
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
}
