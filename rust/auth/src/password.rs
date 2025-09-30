//! Password hashing utilities

use cloudshuttle_error_handling::CloudShuttleError;
use rand::Rng;
use std::fmt;

/// Password hashing service
pub struct PasswordHasher {
    cost: u32,
}

impl PasswordHasher {
    /// Create a new password hasher with default cost
    pub fn new() -> Self {
        Self { cost: 12 }
    }

    /// Create a new password hasher with custom cost
    pub fn with_cost(cost: u32) -> Self {
        Self { cost }
    }

    /// Hash a password
    pub fn hash(&self, password: &str) -> Result<String, PasswordHashError> {
        bcrypt::hash(password, self.cost)
            .map_err(|e| PasswordHashError::HashError(e.to_string()))
    }

    /// Verify a password against a hash
    pub fn verify(&self, password: &str, hash: &str) -> Result<bool, PasswordHashError> {
        bcrypt::verify(password, hash)
            .map_err(|e| PasswordHashError::VerifyError(e.to_string()))
    }

    /// Check if a hash needs to be upgraded (cost changed)
    pub fn needs_upgrade(&self, hash: &str) -> bool {
        bcrypt::get_rounds(hash).map_or(true, |rounds| rounds != self.cost)
    }
}

impl Default for PasswordHasher {
    fn default() -> Self {
        Self::new()
    }
}

/// Password hash errors
#[derive(Debug, thiserror::Error)]
pub enum PasswordHashError {
    #[error("Password hashing failed: {0}")]
    HashError(String),

    #[error("Password verification failed: {0}")]
    VerifyError(String),
}

impl From<PasswordHashError> for CloudShuttleError {
    fn from(err: PasswordHashError) -> Self {
        CloudShuttleError::crypto(err.to_string())
    }
}

/// Secure token generator for password reset, email verification, etc.
pub struct TokenGenerator {
    length: usize,
}

impl TokenGenerator {
    /// Create a new token generator with default length (32)
    pub fn new() -> Self {
        Self { length: 32 }
    }

    /// Create a new token generator with custom length
    pub fn with_length(length: usize) -> Self {
        Self { length }
    }

    /// Generate a secure random token
    pub fn generate_token(&self) -> Result<String, CloudShuttleError> {
        let charset = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                        abcdefghijklmnopqrstuvwxyz\
                        0123456789";
        let charset_len = charset.len();

        let mut rng = rand::thread_rng();
        let token_bytes: Vec<u8> = (0..self.length)
            .map(|_| charset[rng.gen_range(0..charset_len)])
            .collect();

        String::from_utf8(token_bytes)
            .map_err(|e| CloudShuttleError::crypto(format!("Token generation failed: {}", e)))
    }

    /// Generate a secure random numeric token
    pub fn generate_numeric_token(&self, length: usize) -> Result<String, CloudShuttleError> {
        let mut rng = rand::thread_rng();
        let mut token = String::with_capacity(length);

        for _ in 0..length {
            token.push_str(&rng.gen_range(0..10).to_string());
        }

        Ok(token)
    }
}

impl Default for TokenGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Password strength validator
pub struct PasswordValidator {
    min_length: usize,
    require_uppercase: bool,
    require_lowercase: bool,
    require_numbers: bool,
    require_special_chars: bool,
}

impl PasswordValidator {
    /// Create a new password validator with default rules
    pub fn new() -> Self {
        Self {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special_chars: false,
        }
    }

    /// Set minimum length requirement
    pub fn min_length(mut self, length: usize) -> Self {
        self.min_length = length;
        self
    }

    /// Require uppercase characters
    pub fn require_uppercase(mut self, require: bool) -> Self {
        self.require_uppercase = require;
        self
    }

    /// Require lowercase characters
    pub fn require_lowercase(mut self, require: bool) -> Self {
        self.require_lowercase = require;
        self
    }

    /// Require numbers
    pub fn require_numbers(mut self, require: bool) -> Self {
        self.require_numbers = require;
        self
    }

    /// Require special characters
    pub fn require_special_chars(mut self, require: bool) -> Self {
        self.require_special_chars = require;
        self
    }

    /// Validate password strength
    pub fn validate(&self, password: &str) -> Result<(), PasswordValidationError> {
        if password.len() < self.min_length {
            return Err(PasswordValidationError::TooShort {
                min_length: self.min_length,
                actual_length: password.len(),
            });
        }

        if self.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            return Err(PasswordValidationError::MissingUppercase);
        }

        if self.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            return Err(PasswordValidationError::MissingLowercase);
        }

        if self.require_numbers && !password.chars().any(|c| c.is_numeric()) {
            return Err(PasswordValidationError::MissingNumbers);
        }

        if self.require_special_chars && !password.chars().any(|c| !c.is_alphanumeric()) {
            return Err(PasswordValidationError::MissingSpecialChars);
        }

        Ok(())
    }

    /// Get password requirements as a string
    pub fn requirements_text(&self) -> String {
        let mut requirements = Vec::new();

        requirements.push(format!("At least {} characters long", self.min_length));

        if self.require_uppercase {
            requirements.push("At least one uppercase letter".to_string());
        }

        if self.require_lowercase {
            requirements.push("At least one lowercase letter".to_string());
        }

        if self.require_numbers {
            requirements.push("At least one number".to_string());
        }

        if self.require_special_chars {
            requirements.push("At least one special character".to_string());
        }

        requirements.join(", ")
    }
}

impl Default for PasswordValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Password validation errors
#[derive(Debug, thiserror::Error)]
pub enum PasswordValidationError {
    #[error("Password too short: minimum {min_length} characters, got {actual_length}")]
    TooShort { min_length: usize, actual_length: usize },

    #[error("Password must contain at least one uppercase letter")]
    MissingUppercase,

    #[error("Password must contain at least one lowercase letter")]
    MissingLowercase,

    #[error("Password must contain at least one number")]
    MissingNumbers,

    #[error("Password must contain at least one special character")]
    MissingSpecialChars,
}
