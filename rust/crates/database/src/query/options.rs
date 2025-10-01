//! Query execution options and configuration

use serde::{Deserialize, Serialize};

/// Query execution options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryOptions {
    pub timeout: Option<std::time::Duration>,
    pub retry_count: u32,
    pub isolation_level: Option<String>,
    pub read_only: bool,
    pub defer_constraints: bool,
    pub query_plan: bool,
    pub explain_plan: bool,
}

impl QueryOptions {
    /// Create new query options with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set query timeout
    pub fn with_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set retry count for failed queries
    pub fn with_retry(mut self, count: u32) -> Self {
        self.retry_count = count;
        self
    }

    /// Set transaction isolation level
    pub fn with_isolation_level<S: Into<String>>(mut self, level: S) -> Self {
        self.isolation_level = Some(level.into());
        self
    }

    /// Mark query as read-only
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    /// Enable constraint deferral
    pub fn defer_constraints(mut self, defer: bool) -> Self {
        self.defer_constraints = defer;
        self
    }

    /// Enable query plan analysis
    pub fn with_query_plan(mut self, enable: bool) -> Self {
        self.query_plan = enable;
        self
    }

    /// Enable EXPLAIN plan analysis
    pub fn with_explain_plan(mut self, enable: bool) -> Self {
        self.explain_plan = enable;
        self
    }

    /// Check if query has a timeout configured
    pub fn has_timeout(&self) -> bool {
        self.timeout.is_some()
    }

    /// Check if query should be retried on failure
    pub fn should_retry(&self) -> bool {
        self.retry_count > 0
    }

    /// Get timeout duration if configured
    pub fn timeout(&self) -> Option<std::time::Duration> {
        self.timeout
    }

    /// Check if query plan analysis is enabled
    pub fn should_analyze_plan(&self) -> bool {
        self.query_plan || self.explain_plan
    }
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self {
            timeout: None,
            retry_count: 0,
            isolation_level: None,
            read_only: false,
            defer_constraints: false,
            query_plan: false,
            explain_plan: false,
        }
    }
}

/// Query execution context with options
#[derive(Debug, Clone)]
pub struct QueryExecutionContext {
    pub options: QueryOptions,
    pub start_time: std::time::Instant,
    pub attempt_count: u32,
    pub connection_id: Option<String>,
    pub query_id: uuid::Uuid,
}

impl QueryExecutionContext {
    /// Create new execution context
    pub fn new(options: QueryOptions) -> Self {
        Self {
            options,
            start_time: std::time::Instant::now(),
            attempt_count: 0,
            connection_id: None,
            query_id: uuid::Uuid::new_v4(),
        }
    }

    /// Increment attempt count (for retry logic)
    pub fn increment_attempt(&mut self) {
        self.attempt_count += 1;
    }

    /// Set connection identifier
    pub fn with_connection_id<S: Into<String>>(mut self, id: S) -> Self {
        self.connection_id = Some(id.into());
        self
    }

    /// Check if execution has timed out
    pub fn is_timed_out(&self) -> bool {
        if let Some(timeout) = self.options.timeout {
            self.elapsed() > timeout
        } else {
            false
        }
    }

    /// Get elapsed execution time
    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    /// Check if more retry attempts are available
    pub fn can_retry(&self) -> bool {
        self.attempt_count < self.options.retry_count
    }

    /// Get remaining retry attempts
    pub fn remaining_retries(&self) -> u32 {
        if self.attempt_count >= self.options.retry_count {
            0
        } else {
            self.options.retry_count - self.attempt_count
        }
    }
}

/// Query performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryMetrics {
    pub query_type: QueryType,
    pub execution_time: std::time::Duration,
    pub rows_affected: Option<u64>,
    pub rows_returned: Option<u64>,
    pub connection_time: std::time::Duration,
    pub parsing_time: std::time::Duration,
    pub planning_time: std::time::Duration,
    pub cache_hit: bool,
    pub prepared_statement: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryType {
    Select,
    Insert,
    Update,
    Delete,
    Other,
}

impl QueryMetrics {
    /// Create metrics for a successful query
    pub fn new(query_type: QueryType, execution_time: std::time::Duration) -> Self {
        Self {
            query_type,
            execution_time,
            rows_affected: None,
            rows_returned: None,
            connection_time: std::time::Duration::ZERO,
            parsing_time: std::time::Duration::ZERO,
            planning_time: std::time::Duration::ZERO,
            cache_hit: false,
            prepared_statement: false,
        }
    }

    /// Add rows affected count
    pub fn with_rows_affected(mut self, count: u64) -> Self {
        self.rows_affected = Some(count);
        self
    }

    /// Add rows returned count
    pub fn with_rows_returned(mut self, count: u64) -> Self {
        self.rows_returned = Some(count);
        self
    }

    /// Set connection acquisition time
    pub fn with_connection_time(mut self, time: std::time::Duration) -> Self {
        self.connection_time = time;
        self
    }

    /// Set query parsing time
    pub fn with_parsing_time(mut self, time: std::time::Duration) -> Self {
        self.parsing_time = time;
        self
    }

    /// Set query planning time
    pub fn with_planning_time(mut self, time: std::time::Duration) -> Self {
        self.planning_time = time;
        self
    }

