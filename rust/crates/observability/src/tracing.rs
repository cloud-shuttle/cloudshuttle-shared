//! Distributed tracing utilities
//!
//! This module provides distributed tracing functionality,
//! span management, and trace context propagation.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Trace ID for distributed tracing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TraceId(pub String);

impl TraceId {
    /// Generate a new random trace ID
    pub fn generate() -> Self {
        use rand::{RngCore, rngs::OsRng};
        let mut bytes = [0u8; 16];
        OsRng.fill_bytes(&mut bytes);
        Self(hex::encode(bytes))
    }

    /// Create from string
    pub fn from_string(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Get as string reference
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for TraceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Span ID for distributed tracing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SpanId(pub String);

impl SpanId {
    /// Generate a new random span ID
    pub fn generate() -> Self {
        use rand::{RngCore, rngs::OsRng};
        let mut bytes = [0u8; 8];
        OsRng.fill_bytes(&mut bytes);
        Self(hex::encode(bytes))
    }

    /// Create from string
    pub fn from_string(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Get as string reference
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SpanId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Trace span builder
#[derive(Debug, Clone)]
pub struct SpanBuilder {
    name: String,
    trace_id: Option<TraceId>,
    parent_span_id: Option<SpanId>,
    attributes: HashMap<String, String>,
}

impl SpanBuilder {
    /// Create a new span builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            trace_id: None,
            parent_span_id: None,
            attributes: HashMap::new(),
        }
    }

    /// Set trace ID
    pub fn with_trace_id(mut self, trace_id: TraceId) -> Self {
        self.trace_id = Some(trace_id);
        self
    }

    /// Set parent span ID
    pub fn with_parent_span_id(mut self, span_id: SpanId) -> Self {
        self.parent_span_id = Some(span_id);
        self
    }

    /// Add an attribute
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    /// Build the span
    pub fn build(self) -> Span {
        Span {
            id: SpanId::generate(),
            name: self.name,
            trace_id: self.trace_id.unwrap_or_else(TraceId::generate),
            parent_span_id: self.parent_span_id,
            attributes: self.attributes,
            start_time: std::time::Instant::now(),
            end_time: None,
        }
    }
}

/// Trace span
#[derive(Debug, Clone)]
pub struct Span {
    id: SpanId,
    name: String,
    trace_id: TraceId,
    parent_span_id: Option<SpanId>,
    attributes: HashMap<String, String>,
    start_time: std::time::Instant,
    end_time: Option<std::time::Instant>,
}

impl Span {
    /// Get span ID
    pub fn id(&self) -> &SpanId {
        &self.id
    }

    /// Get trace ID
    pub fn trace_id(&self) -> &TraceId {
        &self.trace_id
    }

    /// Get span name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get parent span ID
    pub fn parent_span_id(&self) -> Option<&SpanId> {
        self.parent_span_id.as_ref()
    }

    /// Add an attribute
    pub fn set_attribute(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.attributes.insert(key.into(), value.into());
    }

    /// Get an attribute
    pub fn get_attribute(&self, key: &str) -> Option<&String> {
        self.attributes.get(key)
    }

    /// End the span
    pub fn end(&mut self) {
        self.end_time = Some(std::time::Instant::now());
    }

    /// Get span duration
    pub fn duration(&self) -> Option<std::time::Duration> {
        self.end_time.map(|end| end.duration_since(self.start_time))
    }
}

/// Tracing middleware for HTTP requests
#[derive(Debug, Clone)]
pub struct TracingMiddleware {
    service_name: String,
}

impl TracingMiddleware {
    /// Create a new tracing middleware
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
        }
    }

    /// Get service name
    pub fn service_name(&self) -> &str {
        &self.service_name
    }
}

/// Trace context for propagating trace information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceContext {
    pub trace_id: TraceId,
    pub span_id: SpanId,
    pub parent_span_id: Option<SpanId>,
}

impl TraceContext {
    /// Create a new root trace context
    pub fn new() -> Self {
        Self {
            trace_id: TraceId::generate(),
            span_id: SpanId::generate(),
            parent_span_id: None,
        }
    }

    /// Create a child context
    pub fn child(&self) -> Self {
        Self {
            trace_id: self.trace_id.clone(),
            span_id: SpanId::generate(),
            parent_span_id: Some(self.span_id.clone()),
        }
    }

    /// Convert to HTTP headers
    pub fn to_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("x-trace-id".to_string(), self.trace_id.0.clone());
        headers.insert("x-span-id".to_string(), self.span_id.0.clone());
        if let Some(parent_span_id) = &self.parent_span_id {
            headers.insert("x-parent-span-id".to_string(), parent_span_id.0.clone());
        }
        headers
    }

    /// Create from HTTP headers
    pub fn from_headers(headers: &HashMap<String, String>) -> Option<Self> {
        let trace_id = TraceId(headers.get("x-trace-id")?.clone());
        let span_id = SpanId(headers.get("x-span-id")?.clone());
        let parent_span_id = headers.get("x-parent-span-id").cloned().map(SpanId);

        Some(Self {
            trace_id,
            span_id,
            parent_span_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_id_generation() {
        let id1 = TraceId::generate();
        let id2 = TraceId::generate();
        assert_ne!(id1, id2);
        assert!(!id1.0.is_empty());
    }

    #[test]
    fn test_span_id_generation() {
        let id1 = SpanId::generate();
        let id2 = SpanId::generate();
        assert_ne!(id1, id2);
        assert!(!id1.0.is_empty());
    }

    #[test]
    fn test_span_builder() {
        let span = SpanBuilder::new("test-operation")
            .with_attribute("key", "value")
            .build();

        assert_eq!(span.name(), "test-operation");
        assert_eq!(span.get_attribute("key"), Some(&"value".to_string()));
        assert!(span.duration().is_none());
    }

    #[test]
    fn test_trace_context() {
        let context = TraceContext::new();
        let child_context = context.child();

        assert_eq!(context.trace_id, child_context.trace_id);
        assert_ne!(context.span_id, child_context.span_id);
        assert_eq!(child_context.parent_span_id, Some(context.span_id));
    }

    #[test]
    fn test_trace_context_headers() {
        let context = TraceContext::new();
        let headers = context.to_headers();

        assert!(headers.contains_key("x-trace-id"));
        assert!(headers.contains_key("x-span-id"));
        assert!(!headers.contains_key("x-parent-span-id"));

        let restored = TraceContext::from_headers(&headers).unwrap();
        assert_eq!(context.trace_id, restored.trace_id);
        assert_eq!(context.span_id, restored.span_id);
    }
}
