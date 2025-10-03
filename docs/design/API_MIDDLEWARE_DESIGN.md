# API Middleware Design Document

## Overview

This document outlines the design for API middleware components in the CloudShuttle Gateway, including authentication, rate limiting, and CORS validation. These components provide essential security and performance controls for API endpoints.

## Architecture

### Middleware Stack

```rust
pub struct SharedApiService {
    // Response handling
    // Middleware components (future)
    auth_middleware: AuthMiddleware,
    rate_limit_middleware: RateLimitMiddleware,
    cors_middleware: CorsMiddleware,
}
```

### Configuration

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiConfig {
    use_shared_api: bool,
    use_shared_middleware: bool,  // Master switch for middleware

    // Individual middleware controls
    enable_authentication: bool,
    enable_rate_limiting: bool,
    enable_cors: bool,

    // Middleware-specific settings
    auth_required_by_default: bool,
    rate_limit_requests_per_minute: u32,
    cors_allowed_origins: Vec<String>,
}
```

## Authentication Middleware

### Current Implementation

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
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                self.create_error_response("Missing Authorization header".to_string()),
            )
        })?;

    if !auth_header.starts_with("Bearer ") {
        return Err((
            StatusCode::UNAUTHORIZED,
            self.create_error_response("Invalid Authorization header format".to_string()),
        ));
    }

    let token = &auth_header[7..];
    if token.is_empty() {
        return Err((
            StatusCode::UNAUTHORIZED,
            self.create_error_response("Invalid authentication token".to_string()),
        ));
    }

    Ok(())
}
```

### Future Enhancement: cloudshuttle-auth Integration

When cloudshuttle-auth provides middleware:

```rust
pub fn authenticate_request(
    &self,
    headers: &HeaderMap,
    config: &ApiConfig,
) -> Result<UserContext, (StatusCode, Json<ApiResponse<()>>)> {
    if !config.enable_authentication {
        return Ok(UserContext::anonymous());
    }

    match self.auth_middleware.validate_request(headers) {
        Ok(user_context) => {
            // Store user context for request processing
            self.set_user_context(user_context.clone());
            Ok(user_context)
        }
        Err(AuthError::MissingCredentials) => {
            Err((
                StatusCode::UNAUTHORIZED,
                Json(self.response_builder.error("Authentication required"))
            ))
        }
        Err(AuthError::InvalidToken) => {
            Err((
                StatusCode::UNAUTHORIZED,
                Json(self.response_builder.error("Invalid authentication token"))
            ))
        }
        Err(AuthError::ExpiredToken) => {
            Err((
                StatusCode::UNAUTHORIZED,
                Json(self.response_builder.error("Authentication token expired"))
            ))
        }
    }
}
```

### Authentication Types

```rust
#[derive(Debug, Clone)]
pub enum AuthMethod {
    None,
    BearerToken,
    ApiKey,
    OIDC,
    MutualTLS,
}

#[derive(Debug, Clone)]
pub struct UserContext {
    pub user_id: Option<String>,
    pub tenant_id: Option<String>,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub auth_method: AuthMethod,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub enum AuthError {
    MissingCredentials,
    InvalidCredentials,
    ExpiredCredentials,
    InsufficientPermissions,
    AccountDisabled,
}
```

### Endpoint-Level Configuration

```rust
#[derive(Debug, Clone)]
pub struct EndpointAuth {
    pub required: bool,
    pub allowed_methods: Vec<AuthMethod>,
    pub required_permissions: Vec<String>,
    pub rate_limit_override: Option<u32>,
}

// Usage in endpoint handlers
pub async fn secure_endpoint(
    auth: UserContext,
    body: Json<RequestData>,
) -> Result<Json<ApiResponse<ResponseData>>, StatusCode> {
    // Authentication already validated by middleware
    // Authorization checks can be performed here
    if !auth.permissions.contains(&"admin".to_string()) {
        return Err(StatusCode::FORBIDDEN);
    }

    // Process request
    let result = process_data(body.0, &auth).await?;
    Ok(Json(ApiResponse::success(result)))
}
```

## Rate Limiting Middleware

### Current Implementation

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

### Future Enhancement: Advanced Rate Limiting

When rate limiting middleware is available:

