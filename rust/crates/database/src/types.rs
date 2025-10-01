//! Common database types and traits

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, postgres::PgRow};
use uuid::Uuid;
use cloudshuttle_error_handling::database_error::DatabaseResult;

/// Common database entity fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseEntity {
    pub id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl BaseEntity {
    pub fn new() -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update_timestamp(mut self) -> Self {
        self.updated_at = chrono::Utc::now();
        self
    }
}

impl Default for BaseEntity {
    fn default() -> Self {
        Self::new()
    }
}

/// Soft delete support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftDeleteEntity {
    #[serde(flatten)]
    pub base: BaseEntity,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl SoftDeleteEntity {
    pub fn new() -> Self {
        Self {
            base: BaseEntity::new(),
            deleted_at: None,
        }
    }

    pub fn soft_delete(mut self) -> Self {
        self.deleted_at = Some(chrono::Utc::now());
        self.base = self.base.update_timestamp();
        self
    }

    pub fn restore(mut self) -> Self {
        self.deleted_at = None;
        self.base = self.base.update_timestamp();
        self
    }

    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
}

/// Versioned entity for optimistic locking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedEntity {
    #[serde(flatten)]
    pub base: BaseEntity,
    pub version: i32,
}

impl VersionedEntity {
    pub fn new() -> Self {
        Self {
            base: BaseEntity::new(),
            version: 1,
        }
    }

    pub fn increment_version(mut self) -> Self {
        self.version += 1;
        self.base = self.base.update_timestamp();
        self
    }
}

/// Tenant-aware entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantEntity {
    #[serde(flatten)]
    pub base: BaseEntity,
    pub tenant_id: Uuid,
}

impl TenantEntity {
    pub fn new(tenant_id: Uuid) -> Self {
        Self {
            base: BaseEntity::new(),
            tenant_id,
        }
    }
}

/// Full-featured entity combining all traits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullEntity {
    #[serde(flatten)]
    pub tenant: TenantEntity,
    pub version: i32,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl FullEntity {
    pub fn new(tenant_id: Uuid) -> Self {
        Self {
            tenant: TenantEntity::new(tenant_id),
            version: 1,
            deleted_at: None,
        }
    }

    pub fn update(mut self) -> Self {
        self.tenant.base = self.tenant.base.update_timestamp();
        self.version += 1;
        self
    }

    pub fn soft_delete(mut self) -> Self {
        self.deleted_at = Some(chrono::Utc::now());
        self.update()
    }

    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
}

/// Pagination metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageMeta {
    pub page: usize,
    pub per_page: usize,
    pub total: usize,
    pub total_pages: usize,
    pub has_next: bool,
    pub has_prev: bool,
}

impl PageMeta {
    pub fn new(page: usize, per_page: usize, total: usize) -> Self {
        let total_pages = (total + per_page - 1) / per_page;
        Self {
            page,
            per_page,
            total,
            total_pages,
            has_next: page < total_pages,
            has_prev: page > 1,
        }
    }
}

/// Sort direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortDir {
    #[serde(rename = "asc")]
    Asc,
    #[serde(rename = "desc")]
    Desc,
}

impl SortDir {
    pub fn to_sql(&self) -> &'static str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }
}

/// Sort specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortSpec {
    pub field: String,
    pub direction: SortDir,
}

impl SortSpec {
    pub fn asc(field: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            direction: SortDir::Asc,
        }
    }

    pub fn desc(field: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            direction: SortDir::Desc,
        }
    }
}

/// Filter operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterOp {
    #[serde(rename = "eq")]
    Eq,
    #[serde(rename = "ne")]
    Ne,
    #[serde(rename = "gt")]
    Gt,
    #[serde(rename = "gte")]
    Gte,
    #[serde(rename = "lt")]
    Lt,
    #[serde(rename = "lte")]
    Lte,
    #[serde(rename = "like")]
    Like,
    #[serde(rename = "in")]
    In,
    #[serde(rename = "nin")]
    Nin,
    #[serde(rename = "null")]
    Null,
    #[serde(rename = "nnull")]
    NotNull,
}

