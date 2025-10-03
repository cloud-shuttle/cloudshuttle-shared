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
//! use cloudshuttle_api::{ApiResponse, PaginatedResponse, PaginationMeta};
//!
//! // Create a successful response
//! let response: ApiResponse<Vec<String>> = ApiResponse::success(vec!["item1".to_string()]);
//!
//! // Create a paginated response
//! let data = vec!["item1".to_string(), "item2".to_string()];
//! let pagination = PaginationMeta::new(1, 20, 100);
//! let paginated_response: ApiResponse<PaginatedResponse<String>> =
//!     ApiResponse::success(PaginatedResponse::new(data, pagination));
//! ```

pub mod response;
pub mod pagination;
pub mod error;
pub mod validation;
pub mod service;
pub mod rate_limit;
pub mod cors;
pub mod docs;
pub mod request_tracing;

// Re-export main types
pub use response::{ApiResponse, ApiResult};
pub use pagination::{PaginationParams, PaginatedResponse, PaginationMeta};
pub use error::ApiError;
pub use validation::RequestValidator;
pub use service::{ApiService, ApiServiceConfig};
pub use rate_limit::{InMemoryRateLimiter, RateLimitConfig, RateLimitResult, RateLimitMiddleware};
pub use cors::{CorsConfig, CorsMiddleware, CorsResult};
pub use docs::{
    ApiDocsBuilder,
    cloudshuttle_api_docs,
    axum_integration::OpenApiExt,
    responses,
    SchemaAddon,
    UtoipaToSchema,
};
pub use request_tracing::{TracingConfig, TracingContext, TracingMiddleware, RequestTracing};
