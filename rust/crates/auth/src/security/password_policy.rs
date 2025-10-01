//! Password policy management and strength validation

use crate::types::{AuthResult, AuthError};
use std::collections::HashSet;

/// Password strength assessment result
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

/// Password policy validator
pub struct PasswordPolicy;

impl PasswordPolicy {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_strength_validation() {
        // Strong password
        let strong = PasswordPolicy::validate_password_strength("MySecurePass123!").unwrap();
        assert!(strong.is_strong);
        assert!(strong.score >= 70);

        // Weak password
        assert!(PasswordPolicy::validate_password_strength("weak").is_err());
        assert!(PasswordPolicy::validate_password_strength("nouppercase123").is_err());
    }

    #[test]
    fn test_entropy_calculation() {
        let entropy1 = PasswordPolicy::calculate_entropy("password");
        let entropy2 = PasswordPolicy::calculate_entropy("MySecurePass123!");
        assert!(entropy2 > entropy1);
    }
}
