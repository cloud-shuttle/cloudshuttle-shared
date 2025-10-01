//! Database transaction management

use sqlx::{PgConnection, Transaction, postgres::PgRow};
use std::marker::PhantomData;
use cloudshuttle_error_handling::database_error::{DatabaseResult, DatabaseError};

/// Database transaction wrapper with automatic rollback on drop
pub struct DatabaseTransaction<'a> {
    tx: Option<Transaction<'a, sqlx::Postgres>>,
    committed: bool,
}

impl<'a> DatabaseTransaction<'a> {
    /// Create a new transaction wrapper
    pub fn new(tx: Transaction<'a, sqlx::Postgres>) -> Self {
        Self {
            tx: Some(tx),
            committed: false,
        }
    }

    /// Execute a query within the transaction
    pub async fn execute(&mut self, query: &str, params: &[&(dyn sqlx::Encode<'_, sqlx::Postgres> + Sync)]) -> DatabaseResult<u64> {
        let tx = self.tx.as_mut().ok_or_else(|| {
            sqlx::Error::Configuration("Transaction already committed or rolled back".into())
        })?;

        // TODO: Implement transaction execution
        // For now, return a placeholder
        Ok(0)
    }

    /// Execute a SELECT query and return the first row
    pub async fn fetch_one<T>(&mut self, query: &str) -> DatabaseResult<T>
    where
        T: for<'r> sqlx::FromRow<'r, PgRow> + Send + Unpin,
    {
        let tx = self.tx.as_mut().ok_or_else(|| {
            sqlx::Error::Configuration("Transaction already committed or rolled back".into())
        })?;

        // TODO: Implement transaction fetch_one
        Err(DatabaseError::Query { message: "Transaction fetch_one not implemented".to_string() })
    }

    /// Execute a SELECT query and return optional first row
    pub async fn fetch_optional<T>(&mut self, query: &str) -> DatabaseResult<Option<T>>
    where
        T: for<'r> sqlx::FromRow<'r, PgRow> + Send + Unpin,
    {
        let tx = self.tx.as_mut().ok_or_else(|| {
            sqlx::Error::Configuration("Transaction already committed or rolled back".into())
        })?;

        // TODO: Implement transaction fetch_optional
        Ok(None)
    }

    /// Execute a SELECT query and return all rows
    pub async fn fetch_all<T>(&mut self, query: &str) -> DatabaseResult<Vec<T>>
    where
        T: for<'r> sqlx::FromRow<'r, PgRow> + Send + Unpin,
    {
        let tx = self.tx.as_mut().ok_or_else(|| {
            sqlx::Error::Configuration("Transaction already committed or rolled back".into())
        })?;

        // TODO: Implement transaction fetch_all
        Ok(Vec::new())
    }

    /// Commit the transaction
    pub async fn commit(mut self) -> DatabaseResult<()> {
        if let Some(tx) = self.tx.take() {
            tx.commit().await?;
            self.committed = true;
            tracing::debug!("Database transaction committed");
        }
        Ok(())
    }

    /// Rollback the transaction
    pub async fn rollback(mut self) -> DatabaseResult<()> {
        if let Some(tx) = self.tx.take() {
            tx.rollback().await?;
            self.committed = true;
            tracing::debug!("Database transaction rolled back");
        }
        Ok(())
    }
}

impl<'a> Drop for DatabaseTransaction<'a> {
    fn drop(&mut self) {
        if !self.committed && self.tx.is_some() {
            tracing::warn!("Database transaction was not explicitly committed or rolled back, auto-rollback will occur");
        }
    }
}

/// Transaction result type
pub type TransactionResult<T> = DatabaseResult<T>;

/// Transaction builder for complex operations
pub struct TransactionBuilder<'a> {
    tx: DatabaseTransaction<'a>,
}

impl<'a> TransactionBuilder<'a> {
    pub fn new(tx: DatabaseTransaction<'a>) -> Self {
        Self { tx }
    }

