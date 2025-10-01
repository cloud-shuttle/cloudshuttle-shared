# 📊 Metrics Module Breakdown Plan

**File**: `rust/crates/observability/src/metrics.rs` (323 lines)
**Target**: 3 focused modules under 300 lines each
**Status**: 🚨 CRITICAL - Immediate breakdown required

## 📊 Current Analysis

### **Single Responsibility Violations**
The `metrics.rs` file currently handles:
1. **Metrics Collection** (counter, gauge, histogram creation)
2. **Metrics Recording** (HTTP metrics, error metrics, custom metrics)
3. **Metrics Exposition** (Prometheus format output)
4. **Metrics Middleware** (HTTP request/response tracking)
5. **Metrics Storage** (in-memory storage and retrieval)
6. **Metrics Configuration** (registry setup and management)

### **Dependencies & Complexity**
- **External Crates**: `prometheus`, `lazy_static`, `serde`
- **Internal Dependencies**: HTTP request types, error types
- **Complexity**: Multiple concerns mixed in single file

## 🏗️ Breakdown Architecture

### **Module 1: `metrics_collection.rs` (120 lines)**
**Responsibility**: Metrics creation, storage, and basic operations

```rust
//! Metrics collection and storage utilities

use prometheus::{Encoder, TextEncoder, Registry, Counter, Gauge, Histogram, HistogramOpts, Opts};
use std::collections::HashMap;

/// Metrics collection registry
pub struct MetricsRegistry {
    registry: Registry,
    counters: HashMap<String, Counter>,
    gauges: HashMap<String, Gauge>,
    histograms: HashMap<String, Histogram>,
}

impl MetricsRegistry {
    /// Create new metrics registry
    pub fn new() -> Self {
        // Implementation
    }

    /// Create counter metric
    pub fn create_counter(&mut self, name: &str, help: &str) -> prometheus::Result<Counter> {
        // Implementation
    }

    /// Create gauge metric
    pub fn create_gauge(&mut self, name: &str, help: &str) -> prometheus::Result<Gauge> {
        // Implementation
    }

    /// Create histogram metric
    pub fn create_histogram(&mut self, name: &str, help: &str, buckets: Vec<f64>) -> prometheus::Result<Histogram> {
        // Implementation
    }

    /// Get counter by name
    pub fn get_counter(&self, name: &str) -> Option<&Counter> {
        // Implementation
    }

    /// Get gauge by name
    pub fn get_gauge(&self, name: &str) -> Option<&Gauge> {
        // Implementation
    }

    /// Get histogram by name
    pub fn get_histogram(&self, name: &str) -> Option<&Histogram> {
        // Implementation
    }
}
```

### **Module 2: `metrics_recording.rs` (120 lines)**
**Responsibility**: Metrics value recording and updates

```rust
//! Metrics recording and value updates

use prometheus::{Counter, Gauge, Histogram};
use std::time::{Duration, Instant};

/// HTTP request metrics
#[derive(Debug, Clone)]
pub struct RequestMetrics {
    pub method: String,
    pub path: String,
    pub status_code: u16,
    pub duration_ms: f64,
    pub start_time: Instant,
}

/// Error metrics tracking
#[derive(Debug, Clone)]
pub struct ErrorMetrics {
    pub error_type: String,
    pub count: u64,
    pub last_occurrence: Instant,
}

/// Metrics recorder for updating metric values
pub struct MetricsRecorder {
    registry: std::sync::Arc<crate::metrics_collection::MetricsRegistry>,
}

impl MetricsRecorder {
    /// Record HTTP request metrics
    pub fn record_request(&self, metrics: RequestMetrics) {
        // Implementation
    }

    /// Record error occurrence
    pub fn record_error(&self, error_type: &str) {
        // Implementation
    }

    /// Record custom counter increment
    pub fn increment_counter(&self, name: &str, value: f64) {
        // Implementation
    }

    /// Update gauge value
    pub fn set_gauge(&self, name: &str, value: f64) {
        // Implementation
    }

    /// Record histogram observation
    pub fn observe_histogram(&self, name: &str, value: f64) {
        // Implementation
    }

    /// Record business metrics
    pub fn record_business_metric(&self, name: &str, value: f64, labels: HashMap<String, String>) {
        // Implementation
    }
}
```

### **Module 3: `metrics_middleware.rs` (80 lines)**
**Responsibility**: HTTP middleware for automatic metrics collection

