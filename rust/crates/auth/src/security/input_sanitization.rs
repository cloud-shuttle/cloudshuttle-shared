//! Input sanitization and validation utilities

use crate::types::{AuthResult, AuthError};

/// Input sanitization validator
pub struct InputSanitizer;

impl InputSanitizer {
    /// Validate email format and security
    pub fn validate_email(email: &str) -> AuthResult<String> {
        // Basic email validation
        if !email.contains('@') || email.len() < 3 {
            return Err(AuthError::InvalidCredentials);
        }

        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return Err(AuthError::InvalidCredentials);
        }

        let local = parts[0];
        let domain = parts[1];

        if local.is_empty() || domain.is_empty() || !domain.contains('.') {
            return Err(AuthError::InvalidCredentials);
        }

        // Security checks
        if Self::contains_suspicious_patterns(email) {
            return Err(AuthError::InvalidCredentials);
        }

        Ok(email.to_string())
    }

    /// Check for suspicious patterns in email/username
    fn contains_suspicious_patterns(input: &str) -> bool {
        let suspicious = [
            "<script", "javascript:", "vbscript:", "onload=", "onerror=",
            "<iframe", "<object", "<embed", "eval(", "alert(",
            "../../../", "..\\..\\", "etc/passwd", "boot.ini"
        ];

        let lowercase = input.to_lowercase();
        suspicious.iter().any(|pattern| lowercase.contains(pattern))
    }

    /// Sanitize input for safe database storage
    pub fn sanitize_for_database(input: &str) -> String {
        // Remove null bytes and other dangerous characters
        input.chars()
            .filter(|&c| c != '\0' && c != '\r' && c != '\n')
            .collect::<String>()
            .trim()
            .to_string()
    }

    /// Validate input length limits
    pub fn validate_length(input: &str, min: usize, max: usize, _field_name: &str) -> AuthResult<()> {
        if input.len() < min {
            return Err(AuthError::InvalidCredentials);
        }
        if input.len() > max {
            return Err(AuthError::InvalidCredentials);
        }
        Ok(())
    }

    /// Check if input contains SQL injection patterns
    pub fn detect_sql_injection(input: &str) -> bool {
        let patterns = [
            "'", "\"", ";", "--", "/*", "*/", "xp_", "sp_",
            "union", "select", "insert", "update", "delete", "drop",
            "exec", "execute", "script", "javascript", "vbscript"
        ];

        let lowercase = input.to_lowercase();
        patterns.iter().any(|pattern| lowercase.contains(pattern))
    }

    /// Check if input contains XSS patterns
    pub fn detect_xss(input: &str) -> bool {
        let patterns = [
            "<script", "</script>", "<iframe", "</iframe>", "<object", "</object>",
            "<embed", "</embed>", "javascript:", "vbscript:", "data:", "onload=",
            "onerror=", "onclick=", "onmouseover=", "<img", "<link", "<meta"
        ];

        let lowercase = input.to_lowercase();
        patterns.iter().any(|pattern| lowercase.contains(pattern))
    }

    /// Sanitize HTML content (basic XSS prevention)
    pub fn sanitize_html(input: &str) -> String {
        input
            .replace("<script", "&lt;script")
            .replace("</script", "&lt;/script")
            .replace("<iframe", "&lt;iframe")
            .replace("</iframe", "&lt;/iframe")
            .replace("javascript:", "")
            .replace("vbscript:", "")
            .replace("onload=", "onload&#x3D;")
            .replace("onerror=", "onerror&#x3D;")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        assert!(InputSanitizer::validate_email("user@example.com").is_ok());
        assert!(InputSanitizer::validate_email("invalid-email").is_err());
        assert!(InputSanitizer::validate_email("<script>alert('xss')</script>@evil.com").is_err());
    }

    #[test]
    fn test_sql_injection_detection() {
        assert!(InputSanitizer::detect_sql_injection("'; DROP TABLE users; --"));
        assert!(InputSanitizer::detect_sql_injection("SELECT * FROM users"));
        assert!(!InputSanitizer::detect_sql_injection("normal input"));
    }

    #[test]
    fn test_xss_detection() {
        assert!(InputSanitizer::detect_xss("<script>alert('xss')</script>"));
        assert!(InputSanitizer::detect_xss("javascript:evil()"));
        assert!(!InputSanitizer::detect_xss("normal text"));
    }

    #[test]
    fn test_html_sanitization() {
        let input = "<script>alert('xss')</script><p>safe content</p>";
        let sanitized = InputSanitizer::sanitize_html(input);
        assert!(!sanitized.contains("<script"));
        assert!(sanitized.contains("&lt;script"));
        assert!(sanitized.contains("<p>safe content</p>"));
    }
}
