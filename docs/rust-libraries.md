# Rust Shared Libraries Guide

This guide covers the Rust shared libraries available in CloudShuttle.

## Available Libraries

### 1. Error Handling (`cloudshuttle-error-handling`)

Standardized error types and handling across all Rust services.

**Key Features:**
- Base error types (`CloudShuttleError`)
- Service-specific error traits (`ServiceError`)
- HTTP API error mappings
- Database error handling
- Error handling macros

**Usage:**
```rust
use cloudshuttle_error_handling::{CloudShuttleError, ServiceError};

// Using base error types
fn process_data() -> Result<(), CloudShuttleError> {
    Ok(())
}

// Implementing service-specific errors
#[derive(Debug, thiserror::Error)]
#[error("Auth service error: {message}")]
pub struct AuthError {
    message: String,
    code: String,
}

impl ServiceError for AuthError {
    fn error_code(&self) -> &'static str { &self.code }
    fn http_status(&self) -> http::StatusCode { StatusCode::UNAUTHORIZED }
    fn user_message(&self) -> String { self.message.clone() }
}
```

### 2. Database Layer (`cloudshuttle-database`)

Common database utilities, connection management, and query helpers.

**Key Features:**
- Database connection management
- Query builder helpers and pagination
- Transaction management
- Migration utilities
- Common database types

**Usage:**
```rust
use cloudshuttle_database::{DatabaseConnection, QueryHelper};

let db = DatabaseConnection::new("postgresql://...").await?;
let user = db.find_by_id::<User>("users", user_id).await?;
```

### 3. Authentication (`cloudshuttle-auth`)

Common authentication utilities and JWT handling.

**Key Features:**
- JWT token creation and validation
- Password hashing and verification
- Token generation utilities
- Password strength validation

**Usage:**
```rust
use cloudshuttle_auth::{JwtService, PasswordHasher};

let jwt_service = JwtService::new("secret")?;
let token = jwt_service.create_access_token("user123", tenant_id, vec!["admin".to_string()])?;

let hasher = PasswordHasher::new();
let hash = hasher.hash("password123")?;
```

### 4. Observability (`cloudshuttle-observability`)

Centralized logging, metrics, and tracing utilities.

**Key Features:**
- Structured logging with context
- Metrics collection and reporting
- Distributed tracing
- Health check utilities

**Usage:**
```rust
use cloudshuttle_observability::{Logger, MetricsCollector};

let logger = Logger::new("my-service", "1.0.0");
logger.info("User logged in", &[("user_id", &user_id)]);

let metrics = MetricsCollector::new();
let counter = metrics.counter("requests_total", "Total requests")?;
counter.inc();
```

### 5. Configuration (`cloudshuttle-config`)

Configuration management and validation.

**Key Features:**
- Configuration loading from multiple sources
- Configuration validation
- Environment detection
- Secret management

### 6. API Utilities (`cloudshuttle-api`)

Common API utilities, response formatting, and request handling.

**Key Features:**
- Standardized API responses
- Pagination utilities
- Query filtering and sorting
- API middleware

### 7. Validation (`cloudshuttle-validation`)

Input validation utilities and common validation rules.

**Key Features:**
- Custom validation rules
- Input sanitization
- Validation macros

### 8. Cryptography (`cloudshuttle-crypto`)

Cryptographic utilities for encryption, hashing, and secure operations.

**Key Features:**
- Data encryption/decryption
- Secure random generation
- Digital signatures

## Development

### Adding a New Library

1. Create a new directory under `rust/`
2. Add `Cargo.toml` with proper dependencies
3. Implement the library following the established patterns
4. Add comprehensive tests
5. Update documentation

### Dependencies

All libraries should depend on `cloudshuttle-error-handling` for consistent error handling.

### Testing

Each library should have comprehensive unit tests and integration tests where applicable.

### Documentation

All public APIs should be documented with examples.

## Version Compatibility

Libraries follow semantic versioning and maintain backward compatibility within major versions.
