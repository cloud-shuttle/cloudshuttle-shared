# CloudShuttle Shared Crates Overview

This document provides a comprehensive overview of all available crates in the CloudShuttle shared library, their functionality, and how they work together to provide a complete backend service framework.

## 📦 Available Crates

### 1. 🔧 `cloudshuttle-error-handling` - Error Management
**Purpose**: Standardized error handling across all CloudShuttle services

**Key Features**:
- `CloudShuttleError` - Common error types for configuration, validation, database, and API errors
- `ServiceError` trait - For implementing service-specific errors with HTTP status codes
- `ApiError` - HTTP-friendly error responses with status codes and user messages
- `DatabaseError` - Database-specific error handling and mapping

**Use Cases**:
- Consistent error responses across microservices
- HTTP status code mapping
- Structured error logging and monitoring
- API error response formatting

**Example**:
```rust
use cloudshuttle_error_handling::{CloudShuttleError, ServiceError, ApiError};

#[derive(Debug, thiserror::Error)]
#[error("User not found: {user_id}")]
pub struct UserNotFoundError {
    user_id: String,
}

impl ServiceError for UserNotFoundError {
    fn error_code(&self) -> &'static str { "USER_NOT_FOUND" }
    fn http_status(&self) -> http::StatusCode { http::StatusCode::NOT_FOUND }
    fn user_message(&self) -> String { format!("User '{}' was not found", self.user_id) }
}
```

---

### 2. 🗄️ `cloudshuttle-database` - Database Operations
**Purpose**: Database connection management, query helpers, and migrations

**Key Features**:
- `DatabaseConnection` - Async PostgreSQL connection management
- `ConnectionPool` - Advanced connection pooling with health checks and metrics
- `QueryHelper` - Common database operations (find_by_id, exists, count)
- `MigrationRunner` - Database schema migrations with rollback support
- `DatabaseTransaction` - Transaction management with automatic rollback
- Advanced pool types with metrics and health monitoring

**Use Cases**:
- PostgreSQL database operations
- Connection pooling and health monitoring
- Database migrations and schema management
- Transaction management
- Query building and execution

**Example**:
```rust
use cloudshuttle_database::{DatabaseConnection, QueryHelper, DatabaseTransaction};

let db = DatabaseConnection::new("postgresql://...").await?;

// Simple queries
let user = db.find_by_id::<User>("users", user_id).await?;
let exists = db.exists("users", "email", user_email).await?;

// Transactions
let result = db.transaction(|tx| async move {
    tx.execute("INSERT INTO users ...", &[]).await?;
    tx.find_by_id::<User>("users", user_id).await
}).await;
```

---

### 3. 🔐 `cloudshuttle-auth` - Authentication & JWT
**Purpose**: JWT-based authentication and authorization

**Key Features**:
- `JwtService` - JWT token creation, validation, and refresh
- `Claims` - JWT claims structure with user/tenant/role information
- `AuthMiddleware` - Axum middleware for request authentication
- `KeyManager` - Secure key management and rotation
- Password hashing with Argon2
- Token introspection and PKCE support

**Use Cases**:
- User authentication and session management
- API authorization with role-based access control
- Secure token management and refresh
- Multi-tenant application support

**Example**:
```rust
use cloudshuttle_auth::{JwtService, Claims, AuthMiddleware};

let jwt_service = JwtService::new("your-secret-key".as_bytes())?;

// Create token
let claims = Claims::new("user-123", "tenant-456")
    .with_roles(vec!["user".to_string()]);
let token = jwt_service.create_token(&claims)?;

// Validate token
let validated_claims = jwt_service.validate_token(&token)?;
assert_eq!(validated_claims.sub, "user-123");
```

---

### 4. 📊 `cloudshuttle-observability` - Monitoring & Logging
**Purpose**: Comprehensive observability stack for services

**Key Features**:
- `TracingConfig` - Structured logging with configurable levels and formats
- `MetricsCollector` - Prometheus metrics collection and exposition
- `HealthChecker` - Service health monitoring and reporting
- `TracingMiddleware` - Request tracing with distributed tracing support
- `AuditLogger` - Security event auditing and compliance logging
- Performance monitoring and alerting

**Use Cases**:
- Structured logging and log aggregation
- Performance monitoring and alerting
- Service health checks and dependency monitoring
- Distributed tracing across microservices
- Security auditing and compliance

**Example**:
```rust
use cloudshuttle_observability::{init_tracing, register_metrics, HealthChecker};

init_tracing("my-service", tracing::Level::INFO)?;
register_metrics();

let health_checker = HealthChecker::new("my-service")
    .with_database_check(db_pool)
    .with_custom_check("api", || async { /* custom health logic */ });

tracing::info!(user_id = %user.id, action = "login", "User logged in");
```

