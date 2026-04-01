use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::models::{SensitiveType, WhitelistEntry};
use regex::Regex;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::RwLock;

/// Whitelist entry with regex support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistRule {
    pub id: String,
    pub content: String,
    pub pattern: String,    // Regex pattern (can be same as content or a regex)
    pub is_regex: bool,    // Whether this is a regex pattern
    pub sensitive_type: SensitiveType,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub enabled: bool,      // Enable/disable this rule
    pub match_count: u64,   // Number of times this rule matched
}

impl WhitelistRule {
    pub fn from_entry(entry: WhitelistEntry) -> Self {
        Self {
            id: entry.id,
            content: entry.content.clone(),
            pattern: entry.content.clone(),
            is_regex: false,
            sensitive_type: entry.sensitive_type,
            description: entry.description,
            created_at: entry.created_at,
            enabled: true,
            match_count: 0,
        }
    }
    
    pub fn new_regex(
        pattern: String,
        sensitive_type: SensitiveType,
        description: Option<String>,
    ) -> Result<Self, regex::Error> {
        // Validate regex pattern
        Regex::new(&pattern)?;
        
        Ok(Self {
            id: uuid::Uuid::new_v4().to_string(),
            content: pattern.clone(),
            pattern,
            is_regex: true,
            sensitive_type,
            description,
            created_at: Utc::now(),
            enabled: true,
            match_count: 0,
        })
    }
}

/// Whitelist manager with optimized matching
pub struct WhitelistManager {
    rules: RwLock<Vec<WhitelistRule>>,
    compiled_regexes: RwLock<HashMap<String, Regex>>,
    last_updated: RwLock<DateTime<Utc>>,
}

impl WhitelistManager {
    pub fn new() -> Self {
        Self {
            rules: RwLock::new(Vec::new()),
            compiled_regexes: RwLock::new(HashMap::new()),
            last_updated: RwLock::new(Utc::now()),
        }
    }
    
    /// Add a whitelist rule
    pub fn add_rule(&self, rule: WhitelistRule) -> Result<(), String> {
        let mut rules = self.rules.write().map_err(|e| e.to_string())?;
        
        // If it's a regex, validate and compile it
        if rule.is_regex {
            let regex = Regex::new(&rule.pattern)
                .map_err(|e| format!("Invalid regex pattern: {}", e))?;
            
            let mut compiled = self.compiled_regexes.write().map_err(|e| e.to_string())?;
            compiled.insert(rule.id.clone(), regex);
        }
        
        rules.push(rule);
        *self.last_updated.write().map_err(|e| e.to_string())? = Utc::now();
        
        Ok(())
    }
    
    /// Remove a whitelist rule
    pub fn remove_rule(&self, id: &str) -> Result<(), String> {
        let mut rules = self.rules.write().map_err(|e| e.to_string())?;
        
        // Find and remove the rule
        let index = rules.iter().position(|r| r.id == id)
            .ok_or_else(|| format!("Rule not found: {}", id))?;
        
        let rule = rules.remove(index);
        
        // Remove compiled regex if exists
        if rule.is_regex {
            let mut compiled = self.compiled_regexes.write().map_err(|e| e.to_string())?;
            compiled.remove(&rule.id);
        }
        
        *self.last_updated.write().map_err(|e| e.to_string())? = Utc.now();
        
        Ok(())
    }
    
    /// Check if content matches any whitelist rule
    pub fn is_whitelisted(&self, content: &str, sensitive_type: SensitiveType) -> bool {
        let rules = self.rules.read().map_err(|e| {
            eprintln!("Failed to read whitelist: {}", e);
            false
        }).unwrap_or_default();
        
        for rule in rules.iter() {
            // Skip disabled rules
            if !rule.enabled {
                continue;
            }
            
            // Skip if sensitive type doesn't match
            if rule.sensitive_type != sensitive_type {
                continue;
            }
            
            let is_match = if rule.is_regex {
                // Use compiled regex
                let compiled = self.compiled_regexes.read().map_err(|e| {
                    eprintln!("Failed to read compiled regexes: {}", e);
                    false
                }).unwrap_or_default();
                
                compiled.get(&rule.id)
                    .map(|regex| regex.is_match(content))
                    .unwrap_or(false)
            } else {
                // Exact match
                content == rule.content || content.contains(&rule.content)
            };
            
            if is_match {
                // Increment match count
                if let Ok(mut rules) = self.rules.write() {
                    if let Some(rule) = rules.iter_mut().find(|r| r.id == rule.id) {
                        rule.match_count += 1;
                    }
                }
                return true;
            }
        }
        
        false
    }
    
