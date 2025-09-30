# CloudShuttle Shared Libraries

> Reusable code components that eliminate duplication and ensure consistency across all CloudShuttle services

## Overview

The `cloudshuttle-shared` repository contains shared libraries and utilities used across all CloudShuttle services. This repository eliminates code duplication, ensures consistency, and provides a centralized location for common functionality.

## Repository Structure

```
cloudshuttle-shared/
├── rust/                          # Rust shared libraries
│   ├── Cargo.toml                # Workspace configuration
│   └── crates/                   # Individual Rust crates
│       ├── error-handling/       # Error types and handling
│       ├── database/            # Database utilities
│       ├── auth/                # Authentication helpers
│       ├── observability/       # Logging & metrics
│       ├── config/              # Configuration management
│       ├── api/                 # API utilities
│       ├── validation/          # Input validation
│       └── crypto/              # Cryptographic utilities
├── typescript/                   # TypeScript shared libraries
│   ├── package.json             # Workspace configuration
│   └── packages/                # Individual TypeScript packages
│       ├── components/          # React components
│       ├── hooks/               # React hooks
│       ├── types/               # TypeScript types
│       ├── utils/               # Utility functions
│       ├── api/                 # API client utilities
│       └── stores/              # State management
├── docs/                        # Documentation
│   ├── rust-libraries.md        # Rust library guide
│   ├── typescript-libraries.md  # TypeScript library guide
│   ├── versioning.md            # Versioning strategy
│   └── contributing.md          # Contribution guidelines
├── .github/                     # CI/CD workflows
│   └── workflows/
│       ├── ci.yml               # CI pipeline
│       ├── publish.yml          # Automated publishing
│       └── security.yml         # Security scanning
├── scripts/                     # Build and release scripts
│   ├── build-all.sh            # Build all libraries
│   ├── test-all.sh             # Test all libraries
│   └── release.sh              # Automated release
└── README.md                   # This file
```

## Rust Libraries

### 1. Error Handling (`cloudshuttle-error-handling`)

**Purpose**: Standardized error types and handling patterns

**Features**:
- `CloudShuttleError` - Main error enum for all services
- `ServiceError` trait - Service-specific error handling
- `ApiError` - HTTP API error responses
- `DatabaseError` - Database operation errors

**Usage**:
```rust
use cloudshuttle_error_handling::{CloudShuttleError, ServiceError};

// Implement service-specific errors
#[derive(Debug, thiserror::Error)]
pub struct MyServiceError {
    message: String,
}

impl ServiceError for MyServiceError {
    fn error_code(&self) -> &'static str { "MY_SERVICE_ERROR" }
    fn http_status(&self) -> http::StatusCode { http::StatusCode::INTERNAL_SERVER_ERROR }
    fn user_message(&self) -> String { self.message.clone() }
}
```

### 2. Database (`cloudshuttle-database`)

**Purpose**: Common database utilities and connection management

**Features**:
- Connection pooling and management
- Query builders and helpers
- Transaction management
- Migration utilities

### 3. Authentication (`cloudshuttle-auth`)

**Purpose**: JWT handling and authentication utilities

**Features**:
- JWT token creation and validation
- Claims structure and management
- Authentication middleware
- Key management utilities

### 4. Observability (`cloudshuttle-observability`)

**Purpose**: Logging, metrics, and tracing utilities

**Features**:
- Structured logging with context
- Prometheus metrics collection
- Distributed tracing support
- Health check utilities

### 5. Configuration (`cloudshuttle-config`)

**Purpose**: Configuration loading and validation

**Features**:
- Environment-based configuration
- Validation with detailed error messages
- Secret management
- Configuration reloading

### 6. API (`cloudshuttle-api`)

**Purpose**: Common API utilities and response formatting

**Features**:
- Standardized API responses
- Pagination support
- Filtering and sorting utilities
- Request validation

### 7. Validation (`cloudshuttle-validation`)

**Purpose**: Input validation and sanitization

**Features**:
- Common validation rules
- Input sanitization
- Custom validators
- Validation error formatting

### 8. Crypto (`cloudshuttle-crypto`)

**Purpose**: Cryptographic utilities for secure operations

**Features**:
- Password hashing
- Data encryption/decryption
- Digital signatures
- Secure random generation

## TypeScript Libraries

### 1. Components (`@cloudshuttle/components`)

**Purpose**: Reusable React components

**Features**:
- Button, Input, Modal components
- Form components
- Layout components
- Table and data display components

### 2. Hooks (`@cloudshuttle/hooks`)

**Purpose**: Custom React hooks

**Features**:
- `useApi` - API call management
- `useAuth` - Authentication state
- `useLocalStorage` - Local storage persistence
- `usePagination` - Pagination logic

### 3. Types (`@cloudshuttle/types`)

**Purpose**: Shared TypeScript type definitions

**Features**:
- API response types
- User and tenant types
- Form and validation types
- Common utility types

### 4. Utils (`@cloudshuttle/utils`)

**Purpose**: Utility functions

**Features**:
- Data formatters
- Date/time utilities
- String manipulation
- Array and object helpers

### 5. API (`@cloudshuttle/api`)

**Purpose**: API client utilities

