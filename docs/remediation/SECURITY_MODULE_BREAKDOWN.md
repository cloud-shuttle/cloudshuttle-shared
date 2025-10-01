# 🔐 Security Module Breakdown Plan

**File**: `rust/crates/auth/src/security.rs` (439 lines)
**Target**: 4 focused modules under 300 lines each
**Status**: 🚨 CRITICAL - Immediate breakdown required

## 📊 Current Analysis

### **Single Responsibility Violations**
The `security.rs` file currently handles:
1. **Password Policy** (strength validation, scoring)
2. **Input Sanitization** (XSS, SQL injection prevention)
3. **Rate Limiting** (request throttling)
4. **Encryption Utilities** (token generation, hashing)
5. **Security Validation** (length checks, format validation)
6. **Audit Logging** (security event tracking)
7. **Password Hashing** (Argon2 implementation)

### **Dependencies & Complexity**
- **External Crates**: `ring`, `argon2`, `base64`, `regex`
- **Internal Dependencies**: `AuthError`, `AuthResult`, `SecurityValidator`
- **Complexity**: 7 distinct security domains in one file

## 🏗️ Breakdown Architecture

### **Module 1: `password_policy.rs` (120 lines)**
**Responsibility**: Password strength validation and policy enforcement

```rust
//! Password policy management and strength validation

use crate::types::{AuthResult, AuthError};

/// Password strength assessment
#[derive(Debug, Clone)]
pub struct PasswordStrength {
    pub score: u8,
    pub is_strong: bool,
    pub has_uppercase: bool,
    pub has_lowercase: bool,
    pub has_digit: bool,
    pub has_special: bool,
}

/// Password policy validator
pub struct PasswordPolicy;

impl PasswordPolicy {
    /// Validate password strength according to security policies
    pub fn validate_password_strength(password: &str) -> AuthResult<PasswordStrength> {
        // Implementation
    }

    /// Calculate password entropy score
    pub fn calculate_entropy(password: &str) -> f64 {
        // Implementation
    }

    /// Check password against common patterns
    pub fn check_common_patterns(password: &str) -> bool {
        // Implementation
    }
}
```

### **Module 2: `input_sanitization.rs` (120 lines)**
**Responsibility**: Input validation and sanitization against XSS/SQL injection

```rust
//! Input sanitization and validation utilities

use crate::types::{AuthResult, AuthError};

/// Input sanitization validator
pub struct InputSanitizer;

impl InputSanitizer {
    /// Sanitize HTML input to prevent XSS attacks
    pub fn sanitize_html(input: &str) -> String {
        // Implementation
    }

    /// Sanitize SQL input to prevent injection attacks
    pub fn sanitize_sql(input: &str) -> String {
        // Implementation
    }

    /// Validate filename for path traversal attacks
    pub fn validate_filename(filename: &str) -> AuthResult<()> {
        // Implementation
    }

    /// Sanitize URL input
    pub fn sanitize_url(url: &str) -> AuthResult<String> {
        // Implementation
    }
}
```

### **Module 3: `rate_limiting.rs` (80 lines)**
**Responsibility**: Request rate limiting and throttling

```rust
//! Rate limiting and request throttling

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Rate limiter for request throttling
pub struct RateLimiter {
    attempts: HashMap<String, Vec<Instant>>,
    max_attempts: u32,
    window_duration: Duration,
}

impl RateLimiter {
    /// Create new rate limiter
    pub fn new(max_attempts: u32, window_seconds: u64) -> Self {
        // Implementation
    }

    /// Check if request is allowed
    pub fn check_rate_limit(&mut self, key: &str) -> bool {
        // Implementation
    }

    /// Clean up expired entries
    pub fn cleanup(&mut self) {
        // Implementation
    }
}
```

### **Module 4: `encryption.rs` (80 lines)**
**Responsibility**: Encryption utilities and secure token generation

