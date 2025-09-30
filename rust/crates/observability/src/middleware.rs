//! HTTP middleware for observability
//!
//! This module provides Axum middleware for request tracing,
//! metrics collection, and logging.

use std::time::Instant;
use std::collections::HashMap;

/// Observability middleware layer
#[derive(Debug, Clone)]
pub struct ObservabilityLayer {
    service_name: String,
    enable_metrics: bool,
    enable_tracing: bool,
}

impl ObservabilityLayer {
    /// Create a new observability layer
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            enable_metrics: true,
            enable_tracing: true,
        }
    }

    /// Enable or disable metrics collection
    pub fn with_metrics(mut self, enable: bool) -> Self {
        self.enable_metrics = enable;
        self
    }

    /// Enable or disable tracing
    pub fn with_tracing(mut self, enable: bool) -> Self {
        self.enable_tracing = enable;
        self
    }

    /// Get service name
    pub fn service_name(&self) -> &str {
        &self.service_name
    }
}

/// Request context for observability
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// Request ID
    pub request_id: String,
    /// Start time
    pub start_time: Instant,
    /// HTTP method
    pub method: String,
    /// Request path
    pub path: String,
    /// Query parameters
    pub query: Option<String>,
    /// User agent
    pub user_agent: Option<String>,
    /// Remote address
    pub remote_addr: Option<String>,
}

impl RequestContext {
    /// Create a new request context
    pub fn new(method: &str, path: &str) -> Self {
        use rand::{RngCore, rngs::OsRng};

        // Generate a request ID
        let mut bytes = [0u8; 8];
        OsRng.fill_bytes(&mut bytes);
        let request_id = hex::encode(bytes);

        Self {
            request_id,
            start_time: Instant::now(),
            method: method.to_string(),
            path: path.to_string(),
            query: None,
            user_agent: None,
            remote_addr: None,
        }
    }

    /// Set query parameters
    pub fn with_query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }

    /// Set user agent
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Set remote address
    pub fn with_remote_addr(mut self, addr: impl Into<String>) -> Self {
        self.remote_addr = Some(addr.into());
        self
    }

    /// Get request duration
    pub fn duration(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    /// Get request ID
    pub fn request_id(&self) -> &str {
        &self.request_id
    }
}

/// Request metrics for monitoring
#[derive(Debug, Clone)]
pub struct RequestMetrics {
    pub method: String,
    pub path: String,
    pub status_code: u16,
    pub duration_ms: u64,
    pub user_agent: Option<String>,
}

impl RequestMetrics {
    /// Create from request context and response
    pub fn from_context_and_status(context: &RequestContext, status_code: u16) -> Self {
        Self {
            method: context.method.clone(),
            path: context.path.clone(),
            status_code,
            duration_ms: context.duration().as_millis() as u64,
            user_agent: context.user_agent.clone(),
        }
    }
}

/// Error type for observability operations
#[derive(Debug, thiserror::Error)]
pub enum ObservabilityError {
    #[error("Metrics collection failed: {0}")]
    MetricsError(String),

    #[error("Tracing failed: {0}")]
    TracingError(String),

    #[error("Logging failed: {0}")]
    LoggingError(String),
}

/// Result type for observability operations
pub type Result<T> = std::result::Result<T, ObservabilityError>;

/// Axum middleware for observability
#[cfg(feature = "axum")]
pub mod axum_middleware {
    use super::*;
    use axum::{
        extract::{Request, ConnectInfo},
        middleware::Next,
        response::Response,
        http::{HeaderMap, StatusCode},
    };
    use std::net::SocketAddr;

    /// Observability middleware function
    pub async fn observability_middleware(
        ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
        headers: HeaderMap,
        mut request: Request,
        next: Next,
    ) -> Response {
        // Extract request information
        let method = request.method().to_string();
        let path = request.uri().path().to_string();
        let query = request.uri().query().map(|q| q.to_string());

        // Create request context
        let mut context = RequestContext::new(&method, &path)
            .with_query(query.unwrap_or_default())
            .with_remote_addr(remote_addr.to_string());

        // Extract user agent
        if let Some(user_agent) = headers.get("user-agent") {
            if let Ok(ua) = user_agent.to_str() {
                context = context.with_user_agent(ua.to_string());
            }
        }

        // Store context in request extensions
        request.extensions_mut().insert(context.clone());

        // Log request start
        tracing::info!(
            request_id = %context.request_id(),
            method = %method,
            path = %path,
            remote_addr = %remote_addr,
            "Request started"
        );

        // Process request
        let start = Instant::now();
        let response = next.run(request).await;
        let duration = start.elapsed();

        // Extract status code
        let status_code = response.status().as_u16();

        // Create metrics
        let metrics = RequestMetrics::from_context_and_status(&context, status_code);

        // Log request completion
        tracing::info!(
            request_id = %context.request_id(),
            method = %method,
            path = %path,
            status = %status_code,
            duration_ms = duration.as_millis(),
            "Request completed"
        );

        // Record metrics (if metrics collection is enabled)
        // This would integrate with the metrics module

        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_context_creation() {
        let context = RequestContext::new("GET", "/api/health");
        assert_eq!(context.method, "GET");
        assert_eq!(context.path, "/api/health");
        assert!(!context.request_id.is_empty());
    }

    #[test]
    fn test_request_context_with_options() {
        let context = RequestContext::new("POST", "/api/users")
            .with_query("name=test")
            .with_user_agent("test-agent")
            .with_remote_addr("127.0.0.1:8080");

        assert_eq!(context.method, "POST");
        assert_eq!(context.path, "/api/users");
        assert_eq!(context.query, Some("name=test".to_string()));
        assert_eq!(context.user_agent, Some("test-agent".to_string()));
        assert_eq!(context.remote_addr, Some("127.0.0.1:8080".to_string()));
    }

    #[test]
    fn test_request_metrics() {
        let context = RequestContext::new("GET", "/api/health");
        let metrics = RequestMetrics::from_context_and_status(&context, 200);

        assert_eq!(metrics.method, "GET");
        assert_eq!(metrics.path, "/api/health");
        assert_eq!(metrics.status_code, 200);
        assert!(metrics.duration_ms >= 0);
    }

    #[test]
    fn test_observability_layer() {
        let layer = ObservabilityLayer::new("test-service")
            .with_metrics(true)
            .with_tracing(true);

        assert_eq!(layer.service_name(), "test-service");
    }
}
