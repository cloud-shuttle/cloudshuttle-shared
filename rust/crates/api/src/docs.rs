//! Structured API documentation using OpenAPI/Swagger
//!
//! This module provides comprehensive API documentation generation
//! with OpenAPI 3.0 specifications, including schema definitions,
//! endpoint documentation, and interactive Swagger UI.

use utoipa::{openapi::tag::TagBuilder, Modify, ToSchema};

// Re-export utoipa types for convenience
pub use utoipa::{ToSchema as UtoipaToSchema};

// OpenAPI schema definitions for our common types

/// Pagination metadata schema
#[derive(ToSchema)]
#[schema(example = json!({"page": 1, "per_page": 20, "total": 100, "total_pages": 5, "has_next": true, "has_prev": false}))]
pub struct PaginationMetaSchema {
    /// Current page number (1-based)
    pub page: u32,
    /// Items per page
    pub per_page: u32,
    /// Total number of items
    pub total: u64,
    /// Total number of pages
    pub total_pages: u32,
    /// Whether there is a next page
    pub has_next: bool,
    /// Whether there is a previous page
    pub has_prev: bool,
}

/// Generic API response wrapper schema
#[derive(ToSchema)]
#[schema(example = json!({"success": true, "message": null, "data": "example data", "errors": null, "request_id": "req-123", "timestamp": "2024-01-01T12:00:00Z"}))]
pub struct ApiResponseSchema<T> {
    /// Whether the request was successful
    pub success: bool,
    /// Optional success message
    pub message: Option<String>,
    /// Response data (when successful)
    pub data: Option<T>,
    /// Error details (when failed)
    pub errors: Option<Vec<String>>,
    /// Request ID for tracing
    pub request_id: Option<String>,
    /// Response timestamp
    pub timestamp: String,
}

/// Paginated response schema
#[derive(ToSchema)]
#[schema(example = json!({"data": [{"id": 1, "name": "item1"}, {"id": 2, "name": "item2"}], "pagination": {"page": 1, "per_page": 20, "total": 100, "total_pages": 5, "has_next": true, "has_prev": false}}))]
pub struct PaginatedResponseSchema<T> {
    /// Array of items
    pub data: Vec<T>,
    /// Pagination metadata
    pub pagination: PaginationMetaSchema,
}

/// Error response schema
#[derive(ToSchema)]
#[schema(example = json!({"code": "VALIDATION_ERROR", "message": "Invalid input", "status_code": 400, "request_id": "req-123"}))]
pub struct ApiErrorSchema {
    /// Error code for programmatic handling
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// HTTP status code
    pub status_code: u16,
    /// Request ID for tracing
    pub request_id: Option<String>,
}

/// Validation error details
#[derive(ToSchema)]
#[schema(example = json!({"field": "email", "message": "Invalid email format"}))]
pub struct ValidationErrorSchema {
    /// Field name that failed validation
    pub field: String,
    /// Validation error message
    pub message: String,
}

/// Common API responses
pub mod responses {
    use utoipa::ToResponse;

    /// Successful response (200)
    #[derive(ToResponse)]
    pub struct SuccessResponse<T> {
        #[response(status = 200, description = "Successful operation")]
        pub response: crate::ApiResponse<T>,
    }

    /// Created response (201)
    #[derive(ToResponse)]
    pub struct CreatedResponse<T> {
        #[response(status = 201, description = "Resource created successfully")]
        pub response: crate::ApiResponse<T>,
    }

    /// No content response (204)
    #[derive(ToResponse)]
    pub struct NoContentResponse;

    /// Bad request response (400)
    #[derive(ToResponse)]
    pub struct BadRequestResponse {
        #[response(status = 400, description = "Bad request - invalid input")]
        pub response: crate::ApiResponse<()>,
    }

    /// Unauthorized response (401)
    #[derive(ToResponse)]
    pub struct UnauthorizedResponse {
        #[response(status = 401, description = "Authentication required")]
        pub response: crate::ApiResponse<()>,
    }

    /// Forbidden response (403)
    #[derive(ToResponse)]
    pub struct ForbiddenResponse {
        #[response(status = 403, description = "Insufficient permissions")]
        pub response: crate::ApiResponse<()>,
    }

