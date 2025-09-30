//! Base error types for CloudShuttle services

use std::fmt;

/// Base error type for all CloudShuttle services
#[derive(Debug, thiserror::Error)]
pub enum CloudShuttleError {
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Validation error: {0}")]
    Validation(#[from] validator::ValidationErrors),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("API error: {0}")]
    Api(#[from] ApiError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("Cryptography error: {0}")]
    Crypto(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("External service error: {0}")]
    External(String),
}

impl CloudShuttleError {
    /// Create a new configuration error
    pub fn config<S: Into<String>>(message: S) -> Self {
        Self::Config(message.into())
    }

    /// Create a new authentication error
    pub fn auth<S: Into<String>>(message: S) -> Self {
        Self::Auth(message.into())
    }

    /// Create a new authorization error
    pub fn authorization<S: Into<String>>(message: S) -> Self {
        Self::Authorization(message.into())
    }

    /// Create a new internal error
    pub fn internal<S: Into<String>>(message: S) -> Self {
        Self::Internal(message.into())
    }

    /// Create a new external service error
    pub fn external<S: Into<String>>(message: S) -> Self {
        Self::External(message.into())
    }

    /// Create a new HTTP error
    pub fn http<S: Into<String>>(message: S) -> Self {
        Self::Http(message.into())
    }

    /// Create a new cryptography error
    pub fn crypto<S: Into<String>>(message: S) -> Self {
        Self::Crypto(message.into())
    }
}

/// Result type alias for CloudShuttle operations
pub type Result<T> = std::result::Result<T, CloudShuttleError>;
