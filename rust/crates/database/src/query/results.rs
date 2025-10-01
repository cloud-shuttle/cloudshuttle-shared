//! Query result handling and execution tracking

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Result of a database query execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub query: String,
    pub params: Vec<serde_json::Value>,
    pub success: bool,
    pub execution_time: std::time::Duration,
    pub rows_affected: Option<u64>,
    pub rows_returned: Option<u64>,
    pub error_message: Option<String>,
    pub executed_at: DateTime<Utc>,
}

impl QueryResult {
    /// Create a successful query result
    pub fn success<S: Into<String>>(
        query: S,
        params: Vec<serde_json::Value>,
        execution_time: std::time::Duration,
    ) -> Self {
        Self {
            query: query.into(),
            params,
            success: true,
            execution_time,
            rows_affected: None,
            rows_returned: None,
            error_message: None,
            executed_at: Utc::now(),
        }
    }

    /// Create a failed query result
    pub fn failure<S: Into<String>>(
        query: S,
        params: Vec<serde_json::Value>,
        execution_time: std::time::Duration,
        error_message: String,
    ) -> Self {
        Self {
            query: query.into(),
            params,
            success: false,
            execution_time,
            rows_affected: None,
            rows_returned: None,
            error_message: Some(error_message),
            executed_at: Utc::now(),
        }
    }

    /// Add rows affected count
    pub fn with_rows_affected(mut self, rows: u64) -> Self {
        self.rows_affected = Some(rows);
        self
    }

    /// Add rows returned count
    pub fn with_rows_returned(mut self, rows: u64) -> Self {
        self.rows_returned = Some(rows);
        self
    }

    /// Check if the query was successful
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Get the execution time in milliseconds
    pub fn execution_time_ms(&self) -> f64 {
        self.execution_time.as_secs_f64() * 1000.0
    }
}

/// Paginated query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResult<T> {
    pub data: Vec<T>,
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_previous: bool,
}

impl<T> PaginatedResult<T> {
    pub fn new(
        data: Vec<T>,
        total_count: u64,
        page: u32,
        page_size: u32,
    ) -> Self {
        let total_pages = if page_size > 0 {
            ((total_count as f64) / (page_size as f64)).ceil() as u32
        } else {
            0
        };

        Self {
            data,
            total_count,
            page,
            page_size,
            total_pages,
            has_next: page < total_pages,
            has_previous: page > 1,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}

/// Query execution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryStats {
    pub total_queries: u64,
    pub successful_queries: u64,
    pub failed_queries: u64,
    pub total_execution_time: std::time::Duration,
    pub average_execution_time: std::time::Duration,
    pub slowest_query_time: std::time::Duration,
    pub fastest_query_time: std::time::Duration,
}

impl Default for QueryStats {
    fn default() -> Self {
        Self {
            total_queries: 0,
            successful_queries: 0,
            failed_queries: 0,
            total_execution_time: std::time::Duration::ZERO,
            average_execution_time: std::time::Duration::ZERO,
            slowest_query_time: std::time::Duration::ZERO,
            fastest_query_time: std::time::Duration::MAX,
        }
    }
}

impl QueryStats {
    /// Add a query result to the statistics
    pub fn add_result(&mut self, result: &QueryResult) {
        self.total_queries += 1;
        self.total_execution_time += result.execution_time;

        if result.success {
            self.successful_queries += 1;
        } else {
            self.failed_queries += 1;
        }

        // Update min/max times
        if result.execution_time > self.slowest_query_time {
            self.slowest_query_time = result.execution_time;
        }
        if result.execution_time < self.fastest_query_time {
            self.fastest_query_time = result.execution_time;
        }

        // Update average
        if self.total_queries > 0 {
            self.average_execution_time = self.total_execution_time / self.total_queries as u32;
        }
    }

