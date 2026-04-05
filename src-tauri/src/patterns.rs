use regex::Regex;
use lazy_static::lazy_static;
use crate::models::SensitiveType;

lazy_static! {
    // 手机号码匹配规则: 1[3-9]\d{9}
    static ref PHONE_PATTERN: Regex = Regex::new(r"^1[3-9]\d{9}$").unwrap();
    
    // 身份证号码匹配规则: 18位数字，或17位数字+X
    static ref ID_CARD_PATTERN: Regex = Regex::new(r"^(\d{17}[\dXx]|\d{15})$").unwrap();
    
    // 中文姓名匹配规则: 2-4个中文字符
    static ref NAME_PATTERN: Regex = Regex::new(r"^[\u4e00-\u9fff]{2,4}$").unwrap();
    
    // 地址匹配规则: 包含省/市/区等关键字
    static ref ADDRESS_PATTERN: Regex = Regex::new(
        r"(北京|上海|天津|重庆|河北|山西|辽宁|吉林|黑龙江|江苏|浙江|安徽|福建|江西|山东|河南|湖北|湖南|广东|广西|海南|四川|贵州|云南|西藏|陕西|甘肃|青海|宁夏|新疆|台湾|香港|澳门|内蒙古)" 
    ).unwrap();
}

/// 要排除的测试用手机号码
#[allow(dead_code)]
const TEST_PHONE_NUMBERS: &[&str] = &["13800138000", "13800138001"];

/// 要排除的常见非姓名词汇
#[allow(dead_code)]
const EXCLUDE_NAMES: &[&str] = &["测试", "示例", "样本", "数据", "用户", "客户", "商户"];

/// 对文本进行预处理：移除空格、破折号以及非可见字符
#[allow(dead_code)]
pub fn preprocess_text(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_whitespace() && *c != '-' && *c != '—' && *c != '_')
        .collect::<String>()
        .trim()
        .to_string()
}

/// 对用于展示的敏感内容进行部分掩码隐藏
#[allow(dead_code)]
pub fn mask_content(content: &str, sensitive_type: SensitiveType) -> String {
    let chars: Vec<char> = content.chars().collect();
    let len = chars.len();

    match sensitive_type {
        SensitiveType::PhoneNumber => {
            if len >= 11 {
                let prefix: String = chars[0..3].iter().collect();
                let suffix: String = chars[len - 4..].iter().collect();
                format!("{}****{}", prefix, suffix)
            } else {
                "****".to_string()
            }
        }
        SensitiveType::IdCard => {
            if len >= 8 {
                let prefix: String = chars[0..4].iter().collect();
                let suffix: String = chars[len - 4..].iter().collect();
                format!("{}****{}", prefix, suffix)
            } else {
                "****".to_string()
            }
        }
        SensitiveType::Name => {
            if len >= 2 {
                let prefix: String = chars[0..1].iter().collect();
                format!("{}*", prefix)
            } else {
                "*".to_string()
            }
        }
        SensitiveType::Address => {
            if len > 10 {
                let prefix: String = chars[0..5].iter().collect();
                let suffix: String = chars[len - 5..].iter().collect();
                format!("{}...{}", prefix, suffix)
            } else {
                "****".to_string()
            }
        }
    }
}

/// 识别手机号码
#[allow(dead_code)]
pub fn detect_phone_number(text: &str) -> Option<String> {
    let processed = preprocess_text(text);
    
    // 检查是否匹配正则格式
    if !PHONE_PATTERN.is_match(&processed) {
        return None;
    }
    
    // 排除特定测试号码
    if TEST_PHONE_NUMBERS.contains(&processed.as_str()) {
        return None;
    }
    
    // 排除全一样字的号码（如 11111111111）
    if processed.chars().all(|c| c == processed.chars().next().unwrap()) {
        return None;
    }
    
    Some(processed)
}

/// 验证身份证校验码 (按照 GB 11643-1999)
#[allow(dead_code)]
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

/// 识别身份证号码
#[allow(dead_code)]
pub fn detect_id_card(text: &str) -> Option<String> {
    let processed = preprocess_text(text).to_uppercase();
    
    // 检查是否匹配正则格式
    if !ID_CARD_PATTERN.is_match(&processed) {
        return None;
    }
    
    // 对18位长度通过算法进行校验码检查
    if processed.len() == 18 {
        if !validate_id_card_checksum(&processed) {
            return None;
        }
    }
    
    Some(processed)
}

/// 识别姓名
#[allow(dead_code)]
pub fn detect_name(text: &str) -> Option<String> {
    let processed = preprocess_text(text);
    
    // 检查是否匹配正则格式（2-4个中文字符）
    if !NAME_PATTERN.is_match(&processed) {
        return None;
    }
    
    // 排除常用不属姓名范畴的单词
    if EXCLUDE_NAMES.iter().any(|&name| name == processed) {
        return None;
    }
    
    Some(processed)
}

/// 识别地址
#[allow(dead_code)]
pub fn detect_address(text: &str) -> Option<String> {
    let processed = preprocess_text(text);
    
    // 必须包含省、市等前置关键字
    if !ADDRESS_PATTERN.is_match(&processed) {
        return None;
    }
    
    // 不能全为数字或纯字母组成
    if processed.chars().all(|c| c.is_numeric() || c.is_ascii_alphabetic()) {
        return None;
    }
    
    // 必须至少具有 5 个字符长度
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
        assert!(detect_phone_number("13800138000").is_none()); // 测试号
        assert!(detect_phone_number("11111111111").is_none()); // 连续数字
        assert!(detect_phone_number("138 0000 0000").is_some()); // 带有空格的识别
    }
    
    #[test]
    fn test_id_card_detection() {
        // 注意：校验实际受限于校验算法要求
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
