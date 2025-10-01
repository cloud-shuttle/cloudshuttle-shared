//! Example: Authentication Service using CloudShuttle shared libraries
//!
//! This example demonstrates how to build a complete authentication service
//! using the CloudShuttle shared libraries.

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

// Import CloudShuttle shared libraries
use cloudshuttle_auth::{JwtService, Claims, SecurityValidator};
use cloudshuttle_error_handling::{CloudShuttleError, ServiceError};
use cloudshuttle_validation::ValidationService;
use cloudshuttle_database::DatabaseConnection;

#[derive(Clone)]
struct AppState {
    jwt_service: Arc<JwtService>,
    db: Arc<DatabaseConnection>,
    validator: Arc<ValidationService>,
}

#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct LoginResponse {
    access_token: String,
    refresh_token: String,
    token_type: String,
    expires_in: u64,
}

#[derive(Deserialize)]
struct RegisterRequest {
    email: String,
    password: String,
    confirm_password: String,
}

#[derive(Serialize)]
struct UserProfile {
    id: String,
    email: String,
    created_at: String,
    roles: Vec<String>,
}

async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, CloudShuttleError> {
    // Validate input
    state.validator.validate_email(&req.email)?;
    state.validator.validate_password_strength(&req.password)?;

    // Check if user exists and password is correct
    let user = state.db.get_user_by_email(&req.email).await?
        .ok_or_else(|| CloudShuttleError::Authentication("Invalid credentials".to_string()))?;

    // Verify password using CloudShuttle security
    let is_valid = SecurityValidator::verify_password(&req.password, &user.password_hash).await?;
    if !is_valid {
        return Err(CloudShuttleError::Authentication("Invalid credentials".to_string()));
    }

    // Create JWT token
    let claims = Claims::new(&user.id, &user.tenant_id);
    let access_token = state.jwt_service.create_token(&claims)?;

    // Generate refresh token
    let refresh_token = SecurityValidator::generate_secure_token(32);

    // Store refresh token in database
    state.db.store_refresh_token(&user.id, &refresh_token).await?;

    // Log successful authentication
    SecurityAuditor::log_auth_attempt(&req.email, true, None);

    Ok(Json(LoginResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: 3600, // 1 hour
    }))
}

async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<StatusCode, CloudShuttleError> {
    // Validate input
    state.validator.validate_email(&req.email)?;

    if req.password != req.confirm_password {
        return Err(CloudShuttleError::Validation {
            field: "confirm_password".to_string(),
            message: "Passwords do not match".to_string(),
        });
    }

    // Validate password strength
    let strength = SecurityValidator::validate_password_strength(&req.password)?;
    if !strength.is_strong {
        return Err(CloudShuttleError::Validation {
            field: "password".to_string(),
            message: format!("Password is too weak. Score: {}", strength.score),
        });
    }

    // Hash password
    let password_hash = SecurityValidator::hash_password(&req.password).await?;

    // Create user in database
    let user_id = state.db.create_user(&req.email, &password_hash).await?;

    // Log successful registration
    tracing::info!("User registered: {} (ID: {})", req.email, user_id);

    Ok(StatusCode::CREATED)
}

async fn get_profile(
    State(state): State<AppState>,
    claims: Claims, // Extracted from JWT middleware
) -> Result<Json<UserProfile>, CloudShuttleError> {
    // Get user profile from database
    let user = state.db.get_user_by_id(&claims.sub).await?
        .ok_or_else(|| CloudShuttleError::NotFound("User not found".to_string()))?;

    Ok(Json(UserProfile {
        id: user.id,
        email: user.email,
        created_at: user.created_at.to_rfc3339(),
        roles: user.roles,
    }))
}

async fn health_check() -> &'static str {
    "Authentication service is healthy"
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Initialize CloudShuttle services
    let jwt_service = Arc::new(JwtService::new(b"your-secret-key-here")?);
    let db = Arc::new(DatabaseConnection::new("postgresql://localhost/auth_db").await?);
    let validator = Arc::new(ValidationService::new());

    let app_state = AppState {
        jwt_service,
        db,
        validator,
    };

    // Build the application with routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/login", post(login))
        .route("/register", post(register))
        .route("/profile", get(get_profile))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    // Start the server
    let addr = "0.0.0.0:3000";
    tracing::info!("Authentication service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloudshuttle_auth::test_utils::*;

    #[tokio::test]
    async fn test_login_flow() {
        // Test the complete login flow using shared libraries
        let jwt_service = JwtService::new(b"test-key").unwrap();

        // Validate password strength
        let strength = SecurityValidator::validate_password_strength("TestPass123!").unwrap();
        assert!(strength.is_strong);

        // Create and validate JWT
        let claims = Claims::new("user-123", "tenant-456");
        let token = jwt_service.create_token(&claims).unwrap();

        let validated = jwt_service.validate_token(&token).unwrap();
        assert_eq!(validated.sub, "user-123");
        assert_eq!(validated.tenant_id, "tenant-456");
    }
}
