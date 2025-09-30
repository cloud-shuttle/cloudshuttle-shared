//! Configuration validation utilities
//!
//! This module provides validation functions for configuration values,
//! ensuring they meet requirements before being used.

use std::result::Result as StdResult;

/// Error type for configuration validation
#[derive(Debug, thiserror::Error)]
pub enum ConfigValidationError {
    #[error("Required field missing: {0}")]
    RequiredFieldMissing(String),

    #[error("Invalid value for {field}: {value} ({reason})")]
    InvalidValue {
        field: String,
        value: String,
        reason: String,
    },

    #[error("Value out of range for {field}: {value} (expected {min}..{max})")]
    OutOfRange {
        field: String,
        value: i64,
        min: i64,
        max: i64,
    },

    #[error("Invalid format for {field}: {value}")]
    InvalidFormat { field: String, value: String },
}

/// Result type for configuration validation
pub type Result<T> = StdResult<T, ConfigValidationError>;

/// Validate that a required string field is not empty
///
/// # Arguments
/// * `field_name` - Name of the field for error messages
/// * `value` - Value to validate
///
/// # Returns
/// Result with the validated string or error
pub fn validate_required_string(field_name: &str, value: Option<&str>) -> Result<String> {
    match value {
        Some(s) if !s.trim().is_empty() => Ok(s.trim().to_string()),
        _ => Err(ConfigValidationError::RequiredFieldMissing(field_name.to_string())),
    }
}

/// Validate that a number is within a specified range
///
/// # Arguments
/// * `field_name` - Name of the field for error messages
/// * `value` - Value to validate
/// * `min` - Minimum allowed value (inclusive)
/// * `max` - Maximum allowed value (inclusive)
///
/// # Returns
/// Result with the validated number or error
pub fn validate_range(field_name: &str, value: i64, min: i64, max: i64) -> Result<i64> {
    if value < min || value > max {
        return Err(ConfigValidationError::OutOfRange {
            field: field_name.to_string(),
            value,
            min,
            max,
        });
    }
    Ok(value)
}

/// Validate that a string matches a required pattern
///
/// # Arguments
/// * `field_name` - Name of the field for error messages
/// * `value` - Value to validate
/// * `valid_values` - List of allowed values
///
/// # Returns
/// Result with the validated string or error
pub fn validate_one_of(field_name: &str, value: &str, valid_values: &[&str]) -> Result<String> {
    if valid_values.iter().any(|&v| v == value) {
        Ok(value.to_string())
    } else {
        Err(ConfigValidationError::InvalidValue {
            field: field_name.to_string(),
            value: value.to_string(),
            reason: format!("Must be one of: {:?}", valid_values),
        })
    }
}

/// Validate URL format
///
/// # Arguments
/// * `field_name` - Name of the field for error messages
/// * `value` - URL string to validate
///
/// # Returns
/// Result with the validated URL or error
pub fn validate_url(field_name: &str, value: &str) -> Result<String> {
    if value.starts_with("http://") || value.starts_with("https://") {
        Ok(value.to_string())
    } else {
        Err(ConfigValidationError::InvalidFormat {
            field: field_name.to_string(),
            value: value.to_string(),
        })
    }
}

/// Validate email format (basic validation)
///
/// # Arguments
/// * `field_name` - Name of the field for error messages
/// * `value` - Email string to validate
///
/// # Returns
/// Result with the validated email or error
pub fn validate_email(field_name: &str, value: &str) -> Result<String> {
    if value.contains('@') && value.contains('.') && !value.starts_with('@') && !value.ends_with('@') {
        Ok(value.to_string())
    } else {
        Err(ConfigValidationError::InvalidFormat {
            field: field_name.to_string(),
            value: value.to_string(),
        })
    }
}

/// Configuration validator that can validate entire config structs
pub struct ConfigValidator {
    errors: Vec<ConfigValidationError>,
}

impl ConfigValidator {
    /// Create a new configuration validator
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
        }
    }

    /// Record a validation error
    pub fn record_error(&mut self, error: ConfigValidationError) {
        self.errors.push(error);
    }

    /// Validate a required string field
    pub fn validate_required_string(&mut self, field_name: &str, value: Option<&str>) {
        if let Err(e) = validate_required_string(field_name, value) {
            self.record_error(e);
        }
    }

    /// Validate a numeric range
    pub fn validate_range(&mut self, field_name: &str, value: i64, min: i64, max: i64) {
        if let Err(e) = validate_range(field_name, value, min, max) {
            self.record_error(e);
        }
    }

    /// Validate that value is one of allowed values
    pub fn validate_one_of(&mut self, field_name: &str, value: &str, valid_values: &[&str]) {
        if let Err(e) = validate_one_of(field_name, value, valid_values) {
            self.record_error(e);
        }
    }

    /// Validate URL format
    pub fn validate_url(&mut self, field_name: &str, value: &str) {
        if let Err(e) = validate_url(field_name, value) {
            self.record_error(e);
        }
    }

    /// Check if validation passed (no errors)
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get all validation errors
    pub fn errors(&self) -> &[ConfigValidationError] {
        &self.errors
    }

    /// Get the first validation error (if any)
    pub fn first_error(&self) -> Option<&ConfigValidationError> {
        self.errors.first()
    }

    /// Consume the validator and return result
    pub fn into_result<T>(self, value: T) -> Result<T> {
        if self.is_valid() {
            Ok(value)
        } else {
            Err(self.errors.into_iter().next().unwrap())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_required_string() {
        assert!(validate_required_string("test", Some("value")).is_ok());
        assert!(validate_required_string("test", Some("  ")).is_err());
        assert!(validate_required_string("test", None).is_err());
    }

    #[test]
    fn test_validate_range() {
        assert!(validate_range("test", 5, 1, 10).is_ok());
        assert!(validate_range("test", 0, 1, 10).is_err());
        assert!(validate_range("test", 15, 1, 10).is_err());
    }

    #[test]
    fn test_validate_one_of() {
        let valid = ["a", "b", "c"];
        assert!(validate_one_of("test", "a", &valid).is_ok());
        assert!(validate_one_of("test", "d", &valid).is_err());
    }

    #[test]
    fn test_validate_url() {
        assert!(validate_url("test", "https://example.com").is_ok());
        assert!(validate_url("test", "http://example.com").is_ok());
        assert!(validate_url("test", "ftp://example.com").is_err());
        assert!(validate_url("test", "example.com").is_err());
    }

    #[test]
    fn test_config_validator() {
        let mut validator = ConfigValidator::new();

        validator.validate_required_string("name", Some("test"));
        validator.validate_range("port", 8080, 1000, 9999);
        validator.validate_one_of("env", "prod", &["dev", "staging", "prod"]);

        assert!(validator.is_valid());

        // Test with errors
        let mut validator2 = ConfigValidator::new();
        validator2.validate_required_string("name", None);
        validator2.validate_range("port", 80, 1000, 9999);

        assert!(!validator2.is_valid());
        assert_eq!(validator2.errors().len(), 2);
    }
}
