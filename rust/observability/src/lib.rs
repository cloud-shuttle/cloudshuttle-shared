//! # CloudShuttle Observability
//!
//! Centralized logging, metrics, and tracing utilities for CloudShuttle services.
//!
//! This crate provides:
//! - Structured logging with context
//! - Metrics collection and reporting
//! - Distributed tracing
//! - Health check utilities
//! - Observability middleware
//!
//! ## Example
//!
//! ```rust
//! use cloudshuttle_observability::{Logger, MetricsCollector};
//!
//! // Initialize logger
//! let logger = Logger::new("my-service", "1.0.0");
//!
//! // Log with context
//! logger.info("User logged in", &[("user_id", &user_id)]);
//!
//! // Initialize metrics
//! let metrics = MetricsCollector::new();
//! let counter = metrics.counter("requests_total", "Total requests")?;
//! counter.inc();
//! ```

pub mod logging;
pub mod metrics;
pub mod tracing;
pub mod health;
pub mod middleware;

// Re-export main types for convenience
pub use logging::Logger;
pub use metrics::MetricsCollector;
pub use health::{HealthStatus, HealthCheck, HealthState};
