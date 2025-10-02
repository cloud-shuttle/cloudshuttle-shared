//! Advanced validator implementation.
//!
//! This module contains the core AdvancedValidator struct and its
//! validation logic for security scanning, business rules, and sanitization.

use std::collections::HashMap;
use regex::Regex;
use crate::validation::types::{
    AdvancedValidator, SecurityPattern, ValidationRule, ValidationContext,
    AdvancedValidationResult, ValidationErrorDetail, ValidationWarning,
    ValidationSeverity, ValidationConfig, Sanitizer
};

impl AdvancedValidator {
    /// Create a new advanced validator
    pub fn new(config: ValidationConfig) -> Self {
        let mut validator = Self {
            security_patterns: Vec::new(),
            business_rules: HashMap::new(),
            sanitizers: HashMap::new(),
            config,
        };

        validator.initialize_default_patterns();
        validator
    }

    /// Initialize default security patterns
    fn initialize_default_patterns(&mut self) {
        // SQL injection patterns
        self.add_security_pattern(
            "sql_injection",
            r"(?i)(union\s+select|select\s+.*\s+from|insert\s+into|update\s+.*\s+set|delete\s+from|drop\s+table|alter\s+table|create\s+table|exec\s+|execute\s+|script\s+|javascript\s*|vbscript\s*|onload\s*=|onerror\s*=)",
            ValidationSeverity::Critical,
            "Potential SQL injection attack detected",
        );

        // XSS patterns
        self.add_security_pattern(
            "xss_attack",
            r"(?i)(<script|<iframe|<object|<embed|<form|<input|<meta|<link|<style|javascript:|vbscript:|data:|on\w+\s*=)",
            ValidationSeverity::Critical,
            "Potential XSS attack detected",
        );

        // Path traversal patterns
        self.add_security_pattern(
            "path_traversal",
            r"(\.\./|\.\.\\|~|%2e%2e%2f|%2e%2e%5c)",
            ValidationSeverity::Critical,
            "Potential path traversal attack detected",
        );

        // Command injection patterns
        self.add_security_pattern(
            "command_injection",
            r"([;|&`\$\\(\)\[\]\{\}\*\?\+\^])",
            ValidationSeverity::Critical,
            "Potential command injection detected",
        );
    }

    /// Add a security pattern for threat detection
    pub fn add_security_pattern(
        &mut self,
        name: impl Into<String>,
        pattern: &str,
        severity: ValidationSeverity,
        description: impl Into<String>,
    ) {
        let regex = Regex::new(pattern).expect("Invalid regex pattern");
        self.security_patterns.push(SecurityPattern {
            name: name.into(),
            pattern: regex,
            severity,
            description: description.into(),
        });
    }

    /// Add a business validation rule for a field
    pub fn add_business_rule(&mut self, field: impl Into<String>, rule: ValidationRule) {
        self.business_rules.entry(field.into()).or_insert_with(Vec::new).push(rule);
    }

    /// Add a sanitizer for a field
    pub fn add_sanitizer(&mut self, field: impl Into<String>, sanitizer: Box<dyn Sanitizer>) {
        self.sanitizers.insert(field.into(), sanitizer);
    }

    /// Validate input with comprehensive checking
    pub fn validate(&self, context: ValidationContext) -> AdvancedValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut metadata = HashMap::new();
        let mut sanitized_value = None;

        let field_name = context.field_name.clone();
        let field_value = context.field_value.clone();

        // Extract string value for validation
        let string_value = match field_value {
            serde_json::Value::String(s) => s,
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            _ => field_value.to_string(),
        };

        // Length validation
        if string_value.len() > self.config.max_length {
            errors.push(ValidationErrorDetail {
                code: "MAX_LENGTH_EXCEEDED".to_string(),
                message: format!("Input exceeds maximum length of {}", self.config.max_length),
                field: field_name.clone(),
                rule: "length_validation".to_string(),
                severity: ValidationSeverity::Error,
                context: HashMap::new(),
            });

            if self.config.fail_fast {
                return AdvancedValidationResult {
                    is_valid: false,
                    errors,
                    warnings,
                    sanitized_value,
                    metadata,
                };
            }
        }

