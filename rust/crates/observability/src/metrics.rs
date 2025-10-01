//! Metrics collection and recording - orchestrates all metrics operations
//!
//! This module orchestrates multiple metrics domains through specialized sub-modules:
//! - `metrics_collection`: Global metrics registry and static metric definitions
//! - `metrics_recording`: Metrics value recording and updates
//! - `metrics_middleware`: HTTP middleware for automatic metrics collection

pub mod metrics_collection;
pub mod metrics_recording;
pub mod metrics_middleware;

// Re-export for backward compatibility and convenience
pub use metrics_collection::{
    MetricsRegistry, MetricBuilder, PROMETHEUS_REGISTRY, HTTP_REQUEST_COUNT,
    HTTP_REQUEST_DURATION_SECONDS, ACTIVE_CONNECTIONS, DB_POOL_SIZE,
    DB_POOL_ACTIVE, ERROR_COUNT, BUSINESS_METRICS, register_metrics,
};
pub use metrics_recording::{
    MetricsRecorder, RequestMetrics, DatabaseMetrics, HealthMetrics, PerformanceMetrics,
};
pub use metrics_middleware::{MetricsMiddleware, AlertManager};

/// Legacy metrics collector for backward compatibility
pub struct MetricsCollector {
    registry: std::sync::Arc<std::sync::Mutex<MetricsRegistry>>,
    recorder: MetricsRecorder,
}

impl MetricsCollector {
    /// Create new metrics collector (backward compatibility)
    pub fn new() -> Self {
        let registry = std::sync::Arc::new(std::sync::Mutex::new(MetricsRegistry::new()));
        let recorder = MetricsRecorder::new(registry.clone());
        Self { registry, recorder }
    }

    /// Register a custom metric (backward compatibility)
    pub fn register_custom_metric<C: prometheus::core::Collector + 'static>(
        &self,
        name: String,
        metric: C,
    ) -> Result<(), prometheus::Error> {
        let mut registry = self.registry.lock().unwrap();
        registry.register_custom_metric(name, metric)
    }

    /// Increment a business metric (backward compatibility)
    pub fn increment_business_metric(&self, name: &str) {
        self.recorder.increment_business_metric(name);
    }

    /// Increment business metric by value (backward compatibility)
    pub fn increment_business_metric_by(&self, name: &str, value: f64) {
        self.recorder.increment_business_metric_by(name, value);
    }

    /// Set gauge value (backward compatibility)
    pub fn set_gauge(&self, gauge: &prometheus::Gauge, value: f64) {
        self.recorder.set_gauge(gauge, value);
    }

    /// Add to gauge value (backward compatibility)
    pub fn add_to_gauge(&self, gauge: &prometheus::Gauge, value: f64) {
        self.recorder.add_to_gauge(gauge, value);
    }

    /// Observe histogram value (backward compatibility)
    pub fn observe_histogram(&self, histogram: &prometheus::Histogram, value: f64) {
        self.recorder.observe_histogram(histogram, value);
    }

    /// Get metrics in Prometheus format (backward compatibility)
    pub fn gather_metrics(&self) -> Result<String, prometheus::Error> {
        let registry = self.registry.lock().unwrap();
        registry.gather_metrics()
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collector_backward_compatibility() {
        let collector = MetricsCollector::new();

        // Test that old API still works
        collector.increment_business_metric("users_created_total");

        // Test custom metric registration (should not panic)
        let counter = MetricBuilder::new("test_counter", "A test counter")
            .build_counter()
            .unwrap();
        let _ = collector.register_custom_metric("test".to_string(), counter);

        // Test gathering metrics
        let result = collector.gather_metrics();
        assert!(result.is_ok());
    }
}
