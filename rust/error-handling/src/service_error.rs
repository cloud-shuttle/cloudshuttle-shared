//! Service-specific error traits and utilities

use std::error::Error;

/// Trait for service-specific errors that need to be converted to HTTP responses
pub trait ServiceError: Error + Send + Sync {
    /// Get the error code for this error
    fn error_code(&self) -> &'static str;

    /// Get the HTTP status code for this error
    fn http_status(&self) -> http::StatusCode;

    /// Get a user-friendly message for this error
    fn user_message(&self) -> String;

    /// Get additional details about this error (for internal logging)
    fn details(&self) -> Option<serde_json::Value> {
        None
    }
}

/// Standard service error implementation
#[derive(Debug, Clone)]
pub struct StandardServiceError {
    pub code: &'static str,
    pub status: http::StatusCode,
    pub user_message: String,
    pub internal_message: Option<String>,
    pub details: Option<serde_json::Value>,
}

impl StandardServiceError {
    /// Create a new standard service error
    pub fn new(
        code: &'static str,
        status: http::StatusCode,
        user_message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            status,
            user_message: user_message.into(),
            internal_message: None,
            details: None,
        }
    }

    /// Set the internal message
    pub fn with_internal_message(mut self, message: impl Into<String>) -> Self {
        self.internal_message = Some(message.into());
        self
    }

    /// Set additional details
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

impl Error for StandardServiceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl ServiceError for StandardServiceError {
    fn error_code(&self) -> &'static str {
        self.code
    }

    fn http_status(&self) -> http::StatusCode {
        self.status
    }

    fn user_message(&self) -> String {
        self.user_message.clone()
    }

    fn details(&self) -> Option<serde_json::Value> {
        self.details.clone()
    }
}

impl std::fmt::Display for StandardServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.user_message)
    }
}
