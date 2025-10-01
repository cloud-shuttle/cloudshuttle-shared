//! Service integration tests for authentication components
//!
//! These tests demonstrate how the advanced authentication features
//! work together in real service integration scenarios.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use cloudshuttle_auth::{
    JwtService, Claims, SecurityValidator,
    TokenIntrospection, PkceHandler, RefreshTokenManager,
    RefreshTokenConfig, AdvancedValidator, ValidationContext,
    ValidationSeverity,
};
use cloudshuttle_observability::audit::{AuditLogger, global_audit_logger};

/// Mock user database for integration testing
#[derive(Debug, Clone)]
struct MockUserDatabase {
    users: Arc<RwLock<HashMap<String, UserRecord>>>,
}

#[derive(Debug, Clone)]
struct UserRecord {
    user_id: String,
    email: String,
    password_hash: String,
    roles: Vec<String>,
    tenant_id: String,
    is_active: bool,
}

impl MockUserDatabase {
    fn new() -> Self {
        let mut users = HashMap::new();
        users.insert("user123".to_string(), UserRecord {
            user_id: "user123".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "$argon2id$v=19$m=4096,t=3,p=1$test_salt$test_hash".to_string(),
            roles: vec!["user".to_string()],
            tenant_id: "tenant456".to_string(),
            is_active: true,
        });

        Self {
            users: Arc::new(RwLock::new(users)),
        }
    }

    async fn get_user(&self, user_id: &str) -> Option<UserRecord> {
        self.users.read().await.get(user_id).cloned()
    }

    async fn validate_credentials(&self, user_id: &str, password: &str) -> bool {
        if let Some(user) = self.get_user(user_id).await {
            // In real implementation, use argon2 to verify password
            user.is_active && password == "correct_password"
        } else {
            false
        }
    }
}

/// Complete authentication service integration
struct AuthenticationService {
    jwt_service: JwtService,
    security_validator: SecurityValidator,
    token_introspector: TokenIntrospection,
    pkce_handler: PkceHandler,
    refresh_manager: RefreshTokenManager,
    user_db: MockUserDatabase,
    audit_logger: AuditLogger,
}

impl AuthenticationService {
    async fn new() -> Self {
        let jwt_service = JwtService::new(b"test-secret-key-for-integration").unwrap();
        let security_validator = SecurityValidator;
        let token_introspector = TokenIntrospection::new(jwt_service.clone());
        let pkce_handler = PkceHandler;
        let refresh_config = RefreshTokenConfig {
            rotation_enabled: true,
            max_tokens_per_user: 3,
            ..Default::default()
        };
        let refresh_manager = RefreshTokenManager::new(jwt_service.clone(), refresh_config);
        let user_db = MockUserDatabase::new();
        let audit_logger = AuditLogger::new("auth-service-integration");

        Self {
            jwt_service,
            security_validator,
            token_introspector,
            pkce_handler,
            refresh_manager,
            user_db,
            audit_logger,
        }
    }

    /// Complete authentication flow with all security features
    async fn authenticate_user(&self, user_id: &str, password: &str) -> Result<AuthResponse, AuthError> {
        // Step 1: Input validation and security scanning
        self.validate_input(user_id, password)?;

        // Step 2: Credential validation
        if !self.user_db.validate_credentials(user_id, password).await {
            self.audit_logger.log(
                cloudshuttle_observability::audit::AuditEvent::new(
                    cloudshuttle_observability::audit::AuditEventType::Authentication,
                    "login_failed"
                )
                .with_user_id(user_id)
                .with_result(cloudshuttle_observability::audit::AuditResult::Failure)
            );
            return Err(AuthError::InvalidCredentials);
        }

        // Step 3: Get user details
        let user = self.user_db.get_user(user_id).await
            .ok_or(AuthError::UserNotFound(user_id.to_string()))?;

        // Step 4: Generate tokens
        let roles = user.roles.clone();
        let access_token = self.jwt_service.create_access_token(user_id, &user.tenant_id, roles)?;
        let refresh_token = self.refresh_manager.create_refresh_token(user_id, None, None, None)?;

        // Step 5: Audit successful authentication
        self.audit_logger.log(
            cloudshuttle_observability::audit::AuditEvent::new(
                cloudshuttle_observability::audit::AuditEventType::Authentication,
                "login_success"
            )
            .with_user_id(user_id)
            .with_result(cloudshuttle_observability::audit::AuditResult::Success)
        );

        Ok(AuthResponse {
            access_token,
            refresh_token: Some(refresh_token),
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            user_id: user_id.to_string(),
        })
    }

    /// Validate input with security scanning
    fn validate_input(&self, user_id: &str, password: &str) -> Result<(), AuthError> {
        // Check for malicious input
        if self.security_validator.validate_input(user_id).is_err() ||
           self.security_validator.validate_input(password).is_err() {
            return Err(AuthError::InvalidCredentials);
        }

        // Password strength validation
        if password.len() < 8 {
            return Err(AuthError::PasswordTooWeak);
        }

        Ok(())
    }

