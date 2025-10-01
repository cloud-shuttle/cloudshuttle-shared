//! Authentication error types and definitions

use serde::{Deserialize, Serialize};

/// Authentication result type
pub type AuthResult<T> = Result<T, AuthError>;

/// Authentication errors
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
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

    #[error("Invalid token: {0}")]
    InvalidToken(String),

    #[error("Token revoked")]
    TokenRevoked,

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Internal error: {0}")]
    InternalError(String),

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

    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("User already exists: {0}")]
    UserAlreadyExists(String),

    #[error("Password too weak")]
    PasswordTooWeak,

    #[error("Password reset required")]
    PasswordResetRequired,

    #[error("MFA required")]
    MfaRequired,

    #[error("Invalid MFA code")]
    InvalidMfaCode,

    #[error("MFA setup required")]
    MfaSetupRequired,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Service unavailable")]
    ServiceUnavailable,

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("External service error: {0}")]
    ExternalServiceError(String),
}

impl AuthError {
    /// Convert error to HTTP status code
    pub fn to_http_status(&self) -> http::StatusCode {
        match self {
            AuthError::TokenExpired | AuthError::RefreshTokenExpired | AuthError::SessionExpired
            | AuthError::TokenRevoked => {
                http::StatusCode::UNAUTHORIZED
            }
            AuthError::InvalidCredentials
            | AuthError::InvalidRefreshToken
            | AuthError::InvalidMfaCode
            | AuthError::InvalidToken(_)
            | AuthError::InvalidRequest(_)
            | AuthError::PasswordTooWeak => {
                http::StatusCode::BAD_REQUEST
            }
            AuthError::InsufficientPermissions { .. } => http::StatusCode::FORBIDDEN,
            AuthError::UserNotFound(_) => http::StatusCode::NOT_FOUND,
            AuthError::UserAlreadyExists(_) => http::StatusCode::CONFLICT,
            AuthError::AccountLocked | AuthError::AccountDisabled => http::StatusCode::FORBIDDEN,
            AuthError::MfaRequired | AuthError::MfaSetupRequired => http::StatusCode::UNAUTHORIZED,
            AuthError::TooManyFailedAttempts | AuthError::RateLimitExceeded => {
                http::StatusCode::TOO_MANY_REQUESTS
            }
            AuthError::ServiceUnavailable => http::StatusCode::SERVICE_UNAVAILABLE,
            _ => http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Get error code for API responses
    pub fn error_code(&self) -> &'static str {
        match self {
            AuthError::TokenExpired => "TOKEN_EXPIRED",
            AuthError::InvalidCredentials => "INVALID_CREDENTIALS",
            AuthError::InvalidToken(_) => "INVALID_TOKEN",
            AuthError::TokenRevoked => "TOKEN_REVOKED",
            AuthError::InvalidRequest(_) => "INVALID_REQUEST",
            AuthError::InternalError(_) => "INTERNAL_ERROR",
            AuthError::InvalidRefreshToken => "INVALID_REFRESH_TOKEN",
            AuthError::RefreshTokenExpired => "REFRESH_TOKEN_EXPIRED",
            AuthError::SessionExpired => "SESSION_EXPIRED",
            AuthError::InsufficientPermissions { .. } => "INSUFFICIENT_PERMISSIONS",
            AuthError::UserNotFound(_) => "USER_NOT_FOUND",
            AuthError::UserAlreadyExists(_) => "USER_ALREADY_EXISTS",
            AuthError::AccountLocked => "ACCOUNT_LOCKED",
            AuthError::AccountDisabled => "ACCOUNT_DISABLED",
            AuthError::PasswordTooWeak => "PASSWORD_TOO_WEAK",
            AuthError::MfaRequired => "MFA_REQUIRED",
            AuthError::InvalidMfaCode => "INVALID_MFA_CODE",
            AuthError::MfaSetupRequired => "MFA_SETUP_REQUIRED",
            AuthError::TooManyFailedAttempts => "TOO_MANY_FAILED_ATTEMPTS",
            AuthError::RateLimitExceeded => "RATE_LIMIT_EXCEEDED",
            AuthError::ServiceUnavailable => "SERVICE_UNAVAILABLE",
            AuthError::DatabaseError(_) => "DATABASE_ERROR",
            AuthError::ExternalServiceError(_) => "EXTERNAL_SERVICE_ERROR",
            _ => "AUTHENTICATION_ERROR",
        }
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            AuthError::ServiceUnavailable
                | AuthError::DatabaseError(_)
                | AuthError::ExternalServiceError(_)
                | AuthError::InternalError(_)
                | AuthError::RateLimitExceeded
        )
    }

    /// Check if error indicates authentication failure
    pub fn is_auth_failure(&self) -> bool {
        matches!(
            self,
            AuthError::InvalidCredentials
                | AuthError::InvalidRefreshToken
                | AuthError::TokenExpired
                | AuthError::RefreshTokenExpired
                | AuthError::SessionExpired
                | AuthError::InvalidMfaCode
        )
    }

    /// Check if error indicates authorization failure
    pub fn is_authz_failure(&self) -> bool {
        matches!(self, AuthError::InsufficientPermissions { .. })
    }
}

#[cfg(feature = "axum")]
impl axum::response::IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        let status = self.to_http_status();
        let body = serde_json::json!({
            "error": {
                "code": self.error_code(),
                "message": self.to_string(),
                "retryable": self.is_retryable()
            }
        });

        (status, axum::Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_error_status_codes() {
        assert_eq!(AuthError::InvalidCredentials.to_http_status(), http::StatusCode::BAD_REQUEST);
        assert_eq!(AuthError::TokenExpired.to_http_status(), http::StatusCode::UNAUTHORIZED);
        assert_eq!(AuthError::InsufficientPermissions { required: vec![], actual: vec![] }.to_http_status(), http::StatusCode::FORBIDDEN);
        assert_eq!(AuthError::ServiceUnavailable.to_http_status(), http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn test_auth_error_codes() {
        assert_eq!(AuthError::InvalidCredentials.error_code(), "INVALID_CREDENTIALS");
        assert_eq!(AuthError::TokenExpired.error_code(), "TOKEN_EXPIRED");
        assert_eq!(AuthError::ServiceUnavailable.error_code(), "SERVICE_UNAVAILABLE");
    }

    #[test]
    fn test_auth_error_classification() {
        assert!(AuthError::InvalidCredentials.is_auth_failure());
        assert!(AuthError::TokenExpired.is_auth_failure());
        assert!(AuthError::InsufficientPermissions { required: vec![], actual: vec![] }.is_authz_failure());
        assert!(AuthError::ServiceUnavailable.is_retryable());
        assert!(!AuthError::InvalidCredentials.is_retryable());
    }

    #[test]
    fn test_auth_error_serialization() {
        let error = AuthError::InvalidCredentials;
        let json = serde_json::to_string(&error).unwrap();
        let deserialized: AuthError = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, AuthError::InvalidCredentials));
    }
}
