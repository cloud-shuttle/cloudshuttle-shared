//! Common authentication types

use serde::{Deserialize, Serialize};

/// Authentication result type
pub type AuthResult<T> = Result<T, AuthError>;

/// Authentication errors
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Token creation failed: {0}")]
    TokenCreation(String),

    #[error("Token validation failed: {0}")]
    TokenValidation(String),

    #[error("Invalid key: {0}")]
    InvalidKey(String),

    #[error("Unsupported algorithm: {0}")]
    UnsupportedAlgorithm(String),

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid token type: expected {expected}, got {actual}")]
    InvalidTokenType { expected: String, actual: String },

    #[error("Missing token")]
    MissingToken,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Insufficient permissions: required {required:?}, have {actual:?}")]
    InsufficientPermissions { required: Vec<String>, actual: Vec<String> },

    #[error("Account locked")]
    AccountLocked,

    #[error("Account disabled")]
    AccountDisabled,

    #[error("Too many failed attempts")]
    TooManyFailedAttempts,

    #[error("Session expired")]
    SessionExpired,

    #[error("Invalid refresh token")]
    InvalidRefreshToken,

    #[error("Refresh token expired")]
    RefreshTokenExpired,

    #[error("Token not found")]
    TokenNotFound,

    #[error("Database error: {0}")]
    Database(String),

    #[error("External service error: {0}")]
    ExternalService(String),
}

#[cfg(feature = "middleware")]
impl axum::response::IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        use axum::{http::StatusCode, Json};
        use serde_json::json;

        let (status, message) = match self {
            AuthError::TokenCreation(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Authentication service error"),
            AuthError::TokenValidation(_) => (StatusCode::UNAUTHORIZED, "Invalid token"),
            AuthError::InvalidKey(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Authentication service error"),
            AuthError::UnsupportedAlgorithm(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Authentication service error"),
            AuthError::TokenExpired => (StatusCode::UNAUTHORIZED, "Token expired"),
            AuthError::InvalidTokenType { .. } => (StatusCode::UNAUTHORIZED, "Invalid token type"),
            AuthError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid credentials"),
            AuthError::MissingToken => (StatusCode::UNAUTHORIZED, "Authentication required"),
            AuthError::InsufficientPermissions { .. } => (StatusCode::FORBIDDEN, "Insufficient permissions"),
            AuthError::AccountLocked => (StatusCode::LOCKED, "Account locked"),
            AuthError::AccountDisabled => (StatusCode::FORBIDDEN, "Account disabled"),
            AuthError::TooManyFailedAttempts => (StatusCode::TOO_MANY_REQUESTS, "Too many failed attempts"),
            AuthError::SessionExpired => (StatusCode::UNAUTHORIZED, "Session expired"),
            AuthError::InvalidRefreshToken => (StatusCode::UNAUTHORIZED, "Invalid refresh token"),
            AuthError::RefreshTokenExpired => (StatusCode::UNAUTHORIZED, "Refresh token expired"),
            AuthError::TokenNotFound => (StatusCode::UNAUTHORIZED, "Token not found"),
            AuthError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Authentication service error"),
            AuthError::ExternalService(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Authentication service error"),
        };

        let body = Json(json!({
            "error": message,
            "code": format!("AUTH_{}", status.as_u16())
        }));

        (status, body).into_response()
    }
}

/// User authentication data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCredentials {
    pub username: String,
    pub password: String,
}

impl UserCredentials {
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
        }
    }
}

/// Authentication tokens response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_expires_in: u64,
}

impl AuthTokens {
    pub fn new(access_token: String, refresh_token: String, expires_in: u64, refresh_expires_in: u64) -> Self {
        Self {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in,
            refresh_expires_in,
        }
    }

    pub fn bearer(access_token: String, refresh_token: String) -> Self {
        Self::new(access_token, refresh_token, 3600, 604800) // 1 hour, 7 days
    }
}

/// Refresh token request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

impl RefreshTokenRequest {
    pub fn new(refresh_token: impl Into<String>) -> Self {
        Self {
            refresh_token: refresh_token.into(),
        }
    }
}

/// User session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub user_id: String,
    pub tenant_id: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub session_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl UserSession {
    pub fn new(user_id: String, tenant_id: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            user_id,
            tenant_id,
            roles: Vec::new(),
            permissions: Vec::new(),
            session_id: uuid::Uuid::new_v4().to_string(),
            created_at: now,
            expires_at: now + chrono::Duration::hours(1),
            ip_address: None,
            user_agent: None,
        }
    }

    pub fn with_roles(mut self, roles: Vec<String>) -> Self {
        self.roles = roles;
        self
    }

    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.permissions = permissions;
        self
    }

    pub fn with_expiry(mut self, hours: i64) -> Self {
        self.expires_at = self.created_at + chrono::Duration::hours(hours);
        self
    }

    pub fn is_expired(&self) -> bool {
        chrono::Utc::now() > self.expires_at
    }

    pub fn extend(&mut self, hours: i64) {
        self.expires_at = chrono::Utc::now() + chrono::Duration::hours(hours);
    }
}

/// Authentication context for requests
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: String,
    pub tenant_id: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub session_id: Option<String>,
    pub ip_address: Option<String>,
}

impl AuthContext {
    pub fn from_claims(claims: &crate::Claims) -> Self {
        Self {
            user_id: claims.sub.clone(),
            tenant_id: claims.tenant_id.clone(),
            roles: claims.roles.clone(),
            permissions: claims.permissions.clone(),
            session_id: claims.jti.clone(),
            ip_address: None,
        }
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_string())
    }

    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(&permission.to_string())
    }

    pub fn is_admin(&self) -> bool {
        self.has_role("admin")
    }

    pub fn can_access_tenant(&self, tenant_id: &str) -> bool {
        self.tenant_id == tenant_id || self.is_admin()
    }
}

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
        }
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
}

/// Account lockout policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockoutPolicy {
    pub max_attempts: u32,
    pub lockout_duration_minutes: u32,
    pub reset_after_minutes: u32,
}

impl Default for LockoutPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            lockout_duration_minutes: 15,
            reset_after_minutes: 30,
        }
    }
}

/// Multi-factor authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaConfig {
    pub enabled: bool,
    pub required_for_roles: Vec<String>,
    pub allowed_methods: Vec<MfaMethod>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MfaMethod {
    TOTP,
    SMS,
    Email,
    HardwareToken,
}

impl Default for MfaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            required_for_roles: vec!["admin".to_string()],
            allowed_methods: vec![MfaMethod::TOTP],
        }
    }
}
