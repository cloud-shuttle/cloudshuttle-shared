`# 🔑 JWT Module Breakdown Plan

**File**: `rust/crates/auth/src/jwt.rs` (407 lines)
**Target**: 3 focused modules under 300 lines each
**Status**: 🚨 CRITICAL - Immediate breakdown required

## 📊 Current Analysis

### **Single Responsibility Violations**
The `jwt.rs` file currently handles:
1. **Token Operations** (create, validate, refresh)
2. **Claims Management** (structure, validation, custom claims)
3. **Key Management** (algorithm selection, key validation)
4. **Error Handling** (JWT-specific errors)
5. **Algorithm Support** (HS256, RS256, ES256)
6. **Token Parsing** (decode, verify, extract claims)

### **Dependencies & Complexity**
- **External Crates**: `jsonwebtoken`, `serde`, `base64`
- **Internal Dependencies**: `Claims`, `AuthError`, `AuthResult`
- **Complexity**: Token lifecycle management across multiple algorithms

## 🏗️ Breakdown Architecture

### **Module 1: `token_operations.rs` (150 lines)**
**Responsibility**: Core JWT token creation, validation, and refresh operations

```rust
//! JWT token creation, validation, and refresh operations

use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation, Algorithm};
use crate::types::{AuthResult, AuthError};
use crate::{Claims, JwtAlgorithm};

/// JWT token service
pub struct TokenService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    algorithm: Algorithm,
}

impl TokenService {
    /// Create new token service with specified algorithm
    pub fn new(secret: &[u8], algorithm: JwtAlgorithm) -> AuthResult<Self> {
        // Implementation
    }

    /// Create JWT token from claims
    pub fn create_token(&self, claims: &Claims) -> AuthResult<String> {
        // Implementation
    }

    /// Validate and decode JWT token
    pub fn validate_token(&self, token: &str) -> AuthResult<Claims> {
        // Implementation
    }

    /// Refresh token with new expiration
    pub fn refresh_token(&self, token: &str, new_expiry: i64) -> AuthResult<String> {
        // Implementation
    }
}
```

### **Module 2: `claims_management.rs` (120 lines)**
**Responsibility**: JWT claims structure, validation, and custom claim handling

```rust
//! JWT claims structure and validation

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// JWT claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Expiration time
    pub exp: i64,
    /// Issued at time
    pub iat: i64,
    /// Tenant identifier
    pub tenant_id: String,
    /// User roles
    #[serde(default)]
    pub roles: Vec<String>,
    /// Custom claims
    #[serde(flatten)]
    pub custom: std::collections::HashMap<String, serde_json::Value>,
}

impl Claims {
    /// Create new claims for user
    pub fn new(sub: &str, tenant_id: &str) -> Self {
        // Implementation
    }

    /// Add role to claims
    pub fn with_role(mut self, role: &str) -> Self {
        // Implementation
    }

    /// Add custom claim
    pub fn with_custom_claim(mut self, key: &str, value: serde_json::Value) -> Self {
        // Implementation
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        // Implementation
    }

    /// Validate claims structure
    pub fn validate(&self) -> AuthResult<()> {
        // Implementation
    }
}
```

### **Module 3: `key_management.rs` (100 lines)**
**Responsibility**: JWT algorithm selection and cryptographic key management

```rust
//! JWT algorithm and key management

use ring::signature::{Ed25519KeyPair, KeyPair as RingKeyPair};
use jsonwebtoken::{Algorithm, EncodingKey, DecodingKey};
use crate::types::{AuthResult, AuthError};

/// Supported JWT algorithms
#[derive(Debug, Clone, Copy)]
pub enum JwtAlgorithm {
    HS256,
    RS256,
    ES256,
}

impl JwtAlgorithm {
    /// Convert to jsonwebtoken Algorithm
    pub fn to_algorithm(self) -> Algorithm {
        // Implementation
    }

    /// Get algorithm name
    pub fn as_str(&self) -> &'static str {
        // Implementation
    }
}

/// Key management for JWT operations
pub struct KeyManager {
    algorithm: JwtAlgorithm,
    encoding_key: Option<EncodingKey>,
    decoding_key: Option<DecodingKey>,
}

