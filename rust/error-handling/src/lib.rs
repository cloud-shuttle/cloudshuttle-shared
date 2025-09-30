//! # CloudShuttle Error Handling
//!
//! Standardized error types and handling across all CloudShuttle Rust services.
//!
//! This crate provides:
//! - Base error types for common error scenarios
//! - Service-specific error traits
//! - HTTP API error mappings
//! - Database error handling
//! - Error handling macros
//!
//! ## Example
//!
//! ```rust
//! use cloudshuttle_shared::error::{CloudShuttleError, ServiceError};
//!
//! // Using base error types
//! fn process_data() -> Result<(), CloudShuttleError> {
//!     // Some processing logic
//!     Ok(())
//! }
//!
//! // Implementing service-specific errors
//! #[derive(Debug, thiserror::Error)]
//! #[error("Auth service error: {message}")]
//! pub struct AuthError {
//!     message: String,
//!     code: String,
//! }
//!
//! impl ServiceError for AuthError {
//!     fn error_code(&self) -> &'static str { &self.code }
//!     fn http_status(&self) -> http::StatusCode { http::StatusCode::UNAUTHORIZED }
//!     fn user_message(&self) -> String { self.message.clone() }
//! }
//! ```

pub mod error;
pub mod service_error;
pub mod api_error;
pub mod database_error;
pub mod macros;

// Re-export main types for convenience
pub use error::CloudShuttleError;
pub use service_error::ServiceError;
pub use api_error::ApiError;
pub use database_error::DatabaseError;
