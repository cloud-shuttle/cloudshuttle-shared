//! JWT claims structure and validation

use crate::Claims;
use crate::types::{AuthResult, AuthError};

/// Claims validation and management utilities
pub struct ClaimsManager;

impl ClaimsManager {
    /// Validate claims structure and required fields
    pub fn validate_claims(claims: &Claims) -> AuthResult<()> {
        // Validate subject is not empty
        if claims.sub.is_empty() {
            return Err(AuthError::TokenValidation("Subject claim is required".to_string()));
        }

        // Validate tenant_id is not empty
        if claims.tenant_id.is_empty() {
            return Err(AuthError::TokenValidation("Tenant ID claim is required".to_string()));
        }

        // Validate timestamps are reasonable
        if claims.exp <= claims.iat {
            return Err(AuthError::TokenValidation("Expiration time must be after issued time".to_string()));
        }

        // Validate not before time if present
        if let Some(nbf) = claims.nbf {
            if nbf > claims.exp {
                return Err(AuthError::TokenValidation("Not before time cannot be after expiration".to_string()));
            }
        }

        Ok(())
    }

    /// Check if claims are expired
    pub fn is_expired(claims: &Claims) -> bool {
        let now = Self::current_timestamp();
        claims.exp < now
    }

    /// Get seconds until expiry
    pub fn seconds_until_expiry(claims: &Claims) -> i64 {
        let now = Self::current_timestamp() as i64;
        claims.exp as i64 - now
    }

    /// Check if claims are valid (not expired and properly formed)
    pub fn is_valid(claims: &Claims) -> AuthResult<bool> {
        Self::validate_claims(claims)?;
        Ok(!Self::is_expired(claims))
    }

    /// Validate role-based access
    pub fn has_role(claims: &Claims, required_role: &str) -> bool {
        claims.roles.contains(&required_role.to_string())
    }

    /// Validate permission-based access
    pub fn has_permission(claims: &Claims, required_permission: &str) -> bool {
        claims.permissions.contains(&required_permission.to_string())
    }

    /// Validate that claims have any of the required roles
    pub fn has_any_role(claims: &Claims, required_roles: &[&str]) -> bool {
        required_roles.iter().any(|role| Self::has_role(claims, role))
    }

    /// Validate that claims have all required permissions
    pub fn has_all_permissions(claims: &Claims, required_permissions: &[&str]) -> bool {
        required_permissions.iter().all(|perm| Self::has_permission(claims, perm))
    }

    /// Get custom claim value
    pub fn get_custom_claim<'a>(&self, claims: &'a Claims, key: &str) -> Option<&'a serde_json::Value> {
        claims.custom.get(key)
    }

    /// Set custom claim value
    pub fn set_custom_claim(&self, claims: &mut Claims, key: &str, value: serde_json::Value) {
        claims.custom.insert(key.to_string(), value);
    }

    /// Remove custom claim
    pub fn remove_custom_claim(&self, claims: &mut Claims, key: &str) {
        claims.custom.remove(key);
    }

    /// Check if claims represent an access token
    pub fn is_access_token(claims: &Claims) -> bool {
        matches!(claims.token_type.as_deref(), Some("access") | None)
    }

    /// Check if claims represent a refresh token
    pub fn is_refresh_token(claims: &Claims) -> bool {
        matches!(claims.token_type.as_deref(), Some("refresh"))
    }

    /// Validate issuer
    pub fn validate_issuer(claims: &Claims, expected_issuer: &str) -> AuthResult<()> {
        match &claims.iss {
            Some(issuer) if issuer == expected_issuer => Ok(()),
            Some(issuer) => Err(AuthError::TokenValidation(
                format!("Invalid issuer: expected {}, got {}", expected_issuer, issuer)
            )),
            None => Err(AuthError::TokenValidation("Issuer claim is missing".to_string())),
        }
    }

    /// Validate audience
    pub fn validate_audience(claims: &Claims, expected_audience: &str) -> AuthResult<()> {
        match &claims.aud {
            Some(audience) if audience == expected_audience => Ok(()),
            Some(audience) => Err(AuthError::TokenValidation(
                format!("Invalid audience: expected {}, got {}", expected_audience, audience)
            )),
            None => Err(AuthError::TokenValidation("Audience claim is missing".to_string())),
        }
    }

    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

