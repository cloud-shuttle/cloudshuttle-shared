//! HTTP middleware for automatic metrics collection

use std::time::Instant;
use std::sync::{Arc, Mutex};

/// HTTP metrics middleware for automatic request/response tracking
pub struct MetricsMiddleware {
    recorder: super::MetricsRecorder,
}

impl MetricsMiddleware {
    /// Create new metrics middleware
    pub fn new(recorder: super::MetricsRecorder) -> Self {
        Self { recorder }
    }

    /// Wrap an async operation with metrics collection
    pub async fn wrap_async_operation<F, Fut, T, E>(
        &self,
        operation_name: &str,
        operation: F,
    ) -> Result<T, E>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
    {
        let start = Instant::now();
        let result = operation().await;
        let duration_ms = start.elapsed().as_millis() as f64;

        match &result {
            Ok(_) => {
                self.recorder.record_performance_metric(operation_name, duration_ms);
            }
            Err(_) => {
                self.recorder.record_error(&format!("{} failed", operation_name));
                self.recorder.record_performance_metric(operation_name, duration_ms);
            }
        }

        result
    }

    /// Track database operations
    pub async fn track_database_operation<F, Fut, T, E>(
        &self,
        operation_name: &str,
        operation: F,
    ) -> Result<T, E>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
    {
        let start = Instant::now();
        let result = operation().await;
        let duration_seconds = start.elapsed().as_secs_f64();

        match &result {
            Ok(_) => {
                self.recorder.record_database_metric("query_duration", duration_seconds);
            }
            Err(_) => {
                self.recorder.record_database_metric("query_duration", duration_seconds);
                self.recorder.record_error(&format!("database_{}", operation_name));
            }
        }

        result
    }

    /// Create request metrics for HTTP requests
    pub fn create_request_metrics(&self, method: &str, path: &str) -> super::RequestMetrics {
        super::RequestMetrics::new(method, path)
    }

    /// Record custom middleware metrics
    pub fn record_custom_metric(&self, name: &str, value: f64, labels: std::collections::HashMap<String, String>) {
        self.recorder.record_business_metric(name, value, labels);
    }

    /// Record health check with middleware
    pub fn record_health_check(&self, service: &str, status: &str, duration_ms: f64) {
        self.recorder.record_health_metric(service, status, duration_ms);

        // Additional middleware-specific logic
        if status != "healthy" && status != "ok" {
            self.recorder.record_alert("health_check_failed", service, "warn", serde_json::json!({
                "status": status,
                "duration_ms": duration_ms
            }));
        }
    }

    /// Record service alerts through middleware
    pub fn record_service_alert(&self, alert_type: &str, service: &str, severity: &str, details: serde_json::Value) {
        self.recorder.record_alert(alert_type, service, severity, details);
    }

    /// Middleware for Tower service (if using tower framework)
    #[cfg(feature = "tower")]
    pub fn tower_layer(&self) -> tower::Layer<MetricsMiddlewareLayer> {
        tower::Layer::new(MetricsMiddlewareLayer {
            recorder: self.recorder.clone(),
        })
    }
}

/// Tower service layer for metrics collection
#[cfg(feature = "tower")]
pub struct MetricsMiddlewareLayer {
    recorder: super::MetricsRecorder,
}

#[cfg(feature = "tower")]
impl<S> tower::Layer<S> for MetricsMiddlewareLayer {
    type Service = MetricsMiddlewareService<S>;

    fn layer(&self, service: S) -> Self::Service {
        MetricsMiddlewareService {
            service,
            recorder: self.recorder.clone(),
        }
    }
}

/// Tower service wrapper for metrics collection
#[cfg(feature = "tower")]
pub struct MetricsMiddlewareService<S> {
    service: S,
    recorder: super::MetricsRecorder,
}

#[cfg(feature = "tower")]
impl<S, Request> tower::Service<Request> for MetricsMiddlewareService<S>
where
    S: tower::Service<Request>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = MetricsFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let start = Instant::now();
        let future = self.service.call(request);

        MetricsFuture {
            future,
            start,
            recorder: self.recorder.clone(),
        }
    }
}

