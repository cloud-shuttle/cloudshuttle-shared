//! Authorization guards for role and permission-based access control.
//!
//! This module contains middleware guards for enforcing authorization
//! policies based on user roles, permissions, and tenant isolation.

use axum::{extract::Request, middleware::Next, response::{Response, IntoResponse}};
use crate::{Claims, AuthError, MiddlewareFn};

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
