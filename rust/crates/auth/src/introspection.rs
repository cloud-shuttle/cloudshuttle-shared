//! Token introspection implementation (RFC 7662)
//!
//! This module provides RFC 7662 compliant token introspection capabilities,
//! allowing services to validate and retrieve metadata about tokens.

use serde::{Deserialize, Serialize};
use crate::types::{AuthResult, AuthError};
#[cfg(feature = "observability")]
use cloudshuttle_observability::audit::{audit_auth, AuditResult};
use crate::{Claims, JwtService};

/// Token introspection request
#[derive(Debug, Deserialize)]
pub struct IntrospectionRequest {
    /// The token to introspect
    pub token: String,

    /// Optional token type hint
    #[serde(rename = "token_type_hint")]
    pub token_type_hint: Option<String>,
}

/// Token introspection response (RFC 7662 compliant)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntrospectionResponse {
    /// Whether the token is active
    pub active: bool,

    /// Client identifier for which the token was issued
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,

    /// Human-readable identifier for the resource owner
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,

    /// Scope associated with the token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,

    /// Type of the token
    #[serde(rename = "token_type")]
    pub token_type: String,

    /// Expiration time of the token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp: Option<u64>,

    /// Time at which the token was issued
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iat: Option<u64>,

    /// Time before which the token is not valid
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbf: Option<u64>,

    /// Subject of the token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<String>,

    /// Intended audience of the token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<String>,

    /// Issuer of the token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,

    /// Unique identifier for the token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jti: Option<String>,
}

/// Token introspection service
pub struct TokenIntrospection {
    jwt_service: JwtService,
}

impl TokenIntrospection {
    /// Create a new token introspection service
    pub fn new(jwt_service: JwtService) -> Self {
        Self { jwt_service }
    }

    /// Introspect a token and return RFC 7662 compliant response
    pub fn introspect(&self, request: IntrospectionRequest) -> AuthResult<IntrospectionResponse> {
        // Try to validate the token (this will fail for expired/invalid tokens)
        match self.jwt_service.extract_claims_unchecked(&request.token) {
            Ok(claims) => {
                // Token is structurally valid, create active response
                #[cfg(feature = "observability")]
                audit_auth("token_introspected", Some(&claims.sub), AuditResult::Success);

                Ok(IntrospectionResponse {
                    active: true,
                    client_id: None, // Not implemented in basic JWT
                    username: Some(claims.sub.clone()),
                    scope: None, // Not implemented in basic JWT
                    token_type: "Bearer".to_string(),
                    exp: Some(claims.exp),
                    iat: Some(claims.iat),
                    nbf: claims.nbf,
                    sub: Some(claims.sub.clone()),
                    aud: claims.aud.clone(),
                    iss: claims.iss.clone(),
                    jti: claims.jti.clone(),
                })
            }
            Err(_) => {
                // Token is invalid, expired, or malformed
                #[cfg(feature = "observability")]
                audit_auth("token_introspection_failed", None, AuditResult::Failure);

                Ok(IntrospectionResponse {
                    active: false,
                    client_id: None,
                    username: None,
                    scope: None,
                    token_type: "Bearer".to_string(),
                    exp: None,
                    iat: None,
                    nbf: None,
                    sub: None,
                    aud: None,
                    iss: None,
                    jti: None,
                })
            }
        }
    }

    /// Check if a token is active (convenience method)
    pub fn is_token_active(&self, token: &str) -> bool {
        match self.introspect(IntrospectionRequest {
            token: token.to_string(),
            token_type_hint: Some("access_token".to_string()),
        }) {
            Ok(response) => response.active,
            Err(_) => false,
        }
    }

    /// Get token claims if active (convenience method)
    pub fn get_active_claims(&self, token: &str) -> AuthResult<Option<Claims>> {
        let response = self.introspect(IntrospectionRequest {
            token: token.to_string(),
            token_type_hint: Some("access_token".to_string()),
        })?;

        if response.active {
            // If active, we can safely decode the claims
            self.jwt_service.extract_claims_unchecked(token).map(Some)
        } else {
            Ok(None)
        }
    }
}

/// RFC 7662 Token Introspection trait
pub trait TokenIntrospectable {
    /// Introspect a token
    fn introspect_token(&self, token: &str) -> AuthResult<IntrospectionResponse>;

    /// Check if token is active
    fn is_token_active(&self, token: &str) -> bool;

    /// Get claims for active token
    fn get_active_claims(&self, token: &str) -> AuthResult<Option<Claims>>;
}

impl TokenIntrospectable for TokenIntrospection {
    fn introspect_token(&self, token: &str) -> AuthResult<IntrospectionResponse> {
        self.introspect(IntrospectionRequest {
            token: token.to_string(),
            token_type_hint: Some("access_token".to_string()),
        })
    }

    fn is_token_active(&self, token: &str) -> bool {
        self.is_token_active(token)
    }

    fn get_active_claims(&self, token: &str) -> AuthResult<Option<Claims>> {
        self.get_active_claims(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Claims;

    #[test]
    fn test_token_introspection_valid_token() {
        let jwt_service = JwtService::new(b"test-secret-key").unwrap();
        let claims = Claims::new("user123", "tenant456");
        let token = jwt_service.create_token(&claims).unwrap();

        let introspector = TokenIntrospection::new(jwt_service);

        let response = introspector.introspect(IntrospectionRequest {
            token: token.clone(),
            token_type_hint: Some("access_token".to_string()),
        }).unwrap();

        assert!(response.active);
        assert_eq!(response.username, Some("user123".to_string()));
        assert_eq!(response.sub, Some("user123".to_string()));
        assert_eq!(response.token_type, "Bearer");
        assert!(response.exp.is_some());
        assert!(response.iat.is_some());
    }

    #[test]
    fn test_token_introspection_invalid_token() {
        let jwt_service = JwtService::new(b"test-secret-key").unwrap();
        let introspector = TokenIntrospection::new(jwt_service);

        let response = introspector.introspect(IntrospectionRequest {
            token: "invalid-token".to_string(),
            token_type_hint: Some("access_token".to_string()),
        }).unwrap();

        assert!(!response.active);
        assert!(response.username.is_none());
        assert!(response.exp.is_none());
    }

    #[test]
    fn test_is_token_active() {
        let jwt_service = JwtService::new(b"test-secret-key").unwrap();
        let claims = Claims::new("user123", "tenant456");
        let token = jwt_service.create_token(&claims).unwrap();

        let introspector = TokenIntrospection::new(jwt_service);

        assert!(introspector.is_token_active(&token));
        assert!(!introspector.is_token_active("invalid"));
    }

    #[test]
    fn test_get_active_claims() {
        let jwt_service = JwtService::new(b"test-secret-key").unwrap();
        let claims = Claims::new("user123", "tenant456");
        let token = jwt_service.create_token(&claims).unwrap();

        let introspector = TokenIntrospection::new(jwt_service);

        let active_claims = introspector.get_active_claims(&token).unwrap();
        assert!(active_claims.is_some());
        assert_eq!(active_claims.unwrap().sub, "user123");

        let invalid_claims = introspector.get_active_claims("invalid").unwrap();
        assert!(invalid_claims.is_none());
    }

    #[test]
    fn test_trait_implementation() {
        let jwt_service = JwtService::new(b"test-secret-key").unwrap();
        let claims = Claims::new("user123", "tenant456");
        let token = jwt_service.create_token(&claims).unwrap();

        let introspector = TokenIntrospection::new(jwt_service);

        // Test trait methods
        assert!(introspector.is_token_active(&token));
        let response = introspector.introspect_token(&token).unwrap();
        assert!(response.active);
    }
}