---

### 5. ⚙️ `cloudshuttle-config` - Configuration Management
**Purpose**: Centralized configuration loading and validation

**Key Features**:
- `ConfigLoader` - Environment variables, files, and secrets loading
- Configuration validation with detailed error messages
- `Secret` type for sensitive configuration
- Hot reloading support for configuration changes
- Type-safe configuration access with validation

**Use Cases**:
- Multi-environment configuration (dev/staging/prod)
- Secret management and rotation
- Configuration validation and error handling
- Hot reloading for zero-downtime config updates
- Type-safe configuration access

**Example**:
```rust
use cloudshuttle_config::ConfigLoader;
use serde::Deserialize;

#[derive(Debug, Deserialize, Validate)]
pub struct AppConfig {
    pub database_url: String,
    pub port: u16,
    #[validate(range(min = 1, max = 100))]
    pub max_connections: u32,
    #[serde(flatten)]
    pub secrets: SecretConfig,
}

let config: AppConfig = ConfigLoader::new("my-service")
    .with_env_prefix("MY_SERVICE")
    .with_file("config.toml")
    .with_hot_reload()
    .load()?;
```

---

### 6. 🛡️ `cloudshuttle-validation` - Input Validation & Sanitization
**Purpose**: Security-focused input validation and data sanitization

**Key Features**:
- `validate_email`, `validate_password_strength`, `validate_username`
- HTML, SQL, and filename sanitization
- `AdvancedValidator` - Custom validation rules and contexts
- XSS prevention and input security
- Unicode normalization and security checks

**Use Cases**:
- User input validation and sanitization
- XSS and injection attack prevention
- Data integrity and security
- International character handling
- Custom business rule validation

**Example**:
```rust
use cloudshuttle_validation::{validate_email, sanitize_html, AdvancedValidator};

assert!(validate_email("user@example.com").is_ok());
assert!(validate_email("invalid-email").is_err());

let clean_html = sanitize_html("<script>alert('xss')</script>Hello World");
// Result: "Hello World"

let validator = AdvancedValidator::new()
    .with_rule(|input| input.len() >= 8, "Password too short")
    .validate("mypassword")?;
```

---

### 7. 🔒 `cloudshuttle-crypto` - Cryptographic Operations
**Purpose**: Secure cryptographic utilities for data protection

**Key Features**:
- `hash_password`/`verify_password` - Argon2 password hashing
- `encrypt_data`/`decrypt_data` - AES-GCM encryption
- `generate_secure_token` - Cryptographically secure random tokens
- Key derivation and secure key management

**Use Cases**:
- Password storage and verification
- Sensitive data encryption/decryption
- Secure token generation for sessions/API keys
- Cryptographic key management
- Data protection and compliance

**Example**:
```rust
use cloudshuttle_crypto::{hash_password, verify_password, encrypt_data, decrypt_data};

// Password hashing
let hash = hash_password("user-password")?;
assert!(verify_password("user-password", &hash)?);

// Data encryption
let key = b"32-byte-secret-key-for-aes-256-gcm";
let encrypted = encrypt_data(b"sensitive data", key)?;
let decrypted = decrypt_data(&encrypted, key)?;
assert_eq!(decrypted, b"sensitive data");
```

---

### 8. 🌐 `cloudshuttle-api` - API Utilities & Middleware
**Purpose**: Complete API framework with response formatting, middleware, and documentation

**Key Features**:
- `ApiResponse<T>` - Standardized API response format with success/error states
- **Rate Limiting**: Production-ready sliding window rate limiting with presets
- **CORS Middleware**: Full cross-origin resource sharing support
- **Request Tracing**: Request IDs, timing, and context extraction
- **Pagination**: Type-safe pagination with navigation metadata
- **OpenAPI Documentation**: Complete OpenAPI 3.0 specification generation
- **Validation Integration**: Request validation and sanitization
- **Service Architecture**: Unified middleware integration

**Use Cases**:
- REST API development with consistent responses
- API security (rate limiting, CORS, input validation)
- API documentation and developer experience
- Request monitoring and observability
- Pagination and data navigation
- Middleware composition and management

