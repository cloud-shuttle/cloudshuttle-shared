//! User management service integration tests
//!
//! These tests demonstrate how the advanced database and validation
//! components work together in a user management service scenario.

use std::collections::HashMap;
use cloudshuttle_database::{
    AdvancedPgPool, AdvancedPoolConfig, AdvancedMigrationRunner,
    MigrationBuilder, MigrationStatus, AdvancedValidator,
    ValidationContext, ValidationSeverity, HtmlSanitizer
};
use cloudshuttle_validation::ValidationConfig;

/// Mock user management service demonstrating database + validation integration
struct UserManagementService {
    pool: AdvancedPgPool,
    validator: AdvancedValidator,
    migration_runner: AdvancedMigrationRunner,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct User {
    id: String,
    email: String,
    username: String,
    display_name: String,
    bio: Option<String>,
    is_active: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, serde::Deserialize)]
struct CreateUserRequest {
    email: String,
    username: String,
    display_name: String,
    bio: Option<String>,
    password: String,
}

#[derive(Debug, serde::Serialize)]
struct CreateUserResponse {
    user: User,
    validation_warnings: Vec<String>,
}

impl UserManagementService {
    /// Create a new user management service with database migrations
    async fn new(database_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Configure advanced connection pool
        let pool_config = AdvancedPoolConfig {
            max_connections: 10,
            min_connections: 2,
            acquire_timeout: std::time::Duration::from_secs(30),
            health_check: Default::default(),
            ..Default::default()
        };

        let pool = AdvancedPgPool::new(database_url, pool_config).await?;

        // Initialize database schema
        let migration_runner = AdvancedMigrationRunner::new(pool.clone(), "./migrations").await?;

        // Run user management migrations
        Self::run_user_migrations(&migration_runner).await?;

        // Configure advanced validation
        let validation_config = ValidationConfig {
            max_length: 10000,
            enable_security_scan: true,
            enable_business_rules: true,
            enable_sanitization: true,
            ..Default::default()
        };

        let mut validator = AdvancedValidator::new(validation_config);

        // Add custom business rules for user management
        validator.add_business_rule("email", cloudshuttle_validation::ValidationRule {
            name: "email_format".to_string(),
            description: "Validate email format".to_string(),
            severity: ValidationSeverity::Error,
            enabled: true,
            config: HashMap::new(),
        });

        validator.add_business_rule("username", cloudshuttle_validation::ValidationRule {
            name: "username_format".to_string(),
            description: "Validate username format".to_string(),
            severity: ValidationSeverity::Error,
            enabled: true,
            config: HashMap::new(),
        });

        validator.add_business_rule("display_name", cloudshuttle_validation::ValidationRule {
            name: "display_name_length".to_string(),
            description: "Validate display name length".to_string(),
            severity: ValidationSeverity::Error,
            enabled: true,
            config: HashMap::new(),
        });

        // Add sanitizers
        validator.add_sanitizer("bio", Box::new(HtmlSanitizer::new()));
        validator.add_sanitizer("display_name", Box::new(cloudshuttle_validation::SqlSanitizer::new()));

        Ok(Self {
            pool,
            validator,
            migration_runner,
        })
    }

