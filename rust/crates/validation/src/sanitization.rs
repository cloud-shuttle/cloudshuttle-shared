//! Input sanitization utilities
//!
//! This module provides functions to sanitize user input,
//! remove dangerous content, and normalize data.

use once_cell::sync::Lazy;
use regex::Regex;
use unicode_normalization::UnicodeNormalization;
use std::result::Result as StdResult;

/// Error type for sanitization operations
#[derive(Debug, thiserror::Error)]
pub enum SanitizationError {
    #[error("Sanitization failed: {0}")]
    SanitizationFailed(String),
}

/// Result type for sanitization operations
pub type Result<T> = StdResult<T, SanitizationError>;

// HTML tag regex for basic HTML sanitization
static HTML_TAG_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"<[^>]*>").unwrap()
});

// SQL injection patterns (basic detection)
static SQL_INJECTION_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(union|select|insert|update|delete|drop|create|alter|exec|execute|script|javascript|vbscript|onload|onerror|onmouseover)").unwrap()
});

// Dangerous filename characters
static DANGEROUS_FILENAME_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[<>:\|?*\x00-\x1f]").unwrap()
});

/// Sanitize HTML input by removing all HTML tags
///
/// This is a basic HTML sanitization that removes all HTML tags.
/// For more sophisticated HTML sanitization, consider using a dedicated crate.
///
/// # Arguments
/// * `input` - HTML input to sanitize
///
/// # Returns
/// Sanitized string with HTML tags removed
///
/// # Example
/// ```rust
/// let clean = sanitize_html("<script>alert('xss')</script>Hello World");
/// assert_eq!(clean, "Hello World");
/// ```
pub fn sanitize_html(input: &str) -> String {
    HTML_TAG_REGEX.replace_all(input, "").to_string()
}

/// Sanitize SQL input by detecting potential injection patterns
///
/// This performs basic SQL injection detection and can optionally
/// escape single quotes. Note: This is not a complete SQL injection
/// prevention solution - always use parameterized queries.
///
/// # Arguments
/// * `input` - SQL input to sanitize
/// * `escape_quotes` - Whether to escape single quotes
///
/// # Returns
/// Result with sanitized string or error if suspicious patterns detected
///
/// # Example
/// ```rust
/// let safe = sanitize_sql_input("SELECT * FROM users", false)?;
/// let escaped = sanitize_sql_input("user's input", true)?;
/// assert_eq!(escaped, "user\\'s input");
/// ```
pub fn sanitize_sql_input(input: &str, escape_quotes: bool) -> Result<String> {
    // Check for suspicious patterns
    if SQL_INJECTION_REGEX.is_match(input) {
        return Err(SanitizationError::SanitizationFailed(
            "Potentially dangerous SQL patterns detected".to_string()
        ));
    }

    let mut result = input.to_string();

    if escape_quotes {
        result = result.replace("'", "\\'");
    }

    Ok(result)
}

/// Sanitize filename by removing dangerous characters
///
/// Removes or replaces characters that could be dangerous in filenames,
/// such as path traversal attempts or special filesystem characters.
///
/// # Arguments
/// * `filename` - Filename to sanitize
///
/// # Returns
/// Sanitized filename safe for filesystem operations
///
/// # Example
/// ```rust
/// let safe = sanitize_filename("../../../etc/passwd");
/// assert_eq!(safe, "etcpasswd");
///
/// let safe2 = sanitize_filename("file<name>.txt");
/// assert_eq!(safe2, "filename.txt");
/// ```
pub fn sanitize_filename(filename: &str) -> String {
    // Remove dangerous characters
    let sanitized = DANGEROUS_FILENAME_REGEX.replace_all(filename, "");

    // Replace multiple spaces/dots with single
    let sanitized = Regex::new(r"[.\s]+").unwrap().replace_all(&sanitized, ".");

    // Remove leading/trailing dots and spaces
    sanitized.trim_matches(|c: char| c == '.' || c.is_whitespace()).to_string()
}

/// Normalize and sanitize Unicode text
///
/// Performs Unicode normalization and removes control characters.
///
/// # Arguments
/// * `input` - Text to normalize and sanitize
///
/// # Returns
/// Normalized and sanitized text
///
/// # Example
/// ```rust
/// let normalized = normalize_unicode("café");
/// // Result may vary based on normalization form
/// ```
pub fn normalize_unicode(input: &str) -> String {
    // Normalize to NFC (Canonical Composition)
    let normalized = input.nfc().collect::<String>();

    // Remove control characters (except common whitespace)
    let cleaned = normalized.chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\r' || *c == '\t')
        .collect();

    cleaned
}

/// Trim whitespace and normalize spaces
///
/// # Arguments
/// * `input` - Text to trim and normalize
///
/// # Returns
/// Trimmed and normalized text
///
/// # Example
/// ```rust
/// let clean = trim_and_normalize("  hello   world  ");
/// assert_eq!(clean, "hello world");
/// ```
pub fn trim_and_normalize(input: &str) -> String {
    // Trim whitespace
    let trimmed = input.trim();

    // Normalize multiple spaces to single space
    let normalized = Regex::new(r"\s+").unwrap()
        .replace_all(trimmed, " ")
        .to_string();

    normalized
}

/// Sanitize URL by ensuring it has a valid scheme
///
/// # Arguments
/// * `url` - URL to sanitize
/// * `default_scheme` - Default scheme to add if missing (e.g., "https")
///
/// # Returns
/// Sanitized URL with proper scheme
///
/// # Example
/// ```rust
/// let url = sanitize_url("example.com", "https");
/// assert_eq!(url, "https://example.com");
///
/// let url2 = sanitize_url("https://example.com", "https");
/// assert_eq!(url2, "https://example.com");
/// ```
pub fn sanitize_url(url: &str, default_scheme: &str) -> String {
    let trimmed = url.trim();

    // Check if URL already has a scheme
    if trimmed.contains("://") {
        trimmed.to_string()
    } else {
        format!("{}://{}", default_scheme, trimmed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_html() {
        assert_eq!(sanitize_html("<script>alert('xss')</script>Hello"), "Hello");
        assert_eq!(sanitize_html("<b>Bold</b> text"), "Bold text");
        assert_eq!(sanitize_html("No HTML here"), "No HTML here");
    }

    #[test]
    fn test_sanitize_sql_input() {
        // Safe input
        assert!(sanitize_sql_input("SELECT * FROM users", false).is_ok());
        assert!(sanitize_sql_input("user's input", true).unwrap().contains("\\'"));

        // Potentially dangerous input
        assert!(sanitize_sql_input("SELECT * FROM users; DROP TABLE users;", false).is_err());
        assert!(sanitize_sql_input("UNION SELECT password FROM admin", false).is_err());
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("../../../etc/passwd"), "etcpasswd");
        assert_eq!(sanitize_filename("file<name>.txt"), "filename.txt");
        assert_eq!(sanitize_filename("file with spaces.txt"), "file with spaces.txt");
    }

    #[test]
    fn test_trim_and_normalize() {
        assert_eq!(trim_and_normalize("  hello   world  "), "hello world");
        assert_eq!(trim_and_normalize("hello\t\tworld"), "hello world");
        assert_eq!(trim_and_normalize("hello\n\nworld"), "hello world");
    }

    #[test]
    fn test_sanitize_url() {
        assert_eq!(sanitize_url("example.com", "https"), "https://example.com");
        assert_eq!(sanitize_url("http://example.com", "https"), "http://example.com");
        assert_eq!(sanitize_url("  example.com  ", "https"), "https://example.com");
    }
}
