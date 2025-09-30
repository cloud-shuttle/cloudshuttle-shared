//! Transaction management utilities

use cloudshuttle_error_handling::DatabaseError;
use sqlx::{PgPool, Postgres, Transaction};
use std::ops::{Deref, DerefMut};

/// Database transaction wrapper
pub struct DatabaseTransaction<'a> {
    tx: Option<Transaction<'a, Postgres>>,
    committed: bool,
}

impl<'a> DatabaseTransaction<'a> {
    /// Create a new transaction
    pub fn new(tx: Transaction<'a, Postgres>) -> Self {
        Self {
            tx: Some(tx),
            committed: false,
        }
    }

    /// Commit the transaction
    pub async fn commit(mut self) -> Result<(), DatabaseError> {
        if let Some(tx) = self.tx.take() {
            tx.commit().await?;
            self.committed = true;
        }
        Ok(())
    }

    /// Rollback the transaction
    pub async fn rollback(mut self) -> Result<(), DatabaseError> {
        if let Some(tx) = self.tx.take() {
            tx.rollback().await?;
        }
        Ok(())
    }

    /// Check if transaction is committed
    pub fn is_committed(&self) -> bool {
        self.committed
    }

    /// Get a reference to the inner transaction
    pub fn as_inner(&self) -> &Transaction<'a, Postgres> {
        self.tx.as_ref().expect("Transaction should be present")
    }

    /// Get a mutable reference to the inner transaction
    pub fn as_inner_mut(&mut self) -> &mut Transaction<'a, Postgres> {
        self.tx.as_mut().expect("Transaction should be present")
    }
}

impl<'a> Deref for DatabaseTransaction<'a> {
    type Target = Transaction<'a, Postgres>;

    fn deref(&self) -> &Self::Target {
        self.as_inner()
    }
}

impl<'a> DerefMut for DatabaseTransaction<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_inner_mut()
    }
}

impl<'a> Drop for DatabaseTransaction<'a> {
    fn drop(&mut self) {
        // If transaction wasn't explicitly committed or rolled back,
        // it will be rolled back when dropped
        if !self.committed && self.tx.is_some() {
            // Note: In a real implementation, you might want to log this
            // as an implicit rollback
        }
    }
}

/// Transaction manager for handling nested transactions
pub struct TransactionManager<'a> {
    pool: &'a PgPool,
    transactions: Vec<DatabaseTransaction<'a>>,
}

impl<'a> TransactionManager<'a> {
    /// Create a new transaction manager
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            pool,
            transactions: Vec::new(),
        }
    }

    /// Begin a new transaction
    pub async fn begin(&mut self) -> Result<(), DatabaseError> {
        let tx = self.pool.begin().await?;
        self.transactions.push(DatabaseTransaction::new(tx));
        Ok(())
    }

    /// Commit the current transaction
    pub async fn commit(&mut self) -> Result<(), DatabaseError> {
        if let Some(tx) = self.transactions.pop() {
            tx.commit().await?;
        }
        Ok(())
    }

    /// Rollback the current transaction
    pub async fn rollback(&mut self) -> Result<(), DatabaseError> {
        if let Some(tx) = self.transactions.pop() {
            tx.rollback().await?;
        }
        Ok(())
    }

    /// Get the current transaction
    pub fn current(&mut self) -> Option<&mut DatabaseTransaction<'a>> {
        self.transactions.last_mut()
    }

    /// Execute a function within a transaction
    pub async fn execute<F, Fut, T>(&mut self, f: F) -> Result<T, DatabaseError>
    where
        F: FnOnce(&mut DatabaseTransaction<'a>) -> Fut,
        Fut: std::future::Future<Output = Result<T, DatabaseError>>,
    {
        self.begin().await?;
        let result = f(self.current().expect("Transaction should be present")).await;

        match result {
            Ok(value) => {
                self.commit().await?;
                Ok(value)
            }
            Err(err) => {
                self.rollback().await?;
                Err(err)
            }
        }
    }

    /// Get the transaction depth
    pub fn depth(&self) -> usize {
        self.transactions.len()
    }

    /// Check if we're in a transaction
    pub fn in_transaction(&self) -> bool {
        !self.transactions.is_empty()
    }
}

/// Transaction options
#[derive(Debug, Clone)]
pub struct TransactionOptions {
    pub isolation_level: IsolationLevel,
    pub read_only: bool,
    pub deferrable: bool,
}

impl Default for TransactionOptions {
    fn default() -> Self {
        Self {
            isolation_level: IsolationLevel::ReadCommitted,
            read_only: false,
            deferrable: false,
        }
    }
}

/// SQL isolation levels
#[derive(Debug, Clone)]
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
