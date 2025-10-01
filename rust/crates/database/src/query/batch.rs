//! Batch query operations and execution

use serde::{Deserialize, Serialize};

/// Batch operation support
#[derive(Debug, Clone)]
pub struct BatchOperation {
    operations: Vec<BatchItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchItem {
    pub query: String,
    pub params: Vec<serde_json::Value>,
}

impl BatchOperation {
    /// Create a new empty batch operation
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    /// Add a query to the batch
    pub fn add<S: Into<String>>(mut self, query: S, params: Vec<serde_json::Value>) -> Self {
        self.operations.push(BatchItem {
            query: query.into(),
            params,
        });
        self
    }

    /// Get all batch operations
    pub fn operations(&self) -> &[BatchItem] {
        &self.operations
    }

    /// Check if batch is empty
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    /// Get the number of operations in the batch
    pub fn len(&self) -> usize {
        self.operations.len()
    }

    /// Clear all operations
    pub fn clear(&mut self) {
        self.operations.clear();
    }

    /// Add multiple operations at once
    pub fn extend(&mut self, operations: Vec<BatchItem>) {
        self.operations.extend(operations);
    }
}

impl Default for BatchOperation {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoIterator for BatchOperation {
    type Item = BatchItem;
    type IntoIter = std::vec::IntoIter<BatchItem>;

    fn into_iter(self) -> Self::IntoIter {
        self.operations.into_iter()
    }
}

impl<'a> IntoIterator for &'a BatchOperation {
    type Item = &'a BatchItem;
    type IntoIter = std::slice::Iter<'a, BatchItem>;

    fn into_iter(self) -> Self::IntoIter {
        self.operations.iter()
    }
}

/// Batch execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    pub total_operations: usize,
    pub successful_operations: usize,
    pub failed_operations: usize,
    pub results: Vec<BatchItemResult>,
    pub execution_time: std::time::Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchItemResult {
    pub index: usize,
    pub success: bool,
    pub rows_affected: Option<u64>,
    pub error_message: Option<String>,
    pub execution_time: std::time::Duration,
}

impl BatchResult {
    /// Create a new batch result
    pub fn new(total_operations: usize, execution_time: std::time::Duration) -> Self {
        Self {
            total_operations,
            successful_operations: 0,
            failed_operations: 0,
            results: Vec::new(),
            execution_time,
        }
    }

    /// Add a successful result
    pub fn add_success(&mut self, index: usize, rows_affected: Option<u64>, execution_time: std::time::Duration) {
        self.successful_operations += 1;
        self.results.push(BatchItemResult {
            index,
            success: true,
            rows_affected,
            error_message: None,
            execution_time,
        });
    }

    /// Add a failed result
    pub fn add_failure(&mut self, index: usize, error_message: String, execution_time: std::time::Duration) {
        self.failed_operations += 1;
        self.results.push(BatchItemResult {
            index,
            success: false,
            rows_affected: None,
            error_message: Some(error_message),
            execution_time,
        });
    }

    /// Check if all operations succeeded
    pub fn is_success(&self) -> bool {
        self.failed_operations == 0
    }

    /// Get success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_operations == 0 {
            0.0
        } else {
            (self.successful_operations as f64 / self.total_operations as f64) * 100.0
        }
    }

    /// Get total rows affected across all operations
    pub fn total_rows_affected(&self) -> u64 {
        self.results.iter()
            .filter_map(|r| r.rows_affected)
            .sum()
    }
}

/// Batch execution options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOptions {
    pub continue_on_error: bool,
    pub max_parallel_operations: Option<usize>,
    pub timeout_per_operation: Option<std::time::Duration>,
    pub transaction_mode: BatchTransactionMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatchTransactionMode {
    /// Each operation in its own transaction
    PerOperation,
    /// All operations in a single transaction
    AllInOne,
    /// No transactions (auto-commit mode)
    None,
}

impl Default for BatchOptions {
    fn default() -> Self {
        Self {
            continue_on_error: false,
            max_parallel_operations: None,
            timeout_per_operation: None,
            transaction_mode: BatchTransactionMode::PerOperation,
        }
    }
}

/// Batch builder for fluent API
pub struct BatchBuilder {
    operations: Vec<BatchItem>,
    options: BatchOptions,
}

