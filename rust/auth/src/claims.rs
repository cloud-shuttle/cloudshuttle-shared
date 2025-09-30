//! JWT token claims structures

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Standard JWT claims for CloudShuttle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,

    /// Expiration time (Unix timestamp)
    pub exp: usize,

    /// Issued at time (Unix timestamp)
    pub iat: usize,

    /// Tenant identifier
    pub tenant_id: Uuid,

    /// User roles
    pub roles: Vec<String>,

    /// Additional custom claims
    #[serde(flatten)]
    pub custom: std::collections::HashMap<String, serde_json::Value>,
}

impl Claims {
    /// Create new claims with default values
    pub fn new(sub: impl Into<String>, tenant_id: Uuid) -> Self {
        let now = jsonwebtoken::get_current_timestamp() as usize;

        Self {
            sub: sub.into(),
            exp: now + 3600, // 1 hour from now
            iat: now,
            tenant_id,
            roles: Vec::new(),
            custom: std::collections::HashMap::new(),
        }
    }

    /// Set expiration time
    pub fn with_expiration(mut self, exp: usize) -> Self {
        self.exp = exp;
        self
    }

    /// Set issued at time
    pub fn with_issued_at(mut self, iat: usize) -> Self {
        self.iat = iat;
        self
    }

    /// Set roles
    pub fn with_roles(mut self, roles: Vec<String>) -> Self {
        self.roles = roles;
        self
    }

    /// Add a role
    pub fn add_role(mut self, role: impl Into<String>) -> Self {
        self.roles.push(role.into());
        self
    }

    /// Add a custom claim
    pub fn add_custom_claim(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.custom.insert(key.into(), value);
        self
    }

    /// Check if user has a specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_string())
    }

    /// Check if user has any of the specified roles
    pub fn has_any_role(&self, roles: &[&str]) -> bool {
        roles.iter().any(|role| self.has_role(role))
    }

    /// Check if user has all of the specified roles
    pub fn has_all_roles(&self, roles: &[&str]) -> bool {
        roles.iter().all(|role| self.has_role(role))
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        let now = jsonwebtoken::get_current_timestamp() as usize;
        self.exp < now
    }

    /// Get remaining time until expiration in seconds
    pub fn time_until_expiration(&self) -> i64 {
        let now = jsonwebtoken::get_current_timestamp() as usize;
        self.exp as i64 - now as i64
    }
}

/// Refresh token claims (longer-lived than access tokens)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshClaims {
    /// Subject (user ID)
    pub sub: String,

    /// Expiration time (Unix timestamp)
    pub exp: usize,

    /// Issued at time (Unix timestamp)
    pub iat: usize,

    /// Tenant identifier
    pub tenant_id: Uuid,

    /// Token version for invalidation
    pub version: i32,

    /// Device fingerprint
    pub device_id: Option<String>,
}

impl RefreshClaims {
    /// Create new refresh claims
    pub fn new(sub: impl Into<String>, tenant_id: Uuid, version: i32) -> Self {
        let now = jsonwebtoken::get_current_timestamp() as usize;

        Self {
            sub: sub.into(),
            exp: now + 604800, // 7 days from now
            iat: now,
            tenant_id,
            version,
            device_id: None,
        }
    }

    /// Set expiration time
    pub fn with_expiration(mut self, exp: usize) -> Self {
        self.exp = exp;
        self
    }

    /// Set device ID
    pub fn with_device_id(mut self, device_id: impl Into<String>) -> Self {
        self.device_id = Some(device_id.into());
        self
    }
}

/// Email verification token claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailVerificationClaims {
    /// Subject (user ID)
    pub sub: String,

    /// Email address being verified
    pub email: String,

    /// Expiration time (Unix timestamp)
    pub exp: usize,

    /// Issued at time (Unix timestamp)
    pub iat: usize,

    /// Tenant identifier
    pub tenant_id: Uuid,

    /// Verification token hash for security
    pub token_hash: String,
}

impl EmailVerificationClaims {
    /// Create new email verification claims
    pub fn new(sub: impl Into<String>, email: impl Into<String>, tenant_id: Uuid, token_hash: impl Into<String>) -> Self {
        let now = jsonwebtoken::get_current_timestamp() as usize;

        Self {
            sub: sub.into(),
            email: email.into(),
            exp: now + 86400, // 24 hours from now
            iat: now,
            tenant_id,
            token_hash: token_hash.into(),
        }
    }
}

/// Password reset token claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetClaims {
    /// Subject (user ID)
    pub sub: String,

    /// Email address for password reset
    pub email: String,

    /// Expiration time (Unix timestamp)
    pub exp: usize,

    /// Issued at time (Unix timestamp)
    pub iat: usize,

    /// Tenant identifier
    pub tenant_id: Uuid,

    /// Reset token hash for security
    pub token_hash: String,
}

impl PasswordResetClaims {
    /// Create new password reset claims
    pub fn new(sub: impl Into<String>, email: impl Into<String>, tenant_id: Uuid, token_hash: impl Into<String>) -> Self {
        let now = jsonwebtoken::get_current_timestamp() as usize;

        Self {
            sub: sub.into(),
            email: email.into(),
            exp: now + 3600, // 1 hour from now
            iat: now,
            tenant_id,
            token_hash: token_hash.into(),
        }
    }
}
