//! Prometheus metrics collection and management

use prometheus::{register_counter, register_histogram, register_gauge, Counter, Histogram, Gauge, Encoder, TextEncoder};
use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Global Prometheus registry
pub static PROMETHEUS_REGISTRY: Lazy<prometheus::Registry> = Lazy::new(prometheus::Registry::new);

/// HTTP request counter
pub static HTTP_REQUEST_COUNT: Lazy<Counter> = Lazy::new(|| {
    register_counter!("http_requests_total", "Total number of HTTP requests")
        .expect("can not create counter http_requests_total")
});

/// HTTP request duration histogram
pub static HTTP_REQUEST_DURATION_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!("http_request_duration_seconds", "HTTP request duration in seconds")
        .expect("can not create histogram http_request_duration_seconds")
});

/// Active connections gauge
pub static ACTIVE_CONNECTIONS: Lazy<Gauge> = Lazy::new(|| {
    register_gauge!("active_connections", "Number of active connections")
        .expect("can not create gauge active_connections")
});

/// Database connection pool size
pub static DB_POOL_SIZE: Lazy<Gauge> = Lazy::new(|| {
    register_gauge!("db_pool_size", "Database connection pool size")
        .expect("can not create gauge db_pool_size")
});

/// Database connection pool active connections
pub static DB_POOL_ACTIVE: Lazy<Gauge> = Lazy::new(|| {
    register_gauge!("db_pool_active", "Database connection pool active connections")
        .expect("can not create gauge db_pool_active")
});

/// Error counter by type
pub static ERROR_COUNT: Lazy<Counter> = Lazy::new(|| {
    register_counter!("errors_total", "Total number of errors")
        .expect("can not create counter errors_total")
});

/// Business metrics
pub static BUSINESS_METRICS: Lazy<HashMap<String, Counter>> = Lazy::new(|| {
    let mut metrics = HashMap::new();

    // Pre-register common business metrics
    let metric_names = vec![
        "users_created_total",
        "users_logged_in_total",
        "content_created_total",
        "content_published_total",
        "api_calls_total",
        "cache_hits_total",
        "cache_misses_total",
    ];

    for name in metric_names {
        if let Ok(counter) = register_counter!(name, &format!("Business metric: {}", name)) {
            metrics.insert(name.to_string(), counter);
        }
    }

    metrics
});

/// Register all standard metrics
pub fn register_metrics() {
    // Force initialization of all lazy statics
    Lazy::force(&HTTP_REQUEST_COUNT);
    Lazy::force(&HTTP_REQUEST_DURATION_SECONDS);
    Lazy::force(&ACTIVE_CONNECTIONS);
    Lazy::force(&DB_POOL_SIZE);
    Lazy::force(&DB_POOL_ACTIVE);
    Lazy::force(&ERROR_COUNT);
    Lazy::force(&BUSINESS_METRICS);

    tracing::info!("Standard metrics registered");
}

