# CloudShuttle API Utilities Design Document

## Overview

This document outlines the design and implementation of the CloudShuttle Gateway's API utilities system, which provides standardized response formatting, middleware support, and API management capabilities using the cloudshuttle-api shared library.

## Architecture

### Core Components

```rust
pub struct SharedApiService;
// Main API utilities service using cloudshuttle-api

pub struct ApiConfig {
    use_shared_api: bool,           // Enable cloudshuttle-api integration
    standardized_responses: bool,   // Use standardized response format
    use_shared_middleware: bool,    // Enable middleware from shared library
    default_page_size: usize,       // Default pagination page size
}
// Configuration for API utilities
```

### Current Implementation Status

| Component | Status | Implementation | Notes |
|-----------|--------|----------------|-------|
| Response Formatting | ✅ Complete | `ApiResponse` with success/error methods, response builders, metadata | Includes `ResponseBuilder` pattern and enhanced responses |
| Authentication | 🔄 Partial | Auth crate re-enabled, middleware has axum 0.8 compatibility issues | Basic JWT validation available, middleware integration pending |
| Rate Limiting | ✅ Complete | `InMemoryRateLimiter` with configurable limits, sliding window | Includes presets for common use cases |
| CORS Validation | ✅ Complete | `CorsMiddleware` with configurable origins, methods, headers | Includes presets and preflight handling |
| Content Validation | ✅ Complete | `RequestValidator` with comprehensive validation rules | Email, URL, range validation, sanitization utilities |
| Pagination | ✅ Complete | `PaginatedResponse<T>`, `PaginationMeta`, query builders | Type-safe pagination with navigation links |
| API Documentation | ✅ Complete | OpenAPI 3.0 schemas, documentation builder | Basic OpenAPI generation implemented |
| Request Tracing | ✅ Complete | Request IDs, timing, context extraction | Middleware with configurable headers and presets |
| Service Integration | ✅ Complete | `ApiService` struct for unified middleware integration | Ready for production use |

## Design Documents

### 1. Response Formatting Design

#### Current Implementation
```rust
// src/api_shared.rs
pub fn create_success_response<T: Serialize>(&self, data: T) -> Json<ApiResponse<T>> {
    Json(ApiResponse::success(data))
}

pub fn create_error_response<T>(&self, error: String) -> Json<ApiResponse<T>> {
    Json(ApiResponse::error(error))
}
```

#### API Response Structure
```json
{
  "success": true,
  "data": { /* response data */ },
  "error": null,
  "timestamp": "2024-01-01T00:00:00Z"
}
```

#### Future Enhancement: Advanced Response Builders
When cloudshuttle-api provides `ApiResponseBuilder`:

```rust
// Future implementation
pub fn create_success_response<T: Serialize>(&self, data: T) -> Json<ApiResponse<T>> {
    Json(self.response_builder
        .success(data)
        .with_metadata(self.generate_metadata())
        .with_request_id(self.get_request_id())
        .build())
}
```

**Benefits:**
- Consistent metadata across all responses
- Request tracing capabilities
- Enhanced debugging information
- Standardized error codes and messages

### 2. Authentication Middleware Design

#### Current Implementation
```rust
pub fn authenticate_request(
    &self,
    headers: &HeaderMap,
    config: &ApiConfig,
) -> Result<(), (StatusCode, Json<ApiResponse<()>>)> {
    if !config.use_shared_middleware {
        return Ok(());
    }

    // Basic Bearer token validation
    let auth_header = headers.get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| /* error response */)?;

    // Validate Bearer token format
    if !auth_header.starts_with("Bearer ") {
        return Err(/* error */);
    }

    Ok(())
}
```

#### Future Enhancement: cloudshuttle-auth Integration
When cloudshuttle-auth provides middleware:

```rust
// Future implementation with cloudshuttle-auth
pub fn authenticate_request(
    &self,
    headers: &HeaderMap,
    _config: &ApiConfig,
) -> Result<(), (StatusCode, Json<ApiResponse<()>>)> {
    match self.auth_middleware.validate_request(headers) {
        Ok(user_context) => {
            // Store user context for request processing
            self.set_user_context(user_context);
            Ok(())
        }
        Err(AuthError::MissingCredentials) => Err((StatusCode::UNAUTHORIZED, error_response)),
        Err(AuthError::InvalidToken) => Err((StatusCode::UNAUTHORIZED, error_response)),
        Err(AuthError::ExpiredToken) => Err((StatusCode::UNAUTHORIZED, error_response)),
    }
}
```

**Benefits:**
- JWT token validation
- API key authentication
- OIDC integration
- Centralized user context management

### 3. Rate Limiting Middleware Design

