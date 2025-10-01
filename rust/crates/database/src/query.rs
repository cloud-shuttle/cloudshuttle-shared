//! Query builder utilities and helpers

use sqlx::{PgConnection, postgres::PgRow};
use serde::{Deserialize, Serialize};
use cloudshuttle_error_handling::database_error::DatabaseResult;

/// Query builder for dynamic SQL construction
#[derive(Debug, Clone)]
pub struct QueryBuilder {
    table: String,
    select_fields: Vec<String>,
    where_conditions: Vec<String>,
    params: Vec<serde_json::Value>,
    order_by: Vec<String>,
    limit: Option<usize>,
    offset: Option<usize>,
    joins: Vec<String>,
}

impl QueryBuilder {
    /// Create a new query builder for a table
    pub fn new<S: Into<String>>(table: S) -> Self {
        Self {
            table: table.into(),
            select_fields: vec!["*".to_string()],
            where_conditions: Vec::new(),
            params: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
            joins: Vec::new(),
        }
    }

    /// Specify which fields to select
    pub fn select<S: Into<String>>(mut self, fields: Vec<S>) -> Self {
        self.select_fields = fields.into_iter().map(|s| s.into()).collect();
        self
    }

    /// Add a WHERE condition
    pub fn where_eq<S: Into<String>, V: Into<serde_json::Value>>(mut self, field: S, value: V) -> Self {
        let condition = format!("{} = ${}", field.into(), self.params.len() + 1);
        self.where_conditions.push(condition);
        self.params.push(value.into());
        self
    }

    /// Add a WHERE condition with custom operator
    pub fn where_condition<S: Into<String>>(mut self, condition: S) -> Self {
        self.where_conditions.push(condition.into());
        self
    }

    /// Add a parameter value
    pub fn param<V: Into<serde_json::Value>>(mut self, value: V) -> Self {
        self.params.push(value.into());
        self
    }

    /// Add an ORDER BY clause
    pub fn order_by<S: Into<String>>(mut self, field: S) -> Self {
        self.order_by.push(field.into());
        self
    }

    /// Add a descending ORDER BY clause
    pub fn order_by_desc<S: Into<String>>(mut self, field: S) -> Self {
        self.order_by.push(format!("{} DESC", field.into()));
        self
    }

    /// Set LIMIT
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set OFFSET
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Add a JOIN clause
    pub fn join<S: Into<String>>(mut self, join: S) -> Self {
        self.joins.push(join.into());
        self
    }

    /// Build the final SQL query
    pub fn build(&self) -> String {
        let mut query = format!(
            "SELECT {} FROM {}",
            self.select_fields.join(", "),
            self.table
        );

        // Add JOINs
        for join in &self.joins {
            query.push_str(&format!(" {}", join));
        }

        // Add WHERE conditions
        if !self.where_conditions.is_empty() {
            query.push_str(&format!(" WHERE {}", self.where_conditions.join(" AND ")));
        }

        // Add ORDER BY
        if !self.order_by.is_empty() {
            query.push_str(&format!(" ORDER BY {}", self.order_by.join(", ")));
        }

        // Add LIMIT and OFFSET
        if let Some(limit) = self.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }
        if let Some(offset) = self.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        query
    }

    /// Get the parameter values
    pub fn params(&self) -> &[serde_json::Value] {
        &self.params
    }
}

/// Pagination parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    pub page: usize,
    pub per_page: usize,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: 20,
        }
    }
}

impl PaginationParams {
    pub fn new(page: usize, per_page: usize) -> Self {
        Self { page, per_page }
    }

    pub fn offset(&self) -> usize {
        (self.page.saturating_sub(1)) * self.per_page
    }

    pub fn limit(&self) -> usize {
        self.per_page
    }
}

/// Paginated response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationMeta {
    pub page: usize,
    pub per_page: usize,
    pub total: usize,
    pub total_pages: usize,
    pub has_next: bool,
    pub has_prev: bool,
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, page: usize, per_page: usize, total: usize) -> Self {
        let total_pages = (total + per_page - 1) / per_page;
        let has_next = page < total_pages;
        let has_prev = page > 1;

        Self {
            data,
            pagination: PaginationMeta {
                page,
                per_page,
                total,
                total_pages,
                has_next,
                has_prev,
            },
        }
    }
}