```rust
pub fn check_rate_limit(
    &self,
    client_ip: &str,
    endpoint: &str,
    config: &ApiConfig,
) -> Result<RateLimitInfo, (StatusCode, Json<ApiResponse<()>>)> {
    if !config.enable_rate_limiting {
        return Ok(RateLimitInfo::unlimited());
    }

    match self.rate_limiter.check_limit(client_ip, endpoint) {
        Ok(RateLimitResult::Allowed { remaining, reset_time, window }) => {
            let info = RateLimitInfo {
                remaining,
                reset_time,
                window,
                exceeded: false,
            };

            // Add rate limit headers to response
            self.add_rate_limit_headers(&info);
            Ok(info)
        }
        Err(RateLimitError::Exceeded { retry_after, limit, window }) => {
            let info = RateLimitInfo {
                remaining: 0,
                reset_time: retry_after,
                window,
                exceeded: true,
            };

            Err((
                StatusCode::TOO_MANY_REQUESTS,
                Json(self.response_builder.error("Rate limit exceeded")
                    .with_header("Retry-After", retry_after.to_string())
                    .with_header("X-RateLimit-Limit", limit.to_string())
                    .with_header("X-RateLimit-Reset", retry_after.to_string()))
            ))
        }
    }
}
```

### Rate Limiting Strategies

```rust
#[derive(Debug, Clone)]
pub enum RateLimitStrategy {
    FixedWindow {
        requests_per_window: u32,
        window_seconds: u64,
    },
    SlidingWindow {
        requests_per_window: u32,
        window_seconds: u64,
    },
    TokenBucket {
        capacity: u32,
        refill_rate: f64,  // tokens per second
    },
    LeakyBucket {
        capacity: u32,
        leak_rate: f64,   // requests per second
    },
}

#[derive(Debug)]
pub struct RateLimitInfo {
    pub remaining: u32,
    pub reset_time: DateTime<Utc>,
    pub window: Duration,
    pub exceeded: bool,
}
```

### Rate Limit Headers

```rust
impl SharedApiService {
    fn add_rate_limit_headers(&self, info: &RateLimitInfo) {
        // Standard rate limit headers
        self.set_response_header("X-RateLimit-Remaining", info.remaining.to_string());
        self.set_response_header("X-RateLimit-Reset", info.reset_time.timestamp().to_string());
        self.set_response_header("X-RateLimit-Limit", "1000"); // requests per window

        // Custom headers
        self.set_response_header("X-RateLimit-Window", info.window.as_secs().to_string());
    }
}
```

## CORS Middleware

### Current Implementation

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

### Future Enhancement: CORS Middleware

When CORS middleware is available:

```rust
pub fn validate_cors(
    &self,
    origin: Option<&str>,
    method: &str,
    headers: &[&str],
    config: &ApiConfig,
) -> Result<CorsHeaders, (StatusCode, Json<ApiResponse<()>>)> {
    if !config.enable_cors {
        return Ok(CorsHeaders::disabled());
    }

    match self.cors_middleware.validate_request(origin, method, headers) {
        Ok(CorsResult::Allowed {
            allowed_headers,
            exposed_headers,
            allow_credentials,
            max_age,
        }) => {
            let cors_headers = CorsHeaders {
                allowed_headers,
                exposed_headers,
                allow_credentials,
                max_age,
            };

            // Add CORS headers to response
            self.add_cors_headers(&cors_headers);
            Ok(cors_headers)
        }
        Err(CorsError::InvalidOrigin) => {
            Err((
                StatusCode::FORBIDDEN,
                Json(self.response_builder.error("CORS origin not allowed"))
            ))
        }
        Err(CorsError::MethodNotAllowed) => {
            Err((
                StatusCode::METHOD_NOT_ALLOWED,
                Json(self.response_builder.error("CORS method not allowed"))
            ))
        }
        Err(CorsError::HeadersNotAllowed) => {
            Err((
                StatusCode::FORBIDDEN,
                Json(self.response_builder.error("CORS headers not allowed"))
            ))
        }
    }
}
```

### CORS Configuration

```rust
#[derive(Debug, Clone)]
pub struct CorsConfig {
    pub allow_origins: Vec<String>,  // ["https://*.example.com", "http://localhost:*"]
    pub allow_methods: Vec<String>,  // ["GET", "POST", "PUT", "DELETE"]
    pub allow_headers: Vec<String>,  // ["Content-Type", "Authorization", "X-API-Key"]
    pub expose_headers: Vec<String>, // ["X-Custom-Header"]
    pub allow_credentials: bool,
    pub max_age: Option<u32>,        // preflight cache duration in seconds
}

#[derive(Debug)]
pub struct CorsHeaders {
    pub allowed_headers: Vec<String>,
    pub exposed_headers: Vec<String>,
    pub allow_credentials: bool,
    pub max_age: Option<u32>,
}
```

### Preflight Request Handling

