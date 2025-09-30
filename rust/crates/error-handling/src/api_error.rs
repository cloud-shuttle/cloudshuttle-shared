//! API-specific error handling

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// HTTP API error response
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub error: String,
    pub message: String,
    pub code: String,
    pub status_code: u16,
    pub details: Option<serde_json::Value>,
    pub request_id: Option<String>,
    pub timestamp: String,
}

impl ApiErrorResponse {
    pub fn new<S: Into<String>>(error: S, message: S, code: S, status_code: u16) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
            code: code.into(),
            status_code,
            details: None,
            request_id: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }
}

/// API error types
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Bad request: {message}")]
    BadRequest { message: String },

    #[error("Unauthorized: {message}")]
    Unauthorized { message: String },

    #[error("Forbidden: {message}")]
    Forbidden { message: String },

    #[error("Not found: {resource}")]
    NotFound { resource: String },

    #[error("Conflict: {message}")]
    Conflict { message: String },

    #[error("Rate limited: retry after {retry_after} seconds")]
    RateLimited { retry_after: u64 },

    #[error("Internal server error: {message}")]
    InternalServerError { message: String },

    #[error("Service unavailable: {message}")]
    ServiceUnavailable { message: String },

    #[error("Validation error: {field} - {message}")]
    ValidationError { field: String, message: String },
}

impl ApiError {
    pub fn bad_request<S: Into<String>>(message: S) -> Self {
        Self::BadRequest { message: message.into() }
    }

    pub fn unauthorized<S: Into<String>>(message: S) -> Self {
        Self::Unauthorized { message: message.into() }
    }

    pub fn forbidden<S: Into<String>>(message: S) -> Self {
        Self::Forbidden { message: message.into() }
    }

    pub fn not_found<S: Into<String>>(resource: S) -> Self {
        Self::NotFound { resource: resource.into() }
    }

    pub fn conflict<S: Into<String>>(message: S) -> Self {
        Self::Conflict { message: message.into() }
    }

    pub fn rate_limited(retry_after: u64) -> Self {
        Self::RateLimited { retry_after }
    }

    pub fn internal_server_error<S: Into<String>>(message: S) -> Self {
        Self::InternalServerError { message: message.into() }
    }

    pub fn service_unavailable<S: Into<String>>(message: S) -> Self {
        Self::ServiceUnavailable { message: message.into() }
    }

    pub fn validation_error<S: Into<String>>(field: S, message: S) -> Self {
        Self::ValidationError {
            field: field.into(),
            message: message.into(),
        }
    }

    pub fn http_status(&self) -> http::StatusCode {
        match self {
            Self::BadRequest { .. } => http::StatusCode::BAD_REQUEST,
            Self::Unauthorized { .. } => http::StatusCode::UNAUTHORIZED,
            Self::Forbidden { .. } => http::StatusCode::FORBIDDEN,
            Self::NotFound { .. } => http::StatusCode::NOT_FOUND,
            Self::Conflict { .. } => http::StatusCode::CONFLICT,
            Self::RateLimited { .. } => http::StatusCode::TOO_MANY_REQUESTS,
            Self::InternalServerError { .. } => http::StatusCode::INTERNAL_SERVER_ERROR,
            Self::ServiceUnavailable { .. } => http::StatusCode::SERVICE_UNAVAILABLE,
            Self::ValidationError { .. } => http::StatusCode::BAD_REQUEST,
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            Self::BadRequest { .. } => "BAD_REQUEST",
            Self::Unauthorized { .. } => "UNAUTHORIZED",
            Self::Forbidden { .. } => "FORBIDDEN",
            Self::NotFound { .. } => "NOT_FOUND",
            Self::Conflict { .. } => "CONFLICT",
            Self::RateLimited { .. } => "RATE_LIMITED",
            Self::InternalServerError { .. } => "INTERNAL_SERVER_ERROR",
            Self::ServiceUnavailable { .. } => "SERVICE_UNAVAILABLE",
            Self::ValidationError { .. } => "VALIDATION_ERROR",
        }
    }

    pub fn to_response(&self) -> ApiErrorResponse {
        let status_code = self.http_status().as_u16();
        let mut response = ApiErrorResponse::new(
            self.to_string(),
            self.user_message(),
            self.error_code(),
            status_code,
        );

        // Add specific details for certain error types
        match self {
            Self::ValidationError { field, message } => {
                let details = serde_json::json!({
                    "field": field,
                    "validation_message": message
                });
                response = response.with_details(details);
            }
            Self::RateLimited { retry_after } => {
                let details = serde_json::json!({
                    "retry_after_seconds": retry_after
                });
                response = response.with_details(details);
            }
            _ => {}
        }

        response
    }

    pub fn user_message(&self) -> String {
        match self {
            Self::BadRequest { message } => message.clone(),
            Self::Unauthorized { message } => message.clone(),
            Self::Forbidden { message } => message.clone(),
            Self::NotFound { resource } => format!("The requested {} was not found", resource),
            Self::Conflict { message } => message.clone(),
            Self::RateLimited { retry_after } => format!("Too many requests. Please try again in {} seconds", retry_after),
            Self::InternalServerError { .. } => "An internal server error occurred".to_string(),
            Self::ServiceUnavailable { message } => message.clone(),
            Self::ValidationError { field, message } => format!("Invalid {}: {}", field, message),
        }
    }
}

/// Validation error collection for forms/APIs
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationErrors {
    pub errors: HashMap<String, Vec<String>>,
}

impl ValidationErrors {
    pub fn new() -> Self {
        Self {
            errors: HashMap::new(),
        }
    }

    pub fn add_error<S: Into<String>>(&mut self, field: S, message: S) {
        let field = field.into();
        let message = message.into();
        self.errors.entry(field).or_insert_with(Vec::new).push(message);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn error_count(&self) -> usize {
        self.errors.values().map(|v| v.len()).sum()
    }

    pub fn to_api_error(&self) -> ApiError {
        ApiError::ValidationError {
            field: "form".to_string(),
            message: format!("{} validation errors", self.error_count()),
        }
    }
}

impl Default for ValidationErrors {
    fn default() -> Self {
        Self::new()
    }
}

/// Request context for error tracking
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub request_id: String,
    pub user_id: Option<String>,
    pub tenant_id: Option<String>,
    pub method: String,
    pub path: String,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
}

impl RequestContext {
    pub fn new(request_id: String, method: String, path: String) -> Self {
        Self {
            request_id,
            user_id: None,
            tenant_id: None,
            method,
            path,
            user_agent: None,
            ip_address: None,
        }
    }

    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_tenant_id(mut self, tenant_id: String) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }
}

/// Error with request context
pub struct ContextualError {
    pub error: ApiError,
    pub context: RequestContext,
}

impl ContextualError {
    pub fn new(error: ApiError, context: RequestContext) -> Self {
        Self { error, context }
    }

    pub fn log(&self) {
        tracing::error!(
            request_id = %self.context.request_id,
            user_id = ?self.context.user_id,
            tenant_id = ?self.context.tenant_id,
            method = %self.context.method,
            path = %self.context.path,
            error_code = %self.error.error_code(),
            "API error: {}",
            self.error
        );
    }
}


