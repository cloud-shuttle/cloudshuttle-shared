# 🔄 Dependency Modernization Plan
## Updating CloudShuttle Shared Components to Latest Rust Ecosystem

**Priority**: CRITICAL (Blocks all other improvements)
**Timeline**: Week 1-2
**Impact**: Enables compilation with modern services

---

## 🎯 **Problem Statement**

The shared repository uses outdated dependency versions that conflict with modern Rust services:

- **SQLx**: 0.7.0 → Required: 0.8.x (breaking changes)
- **Axum**: 0.7.x → Latest: 0.8.x (API changes)
- **Base64**: Old API → New engine-based API
- **Various ecosystem crates**: Multiple version conflicts

**Result**: Cannot compile alongside services using current ecosystem versions.

---

## 📊 **Current Dependency Analysis**

### **Workspace Dependencies** (`Cargo.toml`)
```toml
[workspace.dependencies]
# Database
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio-rustls", "chrono", "uuid", "json"] }

# Authentication
jsonwebtoken = "9.3"
bcrypt = "0.15"
rand = "0.8"  # ← Conflicts with modern rand 0.9
base64 = "0.22"

# Web framework
axum = "0.7"  # ← Outdated, should be 0.8
tower = "0.5"
hyper = "1.4"

# Observability
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
prometheus = "0.13"
```

### **Individual Crate Issues**

#### **cloudshuttle-database**
- Uses SQLx 0.7 internally despite workspace specifying 0.8
- SQLite linking conflicts with modern SQLx versions
- Missing runtime features for modern async ecosystems

#### **cloudshuttle-auth**
- Module conflicts: `types.rs` vs `types/mod.rs`
- Missing exports: `AuthResult`, `AuthError`, `AuthTokens`
- Deprecated base64 API: `encode_config()` → new engine API
- Uses old rand API patterns

#### **cloudshuttle-validation**
- Validator version conflicts (0.18 vs 0.20)
- Missing modern sanitization features
- Unicode handling inconsistencies

---

## 🛠️ **Modernization Strategy**

### **Phase 1: Dependency Updates**

#### **Update Workspace Dependencies**
```toml
[workspace.dependencies]
# Core async runtime
tokio = { version = "1.40", features = ["full"] }

# Database - CRITICAL UPDATE
sqlx = { version = "0.8", features = [
    "postgres", "runtime-tokio-rustls", "chrono", "uuid", "json",
    "migrate", "macros", "offline"
]}

# Web framework - MAJOR UPDATE
axum = "0.8"
tower = "0.5"
tower-http = "0.6"
hyper = "1.7"

# Authentication - UPDATE REQUIRED
jsonwebtoken = "10.0"
argon2 = { version = "0.5", features = ["std"] }
rand = "0.9"
base64 = "0.22"

# Validation
validator = { version = "0.20", features = ["derive"] }

# Error handling
thiserror = "2.0"
anyhow = "1.0"

# Observability
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
prometheus = "0.13"
```

#### **Rust Version Update**
```toml
rust-version = "1.89"  # Already current
edition = "2021"       # Consider 2024 edition in future
```

### **Phase 2: API Modernization**

#### **Base64 API Migration**
```rust
// OLD (broken)
base64::encode_config(bytes, base64::URL_SAFE_NO_PAD)

// NEW (works)
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
URL_SAFE_NO_PAD.encode(bytes)
```

#### **Rand API Updates**
```rust
// OLD (deprecated)
rand::thread_rng()

// NEW (current)
rand::rng()
```

#### **Axum API Changes**
```rust
// OLD (Axum 0.7)
use axum::extract::Extension;
use axum::response::IntoResponse;

// NEW (Axum 0.8) - Mostly compatible, but check breaking changes
use axum::extract::Request;
use axum::response::Response;
```

### **Phase 3: SQLx Migration**

#### **Update Query Macros**
```rust
// OLD (SQLx 0.7)
sqlx::query!("SELECT * FROM users WHERE id = $1", user_id)

// NEW (SQLx 0.8) - Arguments changed
sqlx::query!("SELECT * FROM users WHERE id = $1")
    .bind(user_id)
```

#### **Connection Pool Configuration**
```rust
// OLD (SQLx 0.7)
PgPoolOptions::new()
    .max_connections(10)

// NEW (SQLx 0.8) - API mostly same, but verify
PgPoolOptions::new()
    .max_connections(10)
```

---

## 🔧 **Implementation Steps**

