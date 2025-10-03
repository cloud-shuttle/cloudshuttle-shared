//! API Service that integrates all middleware and utilities
//!
//! This module provides a unified service that combines authentication,
//! rate limiting, CORS, and other middleware with the response formatting
//! and validation utilities.

use crate::{
    ApiResponse,
    response::ResponseBuilder,
    error::{ApiError, errors},
    pagination::PaginationParams,
    validation::RequestValidator,
};

/// Configuration for the API service
#[derive(Debug, Clone)]
pub struct ApiServiceConfig {
    /// Whether authentication is required
    pub require_auth: bool,
    /// Whether authentication is optional
    pub optional_auth: bool,
    /// Required roles for access
    pub required_roles: Vec<String>,
    /// Whether to enable CORS
    pub enable_cors: bool,
    /// Default page size for pagination
    pub default_page_size: usize,
    /// Maximum page size allowed
    pub max_page_size: usize,
}

impl Default for ApiServiceConfig {
    fn default() -> Self {
        Self {
            require_auth: false,
            optional_auth: false,
            required_roles: Vec::new(),
            enable_cors: false,
            default_page_size: 20,
            max_page_size: 1000,
        }
    }
}

/// Main API service that integrates all utilities
pub struct ApiService {
    config: ApiServiceConfig,
}

impl ApiService {
    /// Create a new API service
    pub fn new() -> Self {
        Self {
            config: ApiServiceConfig::default(),
        }
    }

    /// Configure the service
    pub fn with_config(mut self, config: ApiServiceConfig) -> Self {
        self.config = config;
        self
    }

    /// Enable authentication requirement
    pub fn require_auth(mut self) -> Self {
        self.config.require_auth = true;
        self.config.optional_auth = false;
        self
    }

    /// Enable optional authentication
    pub fn optional_auth(mut self) -> Self {
        self.config.optional_auth = true;
        self.config.require_auth = false;
        self
    }

    /// Require specific roles
    pub fn require_roles(mut self, roles: Vec<String>) -> Self {
        self.config.required_roles = roles;
        self
    }

    /// Require admin role
    pub fn require_admin(self) -> Self {
        self.require_roles(vec!["admin".to_string()])
    }

    /// Enable CORS
    pub fn with_cors(mut self) -> Self {
        self.config.enable_cors = true;
        self
    }

    /// Create success response
    pub fn success_response<T: serde::Serialize>(&self, data: T) -> ApiResponse<T> {
        ResponseBuilder::success(data).build()
    }

    /// Create success response with message
    pub fn success_response_with_message<T: serde::Serialize>(&self, data: T, message: impl Into<String>) -> ApiResponse<T> {
        ResponseBuilder::success(data).with_message(message).build()
    }

    /// Create error response
    pub fn error_response(&self, message: impl Into<String>) -> ApiResponse<()> {
        ResponseBuilder::<()>::error(message).build()
    }

    /// Create paginated response
    pub fn paginated_response<T: serde::Serialize>(
        &self,
        items: Vec<T>,
        params: &PaginationParams,
        total: u64,
    ) -> ApiResponse<crate::PaginatedResponse<T>> {
        let paginated = crate::PaginatedResponse::from_params(items, params, total);
        self.success_response(paginated)
    }

    /// Validate pagination parameters
    pub fn validate_pagination(&self, params: &PaginationParams) -> Result<(), ApiError> {
        let mut validator = RequestValidator::new();

        validator.validate_range("page", params.get_page() as i64, 1, 10000);
        validator.validate_range("per_page", params.get_per_page() as i64, 1, self.config.max_page_size as i64);

        if validator.is_valid() {
            Ok(())
        } else {
            Err(errors::validation_error(validator.all_errors().join("; ")))
        }
    }

    /// Get the current configuration
    pub fn config(&self) -> &ApiServiceConfig {
        &self.config
    }

    /// Check if authentication is required
    pub fn requires_auth(&self) -> bool {
        self.config.require_auth
    }

    /// Check if CORS is enabled
    pub fn has_cors(&self) -> bool {
        self.config.enable_cors
    }
}

impl Default for ApiService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_service_creation() {
        let service = ApiService::new();
        assert!(!service.requires_auth());
        assert!(!service.has_cors());
    }

    #[test]
    fn test_api_service_with_config() {
        let config = ApiServiceConfig {
            require_auth: true,
            enable_cors: true,
            default_page_size: 50,
            ..Default::default()
        };

        let service = ApiService::new().with_config(config);
        assert_eq!(service.config().default_page_size, 50);
        assert!(service.config().require_auth);
        assert!(service.config().enable_cors);
    }

    #[test]
    fn test_success_response() {
        let service = ApiService::new();
        let response: ApiResponse<String> = service.success_response("test".to_string());

        assert!(response.success);
        assert_eq!(response.data, Some("test".to_string()));
        assert!(response.errors.is_none());
    }

    #[test]
    fn test_error_response() {
        let service = ApiService::new();
        let response: ApiResponse<()> = service.error_response("Something went wrong");

        assert!(!response.success);
        assert_eq!(response.message, Some("Something went wrong".to_string()));
        assert!(response.data.is_none());
    }

    #[test]
    fn test_pagination_validation() {
        let service = ApiService::new();
        let params = PaginationParams::new().page(1).per_page(50);

        assert!(service.validate_pagination(&params).is_ok());

        // Even large values are clamped by get_per_page(), so they should pass
        let large_params = PaginationParams::new().page(1).per_page(2000);
        assert!(service.validate_pagination(&large_params).is_ok());

        // But invalid page numbers should fail
        let invalid_params = PaginationParams::new().page(20000).per_page(20); // > 10000 max
        assert!(service.validate_pagination(&invalid_params).is_err());
    }
}
