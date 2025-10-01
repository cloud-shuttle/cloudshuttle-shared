//! Authentication middleware for Axum

#[cfg(feature = "middleware")]
use axum::{
    extract::Request,
    http::{header, HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use std::pin::Pin;
use std::future::Future;
use crate::{JwtService, Claims, AuthResult, AuthError};

#[cfg(feature = "middleware")]
type MiddlewareFuture = Pin<Box<dyn Future<Output = Response> + Send>>;

#[cfg(feature = "middleware")]
type MiddlewareFn = Box<dyn Fn(Request, Next) -> MiddlewareFuture + Send + Sync>;

/// Authentication middleware layer
#[cfg(feature = "middleware")]
pub struct AuthMiddleware {
    jwt_service: Arc<JwtService>,
    required_roles: Vec<String>,
    optional_auth: bool,
}

#[cfg(feature = "middleware")]
impl AuthMiddleware {
    /// Create new middleware requiring authentication
    pub fn new(jwt_service: Arc<JwtService>) -> Self {
        Self {
            jwt_service,
            required_roles: Vec::new(),
            optional_auth: false,
        }
    }

    /// Create new middleware with optional authentication
    pub fn optional(jwt_service: Arc<JwtService>) -> Self {
        Self {
            jwt_service,
            required_roles: Vec::new(),
            optional_auth: true,
        }
    }

    /// Require specific roles for access
    pub fn require_roles(mut self, roles: Vec<String>) -> Self {
        self.required_roles = roles;
        self
    }

    /// Require admin role
    pub fn require_admin(self) -> Self {
        self.require_roles(vec!["admin".to_string()])
    }

    /// Require tenant admin role
    pub fn require_tenant_admin(self) -> Self {
        self.require_roles(vec!["tenant_admin".to_string(), "admin".to_string()])
    }

    /// Convert to Axum middleware function
    pub fn into_layer(self) -> MiddlewareFn {
        let jwt_service = self.jwt_service.clone();
        let required_roles = self.required_roles.clone();
        let optional_auth = self.optional_auth;

        Box::new(move |mut req: Request, next: Next| {
            let jwt_service = jwt_service.clone();
            let required_roles = required_roles.clone();

            Box::pin(async move {
                // Extract token from Authorization header
                let token = extract_token_from_header(req.headers());

                match token {
                    Some(token) => {
                        match jwt_service.validate_token(&token) {
                            Ok(claims) => {
                                // Check role requirements
                                if !required_roles.is_empty() {
                                    if !claims.has_any_role(&required_roles) {
                                        return AuthError::InsufficientPermissions {
                                            required: required_roles,
                                            actual: claims.roles,
                                        }.into_response();
                                    }
                                }

                                // Add claims to request extensions
                                req.extensions_mut().insert(claims);
                                next.run(req).await
                            }
                            Err(e) => {
                                if optional_auth {
                                    // For optional auth, continue without claims
                                    next.run(req).await
                                } else {
                                    e.into_response()
                                }
                            }
                        }
                    }
                    None => {
                        if optional_auth {
                            // For optional auth, continue without claims
                            next.run(req).await
                        } else {
                            AuthError::MissingToken.into_response()
                        }
                    }
                }
            })
        })
    }
}

/// Extract Bearer token from Authorization header
fn extract_token_from_header(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| {
            if value.starts_with("Bearer ") {
                Some(value.trim_start_matches("Bearer ").to_string())
            } else {
                None
            }
        })
}

/// Authentication extractor for handlers
#[cfg(feature = "middleware")]
pub struct AuthenticatedUser(pub Claims);

#[cfg(feature = "middleware")]
impl AuthenticatedUser {
    /// Get user ID
    pub fn user_id(&self) -> &str {
        &self.0.sub
    }

    /// Get tenant ID
    pub fn tenant_id(&self) -> &str {
        &self.0.tenant_id
    }

    /// Check if user has role
    pub fn has_role(&self, role: &str) -> bool {
        self.0.has_role(role)
    }

    /// Check if user has permission
    pub fn has_permission(&self, permission: &str) -> bool {
        self.0.has_permission(permission)
    }

    /// Get roles
    pub fn roles(&self) -> &[String] {
        &self.0.roles
    }

    /// Get permissions
    pub fn permissions(&self) -> &[String] {
        &self.0.permissions
    }
}

#[async_trait::async_trait]
#[cfg(feature = "middleware")]
impl<S> axum::extract::FromRequest<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: axum::extract::Request, _state: &S) -> Result<Self, Self::Rejection> {
        match req.extensions().get::<Claims>() {
            Some(claims) => Ok(AuthenticatedUser(claims.clone())),
            None => Err(AuthError::MissingToken.into_response()),
        }
    }
}

