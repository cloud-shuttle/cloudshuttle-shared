# 🚀 CloudShuttle Shared Libraries - Quick Reference

## How to Add CloudShuttle Libraries to Your Project

### 1. Git Dependencies (Production)
```toml
[dependencies]
cloudshuttle-auth = { git = "https://github.com/your-org/cloudshuttle-shared.git", branch = "main" }
cloudshuttle-database = { git = "https://github.com/your-org/cloudshuttle-shared.git", branch = "main" }
cloudshuttle-error-handling = { git = "https://github.com/your-org/cloudshuttle-shared.git", branch = "main" }
```

### 2. Path Dependencies (Development)
```toml
[dependencies]
cloudshuttle-auth = { path = "../cloudshuttle-shared/rust/crates/auth" }
cloudshuttle-database = { path = "../cloudshuttle-shared/rust/crates/database" }
```

### 3. Version Pinning
```toml
[dependencies]
# Pin to specific commit for maximum stability
cloudshuttle-auth = { git = "https://github.com/cloud-shuttle/cloudshuttle-shared.git", rev = "commit-hash" }

# Or use branches (recommended)
cloudshuttle-auth = { git = "https://github.com/cloud-shuttle/cloudshuttle-shared.git", branch = "main" }
```

---

## 📦 Available Libraries

| Library | Import | Purpose | Key Features |
|---------|--------|---------|--------------|
| `cloudshuttle-auth` | `use cloudshuttle_auth::*` | JWT & Security | Token creation/validation, password security |
| `cloudshuttle-database` | `use cloudshuttle_database::*` | Database Ops | Connection pooling, dynamic queries |
| `cloudshuttle-error-handling` | `use cloudshuttle_error_handling::*` | Error Handling | Structured errors, HTTP responses |
| `cloudshuttle-validation` | `use cloudshuttle_validation::*` | Input Validation | XSS/SQL injection prevention |
| `cloudshuttle-observability` | `use cloudshuttle_observability::*` | Monitoring | Metrics, tracing, logging |

---

## 💡 Most Common Usage Patterns

### Authentication Service
```rust
use cloudshuttle_auth::{JwtService, Claims, SecurityValidator};

let jwt_service = JwtService::new(b"your-secret-key")?;
let claims = Claims::new("user-123", "tenant-456");
let token = jwt_service.create_token(&claims)?;

// Password validation
let strength = SecurityValidator::validate_password_strength("MyPass123!")?;
// Hash password
let hash = SecurityValidator::hash_password("MyPass123!").await?;
```

### Database Operations
```rust
use cloudshuttle_database::{DatabaseConnection, QueryBuilder};

let db = DatabaseConnection::new("postgresql://localhost/mydb").await?;
let query = QueryBuilder::new("users")
    .select(vec!["id", "email"])
    .where_eq("active", true)
    .limit(10)
    .build();

let users = db.execute_query(query.0, &query.1).await?;
```

### Error Handling
```rust
use cloudshuttle_error_handling::CloudShuttleError;

fn process_data(data: &str) -> Result<(), CloudShuttleError> {
    if data.is_empty() {
        return Err(CloudShuttleError::Validation {
            field: "data".to_string(),
            message: "Data cannot be empty".to_string(),
        });
    }
    Ok(())
}
```

---

## 🔧 Feature Flags

Enable optional features as needed:

```toml
[dependencies]
cloudshuttle-auth = { git = "...", features = ["middleware"] }
cloudshuttle-database = { git = "...", features = ["postgres"] }
```

Available features:
- `middleware` - Axum middleware for authentication
- `postgres` - PostgreSQL database support
- `mysql` - MySQL database support

---

## 🔐 Private Repository Setup

### SSH Key Authentication (Recommended)
```bash
# Generate SSH key
ssh-keygen -t ed25519 -C "your-email@example.com"

# Add to GitHub account, then:
git clone git@github.com:your-org/cloudshuttle-shared.git
```

### Personal Access Token
```bash
# Set up credentials
git config --global credential.helper store
echo "https://username:token@github.com" > ~/.git-credentials
```

---

## 🧪 Testing with Shared Libraries

```rust
#[cfg(test)]
mod tests {
    use cloudshuttle_auth::{JwtService, SecurityValidator};

    #[test]
    fn test_auth_flow() {
        let jwt = JwtService::new(b"test-key").unwrap();
        let claims = Claims::new("user-123", "tenant-456");
        let token = jwt.create_token(&claims).unwrap();

        let validated = jwt.validate_token(&token).unwrap();
        assert_eq!(validated.sub, "user-123");
    }
}
```

---

## 🚀 CI/CD Integration

### GitHub Actions
```yaml
- uses: actions-rust-lang/setup-rust-toolchain@v1
- name: Cache Cargo
  uses: actions/cache@v3
  with:
    path: ~/.cargo/registry
    key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
- run: cargo test --workspace
```

---

## 🆘 Troubleshooting

**Build fails with "failed to load source for dependency"**
```bash
cargo clean
cargo update
# Check git access: git ls-remote https://github.com/your-org/cloudshuttle-shared.git
```

**Feature not enabled error**
```toml
# Add to Cargo.toml:
cloudshuttle-auth = { git = "...", features = ["middleware"] }
```

**Version conflicts**
```bash
rm Cargo.lock
cargo check
```

---

## 📋 Quick Checklist

- [ ] ✅ Add dependencies to `Cargo.toml`
- [ ] ✅ Set up git authentication for private repos
- [ ] ✅ Import required modules in your code
- [ ] ✅ Enable feature flags if needed
- [ ] ✅ Run `cargo check` to verify compilation
- [ ] ✅ Run `cargo test` to verify tests pass

---

*For detailed documentation, see `docs/consumption/README.md`*

**Happy coding with CloudShuttle!** 🚀✨
