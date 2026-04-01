use chrono::{DateTime, Utc};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use crate::error::{AppError, AppResult};

/// Log levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Error,
    Warning,
    Info,
    Debug,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Error => "ERROR",
            LogLevel::Warning => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
        }
    }
}

/// Log entry
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub category: String,
    pub message: String,
    pub context: Option<String>,
}

/// Logger configuration
#[derive(Debug, Clone)]
pub struct LoggerConfig {
    pub log_file_path: PathBuf,
    pub max_file_size: u64,
    pub max_backup_files: usize,
    pub console_output: bool,
    pub file_output: bool,
    pub min_level: LogLevel,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        let config_dir = if cfg!(target_os = "windows") {
            dirs::config_dir()
                .map(|p| p.join("SensitiveScanner"))
                .unwrap_or_else(|| PathBuf::from("./logs"))
        } else if cfg!(target_os = "macos") {
            dirs::home_dir()
                .map(|p| p.join("Library/Logs/SensitiveScanner"))
                .unwrap_or_else(|| PathBuf::from("./logs"))
        } else {
            dirs::config_dir()
                .map(|p| p.join("sensitive-scanner/logs"))
                .unwrap_or_else(|| PathBuf::from("./logs"))
        };

        std::fs::create_dir_all(&config_dir).ok();

        Self {
            log_file_path: config_dir.join("scanner.log"),
            max_file_size: 10 * 1024 * 1024, // 10MB
            max_backup_files: 5,
            console_output: true,
            file_output: true,
            min_level: LogLevel::Info,
        }
    }
}

/// Logger implementation
pub struct Logger {
    config: LoggerConfig,
    file_handle: Mutex<Option<File>>,
}

impl Logger {
    pub fn new(config: LoggerConfig) -> AppResult<Self> {
        let logger = Self {
            config,
            file_handle: Mutex::new(None),
        };

        if logger.config.file_output {
            logger.open_log_file()?;
        }

        Ok(logger)
    }

    /// Open log file with rotation support
    fn open_log_file(&self) -> AppResult<()> {
        // Check if file exists and needs rotation
        if self.config.log_file_path.exists() {
            if let Ok(metadata) = std::fs::metadata(&self.config.log_file_path) {
                if metadata.len() > self.config.max_file_size {
                    self.rotate_log_files()?;
                }
            }
        }

        // Open or create log file
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.config.log_file_path)?;

        *self.file_handle.lock().map_err(|e| {
            AppError::Internal(format!("Failed to acquire lock: {}", e))
        })? = Some(file);