impl FilterOp {
    pub fn to_sql(&self, field: &str, param_index: usize) -> String {
        match self {
            Self::Eq => format!("{} = ${}", field, param_index),
            Self::Ne => format!("{} != ${}", field, param_index),
            Self::Gt => format!("{} > ${}", field, param_index),
            Self::Gte => format!("{} >= ${}", field, param_index),
            Self::Lt => format!("{} < ${}", field, param_index),
            Self::Lte => format!("{} <= ${}", field, param_index),
            Self::Like => format!("{} LIKE ${}", field, param_index),
            Self::In => format!("{} = ANY(${})", field, param_index),
            Self::Nin => format!("{} != ALL(${})", field, param_index),
            Self::Null => format!("{} IS NULL", field),
            Self::NotNull => format!("{} IS NOT NULL", field),
        }
    }
}

/// Filter specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterSpec {
    pub field: String,
    pub op: FilterOp,
    pub value: Option<serde_json::Value>,
}

impl FilterSpec {
    pub fn new(field: impl Into<String>, op: FilterOp, value: impl Into<serde_json::Value>) -> Self {
        Self {
            field: field.into(),
            op,
            value: Some(value.into()),
        }
    }

    pub fn null(field: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            op: FilterOp::Null,
            value: None,
        }
    }

    pub fn not_null(field: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            op: FilterOp::NotNull,
            value: None,
        }
    }
}

/// Query specification combining filters, sorting, and pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuerySpec {
    pub filters: Vec<FilterSpec>,
    pub sorts: Vec<SortSpec>,
    pub page: usize,
    pub per_page: usize,
}

impl QuerySpec {
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
            sorts: Vec::new(),
            page: 1,
            per_page: 20,
        }
    }

    pub fn filter(mut self, filter: FilterSpec) -> Self {
        self.filters.push(filter);
        self
    }

    pub fn sort(mut self, sort: SortSpec) -> Self {
        self.sorts.push(sort);
        self
    }

    pub fn page(mut self, page: usize) -> Self {
        self.page = page;
        self
    }

    pub fn per_page(mut self, per_page: usize) -> Self {
        self.per_page = per_page;
        self
    }

    pub fn offset(&self) -> usize {
        (self.page.saturating_sub(1)) * self.per_page
    }

    pub fn limit(&self) -> usize {
        self.per_page
    }
}

impl Default for QuerySpec {
    fn default() -> Self {
        Self::new()
    }
}

/// Database operation result with metadata
#[derive(Debug)]
pub struct OperationResult<T> {
    pub data: T,
    pub rows_affected: Option<u64>,
    pub execution_time: std::time::Duration,
}

impl<T> OperationResult<T> {
    pub fn new(data: T, execution_time: std::time::Duration) -> Self {
        Self {
            data,
            rows_affected: None,
            execution_time,
        }
    }

    pub fn with_rows_affected(mut self, rows: u64) -> Self {
        self.rows_affected = Some(rows);
        self
    }
}

/// Entity repository trait
#[async_trait]
pub trait Repository<T, ID = Uuid> {
    async fn find_by_id(&self, id: ID) -> DatabaseResult<Option<T>>;
    async fn find_all(&self) -> DatabaseResult<Vec<T>>;
    async fn save(&self, entity: T) -> DatabaseResult<T>;
    async fn delete(&self, id: ID) -> DatabaseResult<bool>;
    async fn exists(&self, id: ID) -> DatabaseResult<bool>;
    async fn count(&self) -> DatabaseResult<i64>;
}

/// Paged repository trait
#[async_trait]
pub trait PagedRepository<T, ID = Uuid> {
    async fn find_page(&self, spec: &QuerySpec) -> DatabaseResult<(Vec<T>, PageMeta)>;
}

/// Soft delete repository trait
#[async_trait]
pub trait SoftDeleteRepository<T, ID = Uuid> {
    async fn soft_delete(&self, id: ID) -> DatabaseResult<bool>;
    async fn restore(&self, id: ID) -> DatabaseResult<bool>;
    async fn find_deleted(&self) -> DatabaseResult<Vec<T>>;
}

/// Versioned repository trait for optimistic locking
#[async_trait]
pub trait VersionedRepository<T, ID = Uuid> {
    async fn save_versioned(&self, entity: T, expected_version: i32) -> DatabaseResult<T>;
}

/// Tenant-aware repository trait
#[async_trait]
pub trait TenantRepository<T, ID = Uuid> {
    async fn find_by_tenant(&self, tenant_id: Uuid) -> DatabaseResult<Vec<T>>;
    async fn find_by_tenant_and_id(&self, tenant_id: Uuid, id: ID) -> DatabaseResult<Option<T>>;
}
