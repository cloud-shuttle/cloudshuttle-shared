//! Authentication credentials and token structures

use serde::{Deserialize, Serialize};

/// User authentication data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCredentials {
    pub username: String,
    pub password: String,
}

impl UserCredentials {
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
        }
    }

    /// Validate that credentials are not empty
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.username.trim().is_empty() {
            return Err("Username cannot be empty");
        }
        if self.password.is_empty() {
            return Err("Password cannot be empty");
        }
        Ok(())
    }

    /// Check if username appears to be an email
    pub fn is_email(&self) -> bool {
        self.username.contains('@')
    }
}

/// Authentication tokens response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_expires_in: u64,
}

impl AuthTokens {
    pub fn new(access_token: String, refresh_token: String, expires_in: u64, refresh_expires_in: u64) -> Self {
        Self {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in,
            refresh_expires_in,
        }
    }

    pub fn bearer(access_token: String, refresh_token: String) -> Self {
        Self::new(access_token, refresh_token, 3600, 604800) // 1 hour, 7 days
    }

    /// Check if access token is expired
    pub fn is_access_expired(&self) -> bool {
        // This would need timestamp tracking in a real implementation
        false
    }

    /// Check if refresh token is expired
    pub fn is_refresh_expired(&self) -> bool {
        // This would need timestamp tracking in a real implementation
        false
    }

    /// Get authorization header value
    pub fn authorization_header(&self) -> String {
        format!("{} {}", self.token_type, self.access_token)
    }
}

/// Refresh token request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

impl RefreshTokenRequest {
    pub fn new(refresh_token: impl Into<String>) -> Self {
        Self {
            refresh_token: refresh_token.into(),
        }
    }

    /// Validate refresh token format
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.refresh_token.trim().is_empty() {
            return Err("Refresh token cannot be empty");
        }
        if self.refresh_token.len() < 10 {
            return Err("Refresh token is too short");
        }
        Ok(())
    }
}

/// Login request payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub remember_me: Option<bool>,
}

impl LoginRequest {
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
            remember_me: None,
        }
    }

    pub fn with_remember_me(mut self, remember: bool) -> Self {
        self.remember_me = Some(remember);
        self
    }

    /// Convert to UserCredentials
    pub fn to_credentials(&self) -> UserCredentials {
        UserCredentials::new(&self.username, &self.password)
    }
}

/// Password change request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordChangeRequest {
    pub current_password: String,
    pub new_password: String,
    pub confirm_password: String,
}

impl PasswordChangeRequest {
    pub fn new(current: impl Into<String>, new: impl Into<String>, confirm: impl Into<String>) -> Self {
        Self {
            current_password: current.into(),
            new_password: new.into(),
            confirm_password: confirm.into(),
        }
    }

    /// Validate password change request
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.current_password.is_empty() {
            return Err("Current password is required");
        }
        if self.new_password.is_empty() {
            return Err("New password is required");
        }
        if self.new_password != self.confirm_password {
            return Err("Passwords do not match");
        }
        if self.new_password == self.current_password {
            return Err("New password must be different from current password");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_credentials_validation() {
        let valid = UserCredentials::new("user", "pass");
        assert!(valid.validate().is_ok());

        let empty_username = UserCredentials::new("", "pass");
        assert!(empty_username.validate().is_err());

        let empty_password = UserCredentials::new("user", "");
        assert!(empty_password.validate().is_err());
    }

    #[test]
    fn test_auth_tokens() {
        let tokens = AuthTokens::bearer("access".to_string(), "refresh".to_string());
        assert_eq!(tokens.token_type, "Bearer");
        assert_eq!(tokens.expires_in, 3600);
        assert_eq!(tokens.refresh_expires_in, 604800);
        assert_eq!(tokens.authorization_header(), "Bearer access");
    }

    #[test]
    fn test_refresh_token_request_validation() {
        let valid = RefreshTokenRequest::new("valid_refresh_token");
        assert!(valid.validate().is_ok());

        let empty = RefreshTokenRequest::new("");
        assert!(empty.validate().is_err());

        let too_short = RefreshTokenRequest::new("short");
        assert!(too_short.validate().is_err());
    }

    #[test]
    fn test_password_change_validation() {
        let valid = PasswordChangeRequest::new("old", "new", "new");
        assert!(valid.validate().is_ok());

        let no_match = PasswordChangeRequest::new("old", "new", "different");
        assert!(no_match.validate().is_err());

        let same_password = PasswordChangeRequest::new("same", "same", "same");
        assert!(same_password.validate().is_err());
    }

    #[test]
    fn test_login_request() {
        let request = LoginRequest::new("user", "pass").with_remember_me(true);
        assert_eq!(request.remember_me, Some(true));

        let creds = request.to_credentials();
        assert_eq!(creds.username, "user");
        assert_eq!(creds.password, "pass");
    }
}
