//! Request validation utilities
//!
//! This module provides validation utilities for API requests,
//! including input sanitization and validation rules.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::result::Result as StdResult;

/// Error type for request validation
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Required field missing: {0}")]
    RequiredFieldMissing(String),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    #[error("Field too long: {field} (max {max} characters)")]
    FieldTooLong { field: String, max: usize },

    #[error("Field too short: {field} (min {min} characters)")]
    FieldTooShort { field: String, min: usize },
}

/// Result type for validation operations
pub type Result<T> = StdResult<T, ValidationError>;

/// Request validator for API endpoints
#[derive(Debug)]
pub struct RequestValidator {
    errors: HashMap<String, Vec<String>>,
}

impl RequestValidator {
    /// Create a new request validator
    pub fn new() -> Self {
        Self {
            errors: HashMap::new(),
        }
    }

    /// Add a validation error
    pub fn add_error(&mut self, field: impl Into<String>, message: impl Into<String>) {
        self.errors.entry(field.into())
            .or_insert_with(Vec::new)
            .push(message.into());
    }

    /// Validate that a required field is present and not empty
    pub fn validate_required(&mut self, field: &str, value: Option<&str>) {
        match value {
            Some(s) if !s.trim().is_empty() => {} // Valid
            _ => {
                self.add_error(field, "This field is required");
            }
        }
    }

    /// Validate string length
    pub fn validate_length(&mut self, field: &str, value: &str, min: usize, max: usize) {
        let len = value.len();
        if len < min {
            self.add_error(field, format!("Must be at least {} characters", min));
        }
        if len > max {
            self.add_error(field, format!("Must be at most {} characters", max));
        }
    }

    /// Validate email format (basic validation)
    pub fn validate_email(&mut self, field: &str, email: &str) {
        if email.is_empty() {
            self.add_error(field, "Email is required");
            return;
        }

        // Basic email validation
        if !email.contains('@') || !email.contains('.') {
            self.add_error(field, "Invalid email format");
            return;
        }

        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            self.add_error(field, "Invalid email format");
            return;
        }

        let local = parts[0];
        let domain = parts[1];

        if local.is_empty() || domain.is_empty() {
            self.add_error(field, "Invalid email format");
            return;
        }

        if !domain.contains('.') {
            self.add_error(field, "Invalid email domain");
        }
    }

    /// Validate URL format
    pub fn validate_url(&mut self, field: &str, url: &str) {
        if url.is_empty() {
            self.add_error(field, "URL is required");
            return;
        }

        if !url.starts_with("http://") && !url.starts_with("https://") {
            self.add_error(field, "URL must start with http:// or https://");
        }
    }

    /// Validate numeric range
    pub fn validate_range(&mut self, field: &str, value: i64, min: i64, max: i64) {
        if value < min {
            self.add_error(field, format!("Must be at least {}", min));
        }
        if value > max {
            self.add_error(field, format!("Must be at most {}", max));
        }
    }

    /// Validate that value is one of allowed options
    pub fn validate_one_of(&mut self, field: &str, value: &str, options: &[&str]) {
        if !options.contains(&value) {
            self.add_error(field, format!("Must be one of: {}", options.join(", ")));
        }
    }

    /// Check if validation passed
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get all validation errors
    pub fn errors(&self) -> &HashMap<String, Vec<String>> {
        &self.errors
    }

    /// Get the first error for a field
    pub fn first_error(&self, field: &str) -> Option<&String> {
        self.errors.get(field)?.first()
    }

    /// Get all errors as a flat list
    pub fn all_errors(&self) -> Vec<String> {
        self.errors.values()
            .flatten()
            .cloned()
            .collect()
    }

    /// Get error count
    pub fn error_count(&self) -> usize {
        self.errors.values().map(|v| v.len()).sum()
    }

    /// Clear all errors
    pub fn clear(&mut self) {
        self.errors.clear();
    }
}

