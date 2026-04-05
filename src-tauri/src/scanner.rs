use std::path::Path;
use std::fs;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use walkdir::WalkDir;
use calamine::{Reader, open_workbook, Xlsx, Xls};
use crate::models::{ScanResult, SensitiveType, ScanConfig};
use crate::patterns::{detect_phone_number, detect_id_card, detect_name, detect_address, mask_content};
use crate::db::Database;
use uuid::Uuid;
use chrono::Utc;
use tauri::{AppHandle, Emitter};

#[allow(dead_code)]
pub struct Scanner {
    config: ScanConfig,
    db: Arc<Database>,
    app_handle: Option<AppHandle>,
    is_running: Arc<AtomicBool>,
    is_paused: Arc<AtomicBool>,
    files_scanned: Arc<AtomicU64>,
    results_found: Arc<AtomicU64>,
    start_time: Arc<Mutex<std::time::Instant>>,
    total_files: Arc<AtomicU64>,
}

#[allow(dead_code)]
impl Scanner {
    pub fn new(config: ScanConfig, db: Arc<Database>) -> Self {
        Scanner {
            config,
            db,
            app_handle: None,
            is_running: Arc::new(AtomicBool::new(false)),
            is_paused: Arc::new(AtomicBool::new(false)),
            files_scanned: Arc::new(AtomicU64::new(0)),
            results_found: Arc::new(AtomicU64::new(0)),
            start_time: Arc::new(Mutex::new(std::time::Instant::now())),
            total_files: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn set_app_handle(&mut self, app_handle: AppHandle) {
        self.app_handle = Some(app_handle);
    }

    /// 开始执行整套扫描流程
    pub async fn start_scan(&self) -> Result<(), String> {
        self.is_running.store(true, Ordering::SeqCst);
        self.is_paused.store(false, Ordering::SeqCst);
        *self.start_time.lock().unwrap() = std::time::Instant::now();

        // 正式扫描前预先统计该次所覆盖的全盘文件总数
        for scan_path in &self.config.scan_paths {
            self.count_files(scan_path).await?;
        }

        for scan_path in &self.config.scan_paths {
            self.scan_directory(scan_path).await?;
        }

        self.is_running.store(false, Ordering::SeqCst);
        Ok(())
    }

    /// 统计指定目录内需扫描处理的文件数
    async fn count_files(&self, dir_path: &str) -> Result<(), String> {
        let path = Path::new(dir_path);
        if !path.exists() {
            return Err(format!("扫描的基础盘符路径或者文件夹不存在: {}", dir_path));
        }

        let mut count = 0u64;
        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let file_path = entry.path();

            if file_path.is_dir() {
                continue;
            }

            if self.should_exclude_path(file_path) {
                continue;
            }

            if let Ok(metadata) = fs::metadata(file_path) {
                if metadata.len() > self.config.max_file_size {
                    continue;
                }
            }

            let extension = file_path
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| format!(".{}", s.to_lowercase()))
                .unwrap_or_default();

            if self.config.file_types.contains(&extension) {
                count += 1;
            }
        }

        self.total_files.store(count, Ordering::SeqCst);
        Ok(())
    }

    /// 中途暂停进行中的扫描
    pub fn pause_scan(&self) {
        self.is_paused.store(true, Ordering::SeqCst);
    }

    /// 唤醒并恢复进行中的暂停状态
    pub fn resume_scan(&self) {
        self.is_paused.store(false, Ordering::SeqCst);
    }

    /// 强行中断并结束进程任务
    pub fn stop_scan(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }

    /// 向前台应用界面发送运行轨迹与探测实况等实时消息
    fn emit_progress(&self, current_file: &str) {
        if let Some(app) = &self.app_handle {
            let files_scanned = self.files_scanned.load(Ordering::SeqCst);
            let results_found = self.results_found.load(Ordering::SeqCst);
            let total_files = self.total_files.load(Ordering::SeqCst);
            let elapsed = self.start_time.lock().unwrap().elapsed().as_secs();

            let progress_percentage = if total_files > 0 {
                (files_scanned as f64 / total_files as f64 * 100.0) as u32
            } else {
                0
            };

            let scan_speed = if elapsed > 0 {
                files_scanned / elapsed
            } else {
                0
            };

            let estimated_remaining = if scan_speed > 0 && total_files > files_scanned {
                (total_files - files_scanned) / scan_speed
            } else {
                0
            };

            let progress_data = serde_json::json!({
                "current_file": current_file,
                "files_scanned": files_scanned,
                "results_found": results_found,
                "total_files": total_files,
                "progress_percentage": progress_percentage,
                "scan_speed": scan_speed,
                "elapsed_seconds": elapsed,
                "estimated_remaining": estimated_remaining,
            });

            if let Err(e) = app.emit("scan-progress", progress_data) {
                eprintln!("投递扫描进度信息失败: {}", e);
            }
        }
    }

