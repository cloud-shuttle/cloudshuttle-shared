//! Pagination utilities for API responses
//!
//! This module provides types and utilities for handling paginated
//! API responses with consistent metadata.

use serde::{Deserialize, Serialize};
use std::result::Result as StdResult;

/// Pagination parameters from query string
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PaginationParams {
    /// Page number (1-based)
    pub page: Option<u32>,
    /// Number of items per page
    pub per_page: Option<u32>,
    /// Sort field
    pub sort_by: Option<String>,
    /// Sort direction
    pub sort_order: Option<String>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            per_page: Some(20),
            sort_by: None,
            sort_order: Some("asc".to_string()),
        }
    }
}

impl PaginationParams {
    /// Create pagination params with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set page number
    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    /// Set items per page
    pub fn per_page(mut self, per_page: u32) -> Self {
        self.per_page = Some(per_page);
        self
    }

    /// Set sort field and direction
    pub fn sort(mut self, field: impl Into<String>, order: impl Into<String>) -> Self {
        self.sort_by = Some(field.into());
        self.sort_order = Some(order.into());
        self
    }

    /// Get validated page number
    pub fn page(&self) -> u32 {
        self.page.unwrap_or(1).max(1)
    }

    /// Get validated items per page
    pub fn per_page(&self) -> u32 {
        let per_page = self.per_page.unwrap_or(20);
        per_page.max(1).min(1000) // Max 1000 items per page
    }

    /// Get sort field
    pub fn sort_by(&self) -> Option<&str> {
        self.sort_by.as_deref()
    }

    /// Get sort order
    pub fn sort_order(&self) -> &str {
        self.sort_order.as_deref().unwrap_or("asc")
    }

    /// Calculate offset for database queries
    pub fn offset(&self) -> u64 {
        ((self.page() - 1) * self.per_page()) as u64
    }

    /// Calculate limit for database queries
    pub fn limit(&self) -> u64 {
        self.per_page() as u64
    }
}

/// Pagination metadata for responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationMeta {
    /// Current page number
    pub page: u32,
    /// Items per page
    pub per_page: u32,
    /// Total number of items
    pub total: u64,
    /// Total number of pages
    pub total_pages: u32,
    /// Whether there is a next page
    pub has_next: bool,
    /// Whether there is a previous page
    pub has_prev: bool,
}

impl PaginationMeta {
    /// Create pagination metadata
    pub fn new(page: u32, per_page: u32, total: u64) -> Self {
        let total_pages = if per_page == 0 {
            0
        } else {
            ((total + per_page as u64 - 1) / per_page as u64) as u32
        };

        Self {
            page,
            per_page,
            total,
            total_pages,
            has_next: page < total_pages,
            has_prev: page > 1,
        }
    }

    /// Get the range of items shown on this page
    pub fn item_range(&self) -> (u64, u64) {
        let start = ((self.page - 1) * self.per_page) as u64;
        let end = (start + self.per_page as u64).min(self.total);
        (start + 1, end) // 1-based indexing
    }
}

impl Default for PaginationMeta {
    fn default() -> Self {
        Self::new(1, 20, 0)
    }
}

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    /// The data items
    pub data: Vec<T>,
    /// Pagination metadata
    pub pagination: PaginationMeta,
}

impl<T> PaginatedResponse<T> {
    /// Create a new paginated response
    pub fn new(data: Vec<T>, pagination: PaginationMeta) -> Self {
        Self { data, pagination }
    }

    /// Create from data and pagination parameters
    pub fn from_params(data: Vec<T>, params: &PaginationParams, total: u64) -> Self {
        let page = params.page();
        let per_page = params.per_page();
        let pagination = PaginationMeta::new(page, per_page, total);
        Self::new(data, pagination)
    }

    /// Get the items
    pub fn items(&self) -> &[T] {
        &self.data
    }

    /// Get mutable reference to items
    pub fn items_mut(&mut self) -> &mut Vec<T> {
        &mut self.data
    }

    /// Check if response is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get number of items in this page
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if this is the first page
    pub fn is_first_page(&self) -> bool {
        self.pagination.page == 1
    }

    /// Check if this is the last page
    pub fn is_last_page(&self) -> bool {
        !self.pagination.has_next
    }
}