        Ok(())
    }

    /// Rotate log files
    fn rotate_log_files(&self) -> AppResult<()> {
        // Remove oldest backup if exists
        let oldest_backup = self.config.log_file_path.with_extension(&format!(
            "log.{}",
            self.config.max_backup_files
        ));
        if oldest_backup.exists() {
            std::fs::remove_file(&oldest_backup)?;
        }

        // Rotate existing backups
        for i in (1..self.config.max_backup_files).rev() {
            let old_path = self.config.log_file_path.with_extension(&format!("log.{}", i));
            let new_path = self.config.log_file_path.with_extension(&format!("log.{}", i + 1));

            if old_path.exists() {
                std::fs::rename(&old_path, &new_path)?;
            }
        }

        // Move current log to .log.1
        let backup_path = self.config.log_file_path.with_extension("log.1");
        std::fs::rename(&self.config.log_file_path, &backup_path)?;

        Ok(())
    }

    /// Write log entry
    fn write_log(&self, entry: &LogEntry) -> AppResult<()> {
        let log_line = self.format_log_entry(entry);

        // Console output
        if self.config.console_output {
            println!("{}", log_line);
        }

        // File output
        if self.config.file_output {
            if let Some(mut file) = self.file_handle.lock().map_err(|e| {
                AppError::Internal(format!("Failed to acquire lock: {}", e))
            })?.as_mut() {
                writeln!(file, "{}", log_line)?;

                // Flush to ensure logs are written immediately
                file.flush()?;
            }
        }

        Ok(())
    }

    /// Format log entry
    fn format_log_entry(&self, entry: &LogEntry) -> String {
        let timestamp = entry.timestamp.format("%Y-%m-%d %H:%M:%S%.3f UTC");
        let context = entry.context.as_ref().map(|c| c.as_str()).unwrap_or("");

        format!(
            "[{}][{}][{}] {} {}",
            timestamp,
            entry.level.as_str(),
            entry.category,
            entry.message,
            if context.is_empty() {
                String::new()
            } else {
                format!("| Context: {}", context)
            }
        )
    }

    /// Log at error level
    pub fn error(&self, category: &str, message: &str, context: Option<&str>) {
        if self.config.min_level <= LogLevel::Error {
            let entry = LogEntry {
                timestamp: Utc::now(),
                level: LogLevel::Error,
                category: category.to_string(),
                message: message.to_string(),
                context: context.map(|s| s.to_string()),
            };
            let _ = self.write_log(&entry);
        }
    }

    /// Log at warning level
    pub fn warning(&self, category: &str, message: &str, context: Option<&str>) {
        if self.config.min_level <= LogLevel::Warning {
            let entry = LogEntry {
                timestamp: Utc::now(),
                level: LogLevel::Warning,
                category: category.to_string(),
                message: message.to_string(),
                context: context.map(|s| s.to_string()),
            };
            let _ = self.write_log(&entry);
        }
    }

    /// Log at info level
    pub fn info(&self, category: &str, message: &str, context: Option<&str>) {
        if self.config.min_level <= LogLevel::Info {
            let entry = LogEntry {
                timestamp: Utc::now(),
                level: LogLevel::Info,
                category: category.to_string(),
                message: message.to_string(),
                context: context.map(|s| s.to_string()),
            };
            let _ = self.write_log(&entry);
        }
    }

    /// Log at debug level
    pub fn debug(&self, category: &str, message: &str, context: Option<&str>) {
        if self.config.min_level <= LogLevel::Debug {
            let entry = LogEntry {
                timestamp: Utc::now(),
                level: LogLevel::Debug,
                category: category.to_string(),
                message: message.to_string(),
                context: context.map(|s| s.to_string()),
            };
            let _ = self.write_log(&entry);
        }
    }

    /// Log error with AppError
    pub fn log_error(&self, error: &AppError, context: Option<&str>) {
        if error.should_log() && self.config.min_level <= LogLevel::Error {
            self.error(
                error.category(),
                &error.user_message(),
                Some(&format!("Details: {}", error)),
            );
        }
    }

    /// Get log file path
    pub fn log_file_path(&self) -> &PathBuf {
        &self.config.log_file_path
    }

    /// Read recent log entries
    pub fn read_recent_logs(&self, lines: usize) -> AppResult<Vec<String>> {
        if !self.config.log_file_path.exists() {
            return Ok(Vec::new());
        }

        let content = std::fs::read_to_string(&self.config.log_file_path)?;
        let all_lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

        let start = if all_lines.len() > lines {
            all_lines.len() - lines
        } else {
            0
        };

        Ok(all_lines[start..].to_vec())
    }

    /// Clear log file
    pub fn clear_logs(&self) -> AppResult<()> {
        if self.config.log_file_path.exists() {
            std::fs::write(&self.config.log_file_path, "")?;
        }
        Ok(())
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new(LoggerConfig::default()).expect("Failed to create logger")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_levels() {
        assert_eq!(LogLevel::Error.as_str(), "ERROR");
        assert_eq!(LogLevel::Info.as_str(), "INFO");
    }

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Error > LogLevel::Warning);
        assert!(LogLevel::Warning > LogLevel::Info);
        assert!(LogLevel::Info > LogLevel::Debug);
    }

    #[test]
    fn test_logger_creation() {
        let config = LoggerConfig::default();
        let logger = Logger::new(config);
        assert!(logger.is_ok());
    }
}