/// Metrics collector for gathering metrics data
pub struct MetricsCollector {
    custom_metrics: HashMap<String, Box<dyn prometheus::core::Collector>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            custom_metrics: HashMap::new(),
        }
    }

    /// Register a custom metric
    pub fn register_custom_metric<C: prometheus::core::Collector + 'static>(
        &mut self,
        name: String,
        metric: C,
    ) -> Result<(), prometheus::Error> {
        PROMETHEUS_REGISTRY.register(Box::new(metric))?;
        // Note: We can't store the metric directly due to trait object limitations
        Ok(())
    }

    /// Increment a business metric
    pub fn increment_business_metric(&self, name: &str) {
        if let Some(counter) = BUSINESS_METRICS.get(name) {
            counter.inc();
        } else {
            tracing::warn!("Business metric '{}' not registered", name);
        }
    }

    /// Increment business metric by value
    pub fn increment_business_metric_by(&self, name: &str, value: f64) {
        if let Some(counter) = BUSINESS_METRICS.get(name) {
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

    /// Observe histogram value
    pub fn observe_histogram(&self, histogram: &Histogram, value: f64) {
        histogram.observe(value);
    }

    /// Get metrics in Prometheus format
    pub fn gather_metrics(&self) -> Result<String, prometheus::Error> {
        let encoder = TextEncoder::new();
        let metric_families = PROMETHEUS_REGISTRY.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        String::from_utf8(buffer).map_err(|e| prometheus::Error::Msg(e.to_string()))
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Request metrics middleware helper
pub struct RequestMetrics {
    method: String,
    path: String,
    start_time: std::time::Instant,
}

impl RequestMetrics {
    pub fn new(method: impl Into<String>, path: impl Into<String>) -> Self {
        HTTP_REQUEST_COUNT.inc();
        ACTIVE_CONNECTIONS.inc();

        Self {
            method: method.into(),
            path: path.into(),
            start_time: std::time::Instant::now(),
        }
    }

    pub fn record_success(self) {
        let duration = self.start_time.elapsed().as_secs_f64();
        HTTP_REQUEST_DURATION_SECONDS.observe(duration);
        ACTIVE_CONNECTIONS.dec();
    }

    pub fn record_error(self, status_code: u16) {
        let duration = self.start_time.elapsed().as_secs_f64();
        HTTP_REQUEST_DURATION_SECONDS.observe(duration);
        ACTIVE_CONNECTIONS.dec();
        ERROR_COUNT.inc();
    }
}

impl Drop for RequestMetrics {
    fn drop(&mut self) {
        // Ensure we decrement active connections even if not explicitly called
        ACTIVE_CONNECTIONS.dec();
    }
}

/// Database metrics
pub struct DatabaseMetrics;

impl DatabaseMetrics {
    pub fn set_pool_size(size: f64) {
        DB_POOL_SIZE.set(size);
    }

    pub fn set_pool_active(active: f64) {
        DB_POOL_ACTIVE.set(active);
    }

    pub fn record_query_duration(duration_seconds: f64) {
        // Could add a histogram for query durations
        tracing::debug!("Query duration: {}s", duration_seconds);
    }

    pub fn increment_error_count() {
        ERROR_COUNT.inc();
    }
}

/// Custom metric builder
pub struct MetricBuilder {
    name: String,
    help: String,
    labels: Vec<String>,
}

impl MetricBuilder {
    pub fn new(name: impl Into<String>, help: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            help: help.into(),
            labels: Vec::new(),
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.labels.push(label.into());
        self
    }

    pub fn build_counter(self) -> Result<Counter, prometheus::Error> {
        let mut opts = prometheus::Opts::new(self.name, self.help);
        if !self.labels.is_empty() {
            // For labeled metrics, we'd need to create a CounterVec
            // This is a simplified implementation
            opts = opts.variable_labels(self.labels);
        }
        register_counter!(opts)
    }

    pub fn build_gauge(self) -> Result<Gauge, prometheus::Error> {
        let mut opts = prometheus::Opts::new(self.name, self.help);
        if !self.labels.is_empty() {
            opts = opts.variable_labels(self.labels);
        }
        register_gauge!(opts)
    }

    pub fn build_histogram(self) -> Result<Histogram, prometheus::Error> {
        let opts = prometheus::HistogramOpts::new(self.name, self.help);
        register_histogram!(opts)
    }
}

/// Health metrics
pub struct HealthMetrics;

impl HealthMetrics {
    pub fn record_health_check(status: &str, duration_ms: f64) {
        // Could add specific health check metrics
        tracing::debug!("Health check {} took {}ms", status, duration_ms);
    }

    pub fn record_service_status(service: &str, status: &str) {
        // Could add service status gauges
        tracing::info!("Service {} status: {}", service, status);
    }
}

/// Performance metrics
pub struct PerformanceMetrics;

impl PerformanceMetrics {
    pub fn record_operation_time(operation: &str, duration_ms: f64) {
        // Could add operation-specific histograms
        tracing::debug!("Operation {} took {}ms", operation, duration_ms);
    }

    pub fn record_memory_usage() {
        // Could record memory usage metrics
        if let Ok(usage) = sys_info::mem_info() {
            tracing::debug!("Memory usage: {} KB used, {} KB total", usage.used, usage.total);
        }
    }

    pub fn record_cpu_usage() {
        // Could record CPU usage metrics
        if let Ok(load) = sys_info::loadavg() {
            tracing::debug!("CPU load: {:.2}", load.one);
        }
    }
}

/// Alert manager integration
pub struct AlertManager;

impl AlertManager {
    pub fn alert_high_error_rate(service: &str, error_rate: f64) {
        tracing::error!("High error rate alert for {}: {} errors/second", service, error_rate);
        // In a real implementation, this would send alerts to monitoring systems
    }

    pub fn alert_service_down(service: &str) {
        tracing::error!("Service down alert: {}", service);
        // In a real implementation, this would send critical alerts
    }

    pub fn alert_high_latency(service: &str, latency_ms: f64) {
        tracing::warn!("High latency alert for {}: {}ms", service, latency_ms);
        // In a real implementation, this would send warning alerts
    }
}