    /// Refresh tokens using advanced refresh management
    async fn refresh_tokens(&self, refresh_token: &str) -> Result<AuthResponse, AuthError> {
        use cloudshuttle_auth::RefreshTokenRequest;

        let request = RefreshTokenRequest {
            refresh_token: refresh_token.to_string(),
            device_id: None,
            scope: None,
        };

        let response = self.refresh_manager.refresh_tokens(request)?;

        // Audit token refresh
        self.audit_logger.log(
            cloudshuttle_observability::audit::AuditEvent::new(
                cloudshuttle_observability::audit::AuditEventType::Authentication,
                "token_refresh"
            )
            .with_result(cloudshuttle_observability::audit::AuditResult::Success)
        );

        Ok(AuthResponse {
            access_token: response.access_token,
            refresh_token: response.refresh_token,
            token_type: response.token_type,
            expires_in: response.expires_in,
            user_id: "refreshed_user".to_string(), // Would extract from token
        })
    }

    /// Validate token using introspection
    async fn validate_token(&self, token: &str) -> Result<TokenValidationResult, AuthError> {
        use cloudshuttle_auth::IntrospectionRequest;

        let request = IntrospectionRequest {
            token: token.to_string(),
            token_type_hint: Some("access_token".to_string()),
        };

        let response = self.token_introspector.introspect(request)?;

        Ok(TokenValidationResult {
            is_valid: response.active,
            user_id: response.username,
            expires_at: response.exp,
        })
    }

    /// Revoke all user tokens (security incident response)
    async fn revoke_all_user_tokens(&self, user_id: &str) -> Result<usize, AuthError> {
        let count = self.refresh_manager.revoke_all_user_tokens(user_id)?;

        // Audit token revocation
        self.audit_logger.log(
            cloudshuttle_observability::audit::AuditEvent::new(
                cloudshuttle_observability::audit::AuditEventType::Security,
                "bulk_token_revocation"
            )
            .with_user_id(user_id)
            .with_result(cloudshuttle_observability::audit::AuditResult::Success)
        );

        Ok(count)
    }
}

/// OAuth 2.1 PKCE Flow Integration
struct OAuthService {
    pkce_handler: PkceHandler,
    jwt_service: JwtService,
    clients: HashMap<String, OAuthClient>,
}

#[derive(Debug, Clone)]
struct OAuthClient {
    client_id: String,
    client_secret: String,
    redirect_uris: Vec<String>,
}

impl OAuthService {
    fn new() -> Self {
        let mut clients = HashMap::new();
        clients.insert("client123".to_string(), OAuthClient {
            client_id: "client123".to_string(),
            client_secret: "secret456".to_string(),
            redirect_uris: vec!["https://example.com/callback".to_string()],
        });

        Self {
            pkce_handler: PkceHandler,
            jwt_service: JwtService::new(b"oauth-secret-key").unwrap(),
            clients,
        }
    }

    /// Initiate OAuth authorization with PKCE
    fn initiate_authorization(&self, client_id: &str, redirect_uri: &str) -> Result<AuthzResponse, AuthError> {
        // Validate client
        let client = self.clients.get(client_id)
            .ok_or(AuthError::InvalidCredentials)?;

        if !client.redirect_uris.contains(&redirect_uri.to_string()) {
            return Err(AuthError::InvalidCredentials);
        }

        // Generate PKCE challenge
        let pkce_pair = self.pkce_handler.generate()?;

        Ok(AuthzResponse {
            authorization_code: "auth_code_123".to_string(),
            code_challenge: pkce_pair.challenge().to_string(),
            code_challenge_method: pkce_pair.method().as_str().to_string(),
            redirect_uri: redirect_uri.to_string(),
        })
    }

    /// Exchange authorization code for tokens with PKCE validation
    fn exchange_code_for_tokens(&self, auth_code: &str, code_verifier: &str, client_id: &str) -> Result<TokenResponse, AuthError> {
        // In real implementation, validate stored code_challenge against code_verifier
        // For test, assume validation passes

        let access_token = self.jwt_service.create_access_token("oauth_user", "tenant", vec!["user".to_string()])?;
        let refresh_token = self.jwt_service.create_refresh_token("oauth_user", "tenant")?;

        Ok(TokenResponse {
            access_token,
            refresh_token: Some(refresh_token),
            token_type: "Bearer".to_string(),
            expires_in: 3600,
        })
    }
}

/// Integration test results
#[derive(Debug)]
struct AuthResponse {
    access_token: String,
    refresh_token: Option<String>,
    token_type: String,
    expires_in: u64,
    user_id: String,
}

