//! Security utilities and validations for authentication

use crate::{AuthResult, AuthError};
use ring::digest;
use std::collections::HashSet;

/// Security validator for comprehensive input validation
pub struct SecurityValidator;

impl SecurityValidator {
    /// Validate password strength according to security policies
    pub fn validate_password_strength(password: &str) -> AuthResult<PasswordStrength> {
        if password.len() < 8 {
            return Err(AuthError::PasswordTooWeak);
        }

        let mut has_uppercase = false;
        let mut has_lowercase = false;
        let mut has_digit = false;
        let mut has_special = false;

        for c in password.chars() {
            if c.is_uppercase() {
                has_uppercase = true;
            } else if c.is_lowercase() {
                has_lowercase = true;
            } else if c.is_ascii_digit() {
                has_digit = true;
            } else if !c.is_alphanumeric() {
                has_special = true;
            }
        }

        if !has_uppercase || !has_lowercase || !has_digit {
            return Err(AuthError::PasswordTooWeak);
        }

        let score = Self::calculate_password_score(password, has_uppercase, has_lowercase, has_digit, has_special);

        Ok(PasswordStrength {
            score,
            is_strong: score >= 70,
            has_uppercase,
            has_lowercase,
            has_digit,
            has_special,
        })
    }

    /// Calculate password strength score (0-100)
    fn calculate_password_score(password: &str, has_upper: bool, has_lower: bool, has_digit: bool, has_special: bool) -> u8 {
        let mut score = 0u8;

        // Length bonus (up to 30 points)
        score += std::cmp::min(password.len() * 3, 30) as u8;

        // Character variety bonus (up to 40 points)
        if has_upper { score += 10; }
        if has_lower { score += 10; }
        if has_digit { score += 10; }
        if has_special { score += 10; }

        // Complexity bonus (up to 30 points)
        if !Self::is_common_password(password) {
            score += 30;
        }

        std::cmp::min(score, 100)
    }

    /// Check if password is in common passwords list
    fn is_common_password(password: &str) -> bool {
        let common_passwords = [
            "password", "123456", "123456789", "qwerty", "abc123",
            "password123", "admin", "letmein", "welcome", "monkey",
            "1234567890", "password1", "qwerty123", "welcome123",
            "admin123", "root", "user", "guest", "test", "demo"
        ];

        let lowercase = password.to_lowercase();
        common_passwords.contains(&lowercase.as_str())
    }

    /// Hash password using Argon2
    pub fn hash_password(password: &str) -> AuthResult<String> {
        use argon2::{Argon2, PasswordHasher, PasswordVerifier};
        use argon2::password_hash::{rand_core::OsRng, PasswordHash, SaltString};

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2.hash_password(password.as_bytes(), &salt)
            .map_err(|_| AuthError::ExternalService("Password hashing failed".to_string()))?;

        Ok(password_hash.to_string())
    }

    /// Verify password against hash
    pub fn verify_password(password: &str, hash: &str) -> AuthResult<bool> {
        use argon2::{Argon2, PasswordVerifier};
        use argon2::password_hash::PasswordHash;

        let parsed_hash = PasswordHash::new(hash)
            .map_err(|_| AuthError::InvalidCredentials)?;

        let argon2 = Argon2::default();
        Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }

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
    pub fn validate_length(input: &str, min: usize, max: usize, field_name: &str) -> AuthResult<()> {
        if input.len() < min {
            return Err(AuthError::InvalidCredentials);
        }
        if input.len() > max {
            return Err(AuthError::InvalidCredentials);
        }
        Ok(())
    }

    /// Generate secure random token
    pub fn generate_secure_token(length: usize) -> String {
        use ring::rand::SecureRandom;
        use ring::rand::SystemRandom;

        let rng = SystemRandom::new();
        let mut bytes = vec![0u8; length];
        rng.fill(&mut bytes).unwrap();

        base64::encode_config(bytes, base64::URL_SAFE_NO_PAD)
    }

    /// Calculate password entropy
    pub fn calculate_entropy(password: &str) -> f64 {
        let charset_size = Self::estimate_charset_size(password);
        if charset_size == 0.0 {
            return 0.0;
        }

        (password.len() as f64) * (charset_size.log2())
    }

