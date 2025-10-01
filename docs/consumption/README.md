# CloudShuttle Shared Libraries - Consumption Guide

This guide explains how to consume CloudShuttle shared libraries in other Rust projects.

## 🚀 Quick Start

### Option 1: Git Dependency (Recommended for Production)

Add to your `Cargo.toml`:

```toml
[dependencies]
cloudshuttle-auth = { git = "https://github.com/cloud-shuttle/cloudshuttle-shared.git", branch = "main" }
cloudshuttle-database = { git = "https://github.com/cloud-shuttle/cloudshuttle-shared.git", branch = "main" }
cloudshuttle-error-handling = { git = "https://github.com/cloud-shuttle/cloudshuttle-shared.git", branch = "main" }
```

### Option 2: Path Dependency (Recommended for Development)

For local development, add to your `Cargo.toml`:

```toml
[dependencies]
cloudshuttle-auth = { path = "../cloudshuttle-shared/rust/crates/auth" }
cloudshuttle-database = { path = "../cloudshuttle-shared/rust/crates/database" }
cloudshuttle-error-handling = { path = "../cloudshuttle-shared/rust/crates/error-handling" }
```

---

## 📦 Available Libraries

| Library | Description | Key Features |
|---------|-------------|--------------|
| `cloudshuttle-auth` | Authentication & JWT utilities | JWT creation/validation, password security, MFA support |
| `cloudshuttle-database` | Database operations & queries | Connection pooling, dynamic queries, transactions |
| `cloudshuttle-error-handling` | Error handling & logging | Structured errors, HTTP responses, service error metrics |
| `cloudshuttle-validation` | Input validation & sanitization | XSS/SQL injection prevention, schema validation |
| `cloudshuttle-observability` | Monitoring & metrics | Performance monitoring, distributed tracing |

---

## 🔧 Detailed Configuration

### Git Dependencies with Authentication

For private repositories, configure authentication:

```bash
# Set up Git credentials
git config --global credential.helper store
echo "https://username:token@github.com" > ~/.git-credentials
```

Then in `Cargo.toml`:

```toml
[dependencies]
cloudshuttle-auth = { git = "https://github.com/cloud-shuttle/cloudshuttle-shared.git" }
cloudshuttle-database = { git = "https://github.com/cloud-shuttle/cloudshuttle-shared.git" }
```

### Version Pinning

For production stability, consider pinning to specific versions:

```toml
[dependencies]
# Pin to specific commit for maximum stability
cloudshuttle-auth = { git = "https://github.com/cloud-shuttle/cloudshuttle-shared.git", rev = "commit-hash" }

# Or use branches (recommended for most use cases)
cloudshuttle-auth = { git = "https://github.com/cloud-shuttle/cloudshuttle-shared.git", branch = "main" }

# Or use tags when releases are available
cloudshuttle-auth = { git = "https://github.com/cloud-shuttle/cloudshuttle-shared.git", tag = "v1.0.0" }
```

### Feature Flags

Enable optional features as needed:

```toml
[dependencies]
cloudshuttle-auth = { git = "https://github.com/cloud-shuttle/cloudshuttle-shared.git", features = ["middleware"] }
cloudshuttle-database = { git = "https://github.com/cloud-shuttle/cloudshuttle-shared.git", features = ["postgres", "mysql"] }
```

---

## 💡 Usage Examples

### Authentication Service

```rust
use cloudshuttle_auth::{JwtService, Claims, SecurityValidator};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create JWT service
    let jwt_service = JwtService::new(b"your-secret-key")?;

    // Create claims
    let claims = Claims::new("user-123", "tenant-456");

    // Generate token
    let token = jwt_service.create_token(&claims)?;

    // Validate password
    let is_strong = SecurityValidator::validate_password_strength("MySecurePass123!")?;
    assert!(is_strong.is_strong);

    Ok(())
}
```

### Database Operations

```rust
use cloudshuttle_database::{DatabaseConnection, QueryBuilder};
use cloudshuttle_error_handling::DatabaseResult;

#[tokio::main]
async fn main() -> DatabaseResult<()> {
    // Connect to database
    let db = DatabaseConnection::new("postgresql://localhost/mydb").await?;

    // Build dynamic query
    let query = QueryBuilder::new("users")
        .select(vec!["id", "email", "created_at"])
        .where_eq("active", true)
        .limit(10)
        .build();

    // Execute query
    let users = db.execute_query(query.0, &query.1).await?;

    Ok(())
}
```

### Error Handling

