# CloudShuttle Error Handling

Standardized error types and handling across all CloudShuttle Rust services.

## Features

- Base error types for common error scenarios
- Service-specific error traits with HTTP status mapping
- HTTP API error types
- Database error handling with SQLx integration
- Error handling macros for common patterns

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
cloudshuttle-error-handling = "0.1.0"
```

For SQLx integration:

```toml
[dependencies]
cloudshuttle-error-handling = { version = "0.1.0", features = ["sqlx"] }
```

## Examples

### Basic Error Types

```rust
use cloudshuttle_error_handling::{CloudShuttleError, Result};

fn process_data() -> Result<()> {
    // Some processing logic that might fail
    Ok(())
}
```

### Service-Specific Errors

```rust
use cloudshuttle_error_handling::{ServiceError, service_error};
use http::StatusCode;

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

### Using Macros

```rust
use cloudshuttle_error_handling::{bail, ensure, service_error};

// Return an error
bail!(CloudShuttleError::internal("Something went wrong"));

// Ensure a condition
ensure!(user.is_authenticated(), CloudShuttleError::auth("Not authenticated"));

// Create service errors
let error = service_error!("VALIDATION_ERROR", StatusCode::BAD_REQUEST, "Invalid input");
```

### Database Operations

```rust
use cloudshuttle_error_handling::{db_op, db_find, DatabaseError};

#[cfg(feature = "sqlx")]
async fn find_user(db: &sqlx::PgPool, id: i32) -> Result<User> {
    let user = db_find!(
        sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
            .fetch_optional(db)
            .await,
        "User"
    )?;
    Ok(user)
}
```

## Error Types

- `CloudShuttleError`: Base error enum for all CloudShuttle services
- `ServiceError`: Trait for service-specific errors with HTTP mapping
- `ApiError`: HTTP API specific errors
- `DatabaseError`: Database operation errors
- `StandardServiceError`: Standard implementation of ServiceError

## Macros

- `service_error!`: Create service errors
- `bail!`: Return an error
- `ensure!`: Ensure a condition or return error
- `db_op!`: Wrap database operations
- `db_find!`: Handle optional database results
- `api_error!`: Create API errors
- `validate_required!`: Validate required fields
- `validate_length!`: Validate field length

## License

This project is licensed under MIT OR Apache-2.0.