impl Default for RequestValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation rules for common use cases
pub mod rules {
    use super::*;

    /// Validate a user registration request
    pub fn validate_user_registration(username: &str, email: &str, password: &str) -> Result<()> {
        let mut validator = RequestValidator::new();

        // Username validation
        validator.validate_required("username", Some(username));
        if !username.is_empty() {
            validator.validate_length("username", username, 3, 32);
        }

        // Email validation
        validator.validate_email("email", email);

        // Password validation (basic)
        validator.validate_required("password", Some(password));
        if !password.is_empty() {
            validator.validate_length("password", password, 8, 128);
        }

        if validator.is_valid() {
            Ok(())
        } else {
            Err(ValidationError::ValidationFailed(
                validator.all_errors().join("; ")
            ))
        }
    }

    /// Validate pagination parameters
    pub fn validate_pagination(page: Option<u32>, per_page: Option<u32>) -> Result<(u32, u32)> {
        let mut validator = RequestValidator::new();

        let page = page.unwrap_or(1);
        let per_page = per_page.unwrap_or(20);

        validator.validate_range("page", page as i64, 1, 10000);
        validator.validate_range("per_page", per_page as i64, 1, 1000);

        if validator.is_valid() {
            Ok((page, per_page))
        } else {
            Err(ValidationError::ValidationFailed(
                validator.all_errors().join("; ")
            ))
        }
    }

    /// Validate search query parameters
    pub fn validate_search_query(query: &str, limit: Option<usize>) -> Result<(String, usize)> {
        let mut validator = RequestValidator::new();

        let trimmed_query = query.trim();
        validator.validate_required("query", Some(trimmed_query));
        if !trimmed_query.is_empty() {
            validator.validate_length("query", trimmed_query, 1, 1000);
        }

        let limit = limit.unwrap_or(50);
        validator.validate_range("limit", limit as i64, 1, 1000);

        if validator.is_valid() {
            Ok((trimmed_query.to_string(), limit))
        } else {
            Err(ValidationError::ValidationFailed(
                validator.all_errors().join("; ")
            ))
        }
    }
}

/// Request sanitization utilities
pub mod sanitize {
    /// Sanitize user input by trimming whitespace
    pub fn trim_whitespace(input: &str) -> String {
        input.trim().to_string()
    }

    /// Sanitize string by removing control characters
    pub fn remove_control_chars(input: &str) -> String {
        input.chars()
            .filter(|c| !c.is_control() || *c == '\n' || *c == '\r' || *c == '\t')
            .collect()
    }

    /// Sanitize input for HTML display (basic XSS prevention)
    pub fn escape_html(input: &str) -> String {
        input
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_validator_required_field() {
        let mut validator = RequestValidator::new();

        validator.validate_required("name", Some("John"));
        validator.validate_required("email", None);

        assert!(!validator.is_valid());
        assert_eq!(validator.error_count(), 1);
        assert!(validator.first_error("email").unwrap().contains("required"));
    }

    #[test]
    fn test_request_validator_email() {
        let mut validator = RequestValidator::new();

        validator.validate_email("email", "user@example.com");
        validator.validate_email("email2", "invalid-email");

        assert!(!validator.is_valid());
        assert_eq!(validator.error_count(), 1);
    }

    #[test]
    fn test_validation_rules_user_registration() {
        // Valid registration
        assert!(rules::validate_user_registration("user123", "user@example.com", "password123").is_ok());

        // Invalid registration
        assert!(rules::validate_user_registration("", "user@example.com", "password123").is_err());
        assert!(rules::validate_user_registration("user123", "invalid-email", "password123").is_err());
        assert!(rules::validate_user_registration("user123", "user@example.com", "short").is_err());
    }

    #[test]
    fn test_sanitize_functions() {
        assert_eq!(sanitize::trim_whitespace("  hello  "), "hello");
        assert_eq!(sanitize::escape_html("<script>"), "&lt;script&gt;");
    }
}