/// Pagination links for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationLinks {
    /// Link to first page
    pub first: Option<String>,
    /// Link to previous page
    pub prev: Option<String>,
    /// Link to next page
    pub next: Option<String>,
    /// Link to last page
    pub last: Option<String>,
}

impl PaginationLinks {
    /// Create pagination links for a given path
    pub fn new(path: &str, meta: &PaginationMeta) -> Self {
        let base_url = path.trim_end_matches('/');

        Self {
            first: Some(format!("{}?page=1&per_page={}", base_url, meta.per_page)),
            prev: if meta.has_prev {
                Some(format!("{}?page={}&per_page={}", base_url, meta.page - 1, meta.per_page))
            } else {
                None
            },
            next: if meta.has_next {
                Some(format!("{}?page={}&per_page={}", base_url, meta.page + 1, meta.per_page))
            } else {
                None
            },
            last: Some(format!("{}?page={}&per_page={}", base_url, meta.total_pages, meta.per_page)),
        }
    }
}

/// Pagination query builder for database queries
pub struct PaginationQueryBuilder {
    base_query: String,
    params: PaginationParams,
}

impl PaginationQueryBuilder {
    /// Create a new pagination query builder
    pub fn new(base_query: impl Into<String>) -> Self {
        Self {
            base_query: base_query.into(),
            params: PaginationParams::default(),
        }
    }

    /// Set pagination parameters
    pub fn with_params(mut self, params: PaginationParams) -> Self {
        self.params = params;
        self
    }

    /// Add ORDER BY clause if sorting is specified
    pub fn with_sorting(mut self) -> Self {
        if let Some(sort_by) = &self.params.sort_by {
            let order = self.params.sort_order();
            self.base_query.push_str(&format!(" ORDER BY {} {}", sort_by, order.to_uppercase()));
        }
        self
    }

    /// Add LIMIT and OFFSET clauses
    pub fn with_limits(mut self) -> Self {
        let limit = self.params.limit();
        let offset = self.params.offset();
        self.base_query.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));
        self
    }

    /// Get the final query
    pub fn build(self) -> String {
        self.base_query
    }

    /// Get a count query for total records
    pub fn count_query(&self) -> String {
        let count_query = self.base_query
            .replace("SELECT ", "SELECT COUNT(*) as total FROM (SELECT ")
            .replace(" ORDER BY", ") ORDER BY");

        if count_query.contains("FROM (SELECT") {
            format!("{}) as subquery", count_query)
        } else {
            format!("SELECT COUNT(*) as total FROM ({}) as subquery", self.base_query)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_params() {
        let params = PaginationParams::new()
            .page(2)
            .per_page(10)
            .sort("name", "desc");

        assert_eq!(params.page(), 2);
        assert_eq!(params.per_page(), 10);
        assert_eq!(params.sort_by(), Some("name"));
        assert_eq!(params.sort_order(), "desc");
        assert_eq!(params.offset(), 10);
        assert_eq!(params.limit(), 10);
    }

    #[test]
    fn test_pagination_meta() {
        let meta = PaginationMeta::new(2, 10, 25);

        assert_eq!(meta.page, 2);
        assert_eq!(meta.per_page, 10);
        assert_eq!(meta.total, 25);
        assert_eq!(meta.total_pages, 3);
        assert!(meta.has_next);
        assert!(meta.has_prev);

        let range = meta.item_range();
        assert_eq!(range, (11, 20)); // Items 11-20 on page 2
    }

    #[test]
    fn test_paginated_response() {
        let data = vec!["item1", "item2"];
        let meta = PaginationMeta::new(1, 2, 5);
        let response = PaginatedResponse::new(data, meta);

        assert_eq!(response.len(), 2);
        assert!(!response.is_empty());
        assert!(response.is_first_page());
        assert!(!response.is_last_page());
    }

    #[test]
    fn test_pagination_query_builder() {
        let base = "SELECT * FROM users WHERE active = true";
        let params = PaginationParams::new()
            .page(2)
            .per_page(5)
            .sort("name", "asc");

        let query = PaginationQueryBuilder::new(base)
            .with_params(params)
            .with_sorting()
            .with_limits()
            .build();

        assert!(query.contains("ORDER BY name ASC"));
        assert!(query.contains("LIMIT 5"));
        assert!(query.contains("OFFSET 5"));
    }
}