```rust
use cloudshuttle_error_handling::{CloudShuttleError, ServiceError};

fn process_user_data(data: &str) -> Result<(), CloudShuttleError> {
    if data.is_empty() {
        return Err(CloudShuttleError::Validation {
            field: "data".to_string(),
            message: "Data cannot be empty".to_string(),
        });
    }

    // Process data...
    Ok(())
}
```

---

## 🏗️ Advanced Configuration

### Workspace Setup

For monorepo setups, include shared libraries in your workspace:

```toml
# In your root Cargo.toml
[workspace]
members = [
    "services/auth-service",
    "services/api-gateway",
    "shared/cloudshuttle-shared/rust/crates/*",
]

[workspace.dependencies]
cloudshuttle-auth = { path = "shared/cloudshuttle-shared/rust/crates/auth" }
cloudshuttle-database = { path = "shared/cloudshuttle-shared/rust/crates/database" }
```

### Private Registry

For enterprise setups, configure a private cargo registry:

```bash
# Configure private registry
cargo login --registry my-registry

# Add to .cargo/config.toml
[registries.my-registry]
index = "https://my-registry.com/git/index"
```

Then in `Cargo.toml`:

```toml
[dependencies]
cloudshuttle-auth = { version = "0.2.0", registry = "my-registry" }
```

### Build Optimization

Optimize builds with feature flags:

```toml
[dependencies]
cloudshuttle-auth = { git = "https://github.com/your-org/cloudshuttle-shared.git", default-features = false, features = ["jwt", "security"] }
```

---

## 🔍 Dependency Management

### Updating Dependencies

```bash
# Update all git dependencies
cargo update

# Update specific dependency
cargo update -p cloudshuttle-auth

# Check for outdated dependencies
cargo outdated
```

### Lock File Management

```bash
# Generate new lock file
rm Cargo.lock
cargo check

# Verify lock file integrity
cargo tree --locked
```

---

## 🧪 Testing with Shared Libraries

### Integration Testing

```rust
#[cfg(test)]
mod tests {
    use cloudshuttle_auth::{JwtService, SecurityValidator};
    use cloudshuttle_database::DatabaseConnection;

    #[tokio::test]
    async fn test_user_registration_flow() {
        // Test complete user registration flow using shared libraries
        let jwt_service = JwtService::new(b"test-key").unwrap();

        // Validate password
        let strength = SecurityValidator::validate_password_strength("TestPass123!").unwrap();
        assert!(strength.is_strong);

        // Create JWT
        let claims = Claims::new("user-123", "tenant-456");
        let token = jwt_service.create_token(&claims).unwrap();

        // Verify token
        let validated = jwt_service.validate_token(&token).unwrap();
        assert_eq!(validated.sub, "user-123");
    }
}
```

---

## 🚀 CI/CD Integration

### GitHub Actions Example

```yaml
# .github/workflows/build.yml
name: Build and Test

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v3
    - uses: actions-rust-lang/setup-rust-toolchain@v1

    - name: Cache Cargo
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

    - name: Run tests
      run: cargo test --workspace --verbose
```

### Docker Integration

```dockerfile
# Dockerfile
FROM rust:1.70-slim as builder

# Clone shared libraries
RUN git clone https://github.com/your-org/cloudshuttle-shared.git /shared

WORKDIR /app

# Copy your service code
COPY . .

# Build with shared libraries
RUN cargo build --release
```

---

## 🔐 Security Considerations

### Dependency Verification

```bash
# Verify dependency checksums
cargo tree --locked

# Audit dependencies for vulnerabilities
cargo audit

# Check for outdated dependencies
cargo outdated
```

### Private Repository Access

For private repositories, ensure proper authentication:

```bash
# SSH key setup (recommended)
ssh-keygen -t ed25519 -C "your-email@example.com"
# Add public key to GitHub

# Or use personal access tokens
git config --global credential.helper store
echo "https://username:token@github.com" > ~/.git-credentials
```

---

## 📚 Additional Resources

- [Cargo Reference - Dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html)
- [Git Dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#git-dependencies)
- [Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)
- [Private Registries](https://doc.rust-lang.org/cargo/reference/registries.html)

---

## 🆘 Troubleshooting

### Common Issues

**Issue: "failed to load source for dependency"**
```
Solution: Check git repository access and credentials
cargo clean && cargo update
```

**Issue: "feature not enabled"**
```
Solution: Enable required features in Cargo.toml
cloudshuttle-auth = { git = "...", features = ["middleware"] }
```

**Issue: "version conflict"**
```
Solution: Update Cargo.lock or pin to specific versions
rm Cargo.lock && cargo check
```

**Issue: "authentication failed"**
```
Solution: Configure git credentials or SSH keys
git config --global credential.helper store
```

---

*This guide covers all major approaches for consuming CloudShuttle shared libraries in your projects.*