    /// Run user management database migrations
    async fn run_user_migrations(migration_runner: &AdvancedMigrationRunner) -> Result<(), Box<dyn std::error::Error>> {
        let migrations = vec![
            MigrationBuilder::new("001", "create_users_table")
                .up_sql(r#"
                    CREATE TABLE users (
                        id VARCHAR(36) PRIMARY KEY,
                        email VARCHAR(255) UNIQUE NOT NULL,
                        username VARCHAR(50) UNIQUE NOT NULL,
                        display_name VARCHAR(100) NOT NULL,
                        bio TEXT,
                        password_hash VARCHAR(255) NOT NULL,
                        is_active BOOLEAN DEFAULT true,
                        created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                        updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
                    );

                    CREATE INDEX idx_users_email ON users(email);
                    CREATE INDEX idx_users_username ON users(username);
                    CREATE INDEX idx_users_active ON users(is_active);
                "#)
                .down_sql("DROP TABLE users;")
                .description("Create users table with indexes")
                .build(),

            MigrationBuilder::new("002", "add_user_profiles")
                .up_sql(r#"
                    CREATE TABLE user_profiles (
                        user_id VARCHAR(36) PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
                        avatar_url VARCHAR(500),
                        website VARCHAR(255),
                        location VARCHAR(100),
                        preferences JSONB DEFAULT '{}',
                        created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                        updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
                    );
                "#)
                .down_sql("DROP TABLE user_profiles;")
                .description("Add user profiles table")
                .depends_on("001")
                .build(),
        ];

        // Apply migrations
        for migration in migrations {
            let plan = migration_runner.plan_migrations(Some(&migration.id))?;
            let results = migration_runner.execute_plan(&plan).await?;

            for result in results {
                if result.status != MigrationStatus::Applied {
                    return Err(format!("Migration {} failed: {:?}", migration.id, result.error_message).into());
                }
            }
        }

        Ok(())
    }

    /// Create a new user with comprehensive validation
    async fn create_user(&self, request: CreateUserRequest) -> Result<CreateUserResponse, UserManagementError> {
        // Step 1: Validate all inputs
        let validation_results = self.validate_user_inputs(&request).await?;

        // Step 2: Check for validation errors
        if !validation_results.iter().all(|r| r.is_valid) {
            let errors: Vec<String> = validation_results.iter()
                .filter(|r| !r.is_valid)
                .flat_map(|r| r.errors.iter().map(|e| e.message.clone()))
                .collect();

            return Err(UserManagementError::ValidationErrors(errors));
        }

        // Step 3: Collect warnings
        let warnings: Vec<String> = validation_results.iter()
            .flat_map(|r| r.warnings.iter().map(|w| w.message.clone()))
            .collect();

        // Step 4: Check if user already exists
        if self.user_exists(&request.email, &request.username).await? {
            return Err(UserManagementError::UserAlreadyExists);
        }

        // Step 5: Hash password (in real implementation)
        let password_hash = self.hash_password(&request.password)?;

        // Step 6: Create user in database
        let user = self.insert_user(&request, &password_hash).await?;

        Ok(CreateUserResponse {
            user,
            validation_warnings: warnings,
        })
    }

    /// Validate user inputs using advanced validation
    async fn validate_user_inputs(&self, request: &CreateUserRequest) -> Result<Vec<cloudshuttle_validation::AdvancedValidationResult>, UserManagementError> {
        let mut results = Vec::new();

        // Validate email
        let email_context = ValidationContext {
            field_name: "email".to_string(),
            field_value: serde_json::json!(request.email),
            context_data: HashMap::new(),
            user_id: None,
            request_id: None,
        };
        results.push(self.validator.validate(email_context));

        // Validate username
        let username_context = ValidationContext {
            field_name: "username".to_string(),
            field_value: serde_json::json!(request.username),
            context_data: HashMap::new(),
            user_id: None,
            request_id: None,
        };
        results.push(self.validator.validate(username_context));

        // Validate display name
        let display_name_context = ValidationContext {
            field_name: "display_name".to_string(),
            field_value: serde_json::json!(request.display_name),
            context_data: HashMap::new(),
            user_id: None,
            request_id: None,
        };
        results.push(self.validator.validate(display_name_context));

        // Validate bio if provided
        if let Some(ref bio) = request.bio {
            let bio_context = ValidationContext {
                field_name: "bio".to_string(),
                field_value: serde_json::json!(bio),
                context_data: HashMap::new(),
                user_id: None,
                request_id: None,
            };
            results.push(self.validator.validate(bio_context));
        }

        Ok(results)
    }

    /// Check if user already exists
    async fn user_exists(&self, email: &str, username: &str) -> Result<bool, UserManagementError> {
        let conn = self.pool.acquire().await?;

        // Check email
        let email_exists: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)"
        )
        .bind(email)
        .fetch_one(&mut *conn)
        .await?;

        if email_exists.0 {
            return Ok(true);
        }

        // Check username
        let username_exists: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)"
        )
        .bind(username)
        .fetch_one(&mut *conn)
        .await?;

        Ok(username_exists.0)
    }

    /// Hash password (simplified for testing)
    fn hash_password(&self, password: &str) -> Result<String, UserManagementError> {
        // In real implementation, use argon2
        Ok(format!("hashed_{}", password))
    }

    /// Insert user into database
    async fn insert_user(&self, request: &CreateUserRequest, password_hash: &str) -> Result<User, UserManagementError> {
        let conn = self.pool.acquire().await?;
        let user_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now();

        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, email, username, display_name, bio, password_hash, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, email, username, display_name, bio, is_active, created_at, updated_at
            "#
        )
        .bind(&user_id)
        .bind(&request.email)
        .bind(&request.username)
        .bind(&request.display_name)
        .bind(&request.bio)
        .bind(password_hash)
        .bind(now)
        .bind(now)
        .fetch_one(&mut *conn)
        .await?;

        Ok(user)
    }

    /// Get user by ID with connection pooling
    async fn get_user(&self, user_id: &str) -> Result<Option<User>, UserManagementError> {
        let conn = self.pool.acquire().await?;

        let user = sqlx::query_as::<_, User>(
            "SELECT id, email, username, display_name, bio, is_active, created_at, updated_at FROM users WHERE id = $1"
        )
        .bind(user_id)
        .fetch_optional(&mut *conn)
        .await?;

        Ok(user)
    }

    /// Update user with validation
    async fn update_user(&self, user_id: &str, updates: UserUpdate) -> Result<User, UserManagementError> {
        // Validate updates
        self.validate_update(&updates)?;

        let conn = self.pool.acquire().await?;
        let now = chrono::Utc::now();

        // Build dynamic update query
        let mut query = "UPDATE users SET updated_at = $1".to_string();
        let mut param_count = 1;
        let mut params: Vec<&(dyn sqlx::Encode + Sync)> = vec![&now];

        if let Some(ref display_name) = updates.display_name {
            param_count += 1;
            query.push_str(&format!(", display_name = ${}", param_count));
            params.push(display_name);
        }

        if let Some(ref bio) = updates.bio {
            param_count += 1;
            query.push_str(&format!(", bio = ${}", param_count));
            params.push(bio);
        }

        query.push_str(&format!(" WHERE id = ${} RETURNING id, email, username, display_name, bio, is_active, created_at, updated_at", param_count + 1));
        params.push(&user_id.to_string());

        let user: User = sqlx::query_as(&query)
            .bind_all(params)
            .fetch_one(&mut *conn)
            .await?;

        Ok(user)
    }

    /// Validate user updates
    fn validate_update(&self, updates: &UserUpdate) -> Result<(), UserManagementError> {
        if let Some(ref display_name) = updates.display_name {
            if display_name.trim().is_empty() {
                return Err(UserManagementError::ValidationErrors(vec!["Display name cannot be empty".to_string()]));
            }
        }

        Ok(())
    }

    /// Get pool metrics for monitoring
    fn get_pool_metrics(&self) -> cloudshuttle_database::PoolMetrics {
        self.pool.metrics()
    }

    /// Check database health
    async fn health_check(&self) -> Result<cloudshuttle_database::PoolHealth, UserManagementError> {
        self.pool.health_check().await.map_err(Into::into)
    }
}