    /// Not found response (404)
    #[derive(ToResponse)]
    pub struct NotFoundResponse {
        #[response(status = 404, description = "Resource not found")]
        pub response: crate::ApiResponse<()>,
    }

    /// Conflict response (409)
    #[derive(ToResponse)]
    pub struct ConflictResponse {
        #[response(status = 409, description = "Resource conflict")]
        pub response: crate::ApiResponse<()>,
    }

    /// Too many requests response (429)
    #[derive(ToResponse)]
    pub struct TooManyRequestsResponse {
        #[response(status = 429, description = "Rate limit exceeded")]
        pub response: crate::ApiResponse<()>,
    }

    /// Internal server error response (500)
    #[derive(ToResponse)]
    pub struct InternalServerErrorResponse {
        #[response(status = 500, description = "Internal server error")]
        pub response: crate::ApiResponse<()>,
    }
}


/// OpenAPI documentation modifier for adding common schemas
#[derive(Clone)]
pub struct SchemaAddon;

impl Modify for SchemaAddon {
    fn modify(&self, _openapi: &mut utoipa::openapi::OpenApi) {
        // Schema modification not implemented yet - requires utoipa API changes
    }
}

/// Builder for creating OpenAPI documentation
pub struct ApiDocsBuilder {
    title: String,
    version: String,
}

impl ApiDocsBuilder {
    /// Create a new API documentation builder
    pub fn new(title: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            version: version.into(),
        }
    }

    /// Build the OpenAPI specification
    pub fn build(self) -> utoipa::openapi::OpenApi {
        use utoipa::openapi::*;

        OpenApiBuilder::new()
            .info(
                InfoBuilder::new()
                    .title(self.title)
                    .version(self.version)
                    .build()
            )
            .build()
    }
}

/// Convenience function to create CloudShuttle API documentation
pub fn cloudshuttle_api_docs() -> utoipa::openapi::OpenApi {
    use utoipa::openapi::*;

    OpenApiBuilder::new()
        .info(
            InfoBuilder::new()
                .title("CloudShuttle API")
                .version(env!("CARGO_PKG_VERSION"))
                .contact(Some(
                    ContactBuilder::new()
                        .name(Some("CloudShuttle Team"))
                        .email(Some("team@cloudshuttle.com"))
                        .build()
                ))
                .license(Some(
                    LicenseBuilder::new()
                        .name("MIT OR Apache-2.0")
                        .build()
                ))
                .build()
        )
        .servers(Some(vec![
            ServerBuilder::new()
                .url("https://api.cloudshuttle.com/v1")
                .description(Some("Production API"))
                .build(),
            ServerBuilder::new()
                .url("http://localhost:8080/v1")
                .description(Some("Development API"))
                .build(),
        ]))
        .tags(Some(vec![
            TagBuilder::new()
                .name("Health")
                .description(Some("Health check and monitoring endpoints"))
                .build(),
            TagBuilder::new()
                .name("Management")
                .description(Some("System management and administration"))
                .build(),
        ]))
        .build()
}

/// Axum router extension for serving OpenAPI documentation
pub mod axum_integration {
    use axum::Router;
    use utoipa::OpenApi;

    /// Extension trait for adding OpenAPI documentation to Axum routers
    pub trait OpenApiExt {
        /// Add OpenAPI documentation routes to the router
        fn with_openapi_docs<A: OpenApi>(self, api: A, path: &str) -> Self;
    }

    impl OpenApiExt for Router {
        fn with_openapi_docs<A: OpenApi>(self, _api: A, _path: &str) -> Self {
            // OpenAPI documentation integration not implemented yet
            // Requires proper utoipa_axum integration
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_docs_builder() {
        let docs = ApiDocsBuilder::new("Test API", "1.0.0").build();

        assert_eq!(docs.info.title, "Test API");
        assert_eq!(docs.info.version, "1.0.0");
    }

    #[test]
    fn test_cloudshuttle_api_docs() {
        let docs = cloudshuttle_api_docs();

        assert_eq!(docs.info.title, "CloudShuttle API");
        assert!(docs.info.contact.is_some());
        assert!(docs.info.license.is_some());
        assert!(docs.servers.is_some());
        assert!(docs.tags.is_some());
    }
}
