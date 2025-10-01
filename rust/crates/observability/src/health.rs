//! Health check utilities
//!
//! This module provides health checking functionality for services,
//! including HTTP health checks and health status management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Health status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    /// System is healthy
    Healthy,
    /// System is degraded but functional
    Degraded,
    /// System is unhealthy
    Unhealthy,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Overall health status
    pub status: HealthStatus,
    /// Service name
    pub service: String,
    /// Timestamp of the check
    pub timestamp: String,
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Additional health details
    pub details: HashMap<String, serde_json::Value>,
}

/// Health checker for HTTP endpoints
#[cfg(feature = "axum")]
pub struct HealthChecker {
    client: reqwest::Client,
    config: HealthCheckerConfig,
}

/// Configuration for health checker
#[derive(Debug, Clone)]
pub struct HealthCheckerConfig {
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Connection timeout in seconds
    pub connect_timeout_seconds: u64,
    /// User agent string
    pub user_agent: String,
    /// Whether to follow redirects
    pub follow_redirects: bool,
    /// Whether to verify SSL certificates
    pub verify_ssl: bool,
}

impl Default for HealthCheckerConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 5,
            connect_timeout_seconds: 2,
            user_agent: "CloudShuttle-HealthChecker/1.0".to_string(),
            follow_redirects: true,
            verify_ssl: true,
        }
    }
}

/// Error type for health check operations
#[derive(Debug, thiserror::Error)]
pub enum HealthCheckError {
    #[cfg(feature = "axum")]
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Timeout exceeded")]
    Timeout,

    #[error("Service unavailable")]
    ServiceUnavailable,
}

/// Result type for health check operations
pub type Result<T> = std::result::Result<T, HealthCheckError>;

/// HTTP health check response
#[cfg(feature = "axum")]
#[derive(Debug, Clone)]
pub struct HealthCheckResponse {
    /// HTTP status code
    pub status: reqwest::StatusCode,
    /// Response body
    pub body: String,
    /// Response time
    pub response_time: Duration,
}

#[cfg(feature = "axum")]
impl HealthChecker {
    /// Create a new health checker with default configuration
    pub fn new() -> Self {
        Self::with_config(HealthCheckerConfig::default())
    }

    /// Create a health checker with custom configuration
    pub fn with_config(config: HealthCheckerConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .connect_timeout(Duration::from_secs(config.connect_timeout_seconds))
            .user_agent(&config.user_agent)
            .redirect(if config.follow_redirects {
                reqwest::redirect::Policy::limited(10)
            } else {
                reqwest::redirect::Policy::none()
            })
            .danger_accept_invalid_certs(!config.verify_ssl)
            .build()
            .expect("Failed to create HTTP client");

        Self { client, config }
    }

    /// Check health of an HTTP endpoint
    pub async fn check_health(&self, url: &str) -> Result<HealthCheckResponse> {
        let start_time = Instant::now();

        let response = self.client.get(url).send().await?;
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        let response_time = start_time.elapsed();

        Ok(HealthCheckResponse {
            status,
            body,
            response_time,
        })
    }

    /// Check health with custom headers
    pub async fn check_health_with_headers(
        &self,
        url: &str,
        headers: &HashMap<String, String>,
    ) -> Result<HealthCheckResponse> {
        let start_time = Instant::now();

        let mut request = self.client.get(url);
        for (key, value) in headers {
            request = request.header(key, value);
        }

        let response = request.send().await?;
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        let response_time = start_time.elapsed();

        Ok(HealthCheckResponse {
            status,
            body,
            response_time,
        })
    }
}

/// Create a standard health check response
pub fn create_health_response(
    service: &str,
    status: HealthStatus,
    response_time: Duration,
) -> HealthResponse {
    HealthResponse {
        status,
        service: service.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        response_time_ms: response_time.as_millis() as u64,
        details: HashMap::new(),
    }
}

/// Create a health check Axum route
#[cfg(feature = "axum")]
pub fn create_health_route() -> axum::Router {
    use axum::{routing::get, Json};

    axum::Router::new().route("/health", get(|| async {
        let response = create_health_response(
            "observability-service",
            HealthStatus::Healthy,
            Duration::from_millis(10),
        );
        Json(response)
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_health_checker_config_default() {
        let config = HealthCheckerConfig::default();
        assert_eq!(config.timeout_seconds, 5);
        assert_eq!(config.connect_timeout_seconds, 2);
        assert!(config.user_agent.contains("CloudShuttle"));
    }

    #[test]
    fn test_create_health_response() {
        let response = create_health_response(
            "test-service",
            HealthStatus::Healthy,
            Duration::from_millis(50),
        );

        assert_eq!(response.service, "test-service");
        assert_eq!(response.status, HealthStatus::Healthy);
        assert_eq!(response.response_time_ms, 50);
        assert!(!response.timestamp.is_empty());
    }

    #[cfg(feature = "axum")]
    #[tokio::test]
    async fn test_health_checker_creation() {
        let checker = HealthChecker::new();
        // Just verify it creates without panic
        assert!(true);
    }
}