/// Token validation options for different scenarios
#[derive(Debug, Clone)]
pub enum TokenValidation {
    Strict,
    RefreshToken,
    ApiKey,
}

impl TokenValidation {
    /// Convert to jsonwebtoken Validation settings
    pub fn to_validation(&self, base_validation: &jsonwebtoken::Validation) -> jsonwebtoken::Validation {
        let mut validation = base_validation.clone();

        match self {
            Self::Strict => {
                // All validations enabled
            }
            Self::RefreshToken => {
                validation.validate_exp = false; // Allow expired tokens for refresh
            }
            Self::ApiKey => {
                validation.validate_exp = true;
                // Note: validate_iat not available in current jsonwebtoken version
                validation.validate_nbf = false;
            }
        }

        validation
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_claims_validation() {
        // Valid claims
        let claims = Claims::new("user-123", "tenant-456");
        assert!(ClaimsManager::validate_claims(&claims).is_ok());

        // Invalid: empty subject
        let invalid_claims = Claims {
            sub: "".to_string(),
            ..claims.clone()
        };
        assert!(ClaimsManager::validate_claims(&invalid_claims).is_err());

        // Invalid: empty tenant_id
        let invalid_claims = Claims {
            tenant_id: "".to_string(),
            ..claims.clone()
        };
        assert!(ClaimsManager::validate_claims(&invalid_claims).is_err());
    }

    #[test]
    fn test_expiry_check() {
        let mut claims = Claims::new("user-123", "tenant-456");

        // Not expired
        claims.exp = (chrono::Utc::now() + Duration::hours(1)).timestamp() as u64;
        assert!(!ClaimsManager::is_expired(&claims));

        // Expired
        claims.exp = (chrono::Utc::now() - Duration::hours(1)).timestamp() as u64;
        assert!(ClaimsManager::is_expired(&claims));
    }

    #[test]
    fn test_role_validation() {
        let mut claims = Claims::new("user-123", "tenant-456");
        claims.roles = vec!["admin".to_string(), "user".to_string()];

        assert!(ClaimsManager::has_role(&claims, "admin"));
        assert!(ClaimsManager::has_role(&claims, "user"));
        assert!(!ClaimsManager::has_role(&claims, "moderator"));
    }

    #[test]
    fn test_permission_validation() {
        let mut claims = Claims::new("user-123", "tenant-456");
        claims.permissions = vec!["read".to_string(), "write".to_string()];

        assert!(ClaimsManager::has_permission(&claims, "read"));
        assert!(ClaimsManager::has_permission(&claims, "write"));
        assert!(!ClaimsManager::has_permission(&claims, "delete"));
    }

    #[test]
    fn test_token_type_validation() {
        let mut claims = Claims::new("user-123", "tenant-456");

        // Default is access token
        assert!(ClaimsManager::is_access_token(&claims));
        assert!(!ClaimsManager::is_refresh_token(&claims));

        // Set as refresh token
        claims.token_type = Some("refresh".to_string());
        assert!(!ClaimsManager::is_access_token(&claims));
        assert!(ClaimsManager::is_refresh_token(&claims));
    }

    #[test]
    fn test_issuer_audience_validation() {
        let mut claims = Claims::new("user-123", "tenant-456");
        claims.iss = Some("test-issuer".to_string());
        claims.aud = Some("test-audience".to_string());

        assert!(ClaimsManager::validate_issuer(&claims, "test-issuer").is_ok());
        assert!(ClaimsManager::validate_issuer(&claims, "wrong-issuer").is_err());

        assert!(ClaimsManager::validate_audience(&claims, "test-audience").is_ok());
        assert!(ClaimsManager::validate_audience(&claims, "wrong-audience").is_err());
    }

    #[test]
    fn test_custom_claims() {
        let mut claims = Claims::new("user-123", "tenant-456");
        let manager = ClaimsManager;

        // Set custom claim
        manager.set_custom_claim(&mut claims, "department", serde_json::json!("engineering"));
        assert_eq!(manager.get_custom_claim(&claims, "department"), Some(&serde_json::json!("engineering")));

        // Remove custom claim
        manager.remove_custom_claim(&mut claims, "department");
        assert_eq!(manager.get_custom_claim(&claims, "department"), None);
    }
}
