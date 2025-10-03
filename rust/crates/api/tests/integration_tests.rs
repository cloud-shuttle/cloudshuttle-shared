//! Comprehensive integration tests for the CloudShuttle API utilities system.
//!
//! These tests cover end-to-end functionality of all API utilities including:
//! - Rate limiting middleware
//! - CORS middleware
//! - Request tracing middleware
//! - API documentation generation
//! - Response formatting
//! - Pagination
//! - Validation
//! - Service integration

use cloudshuttle_api::{
    cors::{CorsConfig, CorsMiddleware, presets as cors_presets},
    rate_limit::{InMemoryRateLimiter, RateLimitConfig, presets as rate_limit_presets, RateLimitResult},
    request_tracing::{TracingConfig, TracingMiddleware, presets as tracing_presets},
    ApiService, ApiResponse, PaginatedResponse, PaginationMeta,
    docs::{ApiDocsBuilder, cloudshuttle_api_docs},
};
use serde::{Deserialize, Serialize};

/// Test data structure for API responses
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TestUser {
    id: u32,
    name: String,
    email: String,
}


#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_api_service_integration() {
        let service = ApiService::new()
            .require_auth()
            .with_cors();

        assert!(service.requires_auth());
        assert!(service.has_cors());

        // Test configuration
        let config = service.config();
        assert!(config.require_auth);
        assert!(config.enable_cors);
    }

    #[test]
    fn test_api_documentation_generation() {
        // Test that API documentation can be generated
        let docs = cloudshuttle_api_docs();
        assert_eq!(docs.info.title, "CloudShuttle API");
        assert!(docs.info.version.starts_with("0."));

        // Test custom documentation builder
        let custom_docs = ApiDocsBuilder::new("Custom API", "2.0.0").build();

        assert_eq!(custom_docs.info.title, "Custom API");
        assert_eq!(custom_docs.info.version, "2.0.0");
    }

    #[test]
    fn test_rate_limit_presets() {
        // Test different rate limit presets
        let api_limiter = rate_limit_presets::api_limiter();
        assert_eq!(api_limiter.config().max_requests, 100);

        let auth_limiter = rate_limit_presets::auth_limiter();
        assert_eq!(auth_limiter.config().max_requests, 10);

        let search_limiter = rate_limit_presets::search_limiter();
        assert_eq!(search_limiter.config().max_requests, 50);

        let upload_limiter = rate_limit_presets::upload_limiter();
        assert_eq!(upload_limiter.config().max_requests, 5);
    }

    #[test]
    fn test_cors_presets() {
        // Test different CORS presets
        let permissive = cors_presets::permissive();
        assert!(permissive.allowed_origins.contains(&"*".to_string()));

        let restrictive = cors_presets::restrictive();
        assert!(restrictive.allowed_origins.contains(&"https://app.cloudshuttle.com".to_string()));
        assert!(restrictive.allow_credentials);

        let api_cors = cors_presets::api();
        assert!(api_cors.allowed_origins.len() > 1);
        assert!(api_cors.allow_credentials);
    }

    #[test]
    fn test_tracing_presets() {
        // Test different tracing presets
        let minimal = tracing_presets::minimal();
        assert!(minimal.include_request_id);
        assert!(!minimal.include_timing);

        let standard = tracing_presets::standard();
        assert!(standard.include_request_id);
        assert!(standard.include_timing);

        let verbose = tracing_presets::verbose();
        assert!(verbose.log_requests);

        let production = tracing_presets::production();
        assert!(production.include_request_id);
        assert!(production.include_timing);
        assert!(!production.log_requests); // Logging handled by observability stack
    }

    #[test]
    fn test_rate_limiting_logic() {
        let limiter = InMemoryRateLimiter::new(RateLimitConfig {
            max_requests: 3,
            window_seconds: 60,
            ..Default::default()
        });

        // First 3 requests should be allowed
        for i in 0..3 {
            match limiter.check_limit("test") {
                RateLimitResult::Allowed { remaining, .. } => {
                    assert_eq!(remaining, 2 - i as u32);
                }
                _ => panic!("Expected allowed"),
            }
        }

        // 4th request should be denied
        match limiter.check_limit("test") {
            RateLimitResult::Exceeded { .. } => {}
            _ => panic!("Expected exceeded"),
        }
    }

    #[test]
    fn test_cors_validation() {
        let middleware = CorsMiddleware::with_config(cors_presets::api());

        // Test valid request
        match middleware.validate_request(Some("https://app.cloudshuttle.com"), &axum::http::Method::GET, &[]) {
            cloudshuttle_api::cors::CorsResult::Allowed { .. } => {}
            _ => panic!("Expected allowed"),
        }

        // Test invalid origin
        match middleware.validate_request(Some("https://malicious.com"), &axum::http::Method::GET, &[]) {
            cloudshuttle_api::cors::CorsResult::InvalidOrigin => {}
            _ => panic!("Expected invalid origin"),
        }
    }

    #[test]
    fn test_response_formatting() {
        // Test success response
        let success: ApiResponse<String> = ApiResponse::success("test data".to_string());
        assert!(success.success);
        assert_eq!(success.data, Some("test data".to_string()));
        assert!(success.errors.is_none());

        // Test error response
        let error = ApiResponse::<()>::error("Something went wrong");
        assert!(!error.success);
        assert_eq!(error.message, Some("Something went wrong".to_string()));
        assert!(error.data.is_none());
    }

    #[test]
    fn test_paginated_response_creation() {
        let data = vec!["item1", "item2", "item3"];
        let pagination = PaginationMeta::new(1, 3, 10);
        let response = PaginatedResponse::new(data, pagination);

        assert_eq!(response.data.len(), 3);
        assert_eq!(response.pagination.page, 1);
        assert_eq!(response.pagination.per_page, 3);
        assert_eq!(response.pagination.total, 10);
        assert!(response.pagination.has_next);
    }
}