/// Optional authentication extractor
#[cfg(feature = "middleware")]
pub struct OptionalUser(pub Option<Claims>);

#[cfg(feature = "middleware")]
#[async_trait::async_trait]
impl<S> axum::extract::FromRequest<S> for OptionalUser
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: axum::extract::Request, _state: &S) -> Result<Self, Self::Rejection> {
        let claims = req.extensions().get::<Claims>().cloned();
        Ok(OptionalUser(claims))
    }
}

/// Role-based authorization middleware
pub struct RoleGuard {
    required_roles: Vec<String>,
}

impl RoleGuard {
    pub fn new(roles: Vec<String>) -> Self {
        Self { required_roles: roles }
    }

    pub fn require_admin() -> Self {
        Self::new(vec!["admin".to_string()])
    }

    pub fn require_tenant_admin() -> Self {
        Self::new(vec!["tenant_admin".to_string(), "admin".to_string()])
    }

    pub fn into_layer(self) -> MiddlewareFn {
        let required_roles = self.required_roles;

        Box::new(move |req: Request, next: Next| {
            let required_roles = required_roles.clone();

            Box::pin(async move {
                match req.extensions().get::<Claims>() {
                    Some(claims) => {
                        if claims.has_any_role(&required_roles) {
                            next.run(req).await
                        } else {
                            AuthError::InsufficientPermissions {
                                required: required_roles,
                                actual: claims.roles.clone(),
                            }.into_response()
                        }
                    }
                    None => AuthError::MissingToken.into_response(),
                }
            })
        })
    }
}

/// Permission-based authorization middleware
pub struct PermissionGuard {
    required_permissions: Vec<String>,
}

impl PermissionGuard {
    pub fn new(permissions: Vec<String>) -> Self {
        Self { required_permissions: permissions }
    }

    pub fn into_layer(self) -> MiddlewareFn {
        let required_permissions = self.required_permissions;

        Box::new(move |req: Request, next: Next| {
            let required_permissions = required_permissions.clone();

            Box::pin(async move {
                match req.extensions().get::<Claims>() {
                    Some(claims) => {
                        if claims.has_any_permission(&required_permissions) {
                            next.run(req).await
                        } else {
                            AuthError::InsufficientPermissions {
                                required: required_permissions,
                                actual: claims.permissions.clone(),
                            }.into_response()
                        }
                    }
                    None => AuthError::MissingToken.into_response(),
                }
            })
        })
    }
}

/// Tenant isolation middleware
pub struct TenantGuard;

impl TenantGuard {
    pub fn new() -> Self {
        Self
    }

    pub fn into_layer(self) -> MiddlewareFn {
        Box::new(move |req: Request, next: Next| {
            Box::pin(async move {
                // This would typically extract tenant ID from path or header
                // and verify it matches the user's tenant ID
                // For now, just pass through
                next.run(req).await
            })
        })
    }
}

impl Default for TenantGuard {
    fn default() -> Self {
        Self::new()
    }
}

/// CORS middleware for authenticated endpoints
pub struct CorsAuthLayer;

impl CorsAuthLayer {
    pub fn new() -> Self {
        Self
    }

    pub fn into_layer(self) -> MiddlewareFn {
        Box::new(move |req: Request, next: Next| {
            Box::pin(async move {
                let mut response = next.run(req).await;

                // Add CORS headers for authenticated endpoints
                let headers = response.headers_mut();
                headers.insert(
                    "Access-Control-Allow-Origin",
                    "http://localhost:3000".parse().unwrap(),
                );
                headers.insert(
                    "Access-Control-Allow-Methods",
                    "GET, POST, PUT, DELETE, OPTIONS".parse().unwrap(),
                );
                headers.insert(
                    "Access-Control-Allow-Headers",
                    "Authorization, Content-Type".parse().unwrap(),
                );
                headers.insert("Access-Control-Allow-Credentials", "true".parse().unwrap());

                response
            })
        })
    }
}

impl Default for CorsAuthLayer {
    fn default() -> Self {
        Self::new()
    }
}