    /// 对指定的单一目录进行深度层级的递归排查
    async fn scan_directory(&self, dir_path: &str) -> Result<(), String> {
        let path = Path::new(dir_path);
        if !path.exists() {
            return Err(format!("试图扫描的途径路径并不存在: {}", dir_path));
        }

        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            // 在每一循环阶段检查用户是否抛出了随时随地关闭的强行停止信号
            if !self.is_running.load(Ordering::SeqCst) {
                break;
            }

            while self.is_paused.load(Ordering::SeqCst) {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }

            let file_path = entry.path();

            // 若轮询出来的是文件夹节点本身则跳过，因为 WalkDir 封装了自动切入树形目录
            if file_path.is_dir() {
                continue;
            }

            // 防火墙排外规则机制，判定符合跳过原则的话就丢弃
            if self.should_exclude_path(file_path) {
                continue;
            }

            // 对碰到了体积巨大（超出白名单体积阈值）的文件进行过滤
            if let Ok(metadata) = fs::metadata(file_path) {
                if metadata.len() > self.config.max_file_size {
                    continue;
                }
            }

            // 按业务设定的特定文件类型进行检测筛选（支持对格式限定的放行）
            let extension = file_path
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| format!(".{}", s.to_lowercase()))
                .unwrap_or_default();

            if !self.config.file_types.contains(&extension) {
                continue;
            }

            let file_path_str = file_path.to_string_lossy().to_string();

            // 正式加载此文档内容前，提前把前台控制面板刷新
            self.emit_progress(&file_path_str);

            // 正式下潜分析此文档内容的违规字符
            let _ = self.scan_file(file_path).await;