impl BatchBuilder {
    /// Create a new batch builder
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
            options: BatchOptions::default(),
        }
    }

    /// Add an INSERT operation
    pub fn insert<S: Into<String>>(mut self, table: S, values: serde_json::Map<String, serde_json::Value>) -> Self {
        let columns: Vec<String> = values.keys().cloned().collect();
        let params: Vec<serde_json::Value> = values.values().cloned().collect();

        let placeholders: Vec<String> = (1..=params.len()).map(|i| format!("${}", i)).collect();

        let query = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            table.into(),
            columns.join(", "),
            placeholders.join(", ")
        );

        self.operations.push(BatchItem { query, params });
        self
    }

    /// Add an UPDATE operation
    pub fn update<S: Into<String>>(mut self, table: S, values: serde_json::Map<String, serde_json::Value>, where_clause: S, where_params: Vec<serde_json::Value>) -> Self {
        let set_clause: Vec<String> = values.keys()
            .enumerate()
            .map(|(i, col)| format!("{} = ${}", col, i + 1))
            .collect();

        let mut params = values.values().cloned().collect::<Vec<_>>();
        params.extend(where_params);

        let query = format!(
            "UPDATE {} SET {} WHERE {}",
            table.into(),
            set_clause.join(", "),
            where_clause.into()
        );

        self.operations.push(BatchItem { query, params });
        self
    }

    /// Add a DELETE operation
    pub fn delete<S: Into<String>>(mut self, table: S, where_clause: S, where_params: Vec<serde_json::Value>) -> Self {
        let query = format!("DELETE FROM {} WHERE {}", table.into(), where_clause.into());

        self.operations.push(BatchItem { query, params: where_params });
        self
    }

    /// Add a raw SQL operation
    pub fn raw<S: Into<String>>(mut self, query: S, params: Vec<serde_json::Value>) -> Self {
        self.operations.push(BatchItem {
            query: query.into(),
            params,
        });
        self
    }

    /// Configure batch options
    pub fn with_options(mut self, options: BatchOptions) -> Self {
        self.options = options;
        self
    }

    /// Set continue on error mode
    pub fn continue_on_error(mut self, continue_on_error: bool) -> Self {
        self.options.continue_on_error = continue_on_error;
        self
    }

    /// Set transaction mode
    pub fn transaction_mode(mut self, mode: BatchTransactionMode) -> Self {
        self.options.transaction_mode = mode;
        self
    }

    /// Build the final batch operation
    pub fn build(self) -> (BatchOperation, BatchOptions) {
        (
            BatchOperation { operations: self.operations },
            self.options,
        )
    }
}

impl Default for BatchBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_operation_creation() {
        let batch = BatchOperation::new()
            .add("SELECT 1", vec![])
            .add("SELECT 2", vec![serde_json::json!(42)]);

        assert_eq!(batch.len(), 2);
        assert!(!batch.is_empty());

        let operations: Vec<_> = batch.operations().iter().collect();
        assert_eq!(operations[0].query, "SELECT 1");
        assert_eq!(operations[1].query, "SELECT 2");
        assert_eq!(operations[1].params[0], serde_json::json!(42));
    }

    #[test]
    fn test_batch_result_tracking() {
        let mut result = BatchResult::new(3, std::time::Duration::from_millis(100));

        result.add_success(0, Some(1), std::time::Duration::from_millis(10));
        result.add_success(1, Some(2), std::time::Duration::from_millis(20));
        result.add_failure(2, "Constraint violation".to_string(), std::time::Duration::from_millis(30));

        assert_eq!(result.total_operations, 3);
        assert_eq!(result.successful_operations, 2);
        assert_eq!(result.failed_operations, 1);
        assert!(!result.is_success());
        assert_eq!(result.success_rate(), 66.66666666666666);
        assert_eq!(result.total_rows_affected(), 3);
    }

    #[test]
    fn test_batch_builder_fluent_api() {
        let (batch, options) = BatchBuilder::new()
            .insert("users", serde_json::json!({"name": "Alice", "email": "alice@example.com"}).as_object().unwrap().clone())
            .update(
                "users",
                serde_json::json!({"name": "Bob"}).as_object().unwrap().clone(),
                "id = $2",
                vec![serde_json::json!(1)]
            )
            .delete("users", "id = $1", vec![serde_json::json!(2)])
            .continue_on_error(true)
            .build();

        assert_eq!(batch.len(), 3);
        assert!(options.continue_on_error);

        // Check INSERT query
        let insert_op = &batch.operations()[0];
        assert!(insert_op.query.contains("INSERT INTO users"));
        assert!(insert_op.query.contains("(email, name)"));
        assert!(insert_op.query.contains("VALUES ($1, $2)"));

        // Check UPDATE query
        let update_op = &batch.operations()[1];
        assert!(update_op.query.contains("UPDATE users SET name = $1 WHERE id = $2"));

        // Check DELETE query
        let delete_op = &batch.operations()[2];
        assert!(delete_op.query.contains("DELETE FROM users WHERE id = $1"));
    }

    #[test]
    fn test_batch_options_default() {
        let options = BatchOptions::default();
        assert!(!options.continue_on_error);
        assert!(options.max_parallel_operations.is_none());
        assert!(options.timeout_per_operation.is_none());
        assert!(matches!(options.transaction_mode, BatchTransactionMode::PerOperation));
    }

    #[test]
    fn test_batch_empty_operations() {
        let batch = BatchOperation::new();
        assert!(batch.is_empty());
        assert_eq!(batch.len(), 0);
    }
}
