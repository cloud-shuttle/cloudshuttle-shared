//! Contract tests for CloudShuttle Authentication API
//!
//! These tests use Pact to ensure consumer-driven contract testing
//! between the authentication service and its consumers.

use pact_consumer::prelude::*;
use serde_json::json;
use cloudshuttle_auth::*;

#[tokio::test]
async fn authentication_service_health_contract() {
    let pact_builder = PactBuilder::new("CloudShuttle API Gateway", "CloudShuttle Authentication Service");

    pact_builder
        .interaction("Authentication service health check", |mut i| {
            i.given("authentication service is running");
            i.upon_receiving("GET request to /health");
            i.with_request("GET", "/health");
            i.will_respond_with(200)
                .with_header("Content-Type", "application/json")
                .with_body(json!({
                    "status": "healthy",
                    "message": "Authentication service is operational",
                    "timestamp": "2025-09-20T10:00:00Z",
                    "version": "0.2.0"
                }));
            i
        })
        .await
        .start_mock_server()
        .await;
}

#[tokio::test]
async fn user_login_contract() {
    let pact_builder = PactBuilder::new("CloudShuttle API Gateway", "CloudShuttle Authentication Service");

    pact_builder
        .interaction("User login with valid credentials", |mut i| {
            i.given("user exists with valid credentials");
            i.upon_receiving("POST request to /login with valid credentials");
            i.with_request("POST", "/login")
                .with_header("Content-Type", "application/json")
                .with_body(json!({
                    "username": "testuser",
                    "password": "validpassword123",
                    "remember_me": false
                }));
            i.will_respond_with(200)
                .with_header("Content-Type", "application/json")
                .with_body(json!({
                    "access_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
                    "refresh_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
                    "token_type": "Bearer",
                    "expires_in": 3600,
                    "refresh_expires_in": 604800
                }));
            i
        })
        .await
        .start_mock_server()
        .await;
}

#[tokio::test]
async fn user_login_invalid_credentials_contract() {
    let pact_builder = PactBuilder::new("CloudShuttle API Gateway", "CloudShuttle Authentication Service");

    pact_builder
        .interaction("User login with invalid credentials", |mut i| {
            i.given("user exists but provides invalid credentials");
            i.upon_receiving("POST request to /login with invalid credentials");
            i.with_request("POST", "/login")
                .with_header("Content-Type", "application/json")
                .with_body(json!({
                    "username": "testuser",
                    "password": "wrongpassword",
                    "remember_me": false
                }));
            i.will_respond_with(401)
                .with_header("Content-Type", "application/json")
                .with_body(json!({
                    "error": {
                        "code": "INVALID_CREDENTIALS",
                        "message": "Invalid username or password",
                        "retryable": false
                    }
                }));
            i
        })
        .await
        .start_mock_server()
        .await;
}

#[tokio::test]
async fn token_refresh_contract() {
    let pact_builder = PactBuilder::new("CloudShuttle API Gateway", "CloudShuttle Authentication Service");

    pact_builder
        .interaction("Refresh access token", |mut i| {
            i.given("user has valid refresh token");
            i.upon_receiving("POST request to /refresh with valid refresh token");
            i.with_request("POST", "/refresh")
                .with_header("Content-Type", "application/json")
                .with_body(json!({
                    "refresh_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."
                }));
            i.will_respond_with(200)
                .with_header("Content-Type", "application/json")
                .with_body(json!({
                    "access_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
                    "refresh_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
                    "token_type": "Bearer",
                    "expires_in": 3600,
                    "refresh_expires_in": 604800
                }));
            i
        })
        .await
        .start_mock_server()
        .await;
}

#[tokio::test]
async fn get_user_profile_contract() {
    let pact_builder = PactBuilder::new("CloudShuttle API Gateway", "CloudShuttle Authentication Service");

    pact_builder
        .interaction("Get authenticated user profile", |mut i| {
            i.given("user is authenticated with valid JWT");
            i.upon_receiving("GET request to /me with valid JWT");
            i.with_request("GET", "/me")
                .with_header("Authorization", "Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...");
            i.will_respond_with(200)
                .with_header("Content-Type", "application/json")
                .with_body(json!({
                    "user_id": "user-123",
                    "username": "testuser",
                    "email": "test@example.com",
                    "tenant_id": "tenant-456",
                    "roles": ["user"],
                    "permissions": ["read", "write"],
                    "mfa_enabled": false,
                    "last_login": "2025-09-20T09:00:00Z",
                    "created_at": "2025-01-01T00:00:00Z",
                    "updated_at": "2025-09-20T09:00:00Z"
                }));
            i
        })
        .await
        .start_mock_server()
        .await;
}

#[tokio::test]
async fn user_logout_contract() {
    let pact_builder = PactBuilder::new("CloudShuttle API Gateway", "CloudShuttle Authentication Service");

    pact_builder
        .interaction("User logout", |mut i| {
            i.given("user is authenticated");
            i.upon_receiving("POST request to /logout");
            i.with_request("POST", "/logout")
                .with_header("Authorization", "Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...");
            i.will_respond_with(200)
                .with_header("Content-Type", "application/json")
                .with_body(json!({
                    "success": true,
                    "message": "Logged out successfully",
                    "timestamp": "2025-09-20T10:00:00Z"
                }));
            i
        })
        .await
        .start_mock_server()
        .await;
}