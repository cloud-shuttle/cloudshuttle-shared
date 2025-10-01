//! Security utilities and validations for authentication
//!
//! This module orchestrates multiple security domains through specialized sub-modules:
//! - `password_policy`: Password strength validation and entropy calculation
//! - `input_sanitization`: XSS/SQL injection prevention and input validation
//! - `encryption`: Cryptographic operations and secure token generation
//! - `rate_limiting`: Request throttling and rate limiting (currently stubbed)

pub mod password_policy;
pub mod input_sanitization;
pub mod encryption;
pub mod rate_limiting;

// Re-export for backward compatibility
pub use password_policy::{PasswordPolicy, PasswordStrength};
pub use input_sanitization::InputSanitizer;
pub use encryption::CryptoUtils;
pub use rate_limiting::{RateLimiter, check_rate_limit};

use crate::types::{AuthResult, AuthError};

/// Main security validator - orchestrates all security modules
pub struct SecurityValidator;

impl SecurityValidator {
    /// Comprehensive security validation
    pub fn validate_input(&self, input: &str) -> AuthResult<()> {
        // Orchestrate all security checks
        InputSanitizer::sanitize_html(input);
        if InputSanitizer::detect_xss(input) {
            return Err(AuthError::InvalidCredentials);
        }
        if InputSanitizer::detect_sql_injection(input) {
            return Err(AuthError::InvalidCredentials);
        }
        Ok(())
    }

    /// Validate password strength (delegates to PasswordPolicy)
    pub fn validate_password_strength(password: &str) -> AuthResult<PasswordStrength> {
        PasswordPolicy::validate_password_strength(password)
    }

    /// Hash password (delegates to CryptoUtils)
    pub fn hash_password(password: &str) -> AuthResult<String> {
        CryptoUtils::hash_password(password)
    }

    /// Verify password (delegates to CryptoUtils)
    pub fn verify_password(password: &str, hash: &str) -> AuthResult<bool> {
        CryptoUtils::verify_password(password, hash)
    }

    /// Validate email (delegates to InputSanitizer)
    pub fn validate_email(email: &str) -> AuthResult<String> {
        InputSanitizer::validate_email(email)
    }

    /// Sanitize for database (delegates to InputSanitizer)
    pub fn sanitize_for_database(input: &str) -> String {
        InputSanitizer::sanitize_for_database(input)
    }

    /// Validate length (delegates to InputSanitizer)
    pub fn validate_length(input: &str, min: usize, max: usize, field_name: &str) -> AuthResult<()> {
        InputSanitizer::validate_length(input, min, max, field_name)
    }

    /// Generate secure token (delegates to CryptoUtils)
    pub fn generate_secure_token(length: usize) -> AuthResult<String> {
        CryptoUtils::generate_secure_token(length)
    }

    /// Calculate entropy (delegates to PasswordPolicy)
    pub fn calculate_entropy(password: &str) -> f64 {
        PasswordPolicy::calculate_entropy(password)
    }

    /// Detect SQL injection (delegates to InputSanitizer)
    pub fn detect_sql_injection(input: &str) -> bool {
        InputSanitizer::detect_sql_injection(input)
    }

    /// Detect XSS (delegates to InputSanitizer)
    pub fn detect_xss(input: &str) -> bool {
        InputSanitizer::detect_xss(input)
    }

    /// Sanitize HTML (delegates to InputSanitizer)
    pub fn sanitize_html(input: &str) -> String {
        InputSanitizer::sanitize_html(input)
    }

    /// Check rate limit (delegates to rate_limiting module)
    pub fn check_rate_limit(key: &str, max_requests: u32, window_seconds: u64) -> bool {
        check_rate_limit(key, max_requests, window_seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_strength_validation() {
        // Strong password
        let strong = SecurityValidator::validate_password_strength("MySecurePass123!").unwrap();
        assert!(strong.is_strong);
        assert!(strong.score >= 70);

        // Weak password
        assert!(SecurityValidator::validate_password_strength("weak").is_err());
        assert!(SecurityValidator::validate_password_strength("nouppercase123").is_err());
    }

    #[test]
    fn test_password_hashing() {
        let password = "MyTestPassword123!";
        let hash = SecurityValidator::hash_password(password).unwrap();

        // Hash should be different from password
        assert_ne!(hash, password);

        // Should be able to verify
        assert!(SecurityValidator::verify_password(password, &hash).unwrap());
        assert!(!SecurityValidator::verify_password("wrongpassword", &hash).unwrap());
    }

    #[test]
    fn test_email_validation() {
        assert!(SecurityValidator::validate_email("user@example.com").is_ok());
        assert!(SecurityValidator::validate_email("invalid-email").is_err());
        assert!(SecurityValidator::validate_email("<script>alert('xss')</script>@evil.com").is_err());
    }

    #[test]
    fn test_sql_injection_detection() {
        assert!(SecurityValidator::detect_sql_injection("'; DROP TABLE users; --"));
        assert!(SecurityValidator::detect_sql_injection("SELECT * FROM users"));
        assert!(!SecurityValidator::detect_sql_injection("normal input"));
    }

    #[test]
    fn test_xss_detection() {
        assert!(SecurityValidator::detect_xss("<script>alert('xss')</script>"));
        assert!(SecurityValidator::detect_xss("javascript:evil()"));
        assert!(!SecurityValidator::detect_xss("normal text"));
    }

    #[test]
    fn test_html_sanitization() {
        let input = "<script>alert('xss')</script><p>safe content</p>";
        let sanitized = SecurityValidator::sanitize_html(input);
        assert!(!sanitized.contains("<script"));
        assert!(sanitized.contains("&lt;script"));
        assert!(sanitized.contains("<p>safe content</p>"));
    }

    #[test]
    fn test_entropy_calculation() {
        let entropy1 = SecurityValidator::calculate_entropy("password");
        let entropy2 = SecurityValidator::calculate_entropy("MySecurePass123!");
        assert!(entropy2 > entropy1);
    }

    #[test]
    fn test_secure_token_generation() {
        let token1 = SecurityValidator::generate_secure_token(32).unwrap();
        let token2 = SecurityValidator::generate_secure_token(32).unwrap();

        // Tokens should be different
        assert_ne!(token1, token2);

        // Should be URL-safe base64
        assert!(token1.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_'));
    }

    #[test]
    fn test_comprehensive_validation() {
        let validator = SecurityValidator;
        assert!(validator.validate_input("normal input").is_ok());
        assert!(validator.validate_input("<script>alert('xss')</script>").is_err());
    }
}
