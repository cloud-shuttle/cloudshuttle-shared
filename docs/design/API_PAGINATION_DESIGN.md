# API Pagination Design Document

## Overview

This document outlines the design for pagination in CloudShuttle Gateway APIs, ensuring efficient handling of large datasets and consistent pagination behavior across all endpoints.

## Current Implementation

### Basic Pagination Structure

```rust
pub fn create_paginated_response<T: Serialize>(
    &self,
    items: Vec<T>,
    page: usize,
    limit: usize,
    total: usize,
) -> Json<ApiResponse<serde_json::Value>> {
    let response = serde_json::json!({
        "items": items,
        "pagination": {
            "page": page,
            "limit": limit,
            "total": total,
            "total_pages": (total + limit - 1) / limit
        }
    });
    self.create_success_response(response)
}
```

### Response Format

```json
{
  "success": true,
  "data": {
    "items": [
      { /* item 1 */ },
      { /* item 2 */ }
    ],
    "pagination": {
      "page": 1,
      "limit": 50,
      "total": 125,
      "total_pages": 3
    }
  },
  "error": null,
  "timestamp": "2024-01-01T12:00:00.000Z"
}
```

## Future Enhancement: Typed Pagination

When cloudshuttle-api provides pagination types:

```rust
pub fn create_paginated_response<T: Serialize>(
    &self,
    items: Vec<T>,
    page: usize,
    limit: usize,
    total: usize,
) -> Json<ApiResponse<PaginatedResponse<T>>> {
    let pagination_meta = PaginationMeta {
        page,
        limit,
        total,
        total_pages: (total + limit - 1) / limit,
        has_next: (page * limit) < total,
        has_prev: page > 1,
        links: self.generate_pagination_links(page, limit, total),
    };

    let response = PaginatedResponse {
        items,
        pagination: pagination_meta,
    };

    self.create_success_response(response)
}
```

### Enhanced Pagination Types

```rust
#[derive(Debug, Serialize, Clone)]
pub struct PaginationMeta {
    pub page: usize,
    pub limit: usize,
    pub total: usize,
    pub total_pages: usize,
    pub has_next: bool,
    pub has_prev: bool,
    pub links: PaginationLinks,
}

#[derive(Debug, Serialize, Clone)]
pub struct PaginationLinks {
    pub first: Option<String>,
    pub prev: Option<String>,
    pub next: Option<String>,
    pub last: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub pagination: PaginationMeta,
}
```

### Link Generation

```rust
impl SharedApiService {
    fn generate_pagination_links(
        &self,
        page: usize,
        limit: usize,
        total: usize,
    ) -> PaginationLinks {
        let base_url = self.get_base_url();
        let total_pages = (total + limit - 1) / limit;

        PaginationLinks {
            first: Some(format!("{}?page=1&limit={}", base_url, limit)),
            prev: if page > 1 {
                Some(format!("{}?page={}&limit={}", base_url, page - 1, limit))
            } else {
                None
            },
            next: if page < total_pages {
                Some(format!("{}?page={}&limit={}", base_url, page + 1, limit))
            } else {
                None
            },
            last: Some(format!("{}?page={}&limit={}", base_url, total_pages, limit)),
        }
    }
}
```

## Pagination Strategies

### Offset-Based Pagination

```rust
#[derive(Debug, Deserialize)]
pub struct OffsetPaginationQuery {
    pub page: Option<usize>,  // 1-based page number
    pub limit: Option<usize>, // items per page
}

impl Default for OffsetPaginationQuery {
    fn default() -> Self {
        Self {
            page: Some(1),
            limit: Some(50),
        }
    }
}

impl OffsetPaginationQuery {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if let Some(page) = self.page {
            if page == 0 {
                return Err(ValidationError::InvalidPage("Page must be >= 1"));
            }
        }

        if let Some(limit) = self.limit {
            if limit == 0 || limit > 1000 {
                return Err(ValidationError::InvalidLimit("Limit must be 1-1000"));
            }
        }

        Ok(())
    }

    pub fn offset(&self) -> usize {
        let page = self.page.unwrap_or(1).saturating_sub(1);
        let limit = self.limit.unwrap_or(50);
        page * limit
    }
}
```