/// Future wrapper for metrics collection
#[cfg(feature = "tower")]
pub struct MetricsFuture<F> {
    future: F,
    start: Instant,
    recorder: super::MetricsRecorder,
}

#[cfg(feature = "tower")]
impl<F> std::future::Future for MetricsFuture<F>
where
    F: std::future::Future,
{
    type Output = F::Output;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        let result = unsafe { std::pin::Pin::new_unchecked(&mut this.future) }.poll(cx);

        if result.is_ready() {
            let duration_ms = this.start.elapsed().as_millis() as f64;
            this.recorder.record_performance_metric("http_request", duration_ms);
        }

        result
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

    pub fn alert_resource_exhaustion(resource: &str, usage_percent: f64) {
        tracing::error!("Resource exhaustion alert for {}: {}% usage", resource, usage_percent);
        // In a real implementation, this would send critical alerts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_metrics_middleware_creation() {
        let registry = Arc::new(Mutex::new(crate::metrics::MetricsRegistry::new()));
        let recorder = crate::metrics::MetricsRecorder::new(registry);
        let middleware = MetricsMiddleware::new(recorder);
        // Test passes if no panic occurs
    }

    #[test]
    fn test_request_metrics_creation() {
        let registry = Arc::new(Mutex::new(crate::metrics::MetricsRegistry::new()));
        let recorder = crate::metrics::MetricsRecorder::new(registry);
        let middleware = MetricsMiddleware::new(recorder);

        let request_metrics = middleware.create_request_metrics("GET", "/api/users");
        assert_eq!(request_metrics.method, "GET");
        assert_eq!(request_metrics.path, "/api/users");
    }

    #[test]
    fn test_custom_metric_recording() {
        let registry = Arc::new(Mutex::new(crate::metrics::MetricsRegistry::new()));
        let recorder = crate::metrics::MetricsRecorder::new(registry);
        let middleware = MetricsMiddleware::new(recorder);

        let mut labels = std::collections::HashMap::new();
        labels.insert("endpoint".to_string(), "/api/users".to_string());

        middleware.record_custom_metric("api_calls", 1.0, labels);
        // Test passes if no panic occurs
    }

    #[test]
    fn test_health_check_recording() {
        let registry = Arc::new(Mutex::new(crate::metrics::MetricsRegistry::new()));
        let recorder = crate::metrics::MetricsRecorder::new(registry);
        let middleware = MetricsMiddleware::new(recorder);

        middleware.record_health_check("api", "healthy", 50.0);
        middleware.record_health_check("database", "unhealthy", 200.0);
        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_async_operation_wrapping() {
        let registry = Arc::new(Mutex::new(crate::metrics::MetricsRegistry::new()));
        let recorder = crate::metrics::MetricsRecorder::new(registry);
        let middleware = MetricsMiddleware::new(recorder);

        let result: Result<i32, &str> = middleware
            .wrap_async_operation("test_operation", || async { Ok(42) })
            .await;

        assert_eq!(result, Ok(42));
    }

    #[tokio::test]
    async fn test_database_operation_tracking() {
        let registry = Arc::new(Mutex::new(crate::metrics::MetricsRegistry::new()));
        let recorder = crate::metrics::MetricsRecorder::new(registry);
        let middleware = MetricsMiddleware::new(recorder);

        let result: Result<String, &str> = middleware
            .track_database_operation("select", || async { Ok("data".to_string()) })
            .await;

        assert_eq!(result, Ok("data".to_string()));
    }

    #[test]
    fn test_alert_manager() {
        AlertManager::alert_high_error_rate("api", 10.0);
        AlertManager::alert_service_down("database");
        AlertManager::alert_high_latency("cache", 5000.0);
        AlertManager::alert_resource_exhaustion("memory", 95.0);
        // Test passes if no panic occurs
    }
}
