//! HTTP API error types and handling

use crate::ServiceError;
use http::StatusCode;

/// API-specific errors
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

    #[error("Unprocessable entity: {message}")]
    UnprocessableEntity { message: String },

    #[error("Too many requests")]
    TooManyRequests,

    #[error("Internal server error: {message}")]
    InternalServerError { message: String },

    #[error("Service unavailable: {message}")]
    ServiceUnavailable { message: String },

    #[error("Request timeout")]
    RequestTimeout,
}

impl ApiError {
    /// Create a bad request error
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest {
            message: message.into(),
        }
    }

    /// Create an unauthorized error
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::Unauthorized {
            message: message.into(),
        }
    }

    /// Create a forbidden error
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::Forbidden {
            message: message.into(),
        }
    }

    /// Create a not found error
    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::NotFound {
            resource: resource.into(),
        }
    }

    /// Create a conflict error
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict {
            message: message.into(),
        }
    }

    /// Create an unprocessable entity error
    pub fn unprocessable_entity(message: impl Into<String>) -> Self {
        Self::UnprocessableEntity {
            message: message.into(),
        }
    }

    /// Create an internal server error
    pub fn internal_server_error(message: impl Into<String>) -> Self {
        Self::InternalServerError {
            message: message.into(),
        }
    }

    /// Create a service unavailable error
    pub fn service_unavailable(message: impl Into<String>) -> Self {
        Self::ServiceUnavailable {
            message: message.into(),
        }
    }
}

impl ServiceError for ApiError {
    fn error_code(&self) -> &'static str {
        match self {
            Self::BadRequest { .. } => "BAD_REQUEST",
            Self::Unauthorized { .. } => "UNAUTHORIZED",
            Self::Forbidden { .. } => "FORBIDDEN",
            Self::NotFound { .. } => "NOT_FOUND",
            Self::Conflict { .. } => "CONFLICT",
            Self::UnprocessableEntity { .. } => "UNPROCESSABLE_ENTITY",
            Self::TooManyRequests => "TOO_MANY_REQUESTS",
            Self::InternalServerError { .. } => "INTERNAL_SERVER_ERROR",
            Self::ServiceUnavailable { .. } => "SERVICE_UNAVAILABLE",
            Self::RequestTimeout => "REQUEST_TIMEOUT",
        }
    }

    fn http_status(&self) -> StatusCode {
        match self {
            Self::BadRequest { .. } => StatusCode::BAD_REQUEST,
            Self::Unauthorized { .. } => StatusCode::UNAUTHORIZED,
            Self::Forbidden { .. } => StatusCode::FORBIDDEN,
            Self::NotFound { .. } => StatusCode::NOT_FOUND,
            Self::Conflict { .. } => StatusCode::CONFLICT,
            Self::UnprocessableEntity { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            Self::TooManyRequests => StatusCode::TOO_MANY_REQUESTS,
            Self::InternalServerError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ServiceUnavailable { .. } => StatusCode::SERVICE_UNAVAILABLE,
            Self::RequestTimeout => StatusCode::REQUEST_TIMEOUT,
        }
    }

    fn user_message(&self) -> String {
        match self {
            Self::BadRequest { message } => message.clone(),
            Self::Unauthorized { message } => message.clone(),
            Self::Forbidden { message } => message.clone(),
            Self::NotFound { resource } => format!("{} not found", resource),
            Self::Conflict { message } => message.clone(),
            Self::UnprocessableEntity { message } => message.clone(),
            Self::TooManyRequests => "Too many requests. Please try again later.".to_string(),
            Self::InternalServerError { .. } => "An internal error occurred. Please try again.".to_string(),
            Self::ServiceUnavailable { .. } => "Service is temporarily unavailable. Please try again later.".to_string(),
            Self::RequestTimeout => "Request timed out. Please try again.".to_string(),
        }
    }
}
