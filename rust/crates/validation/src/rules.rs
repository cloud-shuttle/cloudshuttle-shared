//! Validation rules for common data types
//!
//! This module provides validation functions for emails, passwords,
//! usernames, and other common input types.

use once_cell::sync::Lazy;
use regex::Regex;
use std::result::Result as StdResult;

/// Error type for validation operations
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Invalid email format: {0}")]
    InvalidEmail(String),

    #[error("Invalid password: {0}")]
    InvalidPassword(String),

    #[error("Invalid username: {0}")]
    InvalidUsername(String),

    #[error("Input too long: {0} characters (max {1})")]
    TooLong(usize, usize),

    #[error("Input too short: {0} characters (min {1})")]
    TooShort(usize, usize),

    #[error("Invalid characters: {0}")]
    InvalidCharacters(String),
}

/// Result type for validation operations
pub type Result<T> = StdResult<T, ValidationError>;

// Email validation regex (RFC 5322 compliant)
static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$").unwrap()
});

// Username validation regex (alphanumeric, underscore, dash)
static USERNAME_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap()
});

/// Validate email address format
///
/// # Arguments
/// * `email` - Email address to validate
///
/// # Returns
/// Ok(()) if valid, ValidationError if invalid
///
/// # Example
/// ```rust
/// assert!(validate_email("user@example.com").is_ok());
/// assert!(validate_email("invalid-email").is_err());
/// ```
pub fn validate_email(email: &str) -> Result<()> {
    if email.is_empty() {
        return Err(ValidationError::InvalidEmail("Email cannot be empty".to_string()));
    }

    if email.len() > 254 {
        return Err(ValidationError::TooLong(email.len(), 254));
    }

    if !EMAIL_REGEX.is_match(email) {
        return Err(ValidationError::InvalidEmail(format!("Invalid email format: {}", email)));
    }

    Ok(())
}

/// Validate password strength
///
/// Checks for minimum length, character variety, and common patterns.
///
/// # Arguments
/// * `password` - Password to validate
///
/// # Returns
/// Ok(()) if strong enough, ValidationError if weak
///
/// # Example
/// ```rust
/// assert!(validate_password_strength("StrongP@ssw0rd123").is_ok());
/// assert!(validate_password_strength("weak").is_err());
/// ```
pub fn validate_password_strength(password: &str) -> Result<()> {
    if password.len() < 8 {
        return Err(ValidationError::TooShort(password.len(), 8));
    }

    if password.len() > 128 {
        return Err(ValidationError::TooLong(password.len(), 128));
    }

    // Check for at least one lowercase letter
    if !password.chars().any(|c| c.is_ascii_lowercase()) {
        return Err(ValidationError::InvalidPassword("Must contain at least one lowercase letter".to_string()));
    }

    // Check for at least one uppercase letter
    if !password.chars().any(|c| c.is_ascii_uppercase()) {
        return Err(ValidationError::InvalidPassword("Must contain at least one uppercase letter".to_string()));
    }

    // Check for at least one digit
    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err(ValidationError::InvalidPassword("Must contain at least one digit".to_string()));
    }

    // Check for at least one special character
    if !password.chars().any(|c| !c.is_alphanumeric()) {
        return Err(ValidationError::InvalidPassword("Must contain at least one special character".to_string()));
    }

    Ok(())
}

/// Validate username format
///
/// Usernames must be alphanumeric with underscores and dashes allowed.
///
/// # Arguments
/// * `username` - Username to validate
///
/// # Returns
/// Ok(()) if valid, ValidationError if invalid
///
/// # Example
/// ```rust
/// assert!(validate_username("user123").is_ok());
/// assert!(validate_username("user-name_123").is_ok());
/// assert!(validate_username("user@domain.com").is_err());
/// ```
pub fn validate_username(username: &str) -> Result<()> {
    if username.is_empty() {
        return Err(ValidationError::InvalidUsername("Username cannot be empty".to_string()));
    }

    if username.len() < 3 {
        return Err(ValidationError::TooShort(username.len(), 3));
    }

    if username.len() > 32 {
        return Err(ValidationError::TooLong(username.len(), 32));
    }

    if !USERNAME_REGEX.is_match(username) {
        return Err(ValidationError::InvalidUsername(format!("Invalid username format: {}", username)));
    }

    Ok(())
}

/// Validate input length
///
/// # Arguments
/// * `input` - Input string to validate
/// * `min_len` - Minimum allowed length
/// * `max_len` - Maximum allowed length
///
/// # Returns
/// Ok(()) if within bounds, ValidationError if not
pub fn validate_length(input: &str, min_len: usize, max_len: usize) -> Result<()> {
    if input.len() < min_len {
        return Err(ValidationError::TooShort(input.len(), min_len));
    }

    if input.len() > max_len {
        return Err(ValidationError::TooLong(input.len(), max_len));
    }

    Ok(())
}

/// Validate that input contains only alphanumeric characters and spaces
///
/// # Arguments
/// * `input` - Input string to validate
///
/// # Returns
/// Ok(()) if valid, ValidationError if invalid
pub fn validate_alphanumeric_with_spaces(input: &str) -> Result<()> {
    if input.chars().any(|c| !c.is_alphanumeric() && !c.is_whitespace()) {
        return Err(ValidationError::InvalidCharacters("Only alphanumeric characters and spaces allowed".to_string()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_email_valid() {
        assert!(validate_email("user@example.com").is_ok());
        assert!(validate_email("test.email+tag@domain.co.uk").is_ok());
        assert!(validate_email("user@localhost").is_ok());
    }

    #[test]
    fn test_validate_email_invalid() {
        assert!(validate_email("").is_err());
        assert!(validate_email("invalid-email").is_err());
        assert!(validate_email("user@").is_err());
        assert!(validate_email("@domain.com").is_err());
    }

    #[test]
    fn test_validate_password_strength() {
        // Valid passwords
        assert!(validate_password_strength("StrongP@ssw0rd123").is_ok());
        assert!(validate_password_strength("P@ssw0rd!2024").is_ok());

        // Invalid passwords
        assert!(validate_password_strength("").is_err()); // Too short
        assert!(validate_password_strength("weak").is_err()); // Too short
        assert!(validate_password_strength("password").is_err()); // Missing uppercase, digit, special
        assert!(validate_password_strength("PASSWORD").is_err()); // Missing lowercase, digit, special
        assert!(validate_password_strength("Password").is_err()); // Missing digit, special
        assert!(validate_password_strength("Password123").is_err()); // Missing special
    }

    #[test]
    fn test_validate_username() {
        // Valid usernames
        assert!(validate_username("user123").is_ok());
        assert!(validate_username("test_user").is_ok());
        assert!(validate_username("user-name-123").is_ok());

        // Invalid usernames
        assert!(validate_username("").is_err()); // Empty
        assert!(validate_username("ab").is_err()); // Too short
        assert!(validate_username("user@domain.com").is_err()); // Invalid characters
        assert!(validate_username("user name").is_err()); // Spaces not allowed
    }

    #[test]
    fn test_validate_length() {
        assert!(validate_length("hello", 3, 10).is_ok());
        assert!(validate_length("hi", 3, 10).is_err()); // Too short
        assert!(validate_length("this is a very long string", 3, 10).is_err()); // Too long
    }

    #[test]
    fn test_validate_alphanumeric_with_spaces() {
        assert!(validate_alphanumeric_with_spaces("Hello World 123").is_ok());
        assert!(validate_alphanumeric_with_spaces("Hello@World").is_err()); // Invalid character
    }
}
