//! Metrics collection and registry management

use prometheus::{register_counter, register_histogram, register_gauge, Counter, Histogram, Gauge, Encoder, TextEncoder};
use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Global Prometheus registry
pub static PROMETHEUS_REGISTRY: Lazy<prometheus::Registry> = Lazy::new(prometheus::Registry::new);

/// HTTP request counter
pub static HTTP_REQUEST_COUNT: Lazy<Counter> = Lazy::new(|| {
    let counter = Counter::new("http_requests_total", "Total number of HTTP requests")
        .expect("can not create counter http_requests_total");
    PROMETHEUS_REGISTRY.register(Box::new(counter.clone()))
        .expect("can not register counter http_requests_total");
    counter
});

/// HTTP request duration histogram
pub static HTTP_REQUEST_DURATION_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    let histogram = Histogram::with_opts(prometheus::HistogramOpts::new("http_request_duration_seconds", "HTTP request duration in seconds"))
        .expect("can not create histogram http_request_duration_seconds");
    PROMETHEUS_REGISTRY.register(Box::new(histogram.clone()))
        .expect("can not register histogram http_request_duration_seconds");
    histogram
});

/// Active connections gauge
pub static ACTIVE_CONNECTIONS: Lazy<Gauge> = Lazy::new(|| {
    let gauge = Gauge::new("active_connections", "Number of active connections")
        .expect("can not create gauge active_connections");
    PROMETHEUS_REGISTRY.register(Box::new(gauge.clone()))
        .expect("can not register gauge active_connections");
    gauge
});

/// Database connection pool size
pub static DB_POOL_SIZE: Lazy<Gauge> = Lazy::new(|| {
    let gauge = Gauge::new("db_pool_size", "Database connection pool size")
        .expect("can not create gauge db_pool_size");
    PROMETHEUS_REGISTRY.register(Box::new(gauge.clone()))
        .expect("can not register gauge db_pool_size");
    gauge
});

/// Database connection pool active connections
pub static DB_POOL_ACTIVE: Lazy<Gauge> = Lazy::new(|| {
    let gauge = Gauge::new("db_pool_active", "Database connection pool active connections")
        .expect("can not create gauge db_pool_active");
    PROMETHEUS_REGISTRY.register(Box::new(gauge.clone()))
        .expect("can not register gauge db_pool_active");
    gauge
});

/// Error counter by type
pub static ERROR_COUNT: Lazy<Counter> = Lazy::new(|| {
    let counter = Counter::new("errors_total", "Total number of errors")
        .expect("can not create counter errors_total");
    PROMETHEUS_REGISTRY.register(Box::new(counter.clone()))
        .expect("can not register counter errors_total");
    counter
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
        if let Ok(counter) = Counter::new(name, &format!("Business metric: {}", name)) {
            if PROMETHEUS_REGISTRY.register(Box::new(counter.clone())).is_ok() {
                metrics.insert(name.to_string(), counter);
            }
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
pub struct MetricsRegistry {
    custom_metrics: HashMap<String, Box<dyn prometheus::core::Collector>>,
}

impl MetricsRegistry {
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

    /// Get metrics in Prometheus format
    pub fn gather_metrics(&self) -> Result<String, prometheus::Error> {
        let encoder = TextEncoder::new();
        let metric_families = PROMETHEUS_REGISTRY.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        String::from_utf8(buffer).map_err(|e| prometheus::Error::Msg(e.to_string()))
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Metric builder for creating custom metrics
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
        Counter::with_opts(opts)
    }

    pub fn build_gauge(self) -> Result<Gauge, prometheus::Error> {
        let mut opts = prometheus::Opts::new(self.name, self.help);
        if !self.labels.is_empty() {
            opts = opts.variable_labels(self.labels);
        }
        Gauge::with_opts(opts)
    }

    pub fn build_histogram(self) -> Result<Histogram, prometheus::Error> {
        let opts = prometheus::HistogramOpts::new(self.name, self.help);
        Histogram::with_opts(opts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_registration() {
        register_metrics();
        // Test that metrics are registered without panicking
    }

    #[test]
    fn test_metrics_registry_creation() {
        let registry = MetricsRegistry::new();
        assert!(registry.custom_metrics.is_empty());
    }

    #[test]
    fn test_metric_builder() {
        let builder = MetricBuilder::new("test_counter", "A test counter");
        let counter = builder.build_counter();
        assert!(counter.is_ok());
    }

    #[test]
    fn test_business_metrics_initialization() {
        let metrics = Lazy::force(&BUSINESS_METRICS);
        assert!(!metrics.is_empty());
        assert!(metrics.contains_key("users_created_total"));
    }

    #[test]
    fn test_gather_metrics() {
        let registry = MetricsRegistry::new();
        // Initialize the standard metrics
        register_metrics();
        let result = registry.gather_metrics();
        assert!(result.is_ok());
        let metrics_output = result.unwrap();
        assert!(!metrics_output.is_empty());
        // Should contain Prometheus format headers
        assert!(metrics_output.contains("# HELP"));
    }
}
