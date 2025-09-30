//! Secret management utilities
//!
//! This module provides secure handling of sensitive configuration
//! values like passwords, API keys, and encryption keys.

use std::collections::HashMap;
use std::result::Result as StdResult;

/// Error type for secret operations
#[derive(Debug, thiserror::Error)]
pub enum SecretError {
    #[error("Secret not found: {0}")]
    SecretNotFound(String),

    #[error("Invalid secret format: {0}")]
    InvalidFormat(String),

    #[error("Secret decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Environment variable not set: {0}")]
    EnvVarNotSet(String),
}

/// Result type for secret operations
pub type Result<T> = StdResult<T, SecretError>;

/// Secret value that can be either plain text or encrypted
#[derive(Debug, Clone)]
pub enum Secret {
    /// Plain text secret (not recommended for production)
    Plain(String),
    /// Encrypted secret that needs decryption
    Encrypted(String),
}

/// Configuration secret with additional metadata
#[derive(Debug, Clone)]
pub struct ConfigSecret {
    /// The secret value
    pub value: Secret,
    /// Whether this secret is required
    pub required: bool,
    /// Environment variable name if loaded from env
    pub env_var: Option<String>,
    /// Description for documentation
    pub description: Option<String>,
}

impl ConfigSecret {
    /// Create a new plain text secret
    pub fn plain(value: impl Into<String>) -> Self {
        Self {
            value: Secret::Plain(value.into()),
            required: true,
            env_var: None,
            description: None,
        }
    }

    /// Create a new encrypted secret
    pub fn encrypted(value: impl Into<String>) -> Self {
        Self {
            value: Secret::Encrypted(value.into()),
            required: true,
            env_var: None,
            description: None,
        }
    }

    /// Mark as optional
    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }

    /// Set environment variable name
    pub fn env_var(mut self, name: impl Into<String>) -> Self {
        self.env_var = Some(name.into());
        self
    }

    /// Set description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Get the plain text value (decrypts if necessary)
    pub fn get_plain(&self) -> Result<String> {
        match &self.value {
            Secret::Plain(text) => Ok(text.clone()),
            Secret::Encrypted(_) => {
                // In a real implementation, this would decrypt the value
                Err(SecretError::DecryptionFailed("Decryption not implemented".to_string()))
            }
        }
    }
}

/// Secure configuration container
#[derive(Debug)]
pub struct SecureConfig {
    secrets: HashMap<String, ConfigSecret>,
}

impl SecureConfig {
    /// Create a new secure configuration
    pub fn new() -> Self {
        Self {
            secrets: HashMap::new(),
        }
    }

    /// Add a secret to the configuration
    pub fn add_secret(&mut self, key: impl Into<String>, secret: ConfigSecret) {
        self.secrets.insert(key.into(), secret);
    }

    /// Get a secret by key
    pub fn get_secret(&self, key: &str) -> Option<&ConfigSecret> {
        self.secrets.get(key)
    }

    /// Get plain text value of a secret
    pub fn get_plain_value(&self, key: &str) -> Result<String> {
        self.get_secret(key)
            .ok_or_else(|| SecretError::SecretNotFound(key.to_string()))?
            .get_plain()
    }

    /// Load secrets from environment variables
    pub fn load_from_env(&mut self) -> Result<()> {
        for (key, secret) in &mut self.secrets {
            if let Some(env_var) = &secret.env_var {
                match std::env::var(env_var) {
                    Ok(value) => {
                        secret.value = Secret::Plain(value);
                    }
                    Err(std::env::VarError::NotPresent) => {
                        if secret.required {
                            return Err(SecretError::EnvVarNotSet(env_var.clone()));
                        }
                    }
                    Err(_) => {
                        if secret.required {
                            return Err(SecretError::EnvVarNotSet(env_var.clone()));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Validate that all required secrets are present
    pub fn validate(&self) -> Result<()> {
        for (key, secret) in &self.secrets {
            if secret.required {
                match &secret.value {
                    Secret::Plain(text) if text.trim().is_empty() => {
                        return Err(SecretError::SecretNotFound(format!("{} is empty", key)));
                    }
                    Secret::Encrypted(_) => {
                        // Encrypted secrets need decryption capability
                        return Err(SecretError::DecryptionFailed(format!("{} needs decryption", key)));
                    }
                    _ => {} // Plain secret with content is OK
                }
            }
        }
        Ok(())
    }
}

impl Default for SecureConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility function to create a database URL secret
pub fn database_url_secret() -> ConfigSecret {
    ConfigSecret::plain("")
        .env_var("DATABASE_URL")
        .description("Database connection URL")
}

/// Utility function to create an encryption key secret
pub fn encryption_key_secret() -> ConfigSecret {
    ConfigSecret::plain("")
        .env_var("ENCRYPTION_KEY")
        .description("AES-256 encryption key (32 bytes)")
}

/// Utility function to create a JWT secret
pub fn jwt_secret() -> ConfigSecret {
    ConfigSecret::plain("")
        .env_var("JWT_SECRET")
        .description("JWT signing secret")
}

/// Utility function to create an API key secret
pub fn api_key_secret(name: &str) -> ConfigSecret {
    ConfigSecret::plain("")
        .env_var(format!("{}_API_KEY", name.to_uppercase()))
        .description(format!("{} API key", name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_creation() {
        let secret = ConfigSecret::plain("my-secret")
            .env_var("MY_SECRET")
            .description("Test secret");

        assert!(secret.required);
        assert_eq!(secret.env_var, Some("MY_SECRET".to_string()));
        assert_eq!(secret.description, Some("Test secret".to_string()));
    }

    #[test]
    fn test_secret_get_plain() {
        let secret = ConfigSecret::plain("my-secret");
        assert_eq!(secret.get_plain().unwrap(), "my-secret");
    }

    #[test]
    fn test_secure_config() {
        let mut config = SecureConfig::new();

        config.add_secret("db_url", database_url_secret());
        config.add_secret("jwt_secret", jwt_secret().optional());

        // Should fail validation without env vars
        assert!(config.validate().is_err());

        // Set required env var
        std::env::set_var("DATABASE_URL", "postgresql://localhost/test");
        config.load_from_env().unwrap();
        assert!(config.validate().is_ok());

        // Clean up
        std::env::remove_var("DATABASE_URL");
    }

    #[test]
    fn test_utility_functions() {
        let db_secret = database_url_secret();
        assert_eq!(db_secret.env_var, Some("DATABASE_URL".to_string()));

        let api_secret = api_key_secret("github");
        assert_eq!(api_secret.env_var, Some("GITHUB_API_KEY".to_string()));
    }
}
