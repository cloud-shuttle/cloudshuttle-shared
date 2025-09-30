//! API error types and responses
//!
//! This module provides standardized error handling for API responses,
//! including HTTP status codes and user-friendly error messages.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// API error type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    /// Error code for programmatic handling
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// HTTP status code
    pub status_code: u16,
    /// Additional error details
    pub details: Option<HashMap<String, serde_json::Value>>,
    /// Request ID for tracing
    pub request_id: Option<String>,
}

impl ApiError {
    /// Create a new API error
    pub fn new(code: impl Into<String>, message: impl Into<String>, status_code: u16) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            status_code,
            details: None,
            request_id: None,
        }
    }

    /// Create a bad request error (400)
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new("BAD_REQUEST", message, 400)
    }

    /// Create an unauthorized error (401)
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new("UNAUTHORIZED", message, 401)
    }

    /// Create a forbidden error (403)
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new("FORBIDDEN", message, 403)
    }

    /// Create a not found error (404)
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new("NOT_FOUND", message, 404)
    }

    /// Create a conflict error (409)
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new("CONFLICT", message, 409)
    }

    /// Create an internal server error (500)
    pub fn internal_server_error(message: impl Into<String>) -> Self {
        Self::new("INTERNAL_SERVER_ERROR", message, 500)
    }

    /// Create a service unavailable error (503)
    pub fn service_unavailable(message: impl Into<String>) -> Self {
        Self::new("SERVICE_UNAVAILABLE", message, 503)
    }

    /// Add details to the error
    pub fn with_details(mut self, details: HashMap<String, serde_json::Value>) -> Self {
        self.details = Some(details);
        self
    }

    /// Add a single detail
    pub fn with_detail(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        let details = self.details.get_or_insert_with(HashMap::new);
        if let Ok(json_value) = serde_json::to_value(value) {
            details.insert(key.into(), json_value);
        }
        self
    }

    /// Set request ID for tracing
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    /// Get the HTTP status code
    pub fn status_code(&self) -> u16 {
        self.status_code
    }

    /// Check if this is a client error (4xx)
    pub fn is_client_error(&self) -> bool {
        self.status_code >= 400 && self.status_code < 500
    }

    /// Check if this is a server error (5xx)
    pub fn is_server_error(&self) -> bool {
        self.status_code >= 500
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for ApiError {}

/// Common API error codes
pub mod codes {
    pub const VALIDATION_ERROR: &str = "VALIDATION_ERROR";
    pub const AUTHENTICATION_ERROR: &str = "AUTHENTICATION_ERROR";
    pub const AUTHORIZATION_ERROR: &str = "AUTHORIZATION_ERROR";
    pub const RESOURCE_NOT_FOUND: &str = "RESOURCE_NOT_FOUND";
    pub const RESOURCE_CONFLICT: &str = "RESOURCE_CONFLICT";
    pub const RATE_LIMIT_EXCEEDED: &str = "RATE_LIMIT_EXCEEDED";
    pub const INTERNAL_ERROR: &str = "INTERNAL_ERROR";
    pub const EXTERNAL_SERVICE_ERROR: &str = "EXTERNAL_SERVICE_ERROR";
    pub const MAINTENANCE_MODE: &str = "MAINTENANCE_MODE";
}

/// Error response builder for creating consistent error responses
pub struct ErrorResponseBuilder {
    error: ApiError,
}

impl ErrorResponseBuilder {
    /// Start building an error response
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: ApiError::new(code, message, 500),
        }
    }

    /// Set the HTTP status code
    pub fn status(mut self, status_code: u16) -> Self {
        self.error.status_code = status_code;
        self
    }

    /// Add error details
    pub fn details(mut self, details: HashMap<String, serde_json::Value>) -> Self {
        self.error = self.error.with_details(details);
        self
    }

    /// Add a single detail
    pub fn detail(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        self.error = self.error.with_detail(key, value);
        self
    }

    /// Set request ID
    pub fn request_id(mut self, request_id: impl Into<String>) -> Self {
        self.error = self.error.with_request_id(request_id);
        self
    }

    /// Build the error
    pub fn build(self) -> ApiError {
        self.error
    }

    /// Build and convert to response (for Axum)
    #[cfg(feature = "axum")]
    pub fn into_response(self) -> axum::response::Response {
        let error = self.build();
        let status = axum::http::StatusCode::from_u16(error.status_code)
            .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR);

        axum::response::Json(error).into_response()
    }
}

/// Convenience functions for common errors
pub mod errors {
    use super::*;

    pub fn validation_error(message: impl Into<String>) -> ApiError {
        ApiError::bad_request(message)
    }

    pub fn authentication_error(message: impl Into<String>) -> ApiError {
        ApiError::unauthorized(message)
    }

    pub fn authorization_error(message: impl Into<String>) -> ApiError {
        ApiError::forbidden(message)
    }

    pub fn not_found(resource: impl Into<String>) -> ApiError {
        ApiError::not_found(format!("{} not found", resource.into()))
    }

    pub fn conflict(message: impl Into<String>) -> ApiError {
        ApiError::conflict(message)
    }

    pub fn rate_limited() -> ApiError {
        ApiError::new(codes::RATE_LIMIT_EXCEEDED, "Rate limit exceeded", 429)
    }

    pub fn internal_error() -> ApiError {
        ApiError::internal_server_error("Internal server error")
    }

    pub fn service_unavailable() -> ApiError {
        ApiError::service_unavailable("Service temporarily unavailable")
    }

    pub fn maintenance_mode() -> ApiError {
        ApiError::new(codes::MAINTENANCE_MODE, "Service is under maintenance", 503)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_creation() {
        let error = ApiError::bad_request("Invalid input");
        assert_eq!(error.code, "BAD_REQUEST");
        assert_eq!(error.message, "Invalid input");
        assert_eq!(error.status_code, 400);
        assert!(error.is_client_error());
        assert!(!error.is_server_error());
    }

    #[test]
    fn test_error_with_details() {
        let error = ApiError::validation_error("Field required")
            .with_detail("field", "name")
            .with_request_id("req-123");

        assert_eq!(error.code, "BAD_REQUEST");
        assert_eq!(error.status_code, 400);
        assert_eq!(error.request_id, Some("req-123".to_string()));
        assert!(error.details.is_some());
    }

    #[test]
    fn test_error_response_builder() {
        let error = ErrorResponseBuilder::new("TEST_ERROR", "Test message")
            .status(422)
            .detail("field", "email")
            .build();

        assert_eq!(error.code, "TEST_ERROR");
        assert_eq!(error.status_code, 422);
        assert!(error.details.is_some());
    }

    #[test]
    fn test_convenience_errors() {
        let not_found = errors::not_found("user");
        assert_eq!(not_found.status_code, 404);
        assert_eq!(not_found.message, "user not found");

        let rate_limited = errors::rate_limited();
        assert_eq!(rate_limited.status_code, 429);
        assert_eq!(rate_limited.code, codes::RATE_LIMIT_EXCEEDED);
    }
}
