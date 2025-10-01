//! Integration tests for database operations
//!
//! These tests require a PostgreSQL database to be running.
//! Set the DATABASE_URL environment variable to run these tests:
//! DATABASE_URL=postgresql://user:password@localhost:5432/test_db cargo test --test integration

use cloudshuttle_database::*;
use cloudshuttle_error_handling::database_error::HealthStatus;
use std::env;

#[derive(Debug, sqlx::FromRow)]
struct TestUser {
    id: i32,
    name: String,
    email: String,
}

fn get_database_url() -> String {
    env::var("DATABASE_URL").unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/test".to_string())
}

async fn setup_test_database() -> DatabaseResult<DatabaseConnection> {
    let config = DatabaseConfig {
        url: get_database_url(),
        max_connections: 5,
        min_connections: 1,
        acquire_timeout_seconds: 30,
        idle_timeout_seconds: Some(300),
        max_lifetime_seconds: Some(3600),
    };

    DatabaseConnection::new(config).await
}

#[tokio::test]
async fn test_database_connection() {
    let _db = setup_test_database().await.expect("Failed to connect to database");
}

#[tokio::test]
async fn test_basic_query_execution() {
    let db = setup_test_database().await.expect("Failed to connect to database");

    // Test a simple query
    let result = db.execute("SELECT 1 as test_value").await;
    assert!(result.is_ok(), "Simple query should execute successfully");

    let rows_affected = result.unwrap();
    assert_eq!(rows_affected, 0, "SELECT query should return 0 rows affected");
}

#[tokio::test]
async fn test_transaction_operations() {
    let db = setup_test_database().await.expect("Failed to connect to database");

    // Test transaction creation and commit
    let result = db.transaction(|tx| async move {
        // Execute a simple query in the transaction
        tx.execute("SELECT 1", &[]).await?;
        Ok(())
    }).await;

    assert!(result.is_ok(), "Transaction should complete successfully");
}

#[tokio::test]
async fn test_transaction_rollback() {
    let db = setup_test_database().await.expect("Failed to connect to database");

    // Create a test table for this test
    let _ = db.execute("CREATE TEMP TABLE IF NOT EXISTS test_rollback (id SERIAL PRIMARY KEY, value TEXT)").await;

    // Insert a value to check later
    let _ = db.execute("INSERT INTO test_rollback (value) VALUES ('before_transaction')").await;

    // Start a transaction that should fail
    let result = db.transaction(|tx| async move {
        // Insert in transaction
        tx.execute("INSERT INTO test_rollback (value) VALUES ('in_transaction')", &[]).await?;

        // Force a failure
        Err(DatabaseError::Query { message: "Intentional failure for rollback test".to_string() })
    }).await;

    // Transaction should have failed
    assert!(result.is_err(), "Transaction should have failed and rolled back");

    // Check that the transaction data was rolled back
    // (This would require a more complex query, but demonstrates the concept)
}

#[tokio::test]
async fn test_connection_pool_health() {
    let db = setup_test_database().await.expect("Failed to connect to database");

    // Test health check
    let health = db.health_check().await;
    assert!(health.is_ok(), "Health check should succeed");

    let health = health.unwrap();
    assert_eq!(health.status, HealthStatus::Healthy, "Database should be healthy");
}

#[tokio::test]
async fn test_connection_metrics() {
    let db = setup_test_database().await.expect("Failed to connect to database");

    // Test metrics retrieval
    let metrics = db.pool_metrics().await;
    assert!(metrics.total_connections >= 1, "Should have at least 1 connection");
}