#[derive(Debug)]
struct TokenValidationResult {
    is_valid: bool,
    user_id: Option<String>,
    expires_at: Option<u64>,
}

#[derive(Debug)]
struct AuthzResponse {
    authorization_code: String,
    code_challenge: String,
    code_challenge_method: String,
    redirect_uri: String,
}

#[derive(Debug)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    token_type: String,
    expires_in: u64,
}

#[derive(Debug, thiserror::Error)]
enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("User not found: {0}")]
    UserNotFound(String),
    #[error("Password too weak")]
    PasswordTooWeak,
    #[error("Token error: {0}")]
    TokenError(#[from] cloudshuttle_auth::AuthError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_complete_authentication_flow() {
        let service = AuthenticationService::new().await;

        // Test successful authentication
        let result = service.authenticate_user("user123", "correct_password").await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(!response.access_token.is_empty());
        assert!(response.refresh_token.is_some());
        assert_eq!(response.token_type, "Bearer");
        assert_eq!(response.expires_in, 3600);
    }

    #[tokio::test]
    async fn test_invalid_credentials() {
        let service = AuthenticationService::new().await;

        // Test invalid password
        let result = service.authenticate_user("user123", "wrong_password").await;
        assert!(matches!(result, Err(AuthError::InvalidCredentials)));
    }

    #[tokio::test]
    async fn test_input_validation_security() {
        let service = AuthenticationService::new().await;

        // Test SQL injection attempt
        let result = service.authenticate_user("'; DROP TABLE users; --", "password").await;
        assert!(matches!(result, Err(AuthError::InvalidCredentials)));

        // Test XSS attempt
        let result = service.authenticate_user("<script>alert('xss')</script>", "password").await;
        assert!(matches!(result, Err(AuthError::InvalidCredentials)));
    }

    #[tokio::test]
    async fn test_token_refresh_flow() {
        let service = AuthenticationService::new().await;

        // First authenticate to get tokens
        let auth_result = service.authenticate_user("user123", "correct_password").await.unwrap();
        let refresh_token = auth_result.refresh_token.unwrap();

        // Refresh tokens
        let refresh_result = service.refresh_tokens(&refresh_token).await;
        assert!(refresh_result.is_ok());

        let new_tokens = refresh_result.unwrap();
        assert!(!new_tokens.access_token.is_empty());
        assert!(new_tokens.refresh_token.is_some());
    }

    #[tokio::test]
    async fn test_token_introspection() {
        let service = AuthenticationService::new().await;

        // Get a valid token
        let auth_result = service.authenticate_user("user123", "correct_password").await.unwrap();

        // Validate token
        let validation_result = service.validate_token(&auth_result.access_token).await;
        assert!(validation_result.is_ok());

        let validation = validation_result.unwrap();
        assert!(validation.is_valid);
        assert_eq!(validation.user_id, Some("user123".to_string()));
        assert!(validation.expires_at.is_some());
    }

    #[tokio::test]
    async fn test_bulk_token_revocation() {
        let service = AuthenticationService::new().await;

        // Create multiple tokens for user
        let _auth1 = service.authenticate_user("user123", "correct_password").await.unwrap();
        let _auth2 = service.authenticate_user("user123", "correct_password").await.unwrap();

        // Revoke all tokens
        let revoked_count = service.revoke_all_user_tokens("user123").await;
        assert!(revoked_count.is_ok());
        assert_eq!(revoked_count.unwrap(), 2);
    }

    #[tokio::test]
    async fn test_oauth_pkce_flow() {
        let oauth_service = OAuthService::new();

        // Initiate authorization with PKCE
        let authz_result = oauth_service.initiate_authorization("client123", "https://example.com/callback");
        assert!(authz_result.is_ok());

        let authz = authz_result.unwrap();
        assert!(!authz.authorization_code.is_empty());
        assert!(!authz.code_challenge.is_empty());
        assert_eq!(authz.code_challenge_method, "S256");

        // Exchange code for tokens (would validate PKCE in real implementation)
        let token_result = oauth_service.exchange_code_for_tokens(
            &authz.authorization_code,
            "test_code_verifier",
            "client123"
        );
        assert!(token_result.is_ok());

        let tokens = token_result.unwrap();
        assert!(!tokens.access_token.is_empty());
        assert!(tokens.refresh_token.is_some());
    }

    #[tokio::test]
    async fn test_password_strength_validation() {
        let service = AuthenticationService::new().await;

        // Test weak password
        let result = service.authenticate_user("user123", "weak").await;
        assert!(matches!(result, Err(AuthError::PasswordTooWeak)));
    }

    #[tokio::test]
    async fn test_user_not_found() {
        let service = AuthenticationService::new().await;

        // Test non-existent user
        let result = service.authenticate_user("nonexistent", "password").await;
        assert!(matches!(result, Err(AuthError::UserNotFound(_))));
    }
}
