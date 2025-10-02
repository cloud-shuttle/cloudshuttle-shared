//! Authentication extractors for Axum handlers.
//!
//! This module contains request extractors for authenticated users
//! and optional authentication scenarios.

use axum::{
    extract::Request,
    http::{header, HeaderMap},
    response::{IntoResponse, Response},
};
use crate::{Claims, AuthError};

/// Extract Bearer token from Authorization header
pub fn extract_token_from_header(headers: &HeaderMap) -> Option<String> {
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

#[cfg(feature = "middleware")]
#[async_trait::async_trait]
impl<S> axum::extract::FromRequest<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
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

    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        let claims = req.extensions().get::<Claims>().cloned();
        Ok(OptionalUser(claims))
    }
}
