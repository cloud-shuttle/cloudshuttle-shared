//! Request tracing middleware
//!
//! This module provides comprehensive request tracing functionality
//! including unique request IDs, timing information, and request/response
//! logging for observability and debugging.

use std::time::Instant;
use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
    http::{header, HeaderMap, HeaderName},
};
use uuid::Uuid;


/// Request tracing configuration
#[derive(Debug, Clone)]
pub struct TracingConfig {
    /// Whether to include request IDs in responses
    pub include_request_id: bool,
    /// Whether to include timing information in responses
    pub include_timing: bool,
    /// Whether to log requests (requires observability integration)
    pub log_requests: bool,
    /// Header name for request ID (default: "x-request-id")
    pub request_id_header: String,
    /// Header name for response time (default: "x-response-time")
    pub timing_header: String,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            include_request_id: true,
            include_timing: true,
            log_requests: false,
            request_id_header: "x-request-id".to_string(),
            timing_header: "x-response-time".to_string(),
        }
    }
}

/// Tracing context for a request
#[derive(Debug, Clone)]
pub struct TracingContext {
    /// Unique request ID
    pub request_id: String,
    /// Request start time
    pub start_time: Instant,
    /// Request method
    pub method: String,
    /// Request path
    pub path: String,
    /// User agent (if provided)
    pub user_agent: Option<String>,
    /// Client IP address
    pub client_ip: Option<String>,
}

impl TracingContext {
    /// Create a new tracing context for a request
    pub fn new(req: &Request) -> Self {
        let request_id = Self::extract_or_generate_request_id(req.headers());
        let method = req.method().to_string();
        let path = req.uri().path().to_string();
        let user_agent = Self::extract_user_agent(req.headers());
        let client_ip = Self::extract_client_ip(req);

        Self {
            request_id,
            start_time: Instant::now(),
            method,
            path,
            user_agent,
            client_ip,
        }
    }

    /// Extract request ID from headers or generate a new one
    fn extract_or_generate_request_id(headers: &HeaderMap) -> String {
        headers
            .get("x-request-id")
            .or_else(|| headers.get("x-correlation-id"))
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string())
    }

    /// Extract user agent from headers
    fn extract_user_agent(headers: &HeaderMap) -> Option<String> {
        headers
            .get(header::USER_AGENT)
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string())
    }

    /// Extract client IP from request
    fn extract_client_ip(req: &Request) -> Option<String> {
        // Try X-Forwarded-For header first (for proxies/load balancers)
        if let Some(forwarded_for) = req.headers()
            .get("x-forwarded-for")
            .and_then(|h| h.to_str().ok())
        {
            // Take the first IP if there are multiple
            return forwarded_for.split(',').next().map(|s| s.trim().to_string());
        }

        // Try X-Real-IP header
        if let Some(real_ip) = req.headers()
            .get("x-real-ip")
            .and_then(|h| h.to_str().ok())
        {
            return Some(real_ip.to_string());
        }

        // For direct connections, we can't easily get the client IP
        // This would require extracting from the connection info
        None
    }

    /// Calculate elapsed time since request start
    pub fn elapsed_ms(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64() * 1000.0
    }

    /// Format timing as a string (e.g., "123.45ms")
    pub fn timing_string(&self) -> String {
        format!("{:.2}ms", self.elapsed_ms())
    }
}

/// Request tracing middleware
pub struct TracingMiddleware {
    config: TracingConfig,
}

impl TracingMiddleware {
    /// Create new tracing middleware with default config
    pub fn new() -> Self {
        Self {
            config: TracingConfig::default(),
        }
    }

    /// Create new tracing middleware with custom config
    pub fn with_config(config: TracingConfig) -> Self {
        Self { config }
    }

    /// Create the middleware function
    pub fn layer(config: TracingConfig) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
        move |mut req: Request, next: Next| {
            let config = config.clone();
            Box::pin(async move {
                // Create tracing context
                let ctx = TracingContext::new(&req);

                // Add request ID to request extensions for use by handlers
                req.extensions_mut().insert(ctx.clone());

                // Log request if enabled
                // TODO: Integrate with observability crate for logging
                if config.log_requests {
                    // tracing::info!(
                    //     request_id = %ctx.request_id,
                    //     method = %ctx.method,
                    //     path = %ctx.path,
                    //     user_agent = ?ctx.user_agent,
                    //     client_ip = ?ctx.client_ip,
                    //     "Request started"
                    // );
                }

                // Process the request
                let mut response = next.run(req).await;
                let status = response.status();

                // Add headers to response
                let headers = response.headers_mut();

                if config.include_request_id {
                    if let Ok(header_name) = HeaderName::try_from(&config.request_id_header) {
                        headers.insert(header_name, ctx.request_id.parse().unwrap());
                    }
                }

                if config.include_timing {
                    if let Ok(header_name) = HeaderName::try_from(&config.timing_header) {
                        headers.insert(header_name, ctx.timing_string().parse().unwrap());
                    }
                }

                // Log response if enabled
                // TODO: Integrate with observability crate for logging
                if config.log_requests {
                    // tracing::info!(
                    //     request_id = %ctx.request_id,
                    //     method = %ctx.method,
                    //     path = %ctx.path,
                    //     status = %status.as_u16(),
                    //     duration_ms = %ctx.elapsed_ms(),
                    //     "Request completed"
                    // );
                }

                response
            })
        }
    }
}

