# Database Types Refactor Design Document

**File:** `rust/crates/database/src/types.rs`
**Current Size:** 404 lines (🚨 EXCEEDS LIMIT)
**Target:** 4 files × ~100 lines each
**Status:** DESIGN PHASE

## 📊 Problem Analysis

### Current Issues:
- **Single monolithic file** with 404 lines
- **Mixed concerns**: entities, traits, models, migrations
- **Poor maintainability** for LLM analysis and code review
- **Tight coupling** between unrelated functionality

### Content Analysis:
```
BaseEntity & SoftDeleteEntity    ~80 lines
Repository Traits               ~150 lines (TOO LARGE)
Domain Models                   ~100 lines (TOO LARGE)
Migration Types                 ~50 lines
Connection Types                ~24 lines
```

## 🏗️ Proposed Architecture

### 1. entities.rs (Target: ~80 lines)
**Purpose:** Core entity base classes and common functionality

**Contents:**
```rust
//! Base entity types and common functionality

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Common database entity fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseEntity {
    pub id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl BaseEntity {
    pub fn new() -> Self { ... }
    pub fn update_timestamp(mut self) -> Self { ... }
}

impl Default for BaseEntity {
    fn default() -> Self { Self::new() }
}

/// Soft delete support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftDeleteEntity {
    #[serde(flatten)]
    pub base: BaseEntity,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl SoftDeleteEntity {
    pub fn new() -> Self { ... }
    pub fn soft_delete(mut self) -> Self { ... }
}
```

**Dependencies:**
- `serde`, `uuid`, `chrono`

**Tests:**
- BaseEntity creation and updates
- SoftDeleteEntity soft delete functionality
- Default implementations

### 2. traits.rs (Target: ~120 lines)
**Purpose:** Repository traits split by functionality

**Contents:**
```rust
//! Repository traits for database operations

use async_trait::async_trait;
use crate::DatabaseResult;

/// Basic CRUD operations
#[async_trait]
pub trait Repository<T, ID = Uuid> {
    async fn find_by_id(&self, id: ID) -> DatabaseResult<Option<T>>;
    async fn save(&self, entity: T) -> DatabaseResult<T>;
    async fn delete(&self, id: ID) -> DatabaseResult<bool>;
}

/// Query operations
#[async_trait]
pub trait QueryRepository<T> {
    async fn find_all(&self) -> DatabaseResult<Vec<T>>;
    async fn find_by_criteria(&self, criteria: QueryCriteria) -> DatabaseResult<Vec<T>>;
    async fn count(&self) -> DatabaseResult<i64>;
}

/// Batch operations
#[async_trait]
pub trait BatchRepository<T> {
    async fn save_batch(&self, entities: Vec<T>) -> DatabaseResult<Vec<T>>;
    async fn delete_batch(&self, ids: Vec<Uuid>) -> DatabaseResult<usize>;
}
```

**Dependencies:**
- `async_trait`, `uuid`
- Internal: `DatabaseResult`

**Tests:**
- Trait implementations
- Mock repository tests
- Integration with concrete types

### 3. models.rs (Target: ~100 lines)
**Purpose:** Domain-specific data models

**Contents:**
```rust
//! Domain models for database operations

use serde::{Deserialize, Serialize};
use crate::entities::*;

/// Query criteria for filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryCriteria {
    pub filters: Vec<Filter>,
    pub sorting: Vec<SortOrder>,
    pub pagination: Option<Pagination>,
}

/// Database connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout_seconds: u32,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseHealth {
    pub status: HealthStatus,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
```

**Dependencies:**
- `serde`, `chrono`
- Internal: entities

**Tests:**
- Serialization/deserialization
- Validation logic
- Health check logic

### 4. migrations.rs (Target: ~80 lines)
**Purpose:** Migration-related types and utilities

**Contents:**
```rust
//! Migration types and utilities

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Migration record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationRecord {
    pub id: String,
    pub name: String,
    pub executed_at: DateTime<Utc>,
    pub success: bool,
}

/// Migration status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MigrationStatus {
    Pending,
    Executing,
    Completed,
    Failed,
}

/// Migration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationConfig {
    pub table_name: String,
    pub directory: String,
    pub allow_out_of_order: bool,
}
```

**Dependencies:**
- `serde`, `chrono`

**Tests:**
- Migration record creation
- Status transitions
- Configuration validation

## 🔧 Implementation Plan

### Phase 1: Create New Structure
1. Create `database/src/types/` directory
2. Create individual files with content split
3. Update imports and dependencies
4. Ensure compilation

### Phase 2: Update References
1. Update `lib.rs` to re-export from new locations
2. Update all import statements across codebase
3. Run comprehensive tests

### Phase 3: Cleanup
1. Remove old `types.rs` file
2. Update documentation
3. Final integration testing

## 📋 Migration Checklist

### Pre-Migration:
- [ ] Analyze all imports of current types
- [ ] Document public API surface
- [ ] Create backup branch

### During Migration:
- [ ] Create new directory structure
- [ ] Split code by functionality
- [ ] Update lib.rs exports
- [ ] Update all imports
- [ ] Run `cargo check` frequently

### Post-Migration:
- [ ] Run full test suite
- [ ] Update documentation
- [ ] Code review
- [ ] Performance testing

## ✅ Verification

```bash
# Check file sizes
find rust/crates/database/src/types/ -name "*.rs" -exec wc -l {} \;

# Verify compilation
cargo check --package cloudshuttle-database

# Run tests
cargo test --package cloudshuttle-database

# Check no breaking changes
cargo test --workspace
```

## 🎯 Success Criteria

- [ ] All new files ≤ 120 lines
- [ ] No compilation errors
- [ ] All existing tests pass
- [ ] Public API unchanged
- [ ] Documentation updated
- [ ] Performance maintained

## 🚨 Risk Assessment

**High Risk:** Breaking existing imports
**Mitigation:** Comprehensive testing, gradual migration

**Medium Risk:** Performance impact
**Mitigation:** Benchmark before/after migration

**Low Risk:** Logic errors during split
**Mitigation:** Unit tests for each new module
