//! Core error types for CloudShuttle services

use serde::{Deserialize, Serialize};
use std::fmt;

/// Main error type for CloudShuttle services
#[derive(Debug, thiserror::Error)]
pub enum CloudShuttleError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("External service error: {0}")]
    ExternalService(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Timeout error: {0}")]
    Timeout(String),
}

impl CloudShuttleError {
    /// Create a configuration error
    pub fn config<S: Into<String>>(message: S) -> Self {
        Self::Config(message.into())
    }

    /// Create an authentication error
    pub fn auth<S: Into<String>>(message: S) -> Self {
        Self::Auth(message.into())
    }

    /// Create an authorization error
    pub fn authorization<S: Into<String>>(message: S) -> Self {
        Self::Authorization(message.into())
    }

    /// Create a validation error
    pub fn validation<S: Into<String>>(message: S) -> Self {
        Self::Validation(message.into())
    }

    /// Create an internal error
    pub fn internal<S: Into<String>>(message: S) -> Self {
        Self::Internal(message.into())
    }

    /// Get a user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            Self::Config(_) => "Configuration error occurred".to_string(),
            Self::Database(_) => "Database operation failed".to_string(),
            Self::Auth(_) => "Authentication failed".to_string(),
            Self::Authorization(_) => "Access denied".to_string(),
            Self::Validation(_) => "Invalid input provided".to_string(),
            Self::Network(_) => "Network communication failed".to_string(),
            Self::ServiceUnavailable(_) => "Service temporarily unavailable".to_string(),
            Self::RateLimit(_) => "Too many requests, please try again later".to_string(),
            Self::Internal(_) => "An internal error occurred".to_string(),
            Self::ExternalService(_) => "External service error".to_string(),
            Self::Serialization(_) => "Data processing error".to_string(),
            Self::Http(_) => "HTTP request failed".to_string(),
            Self::Io(_) => "File operation failed".to_string(),
            Self::Parse(_) => "Data parsing failed".to_string(),
            Self::Timeout(_) => "Operation timed out".to_string(),
        }
    }

    /// Get HTTP status code for this error
    pub fn http_status(&self) -> http::StatusCode {
        match self {
            Self::Config(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
            Self::Database(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
            Self::Auth(_) => http::StatusCode::UNAUTHORIZED,
            Self::Authorization(_) => http::StatusCode::FORBIDDEN,
            Self::Validation(_) => http::StatusCode::BAD_REQUEST,
            Self::Network(_) => http::StatusCode::BAD_GATEWAY,
            Self::ServiceUnavailable(_) => http::StatusCode::SERVICE_UNAVAILABLE,
            Self::RateLimit(_) => http::StatusCode::TOO_MANY_REQUESTS,
            Self::Internal(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
            Self::ExternalService(_) => http::StatusCode::BAD_GATEWAY,
            Self::Serialization(_) => http::StatusCode::BAD_REQUEST,
            Self::Http(_) => http::StatusCode::BAD_GATEWAY,
            Self::Io(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
            Self::Parse(_) => http::StatusCode::BAD_REQUEST,
            Self::Timeout(_) => http::StatusCode::GATEWAY_TIMEOUT,
        }
    }

    /// Check if this error should be logged as an error (not warning/info)
    pub fn is_error_level(&self) -> bool {
        matches!(
            self,
            Self::Database(_)
                | Self::Network(_)
                | Self::ServiceUnavailable(_)
                | Self::Internal(_)
                | Self::ExternalService(_)
                | Self::Io(_)
        )
    }
}

/// Structured error response for APIs
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub code: Option<String>,
    pub details: Option<serde_json::Value>,
    pub request_id: Option<String>,
}

impl From<&CloudShuttleError> for ErrorResponse {
    fn from(error: &CloudShuttleError) -> Self {
        Self {
            error: error.to_string(),
            message: error.user_message(),
            code: Some(format!("{:?}", error).split('(').next().unwrap_or("UNKNOWN").to_uppercase()),
            details: None,
            request_id: None,
        }
    }
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.error, self.message)
    }
}