        if string_value.len() < self.config.min_length {
            errors.push(ValidationErrorDetail {
                code: "MIN_LENGTH_NOT_MET".to_string(),
                message: format!("Input is below minimum length of {}", self.config.min_length),
                field: field_name.clone(),
                rule: "length_validation".to_string(),
                severity: ValidationSeverity::Error,
                context: HashMap::new(),
            });

            if self.config.fail_fast {
                return AdvancedValidationResult {
                    is_valid: false,
                    errors,
                    warnings,
                    sanitized_value,
                    metadata,
                };
            }
        }

        // Security scanning
        if self.config.enable_security_scan {
            for pattern in &self.security_patterns {
                if pattern.pattern.is_match(&string_value) {
                    let error = ValidationErrorDetail {
                        code: format!("SECURITY_{}", pattern.name.to_uppercase()),
                        message: pattern.description.clone(),
                        field: field_name.clone(),
                        rule: "security_scan".to_string(),
                        severity: pattern.severity,
                        context: HashMap::new(),
                    };

                    match pattern.severity {
                        ValidationSeverity::Critical | ValidationSeverity::Error => {
                            errors.push(error);
                            if self.config.fail_fast {
                                return AdvancedValidationResult {
                                    is_valid: false,
                                    errors,
                                    warnings,
                                    sanitized_value,
                                    metadata,
                                };
                            }
                        }
                        ValidationSeverity::Warning => {
                            warnings.push(ValidationWarning {
                                code: format!("SECURITY_{}", pattern.name.to_uppercase()),
                                message: pattern.description.clone(),
                                field: field_name.clone(),
                                rule: "security_scan".to_string(),
                                context: HashMap::new(),
                            });
                        }
                        ValidationSeverity::Info => {
                            // Log but don't add to warnings
                        }
                    }
                }
            }
        }

        // Input sanitization
        let processed_value = if self.config.enable_sanitization {
            if let Some(sanitizer) = self.sanitizers.get(&field_name) {
                let sanitized = sanitizer.sanitize(&string_value);
                sanitized_value = Some(serde_json::Value::String(sanitized.clone()));

                // Check if sanitization changed the value
                if sanitized != string_value {
                    metadata.insert("sanitized".to_string(), serde_json::Value::Bool(true));
                    metadata.insert("original_length".to_string(), serde_json::json!(string_value.len()));
                    metadata.insert("sanitized_length".to_string(), serde_json::json!(sanitized.len()));
                }

                sanitized
            } else {
                string_value
            }
        } else {
            string_value
        };

        // Business rule validation
        if self.config.enable_business_rules {
            if let Some(rules) = self.business_rules.get(&field_name) {
                for rule in rules {
                    if !rule.enabled {
                        continue;
                    }

                    let rule_result = self.validate_business_rule(rule, &processed_value, &context);
                    match rule_result {
                        Ok(warning) => {
                            if let Some(w) = warning {
                                warnings.push(w);
                            }
                        }
                        Err(error) => {
                            errors.push(error);
                            if self.config.fail_fast {
                                return AdvancedValidationResult {
                                    is_valid: false,
                                    errors,
                                    warnings,
                                    sanitized_value,
                                    metadata,
                                };
                            }
                        }
                    }
                }
            }
        }

        // Add validation metadata
        metadata.insert("processed_length".to_string(), serde_json::json!(processed_value.len()));
        metadata.insert("security_patterns_checked".to_string(), serde_json::json!(self.security_patterns.len()));

