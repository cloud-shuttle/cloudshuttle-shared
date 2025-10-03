//! CORS middleware for API endpoints
//!
//! This module provides configurable CORS (Cross-Origin Resource Sharing)
//! functionality for API endpoints.

use axum::{
    extract::Request,
    middleware::Next,
    response::{Response, IntoResponse},
    http::{header, Method, StatusCode},
};

/// CORS configuration
#[derive(Debug, Clone)]
pub struct CorsConfig {
    /// Allowed origins (use "*" for all)
    pub allowed_origins: Vec<String>,
    /// Allowed HTTP methods
    pub allowed_methods: Vec<Method>,
    /// Allowed headers
    pub allowed_headers: Vec<String>,
    /// Headers exposed to the client
    pub exposed_headers: Vec<String>,
    /// Whether credentials are allowed
    pub allow_credentials: bool,
    /// Max age for preflight cache (in seconds)
    pub max_age: Option<u32>,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec![
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
                Method::HEAD,
                Method::PATCH,
            ],
            allowed_headers: vec![
                "authorization".to_string(),
                "content-type".to_string(),
                "x-requested-with".to_string(),
                "accept".to_string(),
                "origin".to_string(),
            ],
            exposed_headers: vec![],
            allow_credentials: false,
            max_age: Some(86400), // 24 hours
        }
    }
}

impl CorsConfig {
    /// Create a new CORS config with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set allowed origins
    pub fn allowed_origins(mut self, origins: Vec<String>) -> Self {
        self.allowed_origins = origins;
        self
    }

    /// Add an allowed origin
    pub fn allow_origin(mut self, origin: impl Into<String>) -> Self {
        self.allowed_origins.push(origin.into());
        self
    }

    /// Set allowed methods
    pub fn allowed_methods(mut self, methods: Vec<Method>) -> Self {
        self.allowed_methods = methods;
        self
    }

    /// Add an allowed method
    pub fn allow_method(mut self, method: Method) -> Self {
        self.allowed_methods.push(method);
        self
    }

    /// Set allowed headers
    pub fn allowed_headers(mut self, headers: Vec<String>) -> Self {
        self.allowed_headers = headers;
        self
    }

    /// Add an allowed header
    pub fn allow_header(mut self, header: impl Into<String>) -> Self {
        self.allowed_headers.push(header.into());
        self
    }

    /// Set exposed headers
    pub fn exposed_headers(mut self, headers: Vec<String>) -> Self {
        self.exposed_headers = headers;
        self
    }

    /// Add an exposed header
    pub fn expose_header(mut self, header: impl Into<String>) -> Self {
        self.exposed_headers.push(header.into());
        self
    }

    /// Allow credentials
    pub fn allow_credentials(mut self, allow: bool) -> Self {
        self.allow_credentials = allow;
        self
    }

    /// Set max age for preflight cache
    pub fn max_age(mut self, max_age: u32) -> Self {
        self.max_age = Some(max_age);
        self
    }

    /// Check if an origin is allowed
    pub fn is_origin_allowed(&self, origin: &str) -> bool {
        self.allowed_origins.contains(&"*".to_string()) || self.allowed_origins.contains(&origin.to_string())
    }

    /// Check if a method is allowed
    pub fn is_method_allowed(&self, method: &Method) -> bool {
        self.allowed_methods.contains(method)
    }

    /// Check if a header is allowed
    pub fn is_header_allowed(&self, header: &str) -> bool {
        self.allowed_headers.iter().any(|h| h.eq_ignore_ascii_case(header))
    }
}

/// CORS middleware result
#[derive(Debug)]
pub enum CorsResult {
    /// Request is allowed to proceed
    Allowed {
        /// Headers that should be exposed to the client
        exposed_headers: Vec<String>,
    },
    /// Invalid origin
    InvalidOrigin,
    /// Method not allowed
    MethodNotAllowed,
    /// Header not allowed
    HeaderNotAllowed,
}

/// CORS middleware
pub struct CorsMiddleware {
    config: CorsConfig,
}

impl CorsMiddleware {
    /// Create new CORS middleware with default config
    pub fn new() -> Self {
        Self {
            config: CorsConfig::default(),
        }
    }

    /// Create new CORS middleware with custom config
    pub fn with_config(config: CorsConfig) -> Self {
        Self { config }
    }

    /// Validate a CORS request
    pub fn validate_request(&self, origin: Option<&str>, method: &Method, headers: &[&str]) -> CorsResult {
        // Check origin
        if let Some(origin) = origin {
            if !self.config.is_origin_allowed(origin) {
                return CorsResult::InvalidOrigin;
            }
        } else if !self.config.allowed_origins.contains(&"*".to_string()) {
            // If no origin header and we don't allow all origins, deny
            return CorsResult::InvalidOrigin;
        }

        // Check method
        if !self.config.is_method_allowed(method) {
            return CorsResult::MethodNotAllowed;
        }

        // Check headers
        for header in headers {
            if !self.config.is_header_allowed(header) {
                return CorsResult::HeaderNotAllowed;
            }
        }

        CorsResult::Allowed {
            exposed_headers: self.config.exposed_headers.clone(),
        }
    }

    /// Create the middleware function
    pub fn layer(config: CorsConfig) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
        move |req: Request, next: Next| {
            let config = config.clone();
            let middleware = CorsMiddleware::with_config(config);

            // Extract values before moving req
            let origin = req.headers()
                .get(header::ORIGIN)
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string());

            let method = req.method().clone();

