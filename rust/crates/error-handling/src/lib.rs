//! # CloudShuttle Error Handling
//!
//! Common error types and handling utilities used across CloudShuttle services.
//!
//! This crate provides standardized error handling patterns to ensure consistency
//! across all services and proper error propagation.
//!
//! ## Features
//!
//! - Standardized error types for common failure modes
//! - Service-specific error traits
//! - HTTP status code mapping
//! - Structured error responses
//!
//! ## Example
//!
//! ```rust
//! use cloudshuttle_error_handling::{CloudShuttleError, ServiceError};
//!
//! // Using the base error type
//! fn example() -> Result<(), CloudShuttleError> {
//!     Err(CloudShuttleError::Config("Invalid configuration".to_string()))
//! }
//!
//! // Implementing service-specific errors
//! #[derive(Debug, thiserror::Error)]
//! #[error("{message}")]
//! pub struct MyServiceError {
//!     message: String,
//! }
//!
//! impl ServiceError for MyServiceError {
//!     fn error_code(&self) -> &'static str { "MY_SERVICE_ERROR" }
//!     fn http_status(&self) -> http::StatusCode { http::StatusCode::INTERNAL_SERVER_ERROR }
//!     fn user_message(&self) -> String { self.message.clone() }
//! }
//! ```

pub mod error;
pub mod service_error;
pub mod api_error;
pub mod database_error;

// Re-export the main types
pub use error::CloudShuttleError;
pub use service_error::ServiceError;
pub use api_error::ApiError;
pub use database_error::DatabaseError;

/// Result type alias using CloudShuttleError
pub type Result<T> = std::result::Result<T, CloudShuttleError>;