        AdvancedValidationResult {
            is_valid: errors.is_empty() || errors.iter().all(|e| e.severity == ValidationSeverity::Warning),
            errors,
            warnings,
            sanitized_value,
            metadata,
        }
    }

    /// Validate a single business rule
    fn validate_business_rule(
        &self,
        rule: &ValidationRule,
        value: &str,
        context: &ValidationContext,
    ) -> Result<Option<ValidationWarning>, ValidationErrorDetail> {
        // Example business rules - extend as needed
        match rule.name.as_str() {
            "email_format" => {
                if !value.contains('@') || !value.contains('.') {
                    return Err(ValidationErrorDetail {
                        code: "INVALID_EMAIL_FORMAT".to_string(),
                        message: "Invalid email format".to_string(),
                        field: context.field_name.clone(),
                        rule: rule.name.clone(),
                        severity: rule.severity,
                        context: HashMap::new(),
                    });
                }
            }
            "phone_format" => {
                let phone_regex = Regex::new(r"^\+?[\d\s\-\(\)]+$").unwrap();
                if !phone_regex.is_match(value) {
                    return Err(ValidationErrorDetail {
                        code: "INVALID_PHONE_FORMAT".to_string(),
                        message: "Invalid phone number format".to_string(),
                        field: context.field_name.clone(),
                        rule: rule.name.clone(),
                        severity: rule.severity,
                        context: HashMap::new(),
                    });
                }
            }
            "strong_password" => {
                let has_upper = value.chars().any(|c| c.is_uppercase());
                let has_lower = value.chars().any(|c| c.is_lowercase());
                let has_digit = value.chars().any(|c| c.is_digit(10));
                let has_special = value.chars().any(|c| !c.is_alphanumeric());

                if !(has_upper && has_lower && has_digit && has_special && value.len() >= 8) {
                    return Err(ValidationErrorDetail {
                        code: "WEAK_PASSWORD".to_string(),
                        message: "Password must contain uppercase, lowercase, digit, special character, and be at least 8 characters".to_string(),
                        field: context.field_name.clone(),
                        rule: rule.name.clone(),
                        severity: rule.severity,
                        context: HashMap::new(),
                    });
                }
            }
            "no_consecutive_spaces" => {
                if value.contains("  ") {
                    return Ok(Some(ValidationWarning {
                        code: "CONSECUTIVE_SPACES".to_string(),
                        message: "Input contains consecutive spaces".to_string(),
                        field: context.field_name.clone(),
                        rule: rule.name.clone(),
                        context: HashMap::new(),
                    }));
                }
            }
            _ => {
                // Unknown rule - log warning but don't fail
                return Ok(Some(ValidationWarning {
                    code: "UNKNOWN_RULE".to_string(),
                    message: format!("Unknown validation rule: {}", rule.name),
                    field: context.field_name.clone(),
                    rule: rule.name.clone(),
                    context: HashMap::new(),
                }));
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::types::{ValidationRule, ValidationContext};

    #[test]
    fn test_security_pattern_detection() {
        let config = ValidationConfig::default();
        let validator = AdvancedValidator::new(config);

        let context = ValidationContext {
            field_name: "test_field".to_string(),
            field_value: serde_json::json!("UNION SELECT * FROM users"),
            context_data: HashMap::new(),
            user_id: None,
            request_id: None,
        };

        let result = validator.validate(context);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.code.starts_with("SECURITY_")));
    }

    #[test]
    fn test_business_rule_validation() {
        let config = ValidationConfig::default();
        let mut validator = AdvancedValidator::new(config);

        validator.add_business_rule("email", ValidationRule {
            name: "email_format".to_string(),
            description: "Validate email format".to_string(),
            severity: ValidationSeverity::Error,
            enabled: true,
            config: HashMap::new(),
        });

        let context = ValidationContext {
            field_name: "email".to_string(),
            field_value: serde_json::json!("invalid-email"),
            context_data: HashMap::new(),
            user_id: None,
            request_id: None,
        };

        let result = validator.validate(context);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.code == "INVALID_EMAIL_FORMAT"));
    }

    #[test]
    fn test_length_validation() {
        let mut config = ValidationConfig::default();
        config.max_length = 10;
        let validator = AdvancedValidator::new(config);

        let context = ValidationContext {
            field_name: "test_field".to_string(),
            field_value: serde_json::json!("this is a very long string that exceeds the limit"),
            context_data: HashMap::new(),
            user_id: None,
            request_id: None,
        };

        let result = validator.validate(context);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.code == "MAX_LENGTH_EXCEEDED"));
    }
}
