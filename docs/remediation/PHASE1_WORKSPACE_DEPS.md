# Phase 1: Workspace & Dependencies Remediation

**Status:** CRITICAL - Immediate Action Required
**Timeline:** Complete within 24 hours

## 🚨 Issues Identified

### 1. Workspace Configuration Broken
**File:** `Cargo.toml`
**Issue:** Missing `rust-version` in workspace.package section
**Impact:** Cargo workspace fails to parse
**Severity:** CRITICAL

### 2. Dependency Versions Outdated
**Issue:** Multiple dependencies 1-2+ years outdated
**Security Risk:** Potential CVEs in old versions
**Performance Impact:** Missing optimizations and features

## 🔧 Required Fixes

### Workspace Configuration Fix
```toml
[workspace.package]
version = "0.2.0"  # Update version
edition = "2021"
rust-version = "1.89"  # ADD THIS LINE
license = "MIT OR Apache-2.0"
authors = ["CloudShuttle Team"]
repository = "https://github.com/cloud-shuttle/cloudshuttle-shared"
```

### Dependency Updates (September 2025)
```toml
[workspace.dependencies]
# Core async runtime - UPDATE
tokio = { version = "1.40", features = ["full"] }

# Serialization - UPDATE
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Database - UPDATE
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio-rustls", "chrono", "uuid", "json"] }

# Validation - UPDATE
validator = { version = "0.18", features = ["derive"] }

# Authentication - UPDATE
jsonwebtoken = "9.3"
bcrypt = "0.15"

# Web framework - UPDATE
axum = "0.7"
tower = "0.5"
hyper = "1.4"

# Observability - UPDATE
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
prometheus = "0.13"

# Utilities - UPDATE
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
regex = "1.10"
```

## ✅ Verification Steps

### After Fixes:
1. **Workspace compiles:**
   ```bash
   cargo check --workspace
   ```

2. **Dependencies updated:**
   ```bash
   cargo outdated
   # Should show no major version updates needed
   ```

3. **Security audit passes:**
   ```bash
   cargo audit
   # Should show no critical vulnerabilities
   ```

## 📋 Implementation Checklist

- [ ] Add `rust-version = "1.89"` to `[workspace.package]`
- [ ] Update tokio from 1.0 to 1.40+
- [ ] Update sqlx from 0.7 to 0.8
- [ ] Update validator from 0.16 to 0.18
- [ ] Update all authentication dependencies
- [ ] Update web framework dependencies
- [ ] Run `cargo update` to update lockfile
- [ ] Run `cargo check` to verify compilation
- [ ] Run `cargo audit` for security check
- [ ] Commit changes with message: "fix: update workspace config and dependencies"

## 🚨 Risk Mitigation

**If dependencies break API:**
- Use `cargo update --package <name>` for selective updates
- Implement compatibility layers if needed
- Update tests for any breaking changes

**If security audit fails:**
- Pin vulnerable dependencies to secure versions
- Implement workarounds for unresolved CVEs
- Document security exceptions

## 📊 Success Criteria

- [ ] `cargo check` passes without errors
- [ ] `cargo audit` shows no critical vulnerabilities
- [ ] `cargo outdated` shows only patch/minor updates needed
- [ ] Workspace configuration parses correctly
- [ ] All crates build successfully