    /// Get success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_queries == 0 {
            0.0
        } else {
            (self.successful_queries as f64 / self.total_queries as f64) * 100.0
        }
    }

    /// Get average execution time in milliseconds
    pub fn average_execution_time_ms(&self) -> f64 {
        self.average_execution_time.as_secs_f64() * 1000.0
    }
}

/// Query execution context for tracking and debugging
#[derive(Debug, Clone)]
pub struct QueryExecutionContext {
    pub query_id: uuid::Uuid,
    pub query: String,
    pub params: Vec<serde_json::Value>,
    pub start_time: std::time::Instant,
    pub timeout: Option<std::time::Duration>,
    pub retry_count: u32,
    pub tags: std::collections::HashMap<String, String>,
}

impl QueryExecutionContext {
    pub fn new<S: Into<String>>(query: S, params: Vec<serde_json::Value>) -> Self {
        Self {
            query_id: uuid::Uuid::new_v4(),
            query: query.into(),
            params,
            start_time: std::time::Instant::now(),
            timeout: None,
            retry_count: 0,
            tags: std::collections::HashMap::new(),
        }
    }

    pub fn with_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn with_retry_count(mut self, count: u32) -> Self {
        self.retry_count = count;
        self
    }

    pub fn with_tag<S: Into<String>>(mut self, key: S, value: S) -> Self {
        self.tags.insert(key.into(), value.into());
        self
    }

    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    pub fn is_timed_out(&self) -> bool {
        if let Some(timeout) = self.timeout {
            self.elapsed() > timeout
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_query_result_success() {
        let result = QueryResult::success(
            "SELECT * FROM users",
            vec![serde_json::json!(1)],
            Duration::from_millis(50),
        ).with_rows_affected(5);

        assert!(result.is_success());
        assert_eq!(result.query, "SELECT * FROM users");
        assert_eq!(result.rows_affected, Some(5));
        assert!(result.error_message.is_none());
        assert!(result.execution_time_ms() >= 50.0);
    }

    #[test]
    fn test_query_result_failure() {
        let result = QueryResult::failure(
            "SELECT * FROM invalid_table",
            vec![],
            Duration::from_millis(10),
            "Table does not exist".to_string(),
        );

        assert!(!result.is_success());
        assert!(result.error_message.is_some());
        assert_eq!(result.error_message.as_ref().unwrap(), "Table does not exist");
    }

    #[test]
    fn test_paginated_result() {
        let data = vec![1, 2, 3, 4, 5];
        let result = PaginatedResult::new(data.clone(), 25, 1, 5);

        assert_eq!(result.data, data);
        assert_eq!(result.total_count, 25);
        assert_eq!(result.page, 1);
        assert_eq!(result.page_size, 5);
        assert_eq!(result.total_pages, 5);
        assert!(result.has_next);
        assert!(!result.has_previous);
    }

    #[test]
    fn test_query_stats() {
        let mut stats = QueryStats::default();

        let success_result = QueryResult::success("SELECT 1", vec![], Duration::from_millis(10));
        let failure_result = QueryResult::failure("INVALID", vec![], Duration::from_millis(5), "error".to_string());

        stats.add_result(&success_result);
        stats.add_result(&failure_result);

        assert_eq!(stats.total_queries, 2);
        assert_eq!(stats.successful_queries, 1);
        assert_eq!(stats.failed_queries, 1);
        assert_eq!(stats.success_rate(), 50.0);
    }

    #[test]
    fn test_query_execution_context() {
        let context = QueryExecutionContext::new("SELECT * FROM users", vec![serde_json::json!(1)])
            .with_timeout(Duration::from_secs(30))
            .with_tag("table", "users");

        assert!(!context.query_id.is_nil());
        assert_eq!(context.query, "SELECT * FROM users");
        assert_eq!(context.timeout, Some(Duration::from_secs(30)));
        assert_eq!(context.tags.get("table"), Some(&"users".to_string()));
        assert!(!context.is_timed_out());
    }
}
