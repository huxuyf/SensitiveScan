use thiserror::Error;

/// Application error types
#[derive(Error, Debug)]
pub enum AppError {
    #[error("File I/O error: {0}")]
    FileIo(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    #[error("JSON serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Excel parsing error: {0}")]
    Excel(String),

    #[error("Scan error: {0}")]
    Scan(String),

    #[error("Whitelist error: {0}")]
    Whitelist(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Path error: {0}")]
    Path(String),

    #[error("Permission denied: {0}")]
    Permission(String),

    #[error("Operation cancelled")]
    Cancelled,

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl AppError {
    /// Get error category for logging
    pub fn category(&self) -> &'static str {
        match self {
            AppError::FileIo(_) => "FILE_IO",
            AppError::Database(_) => "DATABASE",
            AppError::Regex(_) => "REGEX",
            AppError::Json(_) => "JSON",
            AppError::Excel(_) => "EXCEL",
            AppError::Scan(_) => "SCAN",
            AppError::Whitelist(_) => "WHITELIST",
            AppError::Config(_) => "CONFIG",
            AppError::Path(_) => "PATH",
            AppError::Permission(_) => "PERMISSION",
            AppError::Cancelled => "CANCELLED",
            AppError::Timeout(_) => "TIMEOUT",
            AppError::InvalidInput(_) => "INVALID_INPUT",
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::Internal(_) => "INTERNAL",
        }
    }

    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            AppError::FileIo(_)
                | AppError::Excel(_)
                | AppError::Timeout(_)
                | AppError::Scan(_)
        )
    }

    /// Check if error should be logged to file
    pub fn should_log(&self) -> bool {
        !matches!(self, AppError::Cancelled)
    }

    /// Get user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            AppError::FileIo(e) => {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    "没有文件访问权限".to_string()
                } else if e.kind() == std::io::ErrorKind::NotFound {
                    "文件或目录不存在".to_string()
                } else {
                    format!("文件操作失败: {}", e)
                }
            }
            AppError::Database(e) => {
                format!("数据库错误: {}", e)
            }
            AppError::Regex(e) => {
                format!("正则表达式错误: {}", e)
            }
            AppError::Json(e) => {
                format!("数据格式错误: {}", e)
            }
            AppError::Excel(msg) => {
                format!("Excel 文件解析失败: {}", msg)
            }
            AppError::Scan(msg) => {
                format!("扫描错误: {}", msg)
            }
            AppError::Whitelist(msg) => {
                format!("白名单错误: {}", msg)
            }
            AppError::Config(msg) => {
                format!("配置错误: {}", msg)
            }
            AppError::Path(msg) => {
                format!("路径错误: {}", msg)
            }
            AppError::Permission(msg) => {
                format!("权限错误: {}", msg)
            }
            AppError::Cancelled => {
                "操作已取消".to_string()
            }
            AppError::Timeout(msg) => {
                format!("操作超时: {}", msg)
            }
            AppError::InvalidInput(msg) => {
                format!("输入无效: {}", msg)
            }
            AppError::NotFound(msg) => {
                format!("未找到: {}", msg)
            }
            AppError::Internal(msg) => {
                format!("系统错误: {}", msg)
            }
        }
    }
}

/// Result type alias for convenience
pub type AppResult<T> = Result<T, AppError>;

/// Convert from calamine error
impl From<calamine::Error> for AppError {
    fn from(err: calamine::Error) -> Self {
        AppError::Excel(err.to_string())
    }
}

/// Convert from anyhow error
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_category() {
        let err = AppError::FileIo(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "test"
        ));
        assert_eq!(err.category(), "FILE_IO");
    }

    #[test]
    fn test_recoverable() {
        assert!(AppError::FileIo(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "test"
        )).is_recoverable());

        assert!(!AppError::Database(rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(1),
            None
        )).is_recoverable());
    }

    #[test]
    fn test_user_message() {
        let err = AppError::FileIo(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "test"
        ));
        assert_eq!(err.user_message(), "没有文件访问权限");
    }
}
