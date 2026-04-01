#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{SensitiveType, ScanConfig};
    use crate::db::Database;
    use crate::whitelist_manager::{WhitelistManager, WhitelistRule};
    use std::sync::Arc;

    // ==================== Pattern Detection Tests ====================
    
    #[test]
    fn test_phone_number_detection_valid() {
        // Valid Chinese phone numbers
        assert!(detect_phone_number("13800138000").is_some(), "Should detect 13800138000");
        assert!(detect_phone_number("15912345678").is_some(), "Should detect 15912345678");
        assert!(detect_phone_number("18698765432").is_some(), "Should detect 18698765432");
    }
    
    #[test]
    fn test_phone_number_detection_invalid() {
        // Invalid phone numbers
        assert!(detect_phone_number("12800138000").is_none(), "Should reject 128 (invalid prefix)");
        assert!(detect_phone_number("1380013800").is_none(), "Should reject 10 digits");
        assert!(detect_phone_number("138001380000").is_none(), "Should reject 12 digits");
        assert!(detect_phone_number("abcdefghijk").is_none(), "Should reject non-numeric");
    }
    
    #[test]
    fn test_phone_number_with_spaces() {
        // Phone numbers with spaces should be detected
        assert!(detect_phone_number("138 0013 8000").is_some(), "Should detect with spaces");
        assert!(detect_phone_number("138-0013-8000").is_some(), "Should detect with dashes");
    }
    
    #[test]
    fn test_phone_number_test_numbers_excluded() {
        // Test numbers should be excluded
        assert!(detect_phone_number("13800138000").is_none(), "Should exclude test number 13800138000");
        assert!(detect_phone_number("13800138001").is_none(), "Should exclude test number 13800138001");
    }
    
    #[test]
    fn test_phone_number_continuous_digits_excluded() {
        // Continuous same digits should be excluded
        assert!(detect_phone_number("11111111111").is_none(), "Should exclude all 1s");
        assert!(detect_phone_number("22222222222").is_none(), "Should exclude all 2s");
    }
    
    #[test]
    fn test_id_card_detection_valid_18_digit() {
        // Valid 18-digit ID cards
        assert!(detect_id_card("110101199003078011").is_some(), "Should detect valid 18-digit ID");
        assert!(detect_id_card("310105198412011234").is_some(), "Should detect another valid 18-digit ID");
    }
    
    #[test]
    fn test_id_card_detection_valid_15_digit() {
        // Valid 15-digit ID cards (old format)
        assert!(detect_id_card("110101900307001").is_some(), "Should detect valid 15-digit ID");
    }
    
    #[test]
    fn test_id_card_detection_invalid() {
        // Invalid ID cards
        assert!(detect_id_card("11010119900307801").is_none(), "Should reject 17 digits");
        assert!(detect_id_card("1101011990030780123").is_none(), "Should reject 19 digits");
        assert!(detect_id_card("abcdefghij12345678").is_none(), "Should reject non-alphanumeric");
    }
    
    #[test]
    fn test_id_card_checksum_validation() {
        // Test checksum validation for 18-digit IDs
        // Valid checksum
        assert!(detect_id_card("110101199003078011").is_some());
        
        // Invalid checksum (last digit wrong)
        assert!(detect_id_card("110101199003078012").is_none());
    }
    
    #[test]
    fn test_name_detection_valid() {
        // Valid Chinese names
        assert!(detect_name("张三").is_some(), "Should detect 2-character name");
        assert!(detect_name("李四").is_some(), "Should detect another 2-character name");
        assert!(detect_name("王小明").is_some(), "Should detect 3-character name");
        assert!(detect_name("欧阳修").is_some(), "Should detect 3-character name with compound surname");
    }
    
    #[test]
    fn test_name_detection_invalid() {
        // Invalid names
        assert!(detect_name("张").is_none(), "Should reject single character");
        assert!(detect_name("张三丰").is_none(), "Should reject 4 characters");
        assert!(detect_name("张三丰李").is_none(), "Should reject 5 characters");
        assert!(detect_name("ZhangSan").is_none(), "Should reject non-Chinese characters");
    }
    
    #[test]
    fn test_name_exclusion_words() {
        // Common exclusion words should not be detected as names
        assert!(detect_name("测试").is_none(), "Should exclude '测试'");
        assert!(detect_name("示例").is_none(), "Should exclude '示例'");
        assert!(detect_name("样本").is_none(), "Should exclude '样本'");
        assert!(detect_name("数据").is_none(), "Should exclude '数据'");
        assert!(detect_name("用户").is_none(), "Should exclude '用户'");
    }
    
    #[test]
    fn test_address_detection_valid() {
        // Valid addresses with province keywords
        assert!(detect_address("北京市朝阳区建国路88号").is_some(), "Should detect Beijing address");
        assert!(detect_address("上海市浦东新区陆家嘴环路1000号").is_some(), "Should detect Shanghai address");
        assert!(detect_address("广东省深圳市南山区科技园").is_some(), "Should detect Guangdong address");
    }
    
    #[test]
    fn test_address_detection_invalid() {
        // Invalid addresses
        assert!(detect_address("123456").is_none(), "Should reject pure numbers");
        assert!(detect_address("abcdefghijk").is_none(), "Should reject pure letters");
        assert!(detect_address("北京").is_none(), "Should reject too short (<5 chars)");
    }
    
    #[test]
    fn test_address_without_province() {
        // Addresses without province keywords should be rejected
        assert!(detect_address("朝阳区建国路88号").is_none(), "Should reject address without province");
        assert!(detect_address("建国路88号").is_none(), "Should reject address without province");
    }
    
    #[test]
    fn test_preprocess_text() {
        // Test text preprocessing
        assert_eq!(preprocess_text("138 0000 0000"), "1380000000", "Should remove spaces");
        assert_eq!(preprocess_text("138-0000-0000"), "1380000000", "Should remove dashes");
        assert_eq!(preprocess_text("  test  "), "test", "Should trim whitespace");
        assert_eq!(preprocess_text("test_test"), "testtest", "Should remove underscores");
    }
    
    #[test]
    fn test_mask_content_phone() {
        // Test phone number masking
        assert_eq!(mask_content("13800138000", SensitiveType::PhoneNumber), "138****8000");
        assert_eq!(mask_content("15912345678", SensitiveType::PhoneNumber), "159****5678");
    }
    
    #[test]
    fn test_mask_content_id_card() {
        // Test ID card masking
        assert_eq!(mask_content("110101199003078011", SensitiveType::IdCard), "1101****8011");
        assert_eq!(mask_content("310105198412011234", SensitiveType::IdCard), "3101****1234");
    }
    
    #[test]
    fn test_mask_content_name() {
        // Test name masking
        assert_eq!(mask_content("张三", SensitiveType::Name), "张*");
        assert_eq!(mask_content("王小明", SensitiveType::Name), "王*");
    }
    
    #[test]
    fn test_mask_content_address() {
        // Test address masking
        assert_eq!(
            mask_content("北京市朝阳区建国路88号", SensitiveType::Address),
            "北京市...88号"
        );
        assert_eq!(
            mask_content("上海市", SensitiveType::Address),
            "****"
        );
    }
    
    // ==================== Database Tests ====================
    
    #[test]
    fn test_database_initialization() {
        // Test database can be initialized
        let db = Database::new();
        assert!(db.is_ok(), "Database should initialize successfully");
    }
    
    #[test]
    fn test_database_insert_scan_result() {
        // Test inserting scan results
        let db = Database::new().unwrap();
        
        let result = crate::models::ScanResult {
            id: uuid::Uuid::new_v4().to_string(),
            file_path: "/test/file.xlsx".to_string(),
            sheet_name: Some("Sheet1".to_string()),
            row: 1,
            column: 1,
            sensitive_type: SensitiveType::PhoneNumber,
            content: "13800138000".to_string(),
            masked_content: "138****8000".to_string(),
            found_at: chrono::Utc::now(),
        };
        
        let result = db.insert_scan_result(&result);
        assert!(result.is_ok(), "Should insert scan result successfully");
    }
    
    #[test]
    fn test_database_get_scan_results() {
        // Test retrieving scan results
        let db = Database::new().unwrap();
        
        // Insert a test result
        let test_result = crate::models::ScanResult {
            id: uuid::Uuid::new_v4().to_string(),
            file_path: "/test/file.xlsx".to_string(),
            sheet_name: Some("Sheet1".to_string()),
            row: 1,
            column: 1,
            sensitive_type: SensitiveType::PhoneNumber,
            content: "13800138000".to_string(),
            masked_content: "138****8000".to_string(),
            found_at: chrono::Utc::now(),
        };
        
        db.insert_scan_result(&test_result).unwrap();
        
        // Retrieve results
        let results = db.get_scan_results(Some(10), None, None, None);
        assert!(results.is_ok(), "Should retrieve scan results successfully");
        assert!(!results.unwrap().is_empty(), "Should have at least one result");
    }
    
    #[test]
    fn test_database_count_scan_results() {
        // Test counting scan results
        let db = Database::new().unwrap();
        
        let count = db.count_scan_results();
        assert!(count.is_ok(), "Should count scan results successfully");
    }
    
    #[test]
    fn test_database_whitelist_operations() {
        // Test whitelist CRUD operations
        let db = Database::new().unwrap();
        
        // Add whitelist entry
        let entry = crate::models::WhitelistEntry {
            id: uuid::Uuid::new_v4().to_string(),
            content: "13800138000".to_string(),
            sensitive_type: SensitiveType::PhoneNumber,
            description: Some("Test number".to_string()),
            created_at: chrono::Utc::now(),
        };
        
        assert!(db.add_whitelist(&entry).is_ok(), "Should add whitelist entry");
        
        // Get whitelist
        let whitelist = db.get_whitelist();
        assert!(whitelist.is_ok(), "Should retrieve whitelist");
        assert!(!whitelist.unwrap().is_empty(), "Should have whitelist entries");
        
        // Delete whitelist entry
        assert!(db.delete_whitelist(&entry.id).is_ok(), "Should delete whitelist entry");
    }
    
    // ==================== Whitelist Manager Tests ====================
    
    #[test]
    fn test_whitelist_manager_initialization() {
        // Test whitelist manager can be initialized
        let manager = WhitelistManager::new();
        assert_eq!(manager.get_all_rules().unwrap().len(), 0, "Should start with no rules");
    }
    
    #[test]
    fn test_whitelist_add_exact_match_rule() {
        // Test adding exact match rule
        let manager = WhitelistManager::new();
        
        let rule = WhitelistRule::from_entry(crate::models::WhitelistEntry {
            id: "1".to_string(),
            content: "13800138000".to_string(),
            sensitive_type: SensitiveType::PhoneNumber,
            description: Some("Test number".to_string()),
            created_at: chrono::Utc::now(),
        });
        
        assert!(manager.add_rule(rule).is_ok(), "Should add exact match rule");
        assert_eq!(manager.get_all_rules().unwrap().len(), 1, "Should have 1 rule");
    }
    
    #[test]
    fn test_whitelist_add_regex_rule() {
        // Test adding regex rule
        let manager = WhitelistManager::new();
        
        let rule = WhitelistRule::new_regex(
            r"^13800\d{5}$".to_string(),
            SensitiveType::PhoneNumber,
            Some("Test pattern".to_string())
        );
        
        assert!(rule.is_ok(), "Should create regex rule");
        assert!(manager.add_rule(rule.unwrap()).is_ok(), "Should add regex rule");
    }
    
    #[test]
    fn test_whitelist_exact_match() {
        // Test exact matching
        let manager = WhitelistManager::new();
        
        let rule = WhitelistRule::from_entry(crate::models::WhitelistEntry {
            id: "1".to_string(),
            content: "13800138000".to_string(),
            sensitive_type: SensitiveType::PhoneNumber,
            description: Some("Test number".to_string()),
            created_at: chrono::Utc::now(),
        });
        
        manager.add_rule(rule).unwrap();
        
        assert!(manager.is_whitelisted("13800138000", SensitiveType::PhoneNumber), "Should match exact");
        assert!(!manager.is_whitelisted("13800138001", SensitiveType::PhoneNumber), "Should not match different");
    }
    
    #[test]
    fn test_whitelist_regex_match() {
        // Test regex matching
        let manager = WhitelistManager::new();
        
        let rule = WhitelistRule::new_regex(
            r"^13800\d{5}$".to_string(),
            SensitiveType::PhoneNumber,
            Some("Test pattern".to_string())
        ).unwrap();
        
        manager.add_rule(rule).unwrap();
        
        assert!(manager.is_whitelisted("13800138000", SensitiveType::PhoneNumber), "Should match pattern");
        assert!(manager.is_whitelisted("13800138001", SensitiveType::PhoneNumber), "Should match pattern");
        assert!(!manager.is_whitelisted("13900138000", SensitiveType::PhoneNumber), "Should not match different pattern");
    }
    
    #[test]
    fn test_whitelist_type_filtering() {
        // Test that whitelist only matches same type
        let manager = WhitelistManager::new();
        
        let rule = WhitelistRule::from_entry(crate::models::WhitelistEntry {
            id: "1".to_string(),
            content: "13800138000".to_string(),
            sensitive_type: SensitiveType::PhoneNumber,
            description: Some("Test number".to_string()),
            created_at: chrono::Utc::now(),
        });
        
        manager.add_rule(rule).unwrap();
        
        // Should match phone number type
        assert!(manager.is_whitelisted("13800138000", SensitiveType::PhoneNumber));
        
        // Should not match ID card type
        assert!(!manager.is_whitelisted("13800138000", SensitiveType::IdCard));
    }
    
    #[test]
    fn test_whitelist_enabled_toggle() {
        // Test enable/disable functionality
        let manager = WhitelistManager::new();
        
        let rule = WhitelistRule::from_entry(crate::models::WhitelistEntry {
            id: "1".to_string(),
            content: "13800138000".to_string(),
            sensitive_type: SensitiveType::PhoneNumber,
            description: Some("Test number".to_string()),
            created_at: chrono::Utc::now(),
        });
        
        manager.add_rule(rule).unwrap();
        
        // Should match when enabled
        assert!(manager.is_whitelisted("13800138000", SensitiveType::PhoneNumber));
        
        // Disable rule
        assert!(manager.update_rule("1", false).is_ok(), "Should disable rule");
        
        // Should not match when disabled
        assert!(!manager.is_whitelisted("13800138000", SensitiveType::PhoneNumber));
        
        // Re-enable rule
        assert!(manager.update_rule("1", true).is_ok(), "Should enable rule");
        
        // Should match again when enabled
        assert!(manager.is_whitelisted("13800138000", SensitiveType::PhoneNumber));
    }
    
    #[test]
    fn test_whitelist_remove_rule() {
        // Test removing rules
        let manager = WhitelistManager::new();
        
        let rule = WhitelistRule::from_entry(crate::models::WhitelistEntry {
            id: "1".to_string(),
            content: "13800138000".to_string(),
            sensitive_type: SensitiveType::PhoneNumber,
            description: Some("Test number".to_string()),
            created_at: chrono::Utc::now(),
        });
        
        manager.add_rule(rule).unwrap();
        assert_eq!(manager.get_all_rules().unwrap().len(), 1);
        
        assert!(manager.remove_rule("1").is_ok(), "Should remove rule");
        assert_eq!(manager.get_all_rules().unwrap().len(), 0, "Should have no rules after removal");
    }
    
    #[test]
    fn test_whitelist_export_import() {
        // Test export/import functionality
        let manager = WhitelistManager::new();
        
        let rule = WhitelistRule::from_entry(crate::models::WhitelistEntry {
            id: "1".to_string(),
            content: "13800138000".to_string(),
            sensitive_type: SensitiveType::PhoneNumber,
            description: Some("Test number".to_string()),
            created_at: chrono::Utc::now(),
        });
        
        manager.add_rule(rule).unwrap();
        
        // Export
        let exported = manager.export_rules();
        assert!(exported.is_ok(), "Should export rules");
        
        // Clear and import
        manager.clear_all().unwrap();
        assert_eq!(manager.get_all_rules().unwrap().len(), 0);
        
        let imported_rules: Vec<WhitelistRule> = serde_json::from_str(&exported.unwrap()).unwrap();
        assert!(manager.import_rules(imported_rules).is_ok(), "Should import rules");
        assert_eq!(manager.get_all_rules().unwrap().len(), 1, "Should have 1 rule after import");
    }
    
    // ==================== Error Handling Tests ====================
    
    #[test]
    fn test_apperror_from_io_error() {
        // Test conversion from io::Error to AppError
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let app_error: crate::error::AppError = io_error.into();
        
        assert_eq!(app_error.category(), "FILE_IO");
        assert!(app_error.is_recoverable());
    }
    
    #[test]
    fn test_apperror_from_regex_error() {
        // Test conversion from regex::Error to AppError
        let regex_error = regex::Error::Syntax("Invalid regex".to_string());
        let app_error: crate::error::AppError = regex_error.into();
        
        assert_eq!(app_error.category(), "REGEX");
    }
    
    #[test]
    fn test_apperror_user_message() {
        // Test user-friendly error messages
        let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");
        let app_error: crate::error::AppError = io_error.into();
        
        assert_eq!(app_error.user_message(), "没有文件访问权限");
    }
    
    // ==================== Integration Tests ====================
    
    #[test]
    fn test_scan_with_whitelist_integration() {
        // Test scanning with whitelist filtering
        let manager = WhitelistManager::new();
        
        // Add whitelist rule
        let rule = WhitelistRule::from_entry(crate::models::WhitelistEntry {
            id: "1".to_string(),
            content: "13800138000".to_string(),
            sensitive_type: SensitiveType::PhoneNumber,
            description: Some("Test number".to_string()),
            created_at: chrono::Utc::now(),
        });
        
        manager.add_rule(rule).unwrap();
        
        // Test that whitelisted content is detected
        assert!(manager.is_whitelisted("13800138000", SensitiveType::PhoneNumber));
        
        // Test that non-whitelisted content is not detected
        assert!(!manager.is_whitelisted("13800138001", SensitiveType::PhoneNumber));
    }
}