impl KeyManager {
    /// Create key manager for symmetric algorithms (HS256)
    pub fn new_hmac(secret: &[u8]) -> AuthResult<Self> {
        // Implementation
    }

    /// Create key manager for asymmetric algorithms (RS256, ES256)
    pub fn new_asymmetric(public_key: &[u8], private_key: &[u8], algorithm: JwtAlgorithm) -> AuthResult<Self> {
        // Implementation
    }

    /// Generate new key pair for asymmetric algorithms
    pub fn generate_keypair(algorithm: JwtAlgorithm) -> AuthResult<(Vec<u8>, Vec<u8>)> {
        // Implementation
    }

    /// Get encoding key for token creation
    pub fn encoding_key(&self) -> &EncodingKey {
        // Implementation
    }

    /// Get decoding key for token validation
    pub fn decoding_key(&self) -> &DecodingKey {
        // Implementation
    }
}
```

## 🔄 Refactoring Steps

### **Step 1: Extract Token Operations**
```bash
# Move token operation code to token_operations.rs
grep -A 100 "create_token\|validate_token" jwt.rs > token_operations.rs
# Add TokenService struct and implementation
```

### **Step 2: Extract Claims Management**
```bash
# Move claims-related code to claims_management.rs
grep -A 50 "Claims" jwt.rs > claims_management.rs
# Add Claims struct and validation logic
```

### **Step 3: Extract Key Management**
```bash
# Move key management code to key_management.rs
grep -A 50 "KeyManager\|JwtAlgorithm" jwt.rs > key_management.rs
# Add algorithm enum and key management
```

### **Step 4: Update Main JWT Module**
```rust
//! JWT token management - orchestrates all JWT operations

pub mod token_operations;
pub mod claims_management;
pub mod key_management;

pub use token_operations::TokenService;
pub use claims_management::Claims;
pub use key_management::{KeyManager, JwtAlgorithm};

/// Legacy JWT service for backward compatibility
pub struct JwtService {
    token_service: TokenService,
}

impl JwtService {
    /// Create new JWT service (backward compatibility)
    pub fn new(secret: &[u8]) -> AuthResult<Self> {
        let token_service = TokenService::new(secret, JwtAlgorithm::HS256)?;
        Ok(Self { token_service })
    }

    /// Create token (backward compatibility)
    pub fn create_token(&self, claims: &Claims) -> AuthResult<String> {
        self.token_service.create_token(claims)
    }

    /// Validate token (backward compatibility)
    pub fn validate_token(&self, token: &str) -> AuthResult<Claims> {
        self.token_service.validate_token(token)
    }
}
```

## 🧪 Testing Strategy

### **Unit Tests per Module**
- **token_operations.rs**: 15 test cases (token creation, validation, refresh)
- **claims_management.rs**: 12 test cases (claims validation, custom claims, expiration)
- **key_management.rs**: 8 test cases (algorithm selection, key generation)

### **Integration Tests**
- **End-to-end JWT flow**: Token creation → validation → refresh
- **Algorithm compatibility**: Test all supported algorithms
- **Key rotation**: Test key management scenarios

### **Property-Based Tests**
- **Token validation**: Edge cases for malformed tokens
- **Claims validation**: Invalid claim structures
- **Algorithm validation**: Unsupported algorithm handling

## 📋 Implementation Checklist

- [ ] Create `token_operations.rs` (150 lines max)
- [ ] Create `claims_management.rs` (120 lines max)
- [ ] Create `key_management.rs` (100 lines max)
- [ ] Update `jwt.rs` for backward compatibility (50 lines max)
- [ ] Update `mod.rs` to expose new modules
- [ ] Update all import statements across codebase
- [ ] Run compilation tests
- [ ] Run JWT test suites
- [ ] Update documentation

## 🎯 Success Metrics

- ✅ **File sizes**: All modules under specified limits
- ✅ **Compilation**: Zero errors after refactoring
- ✅ **API compatibility**: Existing JWT usage unchanged
- ✅ **Test coverage**: 95%+ coverage maintained
- ✅ **Performance**: No regression in token operations

## 🚨 Risk Mitigation

- **Backward Compatibility**: Maintain existing JwtService API
- **Migration Path**: Feature flags for gradual adoption
- **Security**: Ensure cryptographic operations remain secure
- **Performance**: Benchmark before/after refactoring