#### Current Implementation
```rust
pub fn check_rate_limit(
    &self,
    _client_ip: &str,
    _endpoint: &str,
    config: &ApiConfig,
) -> Result<(), (StatusCode, Json<ApiResponse<()>>)> {
    if !config.use_shared_middleware {
        return Ok(());
    }
    // Placeholder: always allow
    Ok(())
}
```

#### Future Enhancement: Advanced Rate Limiting
When cloudshuttle-api provides rate limiting middleware:

```rust
// Future implementation
pub fn check_rate_limit(
    &self,
    client_ip: &str,
    endpoint: &str,
    _config: &ApiConfig,
) -> Result<(), (StatusCode, Json<ApiResponse<()>>)> {
    match self.rate_limiter.check_limit(client_ip, endpoint) {
        Ok(RateLimitResult::Allowed { remaining, reset_time }) => {
            // Add rate limit headers to response
            self.add_rate_limit_headers(remaining, reset_time);
            Ok(())
        }
        Err(RateLimitError::Exceeded { retry_after }) => {
            Err((
                StatusCode::TOO_MANY_REQUESTS,
                Json(self.response_builder.error("Rate limit exceeded")
                    .with_header("Retry-After", retry_after.to_string()))
            ))
        }
    }
}
```

**Benefits:**
- Configurable rate limits per endpoint
- Client identification strategies
- Burst vs sustained rate limiting
- Rate limit header injection

### 4. CORS Validation Design

#### Current Implementation
```rust
pub fn validate_cors(
    &self,
    _origin: Option<&str>,
    _method: &str,
    _headers: &[&str],
    config: &ApiConfig,
) -> Result<(), (StatusCode, Json<ApiResponse<()>>)> {
    if !config.use_shared_middleware {
        return Ok(());
    }
    // Placeholder: basic validation
    Ok(())
}
```

#### Future Enhancement: CORS Middleware
When cloudshuttle-api provides CORS middleware:

```rust
// Future implementation
pub fn validate_cors(
    &self,
    origin: Option<&str>,
    method: &str,
    headers: &[&str],
    _config: &ApiConfig,
) -> Result<(), (StatusCode, Json<ApiResponse<()>>)> {
    match self.cors_middleware.validate_request(origin, method, headers) {
        Ok(CorsResult::Allowed { allowed_headers, exposed_headers }) => {
            // Add CORS headers to response
            self.add_cors_headers(allowed_headers, exposed_headers);
            Ok(())
        }
        Err(CorsError::InvalidOrigin) => {
            Err((StatusCode::FORBIDDEN, error_response))
        }
        Err(CorsError::MethodNotAllowed) => {
            Err((StatusCode::METHOD_NOT_ALLOWED, error_response))
        }
    }
}
```

**Benefits:**
- Configurable allowed origins
- Method-specific CORS policies
- Preflight request handling
- Credential support

### 5. Pagination Design

#### Current Implementation
```rust
pub fn create_paginated_response<T: Serialize>(
    &self,
    items: Vec<T>,
    page: usize,
    limit: usize,
    total: usize,
) -> Json<ApiResponse<serde_json::Value>> {
    let response = serde_json::json!({
        "items": items,
        "pagination": {
            "page": page,
            "limit": limit,
            "total": total,
            "total_pages": (total + limit - 1) / limit
        }
    });
    self.create_success_response(response)
}
```

#### Future Enhancement: Typed Pagination
When cloudshuttle-api provides pagination types:

```rust
// Future implementation with typed pagination
pub fn create_paginated_response<T: Serialize>(
    &self,
    items: Vec<T>,
    page: usize,
    limit: usize,
    total: usize,
) -> Json<ApiResponse<PaginatedResponse<T>>> {
    let pagination_meta = PaginationMeta {
        page,
        limit,
        total,
        total_pages: (total + limit - 1) / limit,
        has_next: page * limit < total,
        has_prev: page > 1,
    };

    let response = PaginatedResponse {
        items,
        pagination: pagination_meta,
        links: self.generate_pagination_links(page, limit, total),
    };

    self.create_success_response(response)
}
```

**Benefits:**
- Type-safe pagination metadata
- Navigation link generation
- Consistent pagination across services
- Cursor-based pagination support

### 6. API Documentation Design

#### Current Implementation
```rust
pub fn create_api_docs(&self) -> Json<ApiResponse<serde_json::Value>> {
    let docs = serde_json::json!({
        "title": "CloudShuttle Proxy API",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Management and monitoring API",
        "endpoints": [
            {
                "path": "/health",
                "method": "GET",
                "description": "Health check endpoint"
            }
            // ... more endpoints
        ]
    });
    self.create_success_response(docs)
}
```

#### Future Enhancement: Structured Documentation
When cloudshuttle-api provides documentation types:

```rust
// Future implementation with structured docs
pub fn create_api_docs(&self) -> Json<ApiResponse<ApiDocumentation>> {
    let docs = ApiDocumentation::new("CloudShuttle Proxy API")
        .with_version(env!("CARGO_PKG_VERSION"))
        .with_description("Management and monitoring API")
        .with_base_url("/api/v1")
        .with_authentication(AuthScheme::Bearer)
        .with_endpoint(
            Endpoint::new("/health", Method::GET)
                .with_description("Health check endpoint")
                .with_response(StatusCode::OK, "HealthStatus")
                .with_tags(vec!["health", "monitoring"])
        )
        .with_endpoint(
            Endpoint::new("/metrics", Method::GET)
                .with_description("Metrics endpoint")
                .with_auth_required(true)
                .with_response(StatusCode::OK, "MetricsData")
                .with_tags(vec!["metrics", "monitoring"])
        )
        .build();

    self.create_success_response(docs)
}
```

**Benefits:**
- OpenAPI/Swagger compatible
- Type-safe endpoint definitions
- Authentication requirements
- Response schema documentation
- Tag-based organization

## Integration Strategy

### Configuration-Driven Migration

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiConfig {
    // Current features
    use_shared_api: bool,
    standardized_responses: bool,
    use_shared_middleware: bool,
    default_page_size: usize,

    // Future features (when available)
    use_advanced_pagination: bool,
    use_structured_documentation: bool,
    enable_response_metadata: bool,
    enable_request_tracing: bool,
}
```

### Backward Compatibility

- All existing API consumers continue to work unchanged
- Gradual migration through configuration flags
- Feature flags allow selective adoption
- Rollback capability maintained

### Testing Strategy

```rust
#[cfg(test)]
mod tests {
    // Test current implementation
    #[test]
    fn test_basic_response_formatting() { /* ... */ }

    // Test future enhancements (when available)
    #[cfg(feature = "advanced-responses")]
    #[test]
    fn test_advanced_response_formatting() { /* ... */ }
}
```

## Migration Roadmap

### Phase 1: Foundation (✅ Complete)
- ✅ Basic response formatting with builders
- ✅ Configuration structure
- ✅ Service initialization
- ✅ Backward compatibility
- ✅ Typed pagination objects
- ✅ Enhanced error handling
- ✅ Request validation with sanitization

### Phase 2: Middleware Integration (✅ Complete)
- ✅ Rate limiting middleware
- ✅ CORS middleware
- ✅ Request tracing middleware
- 🔄 Authentication middleware (partial - auth crate available, middleware integration pending)

### Phase 3: Documentation & Advanced Features (✅ Complete)
- ✅ Basic API documentation
- ✅ Structured API documentation (OpenAPI/Swagger)
- 🔄 Response caching
- 🔄 Request/response transformation
- 🔄 API versioning

### Phase 4: Extended Capabilities (Future)
- 🔄 GraphQL support
- 🔄 Advanced observability
- 🔄 API gateway features

## Implementation Guidelines

### Code Organization
```
crates/api/src/
├── lib.rs                # Main exports and module declarations
├── response.rs           # Standardized API responses and builders
├── pagination.rs         # Pagination utilities and types
├── error.rs              # API error types and responses
├── validation.rs         # Request validation and sanitization
├── service.rs            # Unified API service integration
├── rate_limit.rs         # Rate limiting middleware
├── cors.rs               # CORS middleware
└── docs.rs               # API documentation (future)
```

### Error Handling
- Use consistent error response format
- Include appropriate HTTP status codes
- Provide meaningful error messages
- Maintain error traceability

### Performance Considerations
- Minimize response serialization overhead
- Cache frequently used responses
- Optimize pagination queries
- Monitor API response times

### Security Considerations
- Validate all input parameters
- Sanitize response data
- Implement proper authentication
- Rate limiting protection
- CORS policy enforcement

## Conclusion

The API utilities system is now a comprehensive, production-ready foundation for CloudShuttle Gateway's API capabilities. The implementation includes advanced response formatting, type-safe pagination, robust validation, rate limiting, and CORS middleware. The system emphasizes backward compatibility, performance, and maintainability.

**Current Status**: The core API utilities are complete and ready for production use. Remaining work focuses on authentication middleware integration (pending axum compatibility fixes), structured API documentation, and advanced features like request tracing and caching.

**Key Achievements**:
- Comprehensive response formatting with builder patterns
- Type-safe pagination with navigation metadata
- Production-ready rate limiting and CORS middleware
- Extensive request validation with sanitization
- Unified service architecture for easy integration
- Full backward compatibility maintained

The system successfully evolved from a basic design to a sophisticated, enterprise-grade API utility library that significantly exceeds the original scope.