    /// Execute an operation within the transaction
    pub async fn execute<F, Fut, T>(mut self, operation: F) -> DatabaseResult<T>
    where
        F: FnOnce(&mut DatabaseTransaction<'a>) -> Fut,
        Fut: std::future::Future<Output = DatabaseResult<T>>,
    {
        let result = operation(&mut self.tx).await?;
        self.tx.commit().await?;
        Ok(result)
    }

    /// Add a savepoint for partial rollback
    pub async fn savepoint<F, Fut, T>(&mut self, name: &str, operation: F) -> DatabaseResult<T>
    where
        F: FnOnce(&mut DatabaseTransaction<'a>) -> Fut,
        Fut: std::future::Future<Output = DatabaseResult<T>>,
    {
        // Create savepoint
        self.tx.execute(&format!("SAVEPOINT {}", name), &[]).await?;

        match operation(&mut self.tx).await {
            Ok(result) => {
                // Release savepoint
                self.tx.execute(&format!("RELEASE SAVEPOINT {}", name), &[]).await?;
                Ok(result)
            }
            Err(e) => {
                // Rollback to savepoint
                self.tx.execute(&format!("ROLLBACK TO SAVEPOINT {}", name), &[]).await?;
                Err(e)
            }
        }
    }
}

/// Transaction isolation levels
#[derive(Debug, Clone, Copy)]
pub enum IsolationLevel {
    ReadUncommitted,
    ReadCommitted,
    RepeatableRead,
    Serializable,
}

impl IsolationLevel {
    pub fn as_sql(&self) -> &'static str {
        match self {
            Self::ReadUncommitted => "READ UNCOMMITTED",
            Self::ReadCommitted => "READ COMMITTED",
            Self::RepeatableRead => "REPEATABLE READ",
            Self::Serializable => "SERIALIZABLE",
        }
    }
}

/// Transaction options
#[derive(Debug, Clone)]
pub struct TransactionOptions {
    pub isolation_level: Option<IsolationLevel>,
    pub read_only: bool,
    pub deferrable: bool,
}

impl Default for TransactionOptions {
    fn default() -> Self {
        Self {
            isolation_level: None,
            read_only: false,
            deferrable: false,
        }
    }
}

impl TransactionOptions {
    pub fn with_isolation_level(mut self, level: IsolationLevel) -> Self {
        self.isolation_level = Some(level);
        self
    }

    pub fn read_only(mut self) -> Self {
        self.read_only = true;
        self
    }

    pub fn deferrable(mut self) -> Self {
        self.deferrable = true;
        self
    }
}

/// Nested transaction support (savepoints)
pub struct NestedTransaction<'a> {
    parent: &'a mut DatabaseTransaction<'a>,
    savepoint_name: String,
    active: bool,
}

impl<'a> NestedTransaction<'a> {
    pub async fn new(parent: &'a mut DatabaseTransaction<'a>, name: String) -> DatabaseResult<Self> {
        parent.execute(&format!("SAVEPOINT {}", name), &[]).await?;

        Ok(Self {
            parent,
            savepoint_name: name,
            active: true,
        })
    }

    pub async fn commit(mut self) -> DatabaseResult<()> {
        if self.active {
            self.parent.execute(&format!("RELEASE SAVEPOINT {}", self.savepoint_name), &[]).await?;
            self.active = false;
        }
        Ok(())
    }

    pub async fn rollback(mut self) -> DatabaseResult<()> {
        if self.active {
            self.parent.execute(&format!("ROLLBACK TO SAVEPOINT {}", self.savepoint_name), &[]).await?;
            self.active = false;
        }
        Ok(())
    }
}

impl<'a> Drop for NestedTransaction<'a> {
    fn drop(&mut self) {
        if self.active {
            // Rollback on drop if not explicitly committed
            let _ = futures::executor::block_on(
                self.parent.execute(&format!("ROLLBACK TO SAVEPOINT {}", self.savepoint_name), &[])
            );
        }
    }
}