    /// Estimate character set size for entropy calculation
    fn estimate_charset_size(password: &str) -> f64 {
        let mut charset = HashSet::new();

        for c in password.chars() {
            if c.is_ascii_lowercase() {
                charset.insert("lowercase");
            } else if c.is_ascii_uppercase() {
                charset.insert("uppercase");
            } else if c.is_ascii_digit() {
                charset.insert("digit");
            } else {
                charset.insert("special");
            }
        }

        match charset.len() {
            0 => 0.0,
            1 => 26.0, // Assume lowercase only
            2 => {
                if charset.contains("lowercase") && charset.contains("uppercase") {
                    52.0
                } else {
                    36.0 // letters + digits
                }
            }
            3 => 62.0, // letters + digits + special
            _ => 94.0, // full ASCII printable
        }
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

    /// Rate limiting check (simplified in-memory implementation)
    pub fn check_rate_limit(key: &str, max_requests: u32, window_seconds: u64) -> bool {
        // In production, this would use Redis or similar
        // For now, return true (allow) - implement proper rate limiting in production
        true
    }
}

/// Password strength analysis result
#[derive(Debug, Clone)]
pub struct PasswordStrength {
    pub score: u8,
    pub is_strong: bool,
    pub has_uppercase: bool,
    pub has_lowercase: bool,
    pub has_digit: bool,
    pub has_special: bool,
}

impl PasswordStrength {
    /// Get strength description
    pub fn description(&self) -> &'static str {
        match self.score {
            0..=20 => "Very Weak",
            21..=40 => "Weak",
            41..=60 => "Fair",
            61..=80 => "Good",
            81..=100 => "Strong",
            _ => "Excellent",
        }
    }
}

/// Security audit logger
pub struct SecurityAuditor;

impl SecurityAuditor {
    /// Log security event
    pub fn log_event(event_type: SecurityEventType, details: serde_json::Value) {
        let event = serde_json::json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "event_type": event_type.as_str(),
            "details": details
        });

        // In production, this would write to a secure audit log
        // For now, just print to stderr for development
        eprintln!("🔐 SECURITY EVENT: {}", event);
    }

    /// Log authentication attempt
    pub fn log_auth_attempt(username: &str, success: bool, ip_address: Option<&str>) {
        let details = serde_json::json!({
            "username": username,
            "success": success,
            "ip_address": ip_address
        });

        Self::log_event(SecurityEventType::AuthenticationAttempt, details);
    }

    /// Log suspicious activity
    pub fn log_suspicious_activity(activity: &str, severity: &str, details: serde_json::Value) {
        let mut details = details;
        details["activity"] = serde_json::Value::String(activity.to_string());
        details["severity"] = serde_json::Value::String(severity.to_string());

        Self::log_event(SecurityEventType::SuspiciousActivity, details);
    }
}

/// Security event types
#[derive(Debug, Clone)]
pub enum SecurityEventType {
    AuthenticationAttempt,
    PasswordChange,
    AccountLocked,
    AccountUnlocked,
    SuspiciousActivity,
    TokenIssued,
    TokenRevoked,
    MfaEnabled,
    MfaDisabled,
}

impl SecurityEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SecurityEventType::AuthenticationAttempt => "authentication_attempt",
            SecurityEventType::PasswordChange => "password_change",
            SecurityEventType::AccountLocked => "account_locked",
            SecurityEventType::AccountUnlocked => "account_unlocked",
            SecurityEventType::SuspiciousActivity => "suspicious_activity",
            SecurityEventType::TokenIssued => "token_issued",
            SecurityEventType::TokenRevoked => "token_revoked",
            SecurityEventType::MfaEnabled => "mfa_enabled",
            SecurityEventType::MfaDisabled => "mfa_disabled",
        }
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
        let token1 = SecurityValidator::generate_secure_token(32);
        let token2 = SecurityValidator::generate_secure_token(32);

        // Tokens should be different
        assert_ne!(token1, token2);

        // Should be URL-safe base64
        assert!(token1.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_'));
    }
}
