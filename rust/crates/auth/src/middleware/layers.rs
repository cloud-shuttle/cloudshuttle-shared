//! Authentication middleware layers for Axum.
//!
//! This module contains the core authentication middleware layers
//! that handle JWT validation and request authorization.

use std::sync::Arc;
use axum::{extract::Request, middleware::Next, response::{Response, IntoResponse}};
use crate::{JwtService, Claims, AuthError, MiddlewareFn};
use super::extractors::extract_token_from_header;

/// Main authentication middleware
pub struct AuthMiddleware {
    jwt_service: Arc<JwtService>,
    required_roles: Vec<String>,
    optional_auth: bool,
}

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
                    "https://app.cloudshuttle.com".parse().unwrap(),
                );
                headers.insert(
                    "Access-Control-Allow-Methods",
                    "GET, POST, PUT, DELETE, OPTIONS".parse().unwrap(),
                );
                headers.insert(
                    "Access-Control-Allow-Headers",
                    "Authorization, Content-Type".parse().unwrap(),
                );
                headers.insert(
                    "Access-Control-Allow-Credentials",
                    "true".parse().unwrap(),
                );

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