### Cursor-Based Pagination

```rust
#[derive(Debug, Deserialize)]
pub struct CursorPaginationQuery {
    pub cursor: Option<String>,  // base64 encoded cursor
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct CursorPaginationMeta {
    pub has_next: bool,
    pub has_prev: bool,
    pub next_cursor: Option<String>,
    pub prev_cursor: Option<String>,
}

impl CursorPaginationQuery {
    pub fn decode_cursor(&self) -> Result<Option<CursorData>, DecodeError> {
        match &self.cursor {
            Some(cursor_str) => {
                let decoded = base64::decode(cursor_str)?;
                let cursor_data: CursorData = serde_json::from_slice(&decoded)?;
                Ok(Some(cursor_data))
            }
            None => Ok(None),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CursorData {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub sort_value: Option<String>,
}
```

### Keyset Pagination

```rust
#[derive(Debug, Deserialize)]
pub struct KeysetPaginationQuery {
    pub after: Option<String>,  // ID to start after
    pub before: Option<String>, // ID to start before
    pub limit: Option<usize>,
}

impl KeysetPaginationQuery {
    pub fn to_sql_condition(&self) -> String {
        match (&self.after, &self.before) {
            (Some(after), None) => format!("id > {}", after),
            (None, Some(before)) => format!("id < {}", before),
            _ => String::new(),
        }
    }
}
```

## Database Integration

### SQL Query Builders

```rust
pub struct PaginatedQueryBuilder {
    base_query: String,
    count_query: String,
    pagination: Box<dyn PaginationStrategy>,
}

impl PaginatedQueryBuilder {
    pub fn offset_based(base_query: &str, count_query: &str) -> Self {
        Self {
            base_query: base_query.to_string(),
            count_query: count_query.to_string(),
            pagination: Box::new(OffsetPaginationStrategy),
        }
    }

    pub fn cursor_based(base_query: &str) -> Self {
        Self {
            base_query: base_query.to_string(),
            count_query: String::new(), // Not needed for cursor pagination
            pagination: Box::new(CursorPaginationStrategy),
        }
    }

    pub async fn execute<T: FromRow>(
        &self,
        conn: &mut PgConnection,
        pagination: &dyn PaginationQuery,
    ) -> Result<PaginatedResponse<T>, DbError> {
        // Execute count query if needed
        let total = if !self.count_query.is_empty() {
            sqlx::query_scalar(&self.count_query)
                .fetch_one(conn)
                .await?
        } else {
            0 // For cursor pagination, total might not be known
        };

        // Execute paginated query
        let items: Vec<T> = self.pagination
            .build_query(&self.base_query, pagination)?
            .fetch_all(conn)
            .await?;

        // Build pagination metadata
        let pagination_meta = self.pagination
            .build_metadata(pagination, total, items.len());

        Ok(PaginatedResponse {
            items,
            pagination: pagination_meta,
        })
    }
}
```

### Repository Pattern

```rust
#[async_trait]
pub trait PaginatedRepository<T, Q> {
    async fn find_paginated(
        &self,
        query: Q,
        pagination: &dyn PaginationQuery,
    ) -> Result<PaginatedResponse<T>, RepositoryError>;

    async fn count(&self, query: Q) -> Result<usize, RepositoryError>;
}

pub struct UpstreamRepository {
    pool: PgPool,
}

#[async_trait]
impl PaginatedRepository<UpstreamInfo, UpstreamQuery> for UpstreamRepository {
    async fn find_paginated(
        &self,
        query: UpstreamQuery,
        pagination: &dyn PaginationQuery,
    ) -> Result<PaginatedResponse<UpstreamInfo>, RepositoryError> {
        let mut conn = self.pool.acquire().await?;

        let paginated_query = PaginatedQueryBuilder::offset_based(
            "SELECT * FROM upstreams WHERE tenant_id = $1",
            "SELECT COUNT(*) FROM upstreams WHERE tenant_id = $1",
        );

        paginated_query.execute(&mut conn, pagination).await
    }
}
```