    /// Mark as cache hit
    pub fn cache_hit(mut self, hit: bool) -> Self {
        self.cache_hit = hit;
        self
    }

    /// Mark as using prepared statement
    pub fn prepared_statement(mut self, prepared: bool) -> Self {
        self.prepared_statement = prepared;
        self
    }

    /// Get total query time (connection + execution)
    pub fn total_time(&self) -> std::time::Duration {
        self.connection_time + self.execution_time
    }

    /// Get execution time in milliseconds
    pub fn execution_time_ms(&self) -> f64 {
        self.execution_time.as_secs_f64() * 1000.0
    }

    /// Check if query was read-only
    pub fn is_read_only(&self) -> bool {
        matches!(self.query_type, QueryType::Select)
    }
}

/// Query optimization hints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryHints {
    pub index_hints: Vec<String>,
    pub join_hints: Vec<String>,
    pub parallel_execution: Option<u32>,
    pub memory_limit: Option<usize>,
    pub temp_space_limit: Option<usize>,
}

impl Default for QueryHints {
    fn default() -> Self {
        Self {
            index_hints: Vec::new(),
            join_hints: Vec::new(),
            parallel_execution: None,
            memory_limit: None,
            temp_space_limit: None,
        }
    }
}

impl QueryHints {
    /// Add index hint
    pub fn with_index_hint<S: Into<String>>(mut self, hint: S) -> Self {
        self.index_hints.push(hint.into());
        self
    }

    /// Add join hint
    pub fn with_join_hint<S: Into<String>>(mut self, hint: S) -> Self {
        self.join_hints.push(hint.into());
        self
    }

    /// Set parallel execution degree
    pub fn with_parallel_execution(mut self, degree: u32) -> Self {
        self.parallel_execution = Some(degree);
        self
    }

    /// Set memory limit in bytes
    pub fn with_memory_limit(mut self, limit: usize) -> Self {
        self.memory_limit = Some(limit);
        self
    }

    /// Set temporary space limit in bytes
    pub fn with_temp_space_limit(mut self, limit: usize) -> Self {
        self.temp_space_limit = Some(limit);
        self
    }

    /// Check if any hints are configured
    pub fn has_hints(&self) -> bool {
        !self.index_hints.is_empty()
            || !self.join_hints.is_empty()
            || self.parallel_execution.is_some()
            || self.memory_limit.is_some()
            || self.temp_space_limit.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_query_options_default() {
        let options = QueryOptions::default();
        assert!(options.timeout.is_none());
        assert_eq!(options.retry_count, 0);
        assert!(options.isolation_level.is_none());
        assert!(!options.read_only);
        assert!(!options.defer_constraints);
        assert!(!options.query_plan);
        assert!(!options.explain_plan);
    }

    #[test]
    fn test_query_options_fluent_api() {
        let options = QueryOptions::new()
            .with_timeout(Duration::from_secs(30))
            .with_retry(3)
            .with_isolation_level("SERIALIZABLE")
            .read_only(true)
            .with_query_plan(true);

        assert_eq!(options.timeout, Some(Duration::from_secs(30)));
        assert_eq!(options.retry_count, 3);
        assert_eq!(options.isolation_level, Some("SERIALIZABLE".to_string()));
        assert!(options.read_only);
        assert!(options.query_plan);
        assert!(options.has_timeout());
        assert!(options.should_retry());
        assert!(options.should_analyze_plan());
    }

    #[test]
    fn test_query_execution_context() {
        let options = QueryOptions::new().with_timeout(Duration::from_secs(5)).with_retry(3);
        let mut context = QueryExecutionContext::new(options);

        assert!(!context.is_timed_out());
        assert!(context.can_retry());
        assert_eq!(context.remaining_retries(), 3);

        context.increment_attempt();
        assert_eq!(context.attempt_count, 1);

        let context_with_conn = context.with_connection_id("conn_123");
        assert_eq!(context_with_conn.connection_id, Some("conn_123".to_string()));
    }

    #[test]
    fn test_query_metrics() {
        let metrics = QueryMetrics::new(QueryType::Select, Duration::from_millis(50))
            .with_rows_returned(100)
            .with_connection_time(Duration::from_millis(10))
            .cache_hit(true)
            .prepared_statement(true);

        assert!(metrics.is_read_only());
        assert_eq!(metrics.rows_returned, Some(100));
        assert_eq!(metrics.execution_time_ms(), 50.0);
        assert_eq!(metrics.total_time(), Duration::from_millis(60));
        assert!(metrics.cache_hit);
        assert!(metrics.prepared_statement);
    }

    #[test]
    fn test_query_hints() {
        let hints = QueryHints::default()
            .with_index_hint("USE INDEX idx_users_email")
            .with_join_hint("FORCE ORDER")
            .with_parallel_execution(4)
            .with_memory_limit(1024 * 1024); // 1MB

        assert!(hints.has_hints());
        assert_eq!(hints.index_hints.len(), 1);
        assert_eq!(hints.join_hints.len(), 1);
        assert_eq!(hints.parallel_execution, Some(4));
        assert_eq!(hints.memory_limit, Some(1024 * 1024));
    }

    #[test]
    fn test_query_hints_empty() {
        let hints = QueryHints::default();
        assert!(!hints.has_hints());
    }
}
