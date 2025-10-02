//! Core validation types and data structures.
//!
//! This module contains all the fundamental types used by the validation system
//! including severity levels, rules, contexts, and results.

use std::collections::HashMap;
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Validation severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationSeverity {
    /// Information level (non-blocking)
    Info,
    /// Warning level (logged but allowed)
    Warning,
    /// Error level (blocking validation)
    Error,
    /// Critical level (immediate rejection)
    Critical,
}

/// Validation rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: String,
    /// Validation severity
    pub severity: ValidationSeverity,
    /// Whether this rule is enabled
    pub enabled: bool,
    /// Rule-specific configuration
    pub config: HashMap<String, serde_json::Value>,
}

/// Validation context for business rules
#[derive(Debug, Clone)]
pub struct ValidationContext {
    /// Field name being validated
    pub field_name: String,
    /// Field value
    pub field_value: serde_json::Value,
    /// Additional context data
    pub context_data: HashMap<String, serde_json::Value>,
    /// User ID performing validation (if applicable)
    pub user_id: Option<String>,
    /// Request ID for tracing
    pub request_id: Option<String>,
}

/// Validation result with detailed information
#[derive(Debug, Clone)]
pub struct AdvancedValidationResult {
    /// Whether validation passed
    pub is_valid: bool,
    /// Validation errors (if any)
    pub errors: Vec<ValidationErrorDetail>,
    /// Validation warnings (non-blocking)
    pub warnings: Vec<ValidationWarning>,
    /// Sanitized value (if applicable)
    pub sanitized_value: Option<serde_json::Value>,
    /// Validation metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Detailed validation error
#[derive(Debug, Clone)]
pub struct ValidationErrorDetail {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Field that failed validation
    pub field: String,
    /// Validation rule that failed
    pub rule: String,
    /// Severity of the error
    pub severity: ValidationSeverity,
    /// Additional error context
    pub context: HashMap<String, serde_json::Value>,
}

/// Validation warning (non-blocking)
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    /// Warning code
    pub code: String,
    /// Warning message
    pub message: String,
    /// Field that generated the warning
    pub field: String,
    /// Rule that generated the warning
    pub rule: String,
    /// Additional warning context
    pub context: HashMap<String, serde_json::Value>,
}

/// Security threat patterns
#[derive(Debug, Clone)]
pub struct SecurityPattern {
    /// Pattern name
    pub name: String,
    /// Regex pattern to match
    pub pattern: Regex,
    /// Threat severity
    pub severity: ValidationSeverity,
    /// Description of the threat
    pub description: String,
}

/// Validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Maximum input length
    pub max_length: usize,
    /// Minimum input length
    pub min_length: usize,
    /// Whether to enable security scanning
    pub enable_security_scan: bool,
    /// Whether to enable business rule validation
    pub enable_business_rules: bool,
    /// Whether to enable input sanitization
    pub enable_sanitization: bool,
    /// Whether to reject on first error
    pub fail_fast: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            max_length: 10000,
            min_length: 0,
            enable_security_scan: true,
            enable_business_rules: true,
            enable_sanitization: true,
            fail_fast: false,
        }
    }
}

/// Advanced input validator with security scanning
#[derive(Debug)]
pub struct AdvancedValidator {
    /// Security patterns for threat detection
    pub(crate) security_patterns: Vec<SecurityPattern>,
    /// Business validation rules
    pub(crate) business_rules: std::collections::HashMap<String, Vec<ValidationRule>>,
    /// Field-specific sanitizers
    pub(crate) sanitizers: std::collections::HashMap<String, Box<dyn Sanitizer>>,
    /// Global validation configuration
    pub(crate) config: ValidationConfig,
}

/// Sanitizer trait for input cleaning
pub trait Sanitizer: Send + Sync + std::fmt::Debug {
    /// Sanitize input value
    fn sanitize(&self, value: &str) -> String;

    /// Clone the sanitizer
    fn clone_box(&self) -> Box<dyn Sanitizer>;
}