```rust
//! HTTP middleware for automatic metrics collection

use std::time::Instant;
use std::collections::HashMap;

/// HTTP metrics middleware
pub struct MetricsMiddleware {
    recorder: crate::metrics_recording::MetricsRecorder,
}

impl MetricsMiddleware {
    /// Create new metrics middleware
    pub fn new(recorder: crate::metrics_recording::MetricsRecorder) -> Self {
        // Implementation
    }

    /// Wrap HTTP request with metrics collection
    pub async fn handle_request<F, Fut>(
        &self,
        method: &str,
        path: &str,
        handler: F,
    ) -> Result<hyper::Response<hyper::Body>, crate::Error>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<hyper::Response<hyper::Body>, crate::Error>>,
    {
        let start = Instant::now();

        let result = handler().await;

        let duration_ms = start.elapsed().as_millis() as f64;
        let status_code = match &result {
            Ok(response) => response.status().as_u16(),
            Err(_) => 500,
        };

        let request_metrics = crate::metrics_recording::RequestMetrics {
            method: method.to_string(),
            path: path.to_string(),
            status_code,
            duration_ms,
            start_time: start,
        };

        self.recorder.record_request(request_metrics);

        result
    }

    /// Record custom middleware metrics
    pub fn record_custom_metric(&self, name: &str, value: f64, labels: HashMap<String, String>) {
        // Implementation
    }
}
```

## 🔄 Refactoring Steps

### **Step 1: Extract Metrics Collection**
```bash
# Move collection code to metrics_collection.rs
grep -A 50 "create_counter\|create_gauge" metrics.rs > metrics_collection.rs
# Add MetricsRegistry struct and basic operations
```

### **Step 2: Extract Metrics Recording**
```bash
# Move recording code to metrics_recording.rs
grep -A 50 "record_request\|record_error" metrics.rs > metrics_recording.rs
# Add MetricsRecorder struct and value update operations
```

### **Step 3: Extract Metrics Middleware**
```bash
# Move middleware code to metrics_middleware.rs
grep -A 30 "handle_request" metrics.rs > metrics_middleware.rs
# Add HTTP middleware for automatic collection
```

### **Step 4: Update Main Metrics Module**
```rust
//! Metrics collection and recording - orchestrates all metrics operations

pub mod collection;
pub mod recording;
pub mod middleware;

pub use collection::MetricsRegistry;
pub use recording::{MetricsRecorder, RequestMetrics, ErrorMetrics};
pub use middleware::MetricsMiddleware;

/// Legacy metrics collector for backward compatibility
pub struct MetricsCollector {
    registry: std::sync::Arc<MetricsRegistry>,
    recorder: MetricsRecorder,
}

impl MetricsCollector {
    /// Create new metrics collector
    pub fn new() -> Self {
        let registry = std::sync::Arc::new(MetricsRegistry::new());
        let recorder = MetricsRecorder::new(registry.clone());
        Self { registry, recorder }
    }

    /// Get metrics in Prometheus format
    pub fn gather(&self) -> String {
        // Implementation
    }

    /// Record HTTP request (backward compatibility)
    pub fn record_request(&self, method: &str, path: &str, status: u16, duration: f64) {
        let metrics = RequestMetrics {
            method: method.to_string(),
            path: path.to_string(),
            status_code: status,
            duration_ms: duration,
            start_time: std::time::Instant::now(),
        };
        self.recorder.record_request(metrics);
    }
}
```

## 🧪 Testing Strategy

### **Unit Tests per Module**
- **metrics_collection.rs**: 12 test cases (registry operations, metric creation)
- **metrics_recording.rs**: 15 test cases (value updates, error recording)
- **metrics_middleware.rs**: 8 test cases (HTTP wrapping, custom metrics)

### **Integration Tests**
- **Full metrics pipeline**: Collection → Recording → Exposition
- **Middleware integration**: HTTP request tracking
- **Prometheus compatibility**: Exposition format validation

### **Performance Tests**
- **Concurrent updates**: Multi-threaded metrics recording
- **Memory usage**: Metrics storage efficiency
- **Export performance**: Prometheus format generation

## 📋 Implementation Checklist

- [ ] Create `metrics_collection.rs` (120 lines max)
- [ ] Create `metrics_recording.rs` (120 lines max)
- [ ] Create `metrics_middleware.rs` (80 lines max)
- [ ] Update `metrics.rs` for backward compatibility (50 lines max)
- [ ] Update `mod.rs` to expose new modules
- [ ] Update all import statements across codebase
- [ ] Run compilation tests
- [ ] Run metrics test suites
- [ ] Update documentation

## 🎯 Success Metrics

- ✅ **File sizes**: All modules under specified limits
- ✅ **Compilation**: Zero errors after refactoring
- ✅ **API compatibility**: Existing metrics usage unchanged
- ✅ **Test coverage**: 95%+ coverage maintained
- ✅ **Performance**: No regression in metrics operations

## 🚨 Risk Mitigation

- **Data Loss**: Ensure metrics persistence during transition
- **Performance**: Benchmark metrics operations before/after
- **Compatibility**: Maintain Prometheus exposition format
- **Concurrency**: Test multi-threaded metrics recording