## API Endpoint Implementation

### Controller Layer

```rust
pub struct UpstreamController {
    repository: Arc<UpstreamRepository>,
    api_service: Arc<SharedApiService>,
}

impl UpstreamController {
    pub async fn list_upstreams(
        &self,
        Query(pagination): Query<OffsetPaginationQuery>,
        Query(filters): Query<UpstreamFilters>,
    ) -> Result<Json<ApiResponse<PaginatedResponse<UpstreamInfo>>>, StatusCode> {
        // Validate pagination parameters
        pagination.validate()
            .map_err(|_| StatusCode::BAD_REQUEST)?;

        // Build query
        let query = UpstreamQuery::from_filters(filters);

        // Execute paginated query
        let result = self.repository
            .find_paginated(query, &pagination)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // Return paginated response
        Ok(self.api_service.create_paginated_response(
            result.items,
            pagination.page.unwrap_or(1),
            pagination.limit.unwrap_or(50),
            result.pagination.total,
        ))
    }
}
```

### Route Configuration

```rust
pub fn upstream_routes() -> Router {
    Router::new()
        .route("/upstreams", get(list_upstreams))
        .route("/upstreams/:id", get(get_upstream))
        .layer(
            ServiceBuilder::new()
                .layer(ValidateRequestLayer::new(validate_pagination_params))
        )
}
```

## Performance Optimization

### Database Indexing

```sql
-- For offset-based pagination
CREATE INDEX idx_upstreams_tenant_created
ON upstreams (tenant_id, created_at DESC);

-- For cursor-based pagination
CREATE INDEX idx_upstreams_cursor
ON upstreams (id, created_at DESC);

-- For keyset pagination
CREATE UNIQUE INDEX idx_upstreams_id
ON upstreams (id);
```

### Query Optimization

```rust
pub struct OptimizedPaginationQuery {
    base_query: String,
    use_index_hint: bool,
    prefetch_pages: usize,
}

impl OptimizedPaginationQuery {
    pub fn with_index_hint(mut self) -> Self {
        self.use_index_hint = true;
        self
    }

    pub fn with_prefetch(mut self, pages: usize) -> Self {
        self.prefetch_pages = pages;
        self
    }

    pub fn build_sql(&self, pagination: &dyn PaginationQuery) -> String {
        let mut sql = self.base_query.clone();

        if self.use_index_hint {
            sql = format!("{} USE INDEX (idx_upstreams_tenant_created)", sql);
        }

        // Add pagination clause
        let pagination_clause = pagination.to_sql_clause();
        format!("{} {}", sql, pagination_clause)
    }
}
```

### Caching Strategy

```rust
pub struct PaginationCache {
    redis: RedisClient,
    ttl: Duration,
}

impl PaginationCache {
    pub async fn get_or_compute<T, F, Fut>(
        &self,
        cache_key: &str,
        compute_fn: F,
    ) -> Result<T, CacheError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, CacheError>>,
        T: Serialize + DeserializeOwned,
    {
        // Try cache first
        if let Some(cached) = self.redis.get(cache_key).await? {
            return Ok(cached);
        }

        // Compute result
        let result = compute_fn().await?;

        // Cache result
        self.redis.set_ex(cache_key, &result, self.ttl).await?;

        Ok(result)
    }

    pub fn invalidate_pattern(&self, pattern: &str) {
        // Invalidate cache keys matching pattern
        // e.g., "upstreams:tenant:*:page:*"
    }
}
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_meta_calculation() {
        let meta = PaginationMeta::new(1, 50, 125);
        assert_eq!(meta.page, 1);
        assert_eq!(meta.limit, 50);
        assert_eq!(meta.total, 125);
        assert_eq!(meta.total_pages, 3);
        assert!(meta.has_next);
        assert!(!meta.has_prev);
    }

    #[test]
    fn test_offset_pagination_query() {
        let query = OffsetPaginationQuery {
            page: Some(2),
            limit: Some(25),
        };

        assert_eq!(query.offset(), 25); // (2-1) * 25
    }

    #[test]
    fn test_pagination_links_generation() {
        let service = SharedApiService::new(&ApiConfig::default()).unwrap();
        let links = service.generate_pagination_links(2, 50, 150);

        assert_eq!(links.prev, Some("/api/upstreams?page=1&limit=50".to_string()));
        assert_eq!(links.next, Some("/api/upstreams?page=3&limit=50".to_string()));
        assert_eq!(links.first, Some("/api/upstreams?page=1&limit=50".to_string()));
        assert_eq!(links.last, Some("/api/upstreams?page=3&limit=50".to_string()));
    }
}
```

### Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use test_helpers::TestDatabase;

    #[tokio::test]
    async fn test_paginated_upstream_listing() {
        let db = TestDatabase::new().await;
        let repository = UpstreamRepository::new(db.pool());
        let api_service = SharedApiService::new(&ApiConfig::default()).unwrap();

        // Insert test data
        for i in 0..75 {
            repository.create_upstream(format!("upstream-{}", i)).await.unwrap();
        }

        // Test pagination
        let pagination = OffsetPaginationQuery {
            page: Some(2),
            limit: Some(25),
        };

        let result = repository.find_paginated(
            UpstreamQuery::default(),
            &pagination,
        ).await.unwrap();

        assert_eq!(result.items.len(), 25);
        assert_eq!(result.pagination.page, 2);
        assert_eq!(result.pagination.total, 75);
        assert!(result.pagination.has_next);
        assert!(result.pagination.has_prev);
    }
}
```

## Monitoring and Metrics

### Pagination Metrics

```rust
pub struct PaginationMetrics {
    pub total_requests: Counter,
    pub page_size_distribution: Histogram,
    pub response_time: Histogram,
    pub cache_hit_rate: Gauge,
}

impl PaginationMetrics {
    pub fn record_request(&self, page: usize, limit: usize, response_time: Duration) {
        self.total_requests.inc();
        self.page_size_distribution.observe(limit as f64);
        self.response_time.observe(response_time.as_secs_f64());
    }
}
```

### Performance Alerts

- High page numbers (>1000) may indicate inefficient queries
- Large page sizes (>1000) may cause memory issues
- Slow response times (>5s) for paginated queries
- Low cache hit rates (<50%) may indicate cache tuning needed

## Security Considerations

### Parameter Validation

```rust
impl OffsetPaginationQuery {
    pub fn sanitize(mut self) -> Self {
        // Enforce reasonable limits
        self.page = Some(self.page.unwrap_or(1).clamp(1, 10000));
        self.limit = Some(self.limit.unwrap_or(50).clamp(1, 1000));
        self
    }
}
```

### SQL Injection Prevention

```rust
impl dyn PaginationQuery {
    fn to_sql_clause(&self) -> String {
        match self {
            PaginationQuery::Offset { page, limit } => {
                let offset = (page.saturating_sub(1)) * limit;
                format!("LIMIT {} OFFSET {}", limit, offset)
            }
            // Other pagination types...
        }
    }
}
```

## Migration Strategy

### Phase 1: Basic Pagination (Current)
- ✅ JSON-based pagination structure
- ✅ Offset-based pagination
- ✅ Basic validation

### Phase 2: Enhanced Pagination
- 🔄 Typed pagination structures
- 🔄 Link generation
- 🔄 Cursor-based pagination
- 🔄 Performance optimization

### Phase 3: Advanced Features
- 🔄 Keyset pagination for large datasets
- 🔄 Intelligent caching
- 🔄 Analytics and monitoring
- 🔄 GraphQL-style pagination

## Conclusion

The pagination design provides efficient, scalable, and user-friendly navigation through large datasets while maintaining consistent API behavior and performance. The modular design allows for different pagination strategies based on use case requirements.