**Example**:
```rust
use cloudshuttle_api::{
    ApiResponse, ApiService, PaginatedResponse, PaginationMeta,
    rate_limit::presets::api_limiter,
    cors::presets::api_cors,
    request_tracing::presets::standard_tracing,
};

// Create unified API service
let service = ApiService::new()
    .require_auth()
    .with_cors();

// Rate limiting
let rate_limiter = Arc::new(api_limiter());

// CORS configuration
let cors_config = api_cors();

// Request tracing
let tracing_config = standard_tracing();

// Response formatting
let response: ApiResponse<String> = ApiResponse::success("Hello World".to_string());

// Paginated responses
let pagination = PaginationMeta::new(1, 20, 100);
let paginated: ApiResponse<PaginatedResponse<User>> =
    ApiResponse::success(PaginatedResponse::new(users, pagination));
```

---

## 🔗 Crate Dependencies & Integration

```
cloudshuttle-api
├── cloudshuttle-auth (optional)
├── cloudshuttle-error-handling
└── cloudshuttle-validation

cloudshuttle-auth
├── cloudshuttle-crypto
├── cloudshuttle-observability (optional)
└── cloudshuttle-error-handling

cloudshuttle-database
└── cloudshuttle-error-handling

cloudshuttle-observability
└── cloudshuttle-error-handling

cloudshuttle-config
└── cloudshuttle-validation

cloudshuttle-validation
└── (independent)

cloudshuttle-crypto
└── (independent)

cloudshuttle-error-handling
└── (independent)
```

---

## 🚀 Quick Start Examples

### Basic Service Setup
```rust
use cloudshuttle_config::ConfigLoader;
use cloudshuttle_database::DatabaseConnection;
use cloudshuttle_auth::JwtService;
use cloudshuttle_api::{ApiService, ApiResponse};
use cloudshuttle_observability::init_tracing;

// Load configuration
let config: AppConfig = ConfigLoader::new("my-service").load()?;

// Initialize observability
init_tracing("my-service", tracing::Level::INFO)?;

// Connect to database
let db = DatabaseConnection::new(&config.database_url).await?;

// Initialize auth service
let jwt_service = JwtService::new(config.jwt_secret.as_bytes())?;

// Create API service
let api_service = ApiService::new()
    .require_auth()
    .with_cors();

// Your service is ready!
```

### Complete API Endpoint
```rust
use cloudshuttle_api::{ApiResponse, PaginatedResponse, PaginationMeta};
use cloudshuttle_auth::AuthenticatedUser;
use cloudshuttle_validation::validate_email;

#[derive(serde::Serialize)]
struct User { id: u32, name: String, email: String }

async fn list_users(
    auth: AuthenticatedUser,  // From auth middleware
    pagination: PaginationParams,  // From query params
) -> Result<ApiResponse<PaginatedResponse<User>>, ApiError> {
    // Validate pagination
    validate_pagination(&pagination)?;

    // Check authorization
    if !auth.has_role("admin") && !auth.has_role("user_manager") {
        return Err(ApiError::forbidden("Insufficient permissions"));
    }

    // Fetch users from database
    let users = db.find_users(pagination.offset(), pagination.limit()).await?;
    let total = db.count_users().await?;

    let pagination_meta = PaginationMeta::new(
        pagination.get_page(),
        pagination.get_per_page(),
        total
    );

    Ok(ApiResponse::success(PaginatedResponse::new(users, pagination_meta)))
}
```

---

## 📈 Development Status

| Crate | Status | Test Coverage | Features |
|-------|--------|---------------|----------|
| `cloudshuttle-api` | ✅ Production Ready | 48 tests | Complete API framework |
| `cloudshuttle-auth` | 🔄 Partial (JWT ready) | Full coverage | Authentication framework |
| `cloudshuttle-database` | ✅ Production Ready | Full coverage | Database operations |
| `cloudshuttle-observability` | ✅ Production Ready | Full coverage | Monitoring & logging |
| `cloudshuttle-config` | ✅ Production Ready | Full coverage | Configuration management |
| `cloudshuttle-validation` | ✅ Production Ready | Full coverage | Input validation |
| `cloudshuttle-crypto` | ✅ Production Ready | Full coverage | Cryptographic operations |
| `cloudshuttle-error-handling` | ✅ Production Ready | Full coverage | Error management |

---

## 🎯 Best Practices

1. **Error Handling**: Use `cloudshuttle-error-handling` for consistent error responses
2. **Configuration**: Use `cloudshuttle-config` for all configuration needs
3. **Validation**: Validate all user inputs with `cloudshuttle-validation`
4. **Security**: Use `cloudshuttle-auth` + `cloudshuttle-crypto` for authentication
5. **Monitoring**: Implement observability with `cloudshuttle-observability`
6. **Database**: Use `cloudshuttle-database` for all database operations
7. **API Development**: Start with `cloudshuttle-api` for consistent APIs

This crate ecosystem provides a complete, production-ready foundation for building secure, scalable, and maintainable backend services in Rust.
