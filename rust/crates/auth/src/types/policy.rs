//! Authentication security policies and configurations

use serde::{Deserialize, Serialize};

/// Password policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicy {
    pub min_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_special_chars: bool,
    pub prevent_common_passwords: bool,
    pub max_age_days: Option<u32>,
    pub prevent_reuse_count: Option<u32>,
    pub lockout_on_weak_password: bool,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special_chars: false,
            prevent_common_passwords: true,
            max_age_days: Some(90),
            prevent_reuse_count: Some(5),
            lockout_on_weak_password: false,
        }
    }
}

impl PasswordPolicy {
    /// Validate a password against this policy
    pub fn validate_password(&self, password: &str) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if password.len() < self.min_length {
            errors.push(format!("Password must be at least {} characters long", self.min_length));
        }

        if self.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            errors.push("Password must contain at least one uppercase letter".to_string());
        }

        if self.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            errors.push("Password must contain at least one lowercase letter".to_string());
        }

        if self.require_numbers && !password.chars().any(|c| c.is_numeric()) {
            errors.push("Password must contain at least one number".to_string());
        }

        if self.require_special_chars && !password.chars().any(|c| !c.is_alphanumeric()) {
            errors.push("Password must contain at least one special character".to_string());
        }

        if self.prevent_common_passwords && self.is_common_password(password) {
            errors.push("Password is too common, please choose a different one".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Check if password is in a list of common passwords
    fn is_common_password(&self, password: &str) -> bool {
        let common_passwords = [
            "password", "123456", "123456789", "qwerty", "abc123",
            "password123", "admin", "letmein", "welcome", "monkey",
            "1234567890", "password1", "qwerty123", "welcome123"
        ];

        common_passwords.contains(&password.to_lowercase().as_str())
    }

    /// Generate a password strength score (0-100)
    pub fn password_strength_score(&self, password: &str) -> u8 {
        let mut score = 0u8;

        // Length score (up to 30 points)
        let length_score = if password.len() >= self.min_length {
            std::cmp::min(password.len() * 3, 30) as u8
        } else {
            0
        };
        score += length_score;

        // Character variety score (up to 40 points)
        let mut variety_score = 0u8;
        if password.chars().any(|c| c.is_uppercase()) { variety_score += 10; }
        if password.chars().any(|c| c.is_lowercase()) { variety_score += 10; }
        if password.chars().any(|c| c.is_numeric()) { variety_score += 10; }
        if password.chars().any(|c| !c.is_alphanumeric()) { variety_score += 10; }
        score += variety_score;

        // Complexity bonus (up to 30 points)
        if !self.is_common_password(password) {
            score += 30;
        }

        std::cmp::min(score, 100)
    }

    /// Check if password meets minimum requirements
    pub fn meets_minimum_requirements(&self, password: &str) -> bool {
        self.validate_password(password).is_ok()
    }
}

/// Login attempt tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginAttempt {
    pub user_id: String,
    pub ip_address: String,
    pub user_agent: Option<String>,
    pub success: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub failure_reason: Option<String>,
}

impl LoginAttempt {
    pub fn success(user_id: String, ip_address: String) -> Self {
        Self {
            user_id,
            ip_address,
            user_agent: None,
            success: true,
            timestamp: chrono::Utc::now(),
            failure_reason: None,
        }
    }

    pub fn failure(user_id: String, ip_address: String, reason: String) -> Self {
        Self {
            user_id,
            ip_address,
            user_agent: None,
            success: false,
            timestamp: chrono::Utc::now(),
            failure_reason: Some(reason),
        }
    }

    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }

    pub fn is_recent(&self, minutes: i64) -> bool {
        let now = chrono::Utc::now();
        let duration = now.signed_duration_since(self.timestamp);
        duration.num_minutes() <= minutes
    }
}

/// Account lockout policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockoutPolicy {
    pub max_attempts: u32,
    pub lockout_duration_minutes: u32,
    pub reset_after_minutes: u32,
    pub progressive_lockout: bool,
    pub notify_on_lockout: bool,
    pub max_lockout_duration_hours: Option<u32>,
}

impl Default for LockoutPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            lockout_duration_minutes: 15,
            reset_after_minutes: 30,
            progressive_lockout: true,
            notify_on_lockout: true,
            max_lockout_duration_hours: Some(24),
        }
    }
}

impl LockoutPolicy {
    /// Calculate lockout duration for progressive lockout
    pub fn calculate_lockout_duration(&self, attempt_count: u32) -> chrono::Duration {
        if !self.progressive_lockout {
            return chrono::Duration::minutes(self.lockout_duration_minutes as i64);
        }

        // Progressive lockout: each attempt increases duration
        let multiplier = std::cmp::min(attempt_count, 10); // Cap at 10x
        let duration_minutes = (self.lockout_duration_minutes * multiplier) as i64;

        // Cap at maximum duration if set
        if let Some(max_hours) = self.max_lockout_duration_hours {
            let max_minutes = (max_hours * 60) as i64;
            chrono::Duration::minutes(std::cmp::min(duration_minutes, max_minutes))
        } else {
            chrono::Duration::minutes(duration_minutes)
        }
    }

    /// Check if account should be locked based on recent attempts
    pub fn should_lock_account(&self, recent_failures: &[&LoginAttempt]) -> bool {
        recent_failures.len() >= self.max_attempts as usize
    }

    /// Check if lockout period has expired
    pub fn is_lockout_expired(&self, lockout_start: chrono::DateTime<chrono::Utc>, attempt_count: u32) -> bool {
        let lockout_duration = self.calculate_lockout_duration(attempt_count);
        let now = chrono::Utc::now();
        now.signed_duration_since(lockout_start) > lockout_duration
    }