            // For preflight requests, check all requested headers
            let request_headers = if method == Method::OPTIONS {
                req.headers()
                    .get(header::ACCESS_CONTROL_REQUEST_HEADERS)
                    .and_then(|h| h.to_str().ok())
                    .map(|h| h.split(',').map(|s| s.trim().to_string()).collect::<Vec<_>>())
                    .unwrap_or_default()
            } else {
                vec![]
            };

            Box::pin(async move {
                match middleware.validate_request(origin.as_deref(), &method, &request_headers.iter().map(|s| s.as_str()).collect::<Vec<_>>()) {
                    CorsResult::Allowed { exposed_headers } => {
                        let mut response = next.run(req).await;

                        // Add CORS headers to response
                        let headers = response.headers_mut();

                        // Set allowed origin
                        if let Some(origin) = origin {
                            if middleware.config.allowed_origins.contains(&"*".to_string()) {
                                headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
                            } else {
                                headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin.parse().unwrap());
                            }
                        }

                        // Set allowed methods
                        let methods_str = middleware.config.allowed_methods
                            .iter()
                            .map(|m| m.as_str())
                            .collect::<Vec<_>>()
                            .join(", ");
                        headers.insert(header::ACCESS_CONTROL_ALLOW_METHODS, methods_str.parse().unwrap());

                        // Set allowed headers
                        let headers_str = middleware.config.allowed_headers.join(", ");
                        headers.insert(header::ACCESS_CONTROL_ALLOW_HEADERS, headers_str.parse().unwrap());

                        // Set exposed headers if any
                        if !exposed_headers.is_empty() {
                            let exposed_str = exposed_headers.join(", ");
                            headers.insert(header::ACCESS_CONTROL_EXPOSE_HEADERS, exposed_str.parse().unwrap());
                        }

                        // Set credentials
                        if middleware.config.allow_credentials {
                            headers.insert(header::ACCESS_CONTROL_ALLOW_CREDENTIALS, "true".parse().unwrap());
                        }

                        // Set max age for preflight
                        if let Some(max_age) = middleware.config.max_age {
                            headers.insert(header::ACCESS_CONTROL_MAX_AGE, max_age.to_string().parse().unwrap());
                        }

                        response
                    }
                    CorsResult::InvalidOrigin => {
                        (StatusCode::FORBIDDEN, "CORS: Origin not allowed").into_response()
                    }
                    CorsResult::MethodNotAllowed => {
                        (StatusCode::METHOD_NOT_ALLOWED, "CORS: Method not allowed").into_response()
                    }
                    CorsResult::HeaderNotAllowed => {
                        (StatusCode::FORBIDDEN, "CORS: Header not allowed").into_response()
                    }
                }
            })
        }
    }
}

impl Default for CorsMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

/// Pre-configured CORS configurations for common use cases
pub mod presets {
    use super::*;

    /// Create a permissive CORS config for development
    pub fn permissive() -> CorsConfig {
        CorsConfig::new()
            .allowed_origins(vec!["*".to_string()])
            .allow_credentials(false)
    }

    /// Create a restrictive CORS config for production
    pub fn restrictive() -> CorsConfig {
        CorsConfig::new()
            .allowed_origins(vec!["https://app.cloudshuttle.com".to_string()])
            .allow_credentials(true)
            .allow_header("authorization")
            .allow_header("x-api-key")
    }

    /// Create a CORS config for API endpoints
    pub fn api() -> CorsConfig {
        CorsConfig::new()
            .allowed_origins(vec!["https://app.cloudshuttle.com".to_string(), "https://admin.cloudshuttle.com".to_string()])
            .allow_credentials(true)
            .allow_header("authorization")
            .allow_header("content-type")
            .allow_header("x-request-id")
            .max_age(3600) // 1 hour
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cors_config_defaults() {
        let config = CorsConfig::default();
        assert!(config.allowed_origins.contains(&"*".to_string()));
        assert!(config.allowed_methods.contains(&Method::GET));
        assert!(config.allowed_headers.contains(&"authorization".to_string()));
        assert!(!config.allow_credentials);
    }

    #[test]
    fn test_cors_config_builder() {
        let config = CorsConfig::new()
            .allow_origin("https://example.com")
            .allow_method(Method::PATCH)
            .allow_header("x-custom-header")
            .allow_credentials(true);

        assert!(config.is_origin_allowed("https://example.com"));
        assert!(config.is_method_allowed(&Method::PATCH));
        assert!(config.is_header_allowed("x-custom-header"));
        assert!(config.allow_credentials);
    }

    #[test]
    fn test_cors_validation() {
        let middleware = CorsMiddleware::with_config(
            CorsConfig::new()
                .allowed_origins(vec!["https://allowed.com".to_string()])
                .allow_method(Method::POST)
        );

        // Valid request
        match middleware.validate_request(Some("https://allowed.com"), &Method::POST, &[]) {
            CorsResult::Allowed { .. } => {}
            _ => panic!("Expected allowed"),
        }

        // Invalid origin
        match middleware.validate_request(Some("https://notallowed.com"), &Method::POST, &[]) {
            CorsResult::InvalidOrigin => {}
            _ => panic!("Expected invalid origin"),
        }

        // Invalid method
        match middleware.validate_request(Some("https://allowed.com"), &Method::TRACE, &[]) {
            CorsResult::MethodNotAllowed => {}
            _ => panic!("Expected method not allowed"),
        }
    }

    #[test]
    fn test_presets() {
        let permissive = presets::permissive();
        assert!(permissive.allowed_origins.contains(&"*".to_string()));

        let restrictive = presets::restrictive();
        assert!(restrictive.allowed_origins.contains(&"https://app.cloudshuttle.com".to_string()));
        assert!(restrictive.allow_credentials);
    }
}
