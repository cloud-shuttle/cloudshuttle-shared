# 🚀 CloudShuttle Shared Libraries - Usage Guide

This guide shows you exactly how to use CloudShuttle shared libraries in your Rust projects using Git dependencies and tags.

## 📦 Quick Start - Add to Your Project

### Step 1: Add Dependencies to Cargo.toml

```toml
[dependencies]
# Core libraries you need
cloudshuttle-auth = { git = "https://github.com/cloud-shuttle/cloudshuttle-shared.git", tag = "v0.4.0" }
cloudshuttle-database = { git = "https://github.com/cloud-shuttle/cloudshuttle-shared.git", tag = "v0.4.0" }
cloudshuttle-error-handling = { git = "https://github.com/cloud-shuttle/cloudshuttle-shared.git", tag = "v0.4.0" }
cloudshuttle-api = { git = "https://github.com/cloud-shuttle/cloudshuttle-shared.git", tag = "v0.4.0" }

# Optional libraries based on your needs
cloudshuttle-config = { git = "https://github.com/cloud-shuttle/cloudshuttle-shared.git", tag = "v0.4.0" }
cloudshuttle-validation = { git = "https://github.com/cloud-shuttle/cloudshuttle-shared.git", tag = "v0.4.0" }
cloudshuttle-observability = { git = "https://github.com/cloud-shuttle/cloudshuttle-shared.git", tag = "v0.4.0" }
cloudshuttle-crypto = { git = "https://github.com/cloud-shuttle/cloudshuttle-shared.git", tag = "v0.4.0" }
```

### Step 2: Import and Use in Your Code

```rust
use cloudshuttle_auth::{JwtService, Claims};
use cloudshuttle_database::DatabaseConnection;
use cloudshuttle_error_handling::CloudShuttleError;
use cloudshuttle_api::{ApiResponse, ApiService};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create JWT service
    let jwt_service = JwtService::new(b"your-secret-key")?;

    // Connect to database
    let db = DatabaseConnection::new("postgresql://localhost/mydb").await?;

    // Create API service
    let api_service = ApiService::new();

    println!("CloudShuttle libraries ready!");
    Ok(())
}
```

### Step 3: Build and Test

```bash
# Build your project
cargo build

# Run tests
cargo test

# Update dependencies
cargo update
```

## 🏷️ Version Control & Tags

### Using Specific Tags (Production Recommended)

```toml
[dependencies]
# Pin to specific released version
cloudshuttle-api = { git = "https://github.com/cloud-shuttle/cloudshuttle-shared.git", tag = "v0.4.0" }
cloudshuttle-auth = { git = "https://github.com/cloud-shuttle/cloudshuttle-shared.git", tag = "v0.4.0" }
cloudshuttle-database = { git = "https://github.com/cloud-shuttle/cloudshuttle-shared.git", tag = "v0.4.0" }
```

### Using Branches (Development)

```toml
[dependencies]
# Use latest main branch (may have breaking changes)
cloudshuttle-api = { git = "https://github.com/cloud-shuttle/cloudshuttle-shared.git", branch = "main" }
```

### Using Specific Commits (Maximum Stability)

```toml
[dependencies]
# Pin to exact commit for maximum stability
cloudshuttle-api = { git = "https://github.com/cloud-shuttle/cloudshuttle-shared.git", rev = "b4b2d6c" }
```

## 🔑 Available Libraries & Their Purpose

| Library | Purpose | Key Features | When to Use |
|---------|---------|--------------|-------------|
| `cloudshuttle-api` | Complete API Framework | Response formatting, rate limiting, CORS, OpenAPI docs, tracing | Building REST APIs |
| `cloudshuttle-auth` | Authentication & JWT | Token creation/validation, password security, middleware | User authentication |
| `cloudshuttle-database` | Database Operations | Connection pooling, query helpers, transactions, migrations | Database access |
| `cloudshuttle-error-handling` | Error Management | Structured errors, HTTP status mapping, service errors | Error handling |
| `cloudshuttle-config` | Configuration | Environment loading, validation, secrets, hot reloading | App configuration |
| `cloudshuttle-validation` | Input Validation | XSS/SQL prevention, sanitization, custom validators | Data validation |
| `cloudshuttle-observability` | Monitoring & Logging | Metrics, tracing, health checks, audit logging | Observability |
| `cloudshuttle-crypto` | Cryptography | Password hashing, encryption, secure tokens | Security operations |

## ⚙️ Feature Flags

Enable optional features as needed:

```toml
[dependencies]
# Enable Axum middleware for authentication
cloudshuttle-auth = { git = "https://github.com/cloud-shuttle/cloudshuttle-shared.git", tag = "v0.4.0", features = ["middleware"] }

# Enable PostgreSQL support
cloudshuttle-database = { git = "https://github.com/cloud-shuttle/cloudshuttle-shared.git", tag = "v0.4.0", features = ["postgres"] }

# Enable observability features
cloudshuttle-observability = { git = "https://github.com/cloud-shuttle/cloudshuttle-shared.git", tag = "v0.4.0", features = ["axum"] }
```