```rust
//! Encryption utilities and secure token generation

use ring::rand::SystemRandom;
use ring::signature::{Ed25519KeyPair, KeyPair as RingKeyPair};
use crate::types::{AuthResult, AuthError};

/// Cryptographic utilities
pub struct CryptoUtils;

impl CryptoUtils {
    /// Generate secure random token
    pub fn generate_secure_token(length: usize) -> AuthResult<String> {
        // Implementation
    }

    /// Generate cryptographically secure random bytes
    pub fn generate_random_bytes(length: usize) -> AuthResult<Vec<u8>> {
        // Implementation
    }

    /// Hash data using SHA-256
    pub fn sha256_hash(data: &[u8]) -> String {
        // Implementation
    }

    /// Hash password using Argon2
    pub fn hash_password(password: &str) -> AuthResult<String> {
        // Implementation
    }

    /// Verify password against hash
    pub fn verify_password(password: &str, hash: &str) -> AuthResult<bool> {
        // Implementation
    }
}
```

## 🔄 Refactoring Steps

### **Step 1: Extract Password Policy**
```bash
# Move password-related code to password_policy.rs
grep -A 50 "validate_password_strength" security.rs > password_policy.rs
# Add required imports and struct definitions
```

### **Step 2: Extract Input Sanitization**
```bash
# Move sanitization code to input_sanitization.rs
grep -A 30 "sanitize_html\|sanitize_sql" security.rs > input_sanitization.rs
# Add XSS and SQL injection prevention logic
```

### **Step 3: Extract Rate Limiting**
```bash
# Move rate limiting code to rate_limiting.rs
grep -A 20 "check_rate_limit" security.rs > rate_limiting.rs
# Add HashMap-based rate limiting implementation
```

### **Step 4: Extract Encryption Utils**
```bash
# Move encryption code to encryption.rs
grep -A 30 "generate_secure_token\|hash_password" security.rs > encryption.rs
# Add ring and argon2 dependencies
```

### **Step 5: Update Main Security Module**
```rust
//! Main security validator - orchestrates all security modules

pub mod password_policy;
pub mod input_sanitization;
pub mod rate_limiting;
pub mod encryption;

pub use password_policy::PasswordPolicy;
pub use input_sanitization::InputSanitizer;
pub use rate_limiting::RateLimiter;
pub use encryption::CryptoUtils;

/// Main security validator
pub struct SecurityValidator;

impl SecurityValidator {
    /// Comprehensive security validation
    pub fn validate_input(&self, input: &str) -> AuthResult<()> {
        // Orchestrate all security checks
        InputSanitizer::sanitize_html(input);
        // ... coordinate between modules
    }
}
```

## 🧪 Testing Strategy

### **Unit Tests per Module**
- **password_policy.rs**: 15 test cases (strength validation, entropy calculation)
- **input_sanitization.rs**: 20 test cases (XSS prevention, SQL injection)
- **rate_limiting.rs**: 10 test cases (throttling logic, cleanup)
- **encryption.rs**: 12 test cases (token generation, password hashing)

### **Integration Tests**
- **Cross-module validation**: Ensure modules work together
- **Security property testing**: Proptest for edge cases
- **Performance testing**: Benchmark cryptographic operations

## 📋 Implementation Checklist

- [ ] Create `password_policy.rs` (120 lines max)
- [ ] Create `input_sanitization.rs` (120 lines max)
- [ ] Create `rate_limiting.rs` (80 lines max)
- [ ] Create `encryption.rs` (80 lines max)
- [ ] Update `security.rs` to orchestrate modules (50 lines max)
- [ ] Update `mod.rs` to expose new modules
- [ ] Update all import statements across codebase
- [ ] Run compilation tests
- [ ] Run unit test suites
- [ ] Update documentation

## 🎯 Success Metrics

- ✅ **File sizes**: All modules under 120 lines
- ✅ **Compilation**: Zero errors after refactoring
- ✅ **Test coverage**: 95%+ coverage maintained
- ✅ **API compatibility**: Public API unchanged
- ✅ **Performance**: No regression in security operations

## 🚨 Risk Mitigation

- **Breaking Changes**: Use feature flags during transition
- **Testing**: Comprehensive test suite before/after refactoring
- **Documentation**: Update all usage examples
- **CI/CD**: Automated testing of all security modules
