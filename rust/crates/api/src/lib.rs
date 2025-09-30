//! # CloudShuttle API Utilities
//!
//! Common API utilities and response formatting for CloudShuttle services.
//!
//! ## Features
//!
//! - Standardized API responses
//! - Pagination support
//! - Error response formatting
//! - Request validation
//! - API versioning
//!
//! ## Example
//!
//! ```rust
//! use cloudshuttle_api::{ApiResponse, PaginationMeta};
//! use axum::{Json, http::StatusCode};
//!
//! // Create a successful response
//! let response: ApiResponse<Vec<String>> = ApiResponse::success(vec!["item1".to_string()]);
//!
//! // Create a paginated response
//! let paginated = ApiResponse::paginated(
//!     data,
//!     PaginationMeta { page: 1, per_page: 20, total: 100, ..Default::default() }
//! );
//! ```

pub mod response;
pub mod pagination;
pub mod error;
pub mod validation;

// Re-export main types
pub use response::{ApiResponse, ApiResult};
pub use pagination::{PaginationParams, PaginatedResponse, PaginationMeta};
pub use error::ApiError;
pub use validation::RequestValidator;
