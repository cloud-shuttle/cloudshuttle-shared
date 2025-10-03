# API Response Formatting Design Document

## Overview

This document details the design and implementation of standardized API response formatting in the CloudShuttle Gateway, ensuring consistent response structures across all API endpoints.

## Current Implementation

### Response Structure

```rust
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
```

### Success Response Format

```json
{
  "success": true,
  "data": {
    "message": "Operation completed successfully",
    "result": { /* actual data */ }
  },
  "error": null,
  "timestamp": "2024-01-01T12:00:00.000Z"
}
```

### Error Response Format

```json
{
  "success": false,
  "data": null,
  "error": "Detailed error message",
  "timestamp": "2024-01-01T12:00:00.000Z"
}
```

### Implementation Code

```rust
impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            timestamp: chrono::Utc::now(),
        }
    }
}
```

## Usage Examples

### Health Check Endpoint

```rust
pub async fn health_check() -> Json<ApiResponse<HealthStatus>> {
    let status = HealthStatus {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now(),
        uptime_seconds: 3600,
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    Json(ApiResponse::success(status))
}
```

### Error Handling

```rust
pub async fn get_upstream(id: &str) -> Result<Json<ApiResponse<UpstreamInfo>>, StatusCode> {
    match find_upstream(id).await {
        Ok(upstream) => Ok(Json(ApiResponse::success(upstream))),
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error(format!("Upstream not found: {}", e)))
        )),
    }
}
```

## Future Enhancements

### Advanced Response Builder

When cloudshuttle-api provides `ApiResponseBuilder`:

```rust
// Future implementation
pub fn create_enhanced_response<T: Serialize>(
    &self,
    result: Result<T, ProxyError>
) -> Json<ApiResponse<T>> {
    let mut builder = ApiResponseBuilder::new();

    match result {
        Ok(data) => {
            builder.success(data)
                .with_request_id(self.request_id())
                .with_processing_time(self.processing_time())
                .with_metadata(self.generate_metadata())
        }
        Err(error) => {
            builder.error(error.to_string())
                .with_error_code(error.code())
                .with_request_id(self.request_id())
                .with_debug_info(self.debug_info())
        }
    }

    Json(builder.build())
}
```

### Enhanced Response Features

```rust
#[derive(Debug, Serialize)]
pub struct EnhancedApiResponse<T> {
    // Standard fields
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,

    // Enhanced fields (future)
    pub request_id: Option<String>,
    pub processing_time_ms: Option<u64>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub links: Option<HashMap<String, String>>,
    pub debug_info: Option<DebugInfo>,
}
```

### Request Tracing Integration

```rust
impl SharedApiService {
    pub fn set_request_context(&mut self, request_id: String, start_time: Instant) {
        self.request_id = Some(request_id);
        self.request_start = Some(start_time);
    }

    pub fn get_processing_time(&self) -> Option<u64> {
        self.request_start
            .map(|start| start.elapsed().as_millis() as u64)
    }
}
```

## Benefits

### Consistency
- All API endpoints return the same response structure
- Predictable response format for API consumers
- Standardized error handling across the application

### Debugging
- Timestamps for all responses
- Request IDs for tracing
- Processing time metrics
- Debug information when enabled

### API Evolution
- Backward compatible response structure
- Extensible metadata fields
- Version information included
- Link support for HATEOAS

## Migration Strategy

### Phase 1: Standardization
- ✅ Implement basic `ApiResponse<T>` structure
- ✅ Update all endpoints to use standardized responses
- ✅ Add timestamp to all responses

### Phase 2: Enhancement
- 🔄 Add request ID tracking
- 🔄 Include processing time metrics
- 🔄 Add metadata support
- 🔄 Implement debug information

### Phase 3: Advanced Features
- 🔄 HATEOAS link support
- 🔄 Response caching headers
- 🔄 Conditional response support
- 🔄 Bulk operation responses

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_response() {
        let response = ApiResponse::success("test data");
        assert!(response.success);
        assert_eq!(response.data, Some("test data"));
        assert!(response.error.is_none());
        assert!(response.timestamp <= chrono::Utc::now());
    }

    #[test]
    fn test_error_response() {
        let response: ApiResponse<()> = ApiResponse::error("test error");
        assert!(!response.success);
        assert!(response.data.is_none());
        assert_eq!(response.error, Some("test error"));
    }

    #[test]
    fn test_response_serialization() {
        let response = ApiResponse::success(vec![1, 2, 3]);
        let json = serde_json::to_string(&response).unwrap();

        // Verify JSON structure
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"data\":[1,2,3]"));
        assert!(json.contains("\"timestamp\""));
    }
}
```

## Performance Considerations

### Serialization Overhead
- Use efficient JSON serialization
- Consider binary formats for internal APIs
- Cache frequently used response templates

### Memory Usage
- Avoid large response payloads
- Implement pagination for list endpoints
- Stream large responses when possible

### Monitoring
- Track response time percentiles
- Monitor error rates by endpoint
- Alert on unusual response patterns

## Conclusion

The standardized response formatting provides a solid foundation for CloudShuttle Gateway's API, ensuring consistency, debuggability, and future extensibility. The design allows for gradual enhancement while maintaining backward compatibility.