    /// Get number of failed attempts that should reset the counter
    pub fn reset_window_minutes(&self) -> u32 {
        self.reset_after_minutes
    }
}

/// Multi-factor authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaConfig {
    pub enabled: bool,
    pub required_for_roles: Vec<String>,
    pub required_for_all_users: bool,
    pub allowed_methods: Vec<MfaMethod>,
    pub grace_period_days: Option<u32>,
    pub backup_codes_count: u32,
    pub remember_device_days: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MfaMethod {
    TOTP,
    SMS,
    Email,
    HardwareToken,
    BackupCodes,
}

impl Default for MfaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            required_for_roles: vec!["admin".to_string()],
            required_for_all_users: false,
            allowed_methods: vec![MfaMethod::TOTP],
            grace_period_days: Some(7),
            backup_codes_count: 10,
            remember_device_days: Some(30),
        }
    }
}

impl MfaConfig {
    /// Check if MFA is required for a user with given roles
    pub fn is_required_for_user(&self, user_roles: &[String]) -> bool {
        if !self.enabled {
            return false;
        }

        if self.required_for_all_users {
            return true;
        }

        // Check if user has any role that requires MFA
        user_roles.iter().any(|role| self.required_for_roles.contains(role))
    }

    /// Check if a method is allowed
    pub fn is_method_allowed(&self, method: &MfaMethod) -> bool {
        self.allowed_methods.contains(method)
    }

    /// Get available methods as strings
    pub fn available_methods_strings(&self) -> Vec<String> {
        self.allowed_methods.iter().map(|m| format!("{:?}", m)).collect()
    }

    /// Check if device remembering is enabled
    pub fn device_remembering_enabled(&self) -> bool {
        self.remember_device_days.is_some()
    }
}

/// Security audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub event_type: SecurityEventType,
    pub user_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub details: serde_json::Value,
    pub severity: SecuritySeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEventType {
    LoginSuccess,
    LoginFailure,
    Logout,
    PasswordChange,
    AccountLocked,
    AccountUnlocked,
    MfaEnabled,
    MfaDisabled,
    SuspiciousActivity,
    PasswordReset,
    TokenIssued,
    TokenRevoked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl SecurityEvent {
    pub fn login_success(user_id: String, ip_address: String) -> Self {
        Self {
            event_type: SecurityEventType::LoginSuccess,
            user_id: Some(user_id),
            ip_address: Some(ip_address),
            user_agent: None,
            timestamp: chrono::Utc::now(),
            details: serde_json::json!({}),
            severity: SecuritySeverity::Low,
        }
    }

    pub fn login_failure(user_id: String, ip_address: String, reason: &str) -> Self {
        Self {
            event_type: SecurityEventType::LoginFailure,
            user_id: Some(user_id),
            ip_address: Some(ip_address),
            user_agent: None,
            timestamp: chrono::Utc::now(),
            details: serde_json::json!({"reason": reason}),
            severity: SecuritySeverity::Medium,
        }
    }

    pub fn suspicious_activity(ip_address: String, details: serde_json::Value) -> Self {
        Self {
            event_type: SecurityEventType::SuspiciousActivity,
            user_id: None,
            ip_address: Some(ip_address),
            user_agent: None,
            timestamp: chrono::Utc::now(),
            details,
            severity: SecuritySeverity::High,
        }
    }

    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_policy_validation() {
        let policy = PasswordPolicy::default();

        // Valid password
        assert!(policy.validate_password("ValidPass123").is_ok());

        // Invalid passwords
        assert!(policy.validate_password("short").is_err());
        assert!(policy.validate_password("nouppercase123").is_err());
        assert!(policy.validate_password("NOLOWERCASE123").is_err());
        assert!(policy.validate_password("NoNumbers").is_err());
        assert!(policy.validate_password("password").is_err()); // common password
    }

    #[test]
    fn test_password_strength_score() {
        let policy = PasswordPolicy::default();

        assert!(policy.password_strength_score("weak") < 50);
        assert!(policy.password_strength_score("StrongPass123!") > 80);
        assert_eq!(policy.password_strength_score("VeryStrongPassword123!@#"), 100);
    }

    #[test]
    fn test_lockout_policy() {
        let policy = LockoutPolicy::default();

        let duration = policy.calculate_lockout_duration(1);
        assert_eq!(duration.num_minutes(), 15);

        if policy.progressive_lockout {
            let progressive_duration = policy.calculate_lockout_duration(3);
            assert_eq!(progressive_duration.num_minutes(), 45); // 15 * 3
        }
    }

    #[test]
    fn test_mfa_config() {
        let mut config = MfaConfig::default();
        config.required_for_roles = vec!["admin".to_string()];

        assert!(config.is_required_for_user(&["admin".to_string()]));
        assert!(!config.is_required_for_user(&["user".to_string()]));
        assert!(config.is_method_allowed(&MfaMethod::TOTP));
        assert!(!config.is_method_allowed(&MfaMethod::SMS));
    }

    #[test]
    fn test_security_events() {
        let event = SecurityEvent::login_success("user123".to_string(), "127.0.0.1".to_string());
        assert!(matches!(event.event_type, SecurityEventType::LoginSuccess));
        assert_eq!(event.severity, SecuritySeverity::Low);

        let failure_event = SecurityEvent::login_failure("user123".to_string(), "127.0.0.1".to_string(), "wrong_password");
        assert!(matches!(failure_event.event_type, SecurityEventType::LoginFailure));
        assert_eq!(failure_event.severity, SecuritySeverity::Medium);
    }
}