### **Step 1: Update Workspace Dependencies**
1. Update `Cargo.toml` workspace dependencies
2. Run `cargo update` to resolve new versions
3. Fix any immediate compilation errors

### **Step 2: Fix Individual Crates**

#### **cloudshuttle-auth**
1. **Resolve module conflicts**:
   ```bash
   # Either rename or consolidate
   mv types.rs types/mod.rs  # OR
   rm types/mod.rs  # if types.rs is sufficient
   ```

2. **Add missing exports**:
   ```rust
   // In lib.rs
   pub use error::{AuthError, AuthResult};
   pub use types::{AuthTokens};
   ```

3. **Update base64 usage**:
   ```rust
   // Find and replace all encode_config calls
   use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
   ```

#### **cloudshuttle-database**
1. **Update SQLx usage**:
   ```rust
   // Update all query! macros to use .bind() instead of inline params
   sqlx::query!("SELECT * FROM users WHERE id = $1")
       .bind(user_id)
   ```

2. **Fix runtime features**:
   ```toml
   sqlx = { workspace = true, features = ["runtime-tokio-rustls"] }
   ```

#### **cloudshuttle-validation**
1. **Update validator dependency**:
   ```toml
   validator = { version = "0.20", features = ["derive"] }
   ```

2. **Update validation rules** if APIs changed

### **Step 3: Testing & Validation**

#### **Compilation Testing**
```bash
# Test each crate individually
cargo check -p cloudshuttle-auth
cargo check -p cloudshuttle-database
cargo check -p cloudshuttle-validation
cargo check -p cloudshuttle-error-handling

# Test workspace compilation
cargo check --workspace
```

#### **Integration Testing**
```bash
# Test with a modern service
cargo check --manifest-path ../auth-service/Cargo.toml
```

#### **API Compatibility Testing**
- Create test that uses all shared components
- Verify all public APIs work as expected
- Test serialization/deserialization

---

## 🧪 **Testing Strategy**

### **Unit Tests**
- Test each updated dependency individually
- Verify API compatibility
- Performance regression testing

### **Integration Tests**
- Test component combinations
- Cross-component functionality
- Error propagation

### **Compatibility Tests**
- Test with current auth service
- Test with other CloudShuttle services
- Version compatibility matrix

---

## 📋 **Checklist**

### **Dependency Updates**
- [ ] Update workspace `Cargo.toml`
- [ ] Update individual crate `Cargo.toml` files
- [ ] Run `cargo update`
- [ ] Fix compilation errors

### **API Modernization**
- [ ] Fix base64 API usage
- [ ] Update rand API calls
- [ ] Update Axum extractors/responses
- [ ] Fix SQLx query macros

### **Module Structure**
- [ ] Resolve `types.rs` vs `types/mod.rs` conflicts
- [ ] Add missing type exports
- [ ] Update import statements

### **Testing**
- [ ] Individual crate compilation
- [ ] Workspace compilation
- [ ] Integration with auth service
- [ ] API compatibility verification

---

## 🚨 **Risk Mitigation**

### **Breaking Changes**
- **Semantic versioning**: Use major version bump (0.3.0 → 0.4.0)
- **Migration guide**: Document all breaking changes
- **Gradual adoption**: Allow services to migrate incrementally

### **Testing Coverage**
- **Automated tests**: Comprehensive test suite for all APIs
- **Compatibility matrix**: Test with multiple service versions
- **Performance benchmarks**: Ensure no regressions

### **Rollback Plan**
- **Git branches**: Feature branches for each major change
- **Tagged releases**: Version tags for stable points
- **Documentation**: Clear migration and rollback procedures

---

## 📈 **Success Criteria**

- [ ] **100% compilation success** with latest Rust 1.89
- [ ] **Zero dependency conflicts** with modern services
- [ ] **All APIs functional** with updated dependencies
- [ ] **Backward compatibility** maintained where possible
- [ ] **Performance maintained** or improved
- [ ] **All tests passing** with new versions

---

## 📅 **Timeline**

- **Day 1-2**: Update workspace dependencies and fix immediate errors
- **Day 3-4**: Fix individual crate APIs and module conflicts
- **Day 5-6**: Testing, validation, and integration verification
- **Day 7**: Final review and release preparation

---

## 🎯 **Next Steps**

1. Create feature branch for dependency modernization
2. Update workspace `Cargo.toml` with new versions
3. Fix compilation errors systematically
4. Test integration with auth service
5. Prepare migration documentation

---

*This modernization brings the shared repository up to current Rust ecosystem standards, enabling seamless integration with modern services and preventing future compatibility issues.*