impl Default for TracingMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

/// Extractor for getting tracing context in handlers
pub struct RequestTracing(pub TracingContext);

impl RequestTracing {
    /// Get the request ID
    pub fn request_id(&self) -> &str {
        &self.0.request_id
    }

    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> f64 {
        self.0.elapsed_ms()
    }

    /// Get timing as formatted string
    pub fn timing_string(&self) -> String {
        self.0.timing_string()
    }
}

#[cfg(feature = "axum")]
mod axum_integration {
    use super::*;
    use axum::extract::FromRequest;

    #[async_trait::async_trait]
    impl<S> axum::extract::FromRequest<S> for RequestTracing
    where
        S: Send + Sync,
    {
        type Rejection = (StatusCode, &'static str);

        async fn from_request(req: axum::extract::Request, _state: &mut S) -> Result<Self, Self::Rejection> {
            req.extensions()
                .get::<TracingContext>()
                .cloned()
                .map(RequestTracing)
                .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Tracing context not found"))
        }
    }
}

/// Pre-configured tracing configurations for common use cases
pub mod presets {
    use super::*;

    /// Create a minimal tracing config (just request IDs)
    pub fn minimal() -> TracingConfig {
        TracingConfig {
            include_request_id: true,
            include_timing: false,
            log_requests: false,
            ..Default::default()
        }
    }

    /// Create a standard tracing config (request IDs and timing)
    pub fn standard() -> TracingConfig {
        TracingConfig::default()
    }

    /// Create a verbose tracing config (with request logging)
    pub fn verbose() -> TracingConfig {
        TracingConfig {
            include_request_id: true,
            include_timing: true,
            log_requests: true,
            ..Default::default()
        }
    }

    /// Create a production tracing config (optimized for performance)
    pub fn production() -> TracingConfig {
        TracingConfig {
            include_request_id: true,
            include_timing: true,
            log_requests: false, // Logging handled by observability stack
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;
    use axum::body::Body;

    #[test]
    fn test_tracing_config_defaults() {
        let config = TracingConfig::default();
        assert!(config.include_request_id);
        assert!(config.include_timing);
        assert!(!config.log_requests);
        assert_eq!(config.request_id_header, "x-request-id");
        assert_eq!(config.timing_header, "x-response-time");
    }

    #[test]
    fn test_tracing_context_creation() {
        let req = Request::builder()
            .method("GET")
            .uri("/test")
            .header("user-agent", "test-agent")
            .body(Body::empty())
            .unwrap();

        let ctx = TracingContext::new(&req);

        assert!(!ctx.request_id.is_empty());
        assert_eq!(ctx.method, "GET");
        assert_eq!(ctx.path, "/test");
        assert_eq!(ctx.user_agent, Some("test-agent".to_string()));
    }

    #[test]
    fn test_request_id_extraction() {
        let req = Request::builder()
            .header("x-request-id", "custom-request-id")
            .body(Body::empty())
            .unwrap();

        let ctx = TracingContext::new(&req);
        assert_eq!(ctx.request_id, "custom-request-id");
    }

    #[test]
    fn test_client_ip_extraction() {
        let req = Request::builder()
            .header("x-forwarded-for", "192.168.1.1, 10.0.0.1")
            .body(Body::empty())
            .unwrap();

        let ctx = TracingContext::new(&req);
        assert_eq!(ctx.client_ip, Some("192.168.1.1".to_string()));
    }

    #[test]
    fn test_timing_calculation() {
        let req = Request::builder().body(Body::empty()).unwrap();
        let ctx = TracingContext::new(&req);

        // Small delay to ensure timing works
        std::thread::sleep(std::time::Duration::from_millis(1));

        let elapsed = ctx.elapsed_ms();
        assert!(elapsed > 0.0);

        let timing_str = ctx.timing_string();
        assert!(timing_str.ends_with("ms"));
    }

    #[test]
    fn test_presets() {
        let minimal = presets::minimal();
        assert!(minimal.include_request_id);
        assert!(!minimal.include_timing);

        let verbose = presets::verbose();
        assert!(verbose.log_requests);
    }
}
