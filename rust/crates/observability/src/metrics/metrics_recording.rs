//! Metrics recording and value updates

use prometheus::{Counter, Gauge, Histogram};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// HTTP request metrics
#[derive(Debug, Clone)]
pub struct RequestMetrics {
    pub method: String,
    pub path: String,
    pub status_code: u16,
    pub duration_ms: f64,
    pub start_time: std::time::Instant,
}

impl RequestMetrics {
    pub fn new(method: impl Into<String>, path: impl Into<String>) -> Self {
        super::HTTP_REQUEST_COUNT.inc();
        super::ACTIVE_CONNECTIONS.inc();

        Self {
            method: method.into(),
            path: path.into(),
            status_code: 0, // Will be set on completion
            duration_ms: 0.0, // Will be calculated on completion
            start_time: std::time::Instant::now(),
        }
    }

    pub fn record_success(mut self) -> Self {
        self.duration_ms = self.start_time.elapsed().as_millis() as f64;
        self.status_code = 200;
        super::HTTP_REQUEST_DURATION_SECONDS.observe(self.duration_ms / 1000.0);
        super::ACTIVE_CONNECTIONS.dec();
        self
    }

    pub fn record_error(mut self, status_code: u16) -> Self {
        self.duration_ms = self.start_time.elapsed().as_millis() as f64;
        self.status_code = status_code;
        super::HTTP_REQUEST_DURATION_SECONDS.observe(self.duration_ms / 1000.0);
        super::ACTIVE_CONNECTIONS.dec();
        super::ERROR_COUNT.inc();
        self
    }
}

impl Drop for RequestMetrics {
    fn drop(&mut self) {
        // Ensure we decrement active connections even if not explicitly called
        super::ACTIVE_CONNECTIONS.dec();
    }
}

/// Metrics recorder for updating metric values
pub struct MetricsRecorder {
    registry: Arc<Mutex<crate::metrics::MetricsRegistry>>,
}

impl MetricsRecorder {
    pub fn new(registry: Arc<Mutex<crate::metrics::MetricsRegistry>>) -> Self {
        Self { registry }
    }

    /// Record HTTP request metrics
    pub fn record_request(&self, metrics: RequestMetrics) {
        // The RequestMetrics struct handles its own recording
        // This method is for additional custom recording logic
        tracing::debug!(
            "Recorded HTTP request: {} {} -> {} ({}ms)",
            metrics.method,
            metrics.path,
            metrics.status_code,
            metrics.duration_ms
        );
    }

    /// Record error occurrence
    pub fn record_error(&self, error_type: &str) {
        super::ERROR_COUNT.inc();
        tracing::debug!("Recorded error: {}", error_type);
    }

    /// Increment a business metric
    pub fn increment_business_metric(&self, name: &str) {
        if let Some(counter) = super::BUSINESS_METRICS.get(name) {
            counter.inc();
        } else {
            tracing::warn!("Business metric '{}' not registered", name);
        }
    }

    /// Increment business metric by value
    pub fn increment_business_metric_by(&self, name: &str, value: f64) {
        if let Some(counter) = super::BUSINESS_METRICS.get(name) {
            counter.inc_by(value);
        } else {
            tracing::warn!("Business metric '{}' not registered", name);
        }
    }

    /// Set gauge value
    pub fn set_gauge(&self, gauge: &Gauge, value: f64) {
        gauge.set(value);
    }

    /// Add to gauge value
    pub fn add_to_gauge(&self, gauge: &Gauge, value: f64) {
        gauge.add(value);
    }

    /// Subtract from gauge value
    pub fn sub_from_gauge(&self, gauge: &Gauge, value: f64) {
        gauge.sub(value);
    }

    /// Observe histogram value
    pub fn observe_histogram(&self, histogram: &Histogram, value: f64) {
        histogram.observe(value);
    }

    /// Record custom counter increment
    pub fn increment_counter(&self, name: &str, value: f64) {
        // For custom metrics, we'd need to look them up
        // This is a simplified implementation
        tracing::debug!("Custom counter {} incremented by {}", name, value);
    }

    /// Record business metrics with labels
    pub fn record_business_metric(&self, name: &str, value: f64, labels: HashMap<String, String>) {
        self.increment_business_metric_by(name, value);
        tracing::debug!("Business metric {} recorded with labels: {:?}", name, labels);
    }

    /// Record database metrics
    pub fn record_database_metric(&self, metric_type: &str, value: f64) {
        match metric_type {
            "pool_size" => self.set_gauge(&super::DB_POOL_SIZE, value),
            "pool_active" => self.set_gauge(&super::DB_POOL_ACTIVE, value),
            "query_duration" => {
                // Could add a histogram for query durations
                tracing::debug!("Query duration: {}s", value);
            }
            _ => tracing::warn!("Unknown database metric type: {}", metric_type),
        }
    }