## 🔧 Development Setup (Local Development)

For local development with path dependencies:

```toml
[dependencies]
# Use local paths during development
cloudshuttle-api = { path = "../cloudshuttle-shared/rust/crates/api" }
cloudshuttle-auth = { path = "../cloudshuttle-shared/rust/crates/auth" }
cloudshuttle-database = { path = "../cloudshuttle-shared/rust/crates/database" }
```

## 📋 Complete Example Project

### Cargo.toml
```toml
[package]
name = "my-cloudshuttle-service"
version = "0.1.0"
edition = "2021"

[dependencies]
# CloudShuttle shared libraries
cloudshuttle-api = { git = "https://github.com/cloud-shuttle/cloudshuttle-shared.git", tag = "v0.4.0" }
cloudshuttle-auth = { git = "https://github.com/cloud-shuttle/cloudshuttle-shared.git", tag = "v0.4.0", features = ["middleware"] }
cloudshuttle-database = { git = "https://github.com/cloud-shuttle/cloudshuttle-shared.git", tag = "v0.4.0", features = ["postgres"] }
cloudshuttle-error-handling = { git = "https://github.com/cloud-shuttle/cloudshuttle-shared.git", tag = "v0.4.0" }
cloudshuttle-config = { git = "https://github.com/cloud-shuttle/cloudshuttle-shared.git", tag = "v0.4.0" }

# Web framework
axum = "0.8"
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tower = "0.5"
tower-http = "0.5"
```

### main.rs
```rust
use axum::{routing::get, Router};
use cloudshuttle_api::{ApiResponse, ApiService};
use cloudshuttle_auth::AuthMiddleware;
use cloudshuttle_config::ConfigLoader;
use cloudshuttle_database::DatabaseConnection;

#[derive(serde::Deserialize)]
struct AppConfig {
    database_url: String,
    jwt_secret: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config: AppConfig = ConfigLoader::new("my-service").load()?;

    // Connect to database
    let db = DatabaseConnection::new(&config.database_url).await?;

    // Create API service
    let api_service = ApiService::new().require_auth();

    // Build router with authentication
    let app = Router::new()
        .route("/health", get(health_check))
        .layer(AuthMiddleware::new(&config.jwt_secret));

    println!("🚀 Server starting on http://localhost:3000");
    axum::serve(app, ([127, 0, 0, 1], 3000)).await?;

    Ok(())
}

async fn health_check() -> ApiResponse<String> {
    ApiResponse::success("Service is healthy!".to_string())
}
```

## 🔐 Authentication for Private Repositories

### SSH Key Setup (Recommended)
```bash
# Generate SSH key
ssh-keygen -t ed25519 -C "your-email@example.com"

# Add public key to GitHub repository
# Then use SSH URLs:
git@github.com:cloud-shuttle/cloudshuttle-shared.git
```

### Personal Access Token
```bash
# Set up git credentials
git config --global credential.helper store
echo "https://username:token@github.com" > ~/.git-credentials
```

## 📊 Releases & Version History

### Current Release: v0.4.0
- ✅ Complete API utilities framework
- ✅ Production-ready rate limiting, CORS, tracing
- ✅ OpenAPI documentation generation
- ✅ Enterprise-grade error handling

### Previous Releases
- v0.3.0 - Authentication & database foundations
- v0.2.0 - Core error handling & validation
- v0.1.0 - Initial shared library structure

## 🆘 Troubleshooting

### "failed to load source for dependency"
```bash
# Clean and update
cargo clean
cargo update

# Check git access
git ls-remote https://github.com/cloud-shuttle/cloudshuttle-shared.git
```

### Version conflicts
```bash
# Regenerate lock file
rm Cargo.lock
cargo check
```

### Authentication issues
```bash
# Verify credentials
git config --global --list | grep credential
```

## 📚 Additional Resources

- **[CRATES_OVERVIEW.md](CRATES_OVERVIEW.md)** - Detailed crate descriptions and examples
- **[docs/consumption/README.md](docs/consumption/README.md)** - Comprehensive consumption guide
- **[docs/consumption/QUICK_REFERENCE.md](docs/consumption/QUICK_REFERENCE.md)** - Quick reference guide
- **[GitHub Releases](https://github.com/cloud-shuttle/cloudshuttle-shared/releases)** - Release notes and changelogs

---

## 🎯 Getting Started Checklist

- [ ] ✅ Add CloudShuttle dependencies to `Cargo.toml`
- [ ] ✅ Set up Git authentication (SSH key or token)
- [ ] ✅ Import required modules in your code
- [ ] ✅ Enable feature flags if needed
- [ ] ✅ Run `cargo build` to verify compilation
- [ ] ✅ Run `cargo test` to verify tests pass

**Happy coding with CloudShuttle shared libraries!** 🚀✨
