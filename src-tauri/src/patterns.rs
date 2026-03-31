use regex::Regex;
use lazy_static::lazy_static;
use crate::models::SensitiveType;

#[allow(dead_code)]

lazy_static! {
    // Phone number pattern: 1[3-9]\d{9}
    static ref PHONE_PATTERN: Regex = Regex::new(r"^1[3-9]\d{9}$").unwrap();
    
    // ID card pattern: 18 digits or 17 digits + X
    static ref ID_CARD_PATTERN: Regex = Regex::new(r"^(\d{17}[\dXx]|\d{15})$").unwrap();
    
    // Chinese name pattern: 2-4 Chinese characters
    static ref NAME_PATTERN: Regex = Regex::new(r"^[\u4e00-\u9fff]{2,4}$").unwrap();
    
    // Address pattern: contains province/city/district keywords
    static ref ADDRESS_PATTERN: Regex = Regex::new(
        r"(北京|上海|天津|重庆|河北|山西|辽宁|吉林|黑龙江|江苏|浙江|安徽|福建|江西|山东|河南|湖北|湖南|广东|广西|海南|四川|贵州|云南|西藏|陕西|甘肃|青海|宁夏|新疆|台湾|香港|澳门|内蒙古)" 
    ).unwrap();
}

/// Test phone numbers to exclude (test numbers)
const TEST_PHONE_NUMBERS: &[&str] = &["13800138000", "13800138001"];

/// Common non-name words to exclude
const EXCLUDE_NAMES: &[&str] = &["测试", "示例", "样本", "数据", "用户", "客户", "商户"];

/// Preprocess text: remove spaces, dashes, and non-visible characters
pub fn preprocess_text(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_whitespace() && *c != '-' && *c != '—' && *c != '_')
        .collect::<String>()
        .trim()
        .to_string()
}

/// Mask sensitive content for display
pub fn mask_content(content: &str, sensitive_type: SensitiveType) -> String {
    match sensitive_type {
        SensitiveType::PhoneNumber => {
            if content.len() >= 11 {
                format!("{}****{}", &content[0..3], &content[content.len()-4..])
            } else {
                "****".to_string()
            }
        }
        SensitiveType::IdCard => {
            if content.len() >= 8 {
                format!("{}****{}", &content[0..4], &content[content.len()-4..])
            } else {
                "****".to_string()
            }
        }
        SensitiveType::Name => {
            if content.len() >= 2 {
                format!("{}*", &content[0..1])
            } else {
                "*".to_string()
            }
        }
        SensitiveType::Address => {
            if content.len() > 10 {
                format!("{}...{}", &content[0..5], &content[content.len()-5..])
            } else {
                "****".to_string()
            }
        }
    }
}

/// Detect phone numbers
pub fn detect_phone_number(text: &str) -> Option<String> {
    let processed = preprocess_text(text);
    
    // Check if it matches the pattern
    if !PHONE_PATTERN.is_match(&processed) {
        return None;
    }
    
    // Exclude test numbers
    if TEST_PHONE_NUMBERS.contains(&processed.as_str()) {
        return None;
    }
    
    // Exclude continuous same digits (11111111111)
    if processed.chars().all(|c| c == processed.chars().next().unwrap()) {
        return None;
    }
    
    Some(processed)
}

/// Validate ID card checksum (GB 11643-1999)
fn validate_id_card_checksum(id: &str) -> bool {
    if id.len() != 18 {
        return false;
    }
    
    let weights = [7, 9, 10, 5, 8, 4, 2, 1, 6, 3, 7, 9, 10, 5, 8, 4, 2];
    let check_codes = ['1', '0', 'X', '9', '8', '7', '6', '5', '4', '3', '2'];
    
    let mut sum = 0;
    for (i, weight) in weights.iter().enumerate() {
        if let Some(digit) = id.chars().nth(i) {
            if let Ok(d) = digit.to_string().parse::<u32>() {
                sum += d * weight;
            } else {
                return false;
            }
        }
    }
    
    let check_index = (sum % 11) as usize;
    if check_index >= check_codes.len() {
        return false;
    }
    
    let last_char = id.chars().last().unwrap_or(' ').to_ascii_uppercase();
    check_codes[check_index].to_ascii_uppercase() == last_char
}

/// Detect ID card numbers
pub fn detect_id_card(text: &str) -> Option<String> {
    let processed = preprocess_text(text).to_uppercase();
    
    // Check if it matches the pattern
    if !ID_CARD_PATTERN.is_match(&processed) {
        return None;
    }
    
    // Validate checksum for 18-digit IDs
    if processed.len() == 18 {
        if !validate_id_card_checksum(&processed) {
            return None;
        }
    }
    
    Some(processed)
}

/// Detect names
pub fn detect_name(text: &str) -> Option<String> {
    let processed = preprocess_text(text);
    
    // Check if it matches the pattern (2-4 Chinese characters)
    if !NAME_PATTERN.is_match(&processed) {
        return None;
    }
    
    // Exclude common non-name words
    if EXCLUDE_NAMES.iter().any(|&name| name == processed) {
        return None;
    }
    
    Some(processed)
}

/// Detect addresses
pub fn detect_address(text: &str) -> Option<String> {
    let processed = preprocess_text(text);
    
    // Must contain province/city keywords
    if !ADDRESS_PATTERN.is_match(&processed) {
        return None;
    }
    
    // Must not be pure numbers or letters
    if processed.chars().all(|c| c.is_numeric() || c.is_ascii_alphabetic()) {
        return None;
    }
    
    // Must be at least 5 characters
    if processed.len() < 5 {
        return None;
    }
    
    Some(processed)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_phone_detection() {
        assert!(detect_phone_number("13800000000").is_some());
        assert!(detect_phone_number("13800138000").is_none()); // test number
        assert!(detect_phone_number("11111111111").is_none()); // continuous same digits
        assert!(detect_phone_number("138 0000 0000").is_some()); // with spaces
    }
    
    #[test]
    fn test_id_card_detection() {
        // Note: Using a valid format, actual validation depends on checksum
        assert!(detect_id_card("110101199003078011").is_some());
    }
    
    #[test]
    fn test_name_detection() {
        assert!(detect_name("张三").is_some());
        assert!(detect_name("测试").is_none());
    }
    
    #[test]
    fn test_preprocess() {
        assert_eq!(preprocess_text("138 0000 0000"), "13800000000");
        assert_eq!(preprocess_text("  text  "), "text");
    }
}