            self.files_scanned.fetch_add(1, Ordering::SeqCst);
        }

        Ok(())
    }

    /// 查看对应盘符和文件的完整长目录，比对是不是应该排除的安全敏感文件（比如操作系统保留盘文件等）
    fn should_exclude_path(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy().to_lowercase();

        // 系统默认必定剔除并绕过的非普通应用层安全路径和内核目录
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

        // 客制化的针对自身应用特长配置的白名单安全隔离区
        for exclude_path in &self.config.exclude_paths {
            if path_str.contains(&exclude_path.to_lowercase()) {
                return true;
            }
        }

        false
    }

    /// 加载被选中的涉事嫌疑文件的全文体内容并分类解析
    async fn scan_file(&self, file_path: &Path) -> Result<(), String> {
        let extension = file_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        let mut file_counts = std::collections::HashMap::new();

        let _ = match extension.as_str() {
            "xlsx" | "xls" => self.scan_excel_file(file_path, &mut file_counts).await,
            "csv" | "txt" => self.scan_text_file(file_path, &mut file_counts).await,
            _ => Ok(()),
        };

        // 将涉事词频汇总
        let mut sensitive_types_found = Vec::new();
        let total_matches: u32 = file_counts.values().sum();

        if total_matches >= self.config.min_records_threshold {
            for (t, count) in &file_counts {
                if *count > 0 {
                    sensitive_types_found.push(format!("{:?}", t));
                }
            }

            let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("Unknown").to_string();
            let file_size = fs::metadata(file_path).map(|m| m.len()).unwrap_or(0);
            let file_type = format!(".{}", extension);
            let types_str = sensitive_types_found.join("+");
            let file_path_str = file_path.to_string_lossy().to_string();

            let aggr = crate::models::AggregatedResult {
                file_path: file_path_str,
                file_name,
                file_size,
                file_type,
                sensitive_types: types_str,
                count: total_matches,
            };

            let _ = self.db.insert_sensitive_file(&aggr);
            self.results_found.fetch_add(1, Ordering::SeqCst);
        }

        Ok(())
    }

    /// 分解 Excel 文档格式（包含了 XLSX 新格式方案的分析处理）
    async fn scan_excel_file(&self, file_path: &Path, file_counts: &mut std::collections::HashMap<SensitiveType, u32>) -> Result<(), String> {
        match open_workbook::<Xlsx<_>, _>(file_path) {
            Ok(mut workbook) => {
                for sheet_name in workbook.sheet_names() {
                    if !self.is_running.load(Ordering::SeqCst) {
                        break;
                    }

                    if let Ok(range) = workbook.worksheet_range(&sheet_name) {
                        for row in range.rows() {
                            for cell in row.iter() {
                                let cell_value = cell.to_string();

                                if !cell_value.is_empty() {
                                    if self.check_and_store_result(
                                        &cell_value,
                                        file_counts,
                                    ).await {
                                        return Ok(()); // 超过所设计的容灾熔断点报警下限，直接短路判该文件死刑
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(())
            }
            Err(_) => {
                // 如果发现老式的 XLS 二进制存储引擎也能打开，尝试对 XLS 方案解构
                match open_workbook::<Xls<_>, _>(file_path) {
                    Ok(mut workbook) => {
                        for sheet_name in workbook.sheet_names() {
                            if !self.is_running.load(Ordering::SeqCst) {
                                break;
                            }

                            if let Ok(range) = workbook.worksheet_range(&sheet_name) {
                                for row in range.rows() {
                                    for cell in row.iter() {
                                        let cell_value = cell.to_string();

                                        if !cell_value.is_empty() {
                                            if self.check_and_store_result(
                                                &cell_value,
                                                file_counts,
                                            ).await {
                                                return Ok(()); // 超过该文件阈值并发生短路切离逻辑
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Ok(())
                    }
                    Err(e) => Err(format!("工作表内部遭遇彻底破损或者不可查文件类型: {}", e)),
                }
            }
        }
    }

    /// 使用 UTF-8 或退化为 GBK 解码策略来深入检测纯文本文档
    async fn scan_text_file(&self, file_path: &Path, file_counts: &mut std::collections::HashMap<SensitiveType, u32>) -> Result<(), String> {
        match fs::read(file_path) {
            Ok(bytes) => {
                let content = if let Ok(s) = String::from_utf8(bytes.clone()) {
                    s
                } else {
                    // 如果 UTF-8 的标准无法解析这堆 byte 信息，通过 GB18030 字符集来接管提取能力
                    let (decoded, _, _) = encoding_rs::GB18030.decode(&bytes);
                    decoded.to_string()
                };

                for line in content.lines() {
                    if !self.is_running.load(Ordering::SeqCst) {
                        break;
                    }

                    for cell in line.split(',') {
                        if !cell.trim().is_empty() {
                            if self.check_and_store_result(
                                cell.trim(),
                                file_counts,
                            ).await {
                                return Ok(()); // 只要确认超标即产生熔断保护直接退出探测循环
                            }
                        }
                    }
                }
                Ok(())
            }
            Err(e) => Err(format!("从硬盘或者内存池中拉取出文本内容失败: {}", e)),
        }
    }

    /// 把获取到的碎片或者单项匹配细胞传递到各类正则中检验并自动推论是否发生了超出容限的情况
    /// 当所有的各类总超限数已溢出界定的最坏熔断阈值(大概50)的时候给上游一个极短反馈切分信号 True 
    async fn check_and_store_result(
        &self,
        content: &str,
        file_counts: &mut std::collections::HashMap<SensitiveType, u32>,
    ) -> bool {
        for sensitive_type in &self.config.sensitive_types {
            let detected = match sensitive_type {
                SensitiveType::PhoneNumber => detect_phone_number(content),
                SensitiveType::IdCard => detect_id_card(content),
                SensitiveType::Name => detect_name(content),
                SensitiveType::Address => detect_address(content),
            };

            if detected.is_some() {
                let count = file_counts.entry(*sensitive_type).or_insert(0);
                *count += 1;
                
                let total: u32 = file_counts.values().sum();
                if total >= self.config.min_records_threshold {
                    return true;
                }
            }
        }
        
        false
    }

    /// 向前沿的 GUI 展示出底层已经遍历与扫出总条数的量化值
    pub fn get_stats(&self) -> (u64, u64) {
        (
            self.files_scanned.load(Ordering::SeqCst),
            self.results_found.load(Ordering::SeqCst),
        )
    }
}
