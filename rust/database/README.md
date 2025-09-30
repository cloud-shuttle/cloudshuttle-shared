# CloudShuttle Database Layer

Database utilities, connection management, and query helpers for CloudShuttle services.

## Features

- Database connection management with connection pooling
- Query builder helpers and pagination
- Transaction management
- Migration utilities
- Common database types and configuration

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
cloudshuttle-database = "0.1.0"
cloudshuttle-error-handling = "0.1.0"
```

## Usage

### Basic Connection

```rust
use cloudshuttle_database::{DatabaseConnection, DatabaseConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create database connection
    let db = DatabaseConnection::new("postgresql://user:pass@localhost/db").await?;

    // Ping to test connectivity
    db.ping().await?;

    Ok(())
}
```

### Query Helpers

```rust
use cloudshuttle_database::QueryHelper;

let user = db.find_by_id::<User>("users", user_id).await?;
let users = db.find_by_ids::<User>("users", &[id1, id2, id3]).await?;

// Paginated queries
let result = db.find_with_pagination::<User>(
    "SELECT * FROM users WHERE active = true",
    &[],
    1, // page
    20 // per_page
).await?;

println!("Found {} users", result.total_count);
```

### Transactions

```rust
use cloudshuttle_database::DatabaseTransaction;

// Automatic commit/rollback
let result = db.transaction(|tx| async move {
    // Execute queries within transaction
    sqlx::query("INSERT INTO users ...")
        .execute(tx.as_inner())
        .await?;

    sqlx::query("INSERT INTO profiles ...")
        .execute(tx.as_inner())
        .await?;

    Ok(())
}).await;
```

### Configuration

```rust
use cloudshuttle_database::DatabaseConfig;

// From environment variables
let config = DatabaseConfig::from_env()?;

// From URL with custom options
let config = DatabaseConfig::from_url("postgresql://...")?
    .with_max_connections(50)
    .with_min_connections(10)
    .with_connection_timeout(30);
```

### Migrations

```rust
use cloudshuttle_database::MigrationRunner;
use std::path::Path;

let runner = MigrationRunner::new(db);

// Run all pending migrations
runner.run_migrations(Path::new("./migrations")).await?;

// Check migration status
let status = runner.status(Path::new("./migrations")).await?;
println!("Pending: {}, Applied: {}",
    status.pending.len(),
    status.applied.len()
);

// Rollback last migration
runner.rollback_last(Path::new("./migrations")).await?;
```

## Database Types

### Base Entity

```rust
use cloudshuttle_database::BaseEntity;

#[derive(sqlx::FromRow)]
struct User {
    #[sqlx(flatten)]
    base: BaseEntity,
    name: String,
    email: String,
}
```

### Tenant-Scoped Entity

```rust
use cloudshuttle_database::TenantEntity;

#[derive(sqlx::FromRow)]
struct TenantUser {
    #[sqlx(flatten)]
    tenant: TenantEntity,
    name: String,
    role: String,
}
```

### Pagination

```rust
use cloudshuttle_database::{PaginationParams, QueryResult};

let params = PaginationParams {
    page: 1,
    per_page: 20,
    sort_by: Some("created_at".to_string()),
    sort_order: SortOrder::Descending,
};
```

## Migration Files

Create migration files in your migrations directory:

```
migrations/
├── 001_initial_schema.up.sql
├── 001_initial_schema.down.sql
├── 002_add_users_table.up.sql
├── 002_add_users_table.down.sql
```

Example migration file:

```sql
-- 001_initial_schema.up.sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- 001_initial_schema.down.sql
DROP TABLE users;
```

## License

This project is licensed under MIT OR Apache-2.0.
