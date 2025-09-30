//! # CloudShuttle Observability
//!
//! Comprehensive observability utilities for CloudShuttle services.
//!
//! ## Features
//!
//! - Structured logging with configurable levels
//! - Prometheus metrics collection
//! - Distributed tracing support
//! - Health check endpoints
//! - Request tracing middleware
//! - Performance monitoring
//!
//! ## Example
//!
//! ```rust
//! use cloudshuttle_observability::{init_tracing, register_metrics, create_health_route};
//!
//! // Initialize tracing
//! init_tracing("my-service", tracing::Level::INFO)?;
//!
//! // Register metrics
//! register_metrics();
//!
//! // Create health check route
//! let health_route = create_health_route();
//! ```

pub mod logging;
pub mod metrics;
pub mod tracing;
pub mod health;
pub mod middleware;

// Re-export main functions and types
pub use logging::{init_tracing, TracingConfig, LogFormat, Logger, LogLevel, LogSampler, PerformanceLogger};
pub use metrics::{register_metrics, MetricsCollector, HTTP_REQUEST_COUNT as REQUEST_COUNT, HTTP_REQUEST_DURATION_SECONDS as REQUEST_DURATION};
pub use tracing::{TraceId, SpanId, SpanBuilder, Span, TracingMiddleware, TraceContext};
#[cfg(feature = "axum")]
pub use health::{HealthChecker, HealthStatus, HealthResponse, HealthCheckerConfig};
pub use middleware::{ObservabilityLayer, RequestContext, RequestMetrics};

// Conditional exports for axum feature
#[cfg(feature = "axum")]
pub use health::create_health_route;
#[cfg(feature = "axum")]
pub use middleware::axum_middleware;

// Re-export prometheus metrics for convenience
pub use prometheus::{Counter, Gauge, Histogram, register_counter, register_gauge, register_histogram};