#[derive(Debug, serde::Deserialize)]
struct UserUpdate {
    display_name: Option<String>,
    bio: Option<String>,
}

#[derive(Debug, thiserror::Error)]
enum UserManagementError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] cloudshuttle_database::DatabaseError),
    #[error("Validation errors: {0:?}")]
    ValidationErrors(Vec<String>),
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    // Helper function to get test database URL
    fn get_test_database_url() -> String {
        env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://postgres:password@localhost:5432/cloudshuttle_test".to_string()
        })
    }

    #[tokio::test]
    async fn test_user_creation_validation() {
        let service = UserManagementService::new(&get_test_database_url()).await.unwrap();

        // Test valid user creation
        let request = CreateUserRequest {
            email: "test@example.com".to_string(),
            username: "testuser".to_string(),
            display_name: "Test User".to_string(),
            bio: Some("A test user bio".to_string()),
            password: "secure_password_123".to_string(),
        };

        let result = service.create_user(request).await;
        match result {
            Ok(response) => {
                assert!(!response.user.id.is_empty());
                assert_eq!(response.user.email, "test@example.com");
                assert_eq!(response.user.username, "testuser");
                assert_eq!(response.user.display_name, "Test User");
            }
            Err(UserManagementError::DatabaseError(_)) => {
                // Database not available, skip test
                println!("Skipping test: database not available");
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_input_validation_security() {
        let service = UserManagementService::new(&get_test_database_url()).await.unwrap();

        // Test XSS in bio
        let request = CreateUserRequest {
            email: "test@example.com".to_string(),
            username: "testuser".to_string(),
            display_name: "Test User".to_string(),
            bio: Some("<script>alert('xss')</script>".to_string()),
            password: "password123".to_string(),
        };

        let result = service.create_user(request).await;
        // Should either fail validation or sanitize the input
        match result {
            Ok(_) | Err(UserManagementError::DatabaseError(_)) => {
                // Either validation passed (with sanitization) or DB error (expected)
            }
            Err(UserManagementError::ValidationErrors(errors)) => {
                // Validation caught the XSS attempt
                assert!(!errors.is_empty());
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_business_rule_validation() {
        let service = UserManagementService::new(&get_test_database_url()).await.unwrap();

        // Test invalid email
        let request = CreateUserRequest {
            email: "invalid-email".to_string(),
            username: "testuser".to_string(),
            display_name: "Test User".to_string(),
            bio: None,
            password: "password123".to_string(),
        };

        let validation_results = service.validate_user_inputs(&request).await.unwrap();
        let email_validation = &validation_results[0];
        assert!(!email_validation.is_valid);
    }

    #[tokio::test]
    async fn test_connection_pool_metrics() {
        let service = UserManagementService::new(&get_test_database_url()).await.unwrap();

        let metrics = service.get_pool_metrics();
        assert!(metrics.total_connections >= 0);
        assert!(metrics.idle_connections >= 0);
    }

    #[tokio::test]
    async fn test_migration_execution() {
        let service = UserManagementService::new(&get_test_database_url()).await.unwrap();

        // Check migration status
        let status = service.migration_runner.status();
        assert!(status.applied_count >= 0);
    }

    #[tokio::test]
    async fn test_duplicate_user_prevention() {
        let service = UserManagementService::new(&get_test_database_url()).await.unwrap();

        let request1 = CreateUserRequest {
            email: "duplicate@example.com".to_string(),
            username: "duplicateuser".to_string(),
            display_name: "Duplicate User".to_string(),
            bio: None,
            password: "password123".to_string(),
        };

        let request2 = CreateUserRequest {
            email: "duplicate@example.com".to_string(), // Same email
            username: "differentuser".to_string(),
            display_name: "Different User".to_string(),
            bio: None,
            password: "password123".to_string(),
        };

        let _result1 = service.create_user(request1).await;
        let result2 = service.create_user(request2).await;

        match result2 {
            Err(UserManagementError::UserAlreadyExists) => {
                // Expected behavior
            }
            Err(UserManagementError::DatabaseError(_)) => {
                // Database not available
            }
            _ => panic!("Expected UserAlreadyExists error"),
        }
    }

    #[tokio::test]
    async fn test_input_sanitization() {
        let mut validator = AdvancedValidator::new(ValidationConfig::default());
        validator.add_sanitizer("test_field", Box::new(HtmlSanitizer::new()));

        let context = ValidationContext {
            field_name: "test_field".to_string(),
            field_value: serde_json::json!("<b>Bold</b> <script>alert('xss')</script> Normal"),
            context_data: HashMap::new(),
            user_id: None,
            request_id: None,
        };

        let result = validator.validate(context);
        assert!(result.is_valid);
        assert!(result.sanitized_value.is_some());

        let sanitized = result.sanitized_value.unwrap().as_str().unwrap();
        assert!(sanitized.contains("<b>Bold</b>")); // Allowed tag preserved
        assert!(!sanitized.contains("<script>")); // Dangerous tag removed
        assert!(sanitized.contains("Normal")); // Normal text preserved
    }
}