/// Sorting options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortDirection {
    #[serde(rename = "asc")]
    Ascending,
    #[serde(rename = "desc")]
    Descending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortOption {
    pub field: String,
    pub direction: SortDirection,
}

impl SortOption {
    pub fn asc<S: Into<String>>(field: S) -> Self {
        Self {
            field: field.into(),
            direction: SortDirection::Ascending,
        }
    }

    pub fn desc<S: Into<String>>(field: S) -> Self {
        Self {
            field: field.into(),
            direction: SortDirection::Descending,
        }
    }

    pub fn to_sql(&self) -> String {
        match self.direction {
            SortDirection::Ascending => self.field.clone(),
            SortDirection::Descending => format!("{} DESC", self.field),
        }
    }
}

/// Filtering options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterOption {
    pub field: String,
    pub operator: FilterOperator,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterOperator {
    #[serde(rename = "eq")]
    Equal,
    #[serde(rename = "ne")]
    NotEqual,
    #[serde(rename = "gt")]
    GreaterThan,
    #[serde(rename = "gte")]
    GreaterThanOrEqual,
    #[serde(rename = "lt")]
    LessThan,
    #[serde(rename = "lte")]
    LessThanOrEqual,
    #[serde(rename = "like")]
    Like,
    #[serde(rename = "in")]
    In,
    #[serde(rename = "nin")]
    NotIn,
    #[serde(rename = "null")]
    IsNull,
    #[serde(rename = "nnull")]
    IsNotNull,
}

impl FilterOperator {
    pub fn to_sql(&self, param_index: usize) -> String {
        match self {
            Self::Equal => format!("= ${}", param_index),
            Self::NotEqual => format!("!= ${}", param_index),
            Self::GreaterThan => format!("> ${}", param_index),
            Self::GreaterThanOrEqual => format!(">= ${}", param_index),
            Self::LessThan => format!("< ${}", param_index),
            Self::LessThanOrEqual => format!("<= ${}", param_index),
            Self::Like => format!("LIKE ${}", param_index),
            Self::In => format!("= ANY(${})", param_index),
            Self::NotIn => format!("!= ALL(${})", param_index),
            Self::IsNull => "IS NULL".to_string(),
            Self::IsNotNull => "IS NOT NULL".to_string(),
        }
    }
}

/// Query result wrapper with metadata
#[derive(Debug)]
pub struct QueryResult<T> {
    pub data: T,
    pub execution_time: std::time::Duration,
    pub rows_affected: Option<u64>,
}

impl<T> QueryResult<T> {
    pub fn new(data: T, execution_time: std::time::Duration) -> Self {
        Self {
            data,
            execution_time,
            rows_affected: None,
        }
    }

    pub fn with_rows_affected(mut self, rows: u64) -> Self {
        self.rows_affected = Some(rows);
        self
    }
}

/// Batch operation support
pub struct BatchOperation {
    operations: Vec<BatchItem>,
}

pub struct BatchItem {
    pub query: String,
    pub params: Vec<serde_json::Value>,
}

impl BatchOperation {
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    pub fn add<S: Into<String>>(mut self, query: S, params: Vec<serde_json::Value>) -> Self {
        self.operations.push(BatchItem {
            query: query.into(),
            params,
        });
        self
    }

    pub fn operations(&self) -> &[BatchItem] {
        &self.operations
    }
}

/// Query execution options
#[derive(Debug, Clone, Default)]
pub struct QueryOptions {
    pub timeout: Option<std::time::Duration>,
    pub retry_count: u32,
    pub isolation_level: Option<String>,
}

impl QueryOptions {
    pub fn with_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn with_retry(mut self, count: u32) -> Self {
        self.retry_count = count;
        self
    }

    pub fn with_isolation_level<S: Into<String>>(mut self, level: S) -> Self {
        self.isolation_level = Some(level.into());
        self
    }
}