#[cfg(test)]
mod validation_integration_tests {
    use cloudshuttle_api::validation::{RequestValidator, rules};

    #[test]
    fn test_user_registration_validation() {
        // Test successful validation
        let result = rules::validate_user_registration(
            "testuser",
            "test@example.com",
            "password123"
        );
        assert!(result.is_ok());

        // Test validation failures
        let result = rules::validate_user_registration(
            "", // empty username
            "test@example.com",
            "password123"
        );
        assert!(result.is_err());

        let result = rules::validate_user_registration(
            "testuser",
            "invalid-email", // invalid email
            "password123"
        );
        assert!(result.is_err());

        let result = rules::validate_user_registration(
            "testuser",
            "test@example.com",
            "short" // password too short
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_pagination_validation() {
        // Test valid pagination
        let result = rules::validate_pagination(Some(1), Some(20));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), (1, 20));

        // Test invalid pagination
        let result = rules::validate_pagination(Some(0), Some(20)); // page must be >= 1
        assert!(result.is_err());

        let result = rules::validate_pagination(Some(1), Some(2000)); // per_page too large
        assert!(result.is_err());
    }

    #[test]
    fn test_request_validator() {
        let mut validator = RequestValidator::new();

        // Test successful validation
        validator.validate_required("name", Some("John"));
        validator.validate_length("name", "John", 1, 100);
        validator.validate_email("email", "john@example.com");

        assert!(validator.is_valid());

        // Test failed validation
        let mut validator = RequestValidator::new();
        validator.validate_required("name", None); // required field missing
        validator.validate_email("email", "invalid-email"); // invalid email

        assert!(!validator.is_valid());
        assert_eq!(validator.error_count(), 2);
    }
}

#[cfg(test)]
mod pagination_integration_tests {
    use cloudshuttle_api::{PaginationParams, PaginatedResponse, PaginationMeta};

    #[test]
    fn test_pagination_params() {
        let params = PaginationParams::new()
            .page(2)
            .per_page(10)
            .sort("name", "desc");

        assert_eq!(params.get_page(), 2);
        assert_eq!(params.get_per_page(), 10);
        assert_eq!(params.offset(), 10); // (page-1) * per_page
        assert_eq!(params.limit(), 10);

        assert_eq!(params.sort_by(), Some("name"));
        assert_eq!(params.sort_order(), "desc");
    }

    #[test]
    fn test_pagination_meta() {
        let meta = PaginationMeta::new(2, 10, 25);

        assert_eq!(meta.page, 2);
        assert_eq!(meta.per_page, 10);
        assert_eq!(meta.total, 25);
        assert_eq!(meta.total_pages, 3); // ceil(25/10) = 3
        assert!(meta.has_next); // page 2 < total_pages 3
        assert!(meta.has_prev); // page 2 > 1

        let range = meta.item_range();
        assert_eq!(range, (11, 20)); // items 11-20 on page 2
    }

    #[test]
    fn test_paginated_response() {
        let data = vec!["item1", "item2", "item3"];
        let meta = PaginationMeta::new(1, 3, 10);
        let response = PaginatedResponse::new(data, meta);

        assert_eq!(response.items().len(), 3);
        assert!(!response.is_empty());
        assert!(response.is_first_page());
        assert!(!response.is_last_page()); // 10 total items, so more pages exist
    }

    #[test]
    fn test_paginated_response_from_params() {
        let data = vec![1, 2, 3, 4, 5];
        let params = PaginationParams::new().page(2).per_page(2);
        let response = PaginatedResponse::from_params(data, &params, 10);

        assert_eq!(response.data.len(), 5); // Original data unchanged
        assert_eq!(response.pagination.page, 2);
        assert_eq!(response.pagination.per_page, 2);
        assert_eq!(response.pagination.total, 10);
    }
}