**Features**:
- HTTP client configuration
- Request/response interceptors
- Error handling
- Authentication headers

### 6. Stores (`@cloudshuttle/stores`)

**Purpose**: State management stores

**Features**:
- Authentication store
- User preferences store
- Notification store
- Application state stores

## Versioning Strategy

### Semantic Versioning
```
MAJOR.MINOR.PATCH

MAJOR: Breaking changes
MINOR: New features (backward compatible)
PATCH: Bug fixes (backward compatible)
```

### Publishing Cadence
- **Major releases**: Breaking changes, coordinated across services
- **Minor releases**: New features, monthly
- **Patch releases**: Bug fixes, as needed

### Dependency Management
```toml
# In service Cargo.toml
[dependencies]
cloudshuttle-error-handling = "1.2.3"
cloudshuttle-database = "1.1.0"
```

## Development Setup

### Prerequisites
- **Rust**: 1.75+ with Cargo
- **Node.js**: 18+ with npm or pnpm
- **Git**: For repository management

### Local Development
```bash
# Clone the repository
git clone https://github.com/cloudshuttle/cloudshuttle-shared.git
cd cloudshuttle-shared

# Rust development
cd rust
cargo build
cargo test

# TypeScript development
cd ../typescript
pnpm install
pnpm build
pnpm test
```

### Building All Libraries
```bash
# Use the build script
./scripts/build-all.sh

# Or build manually
cd rust && cargo build --release
cd ../typescript && pnpm build
```

### Running Tests
```bash
# Run all tests
./scripts/test-all.sh

# Or test individually
cd rust && cargo test
cd ../typescript && pnpm test
```

## Publishing

### Git-Based Distribution (Private)
For security and IP protection, shared libraries are distributed via Git dependencies rather than public registries:

```bash
# Create and push a version tag
git tag v1.2.3
git push origin v1.2.3

# CI/CD validates all libraries compile and tests pass
# Libraries remain private in this GitHub repository
```

### Usage in Services
```toml
# In service Cargo.toml
[dependencies]
cloudshuttle-error-handling = { git = "https://github.com/cloudshuttle/cloudshuttle-shared.git", tag = "v1.2.3" }
cloudshuttle-database = { git = "https://github.com/cloudshuttle/cloudshuttle-shared.git", tag = "v1.2.3" }
```

```json
// In package.json
{
  "dependencies": {
    "@cloudshuttle/components": "github:cloudshuttle/cloudshuttle-shared#v1.2.3"
  }
}
```

### Security Note
These libraries contain authentication, cryptographic, and business logic that must remain private. They are distributed via Git dependencies to maintain security and IP protection.

## Contributing

### Code Standards
- **Rust**: Follow Rust API guidelines and idioms
- **TypeScript**: Follow Airbnb style guide
- **Testing**: 80%+ code coverage required
- **Documentation**: All public APIs documented

### Pull Request Process
1. Create a feature branch
2. Implement changes with tests
3. Update documentation
4. Create pull request
5. Code review and approval
6. Automated testing passes
7. Merge and tag release

### Adding New Libraries
1. Create the library structure
2. Add to workspace configuration
3. Implement comprehensive tests
4. Add documentation
5. Update CI/CD pipelines

## Migration Guide

### From Monolithic Services
1. **Identify duplicated code** in current services
2. **Extract to shared libraries** following the patterns above
3. **Update service dependencies** to use shared libraries
4. **Test integration** and compatibility
5. **Remove duplicated code** from services

### Example Migration
```rust
// Before: Duplicated error types
#[derive(Debug, thiserror::Error)]
pub enum MyServiceError {
    #[error("Not found: {resource}")]
    NotFound { resource: String },
    // ... more duplicated error types
}

// After: Use shared error types
use cloudshuttle_error_handling::ApiError;

impl From<ApiError> for MyServiceError {
    fn from(error: ApiError) -> Self {
        match error {
            ApiError::NotFound { resource } => MyServiceError::NotFound { resource },
            // ... map other errors
        }
    }
}
```

## Support

### Documentation
- [Rust Libraries Guide](docs/rust-libraries.md)
- [TypeScript Libraries Guide](docs/typescript-libraries.md)
- [Versioning Strategy](docs/versioning.md)
- [Contributing Guide](docs/contributing.md)

### Issues and Support
- **GitHub Issues**: Bug reports and feature requests
- **Discussions**: Questions and community support
- **Documentation**: Comprehensive API documentation

---

## Quality Assurance

### Automated Testing
- **Unit Tests**: All public APIs tested
- **Integration Tests**: Cross-library compatibility
- **Contract Tests**: API compatibility validation
- **Performance Tests**: Benchmarking and optimization

### Code Quality
- **Linting**: Clippy for Rust, ESLint for TypeScript
- **Formatting**: rustfmt and Prettier
- **Security**: Automated security scanning
- **Coverage**: 80%+ test coverage required

### Release Quality
- **Automated Testing**: All tests pass before release
- **API Compatibility**: Breaking changes flagged
- **Documentation**: Updated for all changes
- **Changelog**: Release notes for all changes

This shared libraries repository serves as the foundation for consistent, maintainable, and high-quality code across the entire CloudShuttle platform.