    /// Record health check metrics
    pub fn record_health_metric(&self, service: &str, status: &str, duration_ms: f64) {
        tracing::debug!("Health check {} status: {} took {}ms", service, status, duration_ms);
        // Could add specific health check metrics here
    }

    /// Record performance metrics
    pub fn record_performance_metric(&self, operation: &str, duration_ms: f64) {
        tracing::debug!("Operation {} took {}ms", operation, duration_ms);
        // Could add operation-specific histograms here
    }

    /// Record alert-worthy events
    pub fn record_alert(&self, alert_type: &str, service: &str, severity: &str, details: serde_json::Value) {
        match severity {
            "error" => tracing::error!("Alert {} for {}: {}", alert_type, service, details),
            "warn" => tracing::warn!("Alert {} for {}: {}", alert_type, service, details),
            "info" => tracing::info!("Alert {} for {}: {}", alert_type, service, details),
            _ => tracing::debug!("Alert {} for {}: {}", alert_type, service, details),
        }
    }
}

/// Database metrics helper
pub struct DatabaseMetrics;

impl DatabaseMetrics {
    pub fn set_pool_size(size: f64) {
        super::DB_POOL_SIZE.set(size);
    }

    pub fn set_pool_active(active: f64) {
        super::DB_POOL_ACTIVE.set(active);
    }

    pub fn record_query_duration(duration_seconds: f64) {
        // Could add a histogram for query durations
        tracing::debug!("Query duration: {}s", duration_seconds);
    }

    pub fn increment_error_count() {
        super::ERROR_COUNT.inc();
    }
}

/// Health metrics helper
pub struct HealthMetrics;

impl HealthMetrics {
    pub fn record_health_check(status: &str, duration_ms: f64) {
        // Could add specific health check metrics
        tracing::debug!("Health check status: {} took {}ms", status, duration_ms);
    }

    pub fn record_service_status(service: &str, status: &str) {
        // Could add service status gauges
        tracing::info!("Service {} status: {}", service, status);
    }
}

/// Performance metrics helper
pub struct PerformanceMetrics;

impl PerformanceMetrics {
    pub fn record_operation_time(operation: &str, duration_ms: f64) {
        // Could add operation-specific histograms
        tracing::debug!("Operation {} took {}ms", operation, duration_ms);
    }

    #[cfg(feature = "sys-info")]
    pub fn record_memory_usage() {
        // Could record memory usage metrics
        if let Ok(usage) = sys_info::mem_info() {
            // TODO: Check actual MemInfo fields
            tracing::debug!("Memory info retrieved: {:?}", usage);
        }
    }

    #[cfg(feature = "sys-info")]
    pub fn record_cpu_usage() {
        // Could record CPU usage metrics
        if let Ok(load) = sys_info::loadavg() {
            tracing::debug!("CPU load: {:.2}", load.one);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_metrics_creation() {
        let metrics = RequestMetrics::new("GET", "/api/users");
        assert_eq!(metrics.method, "GET");
        assert_eq!(metrics.path, "/api/users");
        assert_eq!(metrics.status_code, 0);
    }

    #[test]
    fn test_request_metrics_success() {
        let metrics = RequestMetrics::new("GET", "/api/users");
        let completed = metrics.record_success();
        assert_eq!(completed.status_code, 200);
        assert!(completed.duration_ms >= 0.0);
    }

    #[test]
    fn test_request_metrics_error() {
        let metrics = RequestMetrics::new("POST", "/api/users");
        let completed = metrics.record_error(400);
        assert_eq!(completed.status_code, 400);
        assert!(completed.duration_ms >= 0.0);
    }

    #[test]
    fn test_metrics_recorder_creation() {
        let registry = Arc::new(Mutex::new(crate::metrics::MetricsRegistry::new()));
        let recorder = MetricsRecorder::new(registry);
        // Test passes if no panic occurs
    }

    #[test]
    fn test_business_metric_increment() {
        let registry = Arc::new(Mutex::new(crate::metrics::MetricsRegistry::new()));
        let recorder = MetricsRecorder::new(registry);

        // This should not panic even if metric doesn't exist
        recorder.increment_business_metric("nonexistent_metric");
        recorder.increment_business_metric_by("nonexistent_metric", 5.0);
    }

    #[test]
    fn test_database_metrics() {
        DatabaseMetrics::set_pool_size(10.0);
        DatabaseMetrics::set_pool_active(5.0);
        DatabaseMetrics::record_query_duration(0.1);
        DatabaseMetrics::increment_error_count();
        // Test passes if no panic occurs
    }

    #[test]
    fn test_health_metrics() {
        HealthMetrics::record_health_check("ok", 50.0);
        HealthMetrics::record_service_status("api", "healthy");
        // Test passes if no panic occurs
    }

    #[test]
    fn test_performance_metrics() {
        PerformanceMetrics::record_operation_time("database_query", 25.0);
        // Test passes if no panic occurs
    }
}