    /// Get all rules
    pub fn get_all_rules(&self) -> Result<Vec<WhitelistRule>, String> {
        self.rules.read().map_err(|e| e.to_string()).map(|rules| rules.clone())
    }
    
    /// Update rule
    pub fn update_rule(&self, id: &str, enabled: bool) -> Result<(), String> {
        let mut rules = self.rules.write().map_err(|e| e.to_string())?;
        
        let rule = rules.iter_mut()
            .find(|r| r.id == id)
            .ok_or_else(|| format!("Rule not found: {}", id))?;
        
        rule.enabled = enabled;
        
        *self.last_updated.write().map_err(|e| e.to_string())? = Utc::now();
        
        Ok(())
    }
    
    /// Clear all rules
    pub fn clear_all(&self) -> Result<(), String> {
        let mut rules = self.rules.write().map_err(|e| e.to_string())?;
        let mut compiled = self.compiled_regexes.write().map_err(|e| e.to_string())?;
        
        rules.clear();
        compiled.clear();
        
        *self.last_updated.write().map_err(|e| e.to_string())? = Utc::now();
        
        Ok(())
    }
    
    /// Import rules from JSON
    pub fn import_rules(&self, rules: Vec<WhitelistRule>) -> Result<(), String> {
        for rule in rules {
            self.add_rule(rule)?;
        }
        Ok(())
    }
    
    /// Export rules to JSON
    pub fn export_rules(&self) -> Result<String, String> {
        let rules = self.get_all_rules()?;
        serde_json::to_string_pretty(&rules).map_err(|e| e.to_string())
    }
    
    /// Get last update time
    pub fn last_updated(&self) -> Result<DateTime<Utc>, String> {
        *self.last_updated.read().map_err(|e| e.to_string())
    }
}

impl Default for WhitelistManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_whitelist_exact_match() {
        let manager = WhitelistManager::new();
        
        let rule = WhitelistRule::from_entry(WhitelistEntry {
            id: "1".to_string(),
            content: "13800138000".to_string(),
            sensitive_type: SensitiveType::PhoneNumber,
            description: Some("Test number".to_string()),
            created_at: Utc::now(),
        });
        
        assert!(manager.add_rule(rule).is_ok());
        assert!(manager.is_whitelisted("13800138000", SensitiveType::PhoneNumber));
        assert!(!manager.is_whitelisted("13800138001", SensitiveType::PhoneNumber));
    }
    
    #[test]
    fn test_whitelist_regex_match() {
        let manager = WhitelistManager::new();
        
        let rule = WhitelistRule::new_regex(
            r"13800\d{5}".to_string(),
            SensitiveType::PhoneNumber,
            Some("Test pattern".to_string())
        ).unwrap();
        
        assert!(manager.add_rule(rule).is_ok());
        assert!(manager.is_whitelisted("13800138000", SensitiveType::PhoneNumber));
        assert!(manager.is_whitelisted("13800138001", SensitiveType::PhoneNumber));
        assert!(!manager.is_whitelisted("13900138000", SensitiveType::PhoneNumber));
    }
    
    #[test]
    fn test_whitelist_export_import() {
        let manager = WhitelistManager::new();
        
        let rule = WhitelistRule::from_entry(WhitelistEntry {
            id: "1".to_string(),
            content: "13800138000".to_string(),
            sensitive_type: SensitiveType::PhoneNumber,
            description: Some("Test".to_string()),
            created_at: Utc::now(),
        });
        
        manager.add_rule(rule).unwrap();
        
        let exported = manager.export_rules().unwrap();
        let imported: Vec<WhitelistRule> = serde_json::from_str(&exported).unwrap();
        
        assert_eq!(imported.len(), 1);
        assert_eq!(imported[0].content, "13800138000");
    }
}
