//! Query builders and helpers

use crate::DatabaseConnection;
use cloudshuttle_error_handling::DatabaseError;
use sqlx::{PgPool, FromRow, Postgres, QueryBuilder};
use uuid::Uuid;

/// Trait for query helper methods
#[async_trait::async_trait]
pub trait QueryHelper {
    /// Find a record by ID
    async fn find_by_id<T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin>(
        &self,
        table: &str,
        id: Uuid,
    ) -> Result<Option<T>, DatabaseError>;

    /// Find multiple records by IDs
    async fn find_by_ids<T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin>(
        &self,
        table: &str,
        ids: &[Uuid],
    ) -> Result<Vec<T>, DatabaseError>;

    /// Find records with pagination
    async fn find_with_pagination<T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin>(
        &self,
        query: &str,
        params: &[&(dyn sqlx::Encode<'_, Postgres> + Sync)],
        page: u32,
        per_page: u32,
    ) -> Result<PaginatedResult<T>, DatabaseError>;

    /// Count records matching a query
    async fn count(
        &self,
        table: &str,
        where_clause: Option<&str>,
        params: &[&(dyn sqlx::Encode<'_, Postgres> + Sync)],
    ) -> Result<i64, DatabaseError>;
}

#[async_trait::async_trait]
impl QueryHelper for DatabaseConnection {
    async fn find_by_id<T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin>(
        &self,
        table: &str,
        id: Uuid,
    ) -> Result<Option<T>, DatabaseError> {
        let query = format!("SELECT * FROM {} WHERE id = $1", table);
        let result = sqlx::query_as::<_, T>(&query)
            .bind(id)
            .fetch_optional(self.pool())
            .await?;
        Ok(result)
    }

    async fn find_by_ids<T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin>(
        &self,
        table: &str,
        ids: &[Uuid],
    ) -> Result<Vec<T>, DatabaseError> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders: Vec<String> = (1..=ids.len()).map(|i| format!("${}", i)).collect();
        let query = format!(
            "SELECT * FROM {} WHERE id IN ({})",
            table,
            placeholders.join(",")
        );

        let mut sql_query = sqlx::query_as::<_, T>(&query);
        for id in ids {
            sql_query = sql_query.bind(id);
        }

        let results = sql_query.fetch_all(self.pool()).await?;
        Ok(results)
    }

    async fn find_with_pagination<T: for<'r> FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin>(
        &self,
        query: &str,
        params: &[&(dyn sqlx::Encode<'_, Postgres> + Sync)],
        page: u32,
        per_page: u32,
    ) -> Result<PaginatedResult<T>, DatabaseError> {
        let offset = (page.saturating_sub(1)) * per_page;

        // Build the paginated query
        let paginated_query = format!("{} LIMIT {} OFFSET {}", query, per_page, offset);
        let mut sql_query = sqlx::query_as::<_, T>(&paginated_query);

        for param in params {
            sql_query = sql_query.bind(param);
        }

        let records = sql_query.fetch_all(self.pool()).await?;

        // Get total count
        let count_query = format!("SELECT COUNT(*) FROM ({}) AS subquery", query);
        let mut count_sql = sqlx::query_as::<_, (i64,)>(&count_query);

        for param in params {
            count_sql = count_sql.bind(param);
        }

        let (total_count,) = count_sql.fetch_one(self.pool()).await?;

        Ok(PaginatedResult {
            records,
            total_count: total_count as u64,
            page,
            per_page,
            total_pages: ((total_count as u32).saturating_add(per_page.saturating_sub(1))) / per_page,
        })
    }

    async fn count(
        &self,
        table: &str,
        where_clause: Option<&str>,
        params: &[&(dyn sqlx::Encode<'_, Postgres> + Sync)],
    ) -> Result<i64, DatabaseError> {
        let query = if let Some(where_clause) = where_clause {
            format!("SELECT COUNT(*) FROM {} WHERE {}", table, where_clause)
        } else {
            format!("SELECT COUNT(*) FROM {}", table)
        };

        let mut sql_query = sqlx::query_as::<_, (i64,)>(&query);
        for param in params {
            sql_query = sql_query.bind(param);
        }

        let (count,) = sql_query.fetch_one(self.pool()).await?;
        Ok(count)
    }
}

/// Paginated query result
#[derive(Debug, Clone)]
pub struct PaginatedResult<T> {
    pub records: Vec<T>,
    pub total_count: u64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

impl<T> PaginatedResult<T> {
    /// Check if there are more pages
    pub fn has_next_page(&self) -> bool {
        self.page < self.total_pages
    }

    /// Check if there are previous pages
    pub fn has_prev_page(&self) -> bool {
        self.page > 1
    }

    /// Get the next page number
    pub fn next_page(&self) -> Option<u32> {
        if self.has_next_page() {
            Some(self.page + 1)
        } else {
            None
        }
    }

    /// Get the previous page number
    pub fn prev_page(&self) -> Option<u32> {
        if self.has_prev_page() {
            Some(self.page - 1)
        } else {
            None
        }
    }
}

/// Query builder for complex queries
pub struct QueryBuilderHelper<'a> {
    builder: QueryBuilder<'a, Postgres>,
    table: &'a str,
}

impl<'a> QueryBuilderHelper<'a> {
    /// Create a new query builder
    pub fn new(table: &'a str) -> Self {
        let mut builder = QueryBuilder::new("SELECT * FROM ");
        builder.push(table);
        Self { builder, table }
    }

    /// Add WHERE conditions
    pub fn where_clause(mut self, condition: &str) -> Self {
        self.builder.push(" WHERE ");
        self.builder.push(condition);
        self
    }

    /// Add ORDER BY clause
    pub fn order_by(mut self, column: &str, direction: OrderDirection) -> Self {
        self.builder.push(" ORDER BY ");
        self.builder.push(column);
        self.builder.push(" ");
        self.builder.push(match direction {
            OrderDirection::Asc => "ASC",
            OrderDirection::Desc => "DESC",
        });
        self
    }

    /// Add LIMIT clause
    pub fn limit(mut self, limit: u32) -> Self {
        self.builder.push(" LIMIT ");
        self.builder.push(limit.to_string());
        self
    }

    /// Add OFFSET clause
    pub fn offset(mut self, offset: u32) -> Self {
        self.builder.push(" OFFSET ");
        self.builder.push(offset.to_string());
        self
    }

    /// Build the final query
    pub fn build(self) -> String {
        self.builder.into_sql()
    }
}

/// Order direction for queries
#[derive(Debug, Clone, Copy)]
pub enum OrderDirection {
    Asc,
    Desc,
}