```rust
pub async fn handle_cors_preflight(
    origin: Option<&str>,
    method: &str,
    headers: &[&str],
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    match validate_cors(origin, method, headers) {
        Ok(cors_headers) => {
            // Return successful preflight response with CORS headers
            let mut response = Json(ApiResponse::success(()));

            // Add CORS headers to response
            if let Some(response) = response.as_mut() {
                // This would be handled by the framework's response builder
                // to add the appropriate CORS headers
            }

            Ok(response)
        }
        Err((status, error_response)) => Err(status),
    }
}
```

## Middleware Integration

### Middleware Chain

```rust
pub async fn process_api_request<T>(
    request: Request,
    handler: impl Fn(Request, MiddlewareContext) -> Result<T>,
) -> Result<Json<ApiResponse<T>>, StatusCode> {
    // 1. CORS validation (for preflight and actual requests)
    let cors_result = validate_cors(
        request.origin(),
        request.method(),
        request.headers(),
    )?;

    // 2. Rate limiting
    let rate_limit_info = check_rate_limit(
        request.client_ip(),
        request.path(),
    )?;

    // 3. Authentication
    let user_context = authenticate_request(request.headers())?;

    // 4. Authorization (endpoint-specific)
    let middleware_context = MiddlewareContext {
        user: user_context,
        rate_limit: rate_limit_info,
        cors: cors_result,
    };

    // 5. Execute handler
    let result = handler(request, middleware_context).await?;

    Ok(Json(ApiResponse::success(result)))
}
```

### Middleware Context

```rust
#[derive(Debug)]
pub struct MiddlewareContext {
    pub user: UserContext,
    pub rate_limit: RateLimitInfo,
    pub cors: CorsHeaders,
    pub request_id: String,
    pub start_time: Instant,
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderMap, HeaderValue};

    #[test]
    fn test_authentication_missing_header() {
        let service = SharedApiService::new(&ApiConfig::default()).unwrap();
        let headers = HeaderMap::new();

        let result = service.authenticate_request(&headers, &ApiConfig::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_authentication_valid_token() {
        let service = SharedApiService::new(&ApiConfig::default()).unwrap();
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", HeaderValue::from_static("Bearer valid-token"));

        let result = service.authenticate_request(&headers, &ApiConfig::default());
        assert!(result.is_ok());
    }

    #[test]
    fn test_rate_limiting_disabled() {
        let config = ApiConfig {
            use_shared_middleware: false,
            ..Default::default()
        };
        let service = SharedApiService::new(&config).unwrap();

        let result = service.check_rate_limit("127.0.0.1", "/api/test", &config);
        assert!(result.is_ok()); // Should always pass when disabled
    }
}
```

### Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use axum::http::Request;
    use tower::Service;

    #[tokio::test]
    async fn test_full_middleware_chain() {
        // Test complete request processing with all middleware
        let app = create_test_app();

        // Test successful request
        let request = Request::builder()
            .method("GET")
            .uri("/api/health")
            .header("Authorization", "Bearer test-token")
            .body(Body::empty())
            .unwrap();

        let response = app.call(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
```

## Performance Considerations

### Caching
- Cache CORS validation results
- Cache authentication tokens (with TTL)
- Cache rate limit counters

### Async Processing
- Non-blocking middleware operations
- Parallel validation where possible
- Early returns for failed validations

### Monitoring
- Middleware performance metrics
- Authentication success/failure rates
- Rate limiting statistics
- CORS violation tracking

## Security Considerations

### Authentication
- Secure token storage and validation
- Token expiration and refresh
- Account lockout protection
- Audit logging for security events

### Rate Limiting
- Distributed rate limiting for multi-instance deployments
- Client fingerprinting to prevent bypass
- Graduated rate limit responses
- Rate limit bypass for trusted clients

### CORS
- Strict origin validation
- Minimal allowed headers
- Credential handling security
- Preflight request validation

## Migration Strategy

### Phase 1: Basic Middleware (Current)
- ✅ Response formatting
- ✅ Basic authentication placeholder
- ✅ Rate limiting placeholder
- ✅ CORS placeholder

### Phase 2: Enhanced Middleware
- 🔄 JWT token authentication
- 🔄 Configurable rate limiting
- 🔄 CORS policy enforcement
- 🔄 Request tracing

### Phase 3: Advanced Security
- 🔄 Multi-factor authentication
- 🔄 API key management
- 🔄 OAuth2/OIDC integration
- 🔄 Advanced authorization

## Conclusion

The middleware design provides a comprehensive security and performance framework for CloudShuttle Gateway APIs. The modular architecture allows for gradual enhancement while maintaining backward compatibility and performance.
