//! Structured audit logging for security and business events
//!
//! This module provides comprehensive audit logging capabilities extracted
//! from production auth service patterns, applicable to all shared components.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Audit logger for structured event logging
pub struct AuditLogger {
    service_name: String,
    log_level: AuditLevel,
}

impl AuditLogger {
    /// Create a new audit logger for a service
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            log_level: AuditLevel::Standard,
        }
    }

    /// Set the audit log level
    pub fn with_level(mut self, level: AuditLevel) -> Self {
        self.log_level = level;
        self
    }

    /// Log an audit event
    pub fn log(&self, event: AuditEvent) {
        match self.log_level {
            AuditLevel::Minimal => {
                // Only log security events
                if matches!(event.event_type, AuditEventType::Authentication | AuditEventType::Authorization | AuditEventType::Security) {
                    self.log_event(&event);
                }
            }
            AuditLevel::Standard => {
                // Log business + security events
                if !matches!(event.event_type, AuditEventType::DataAccess) {
                    self.log_event(&event);
                }
            }
            AuditLevel::Detailed => {
                // Log all events including debug
                self.log_event(&event);
            }
        }
    }

    fn log_event(&self, event: &AuditEvent) {
        // Use structured logging with the event data
        tracing::info!(
            service = %self.service_name,
            event_type = ?event.event_type,
            user_id = ?event.user_id,
            resource_id = ?event.resource_id,
            action = %event.action,
            result = ?event.result,
            ip_address = ?event.ip_address,
            user_agent = ?event.user_agent,
            "audit_event"
        );
    }
}

/// Audit log level configuration
#[derive(Debug, Clone, Copy)]
pub enum AuditLevel {
    /// Only security-critical events (auth, authz, security)
    Minimal,
    /// Business + security events (excludes data access)
    Standard,
    /// All events including data access and debug
    Detailed,
}

/// Structured audit event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Event timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Type of audit event
    pub event_type: AuditEventType,
    /// User ID if applicable
    pub user_id: Option<String>,
    /// Resource ID being accessed/modified
    pub resource_id: Option<String>,
    /// Action performed
    pub action: String,
    /// Result of the action
    pub result: AuditResult,
    /// IP address of the request
    pub ip_address: Option<String>,
    /// User agent string
    pub user_agent: Option<String>,
    /// Additional metadata
    #[serde(flatten)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Default for AuditEvent {
    fn default() -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            event_type: AuditEventType::DataAccess,
            user_id: None,
            resource_id: None,
            action: "unknown".to_string(),
            result: AuditResult::Success,
            ip_address: None,
            user_agent: None,
            metadata: HashMap::new(),
        }
    }
}

impl AuditEvent {
    /// Create a new audit event
    pub fn new(event_type: AuditEventType, action: impl Into<String>) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            event_type,
            action: action.into(),
            ..Default::default()
        }
    }

    /// Set user ID
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Set resource ID
    pub fn with_resource_id(mut self, resource_id: impl Into<String>) -> Self {
        self.resource_id = Some(resource_id.into());
        self
    }

    /// Set result
    pub fn with_result(mut self, result: AuditResult) -> Self {
        self.result = result;
        self
    }

    /// Set IP address
    pub fn with_ip_address(mut self, ip: impl Into<String>) -> Self {
        self.ip_address = Some(ip.into());
        self
    }

    /// Set user agent
    pub fn with_user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = Some(ua.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

/// Types of audit events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    /// User authentication events (login, logout, token refresh)
    Authentication,
    /// Authorization and permission checks
    Authorization,
    /// Data access and modification events
    DataAccess,
    /// Security-related events (password changes, MFA setup)
    Security,
    /// Administrative actions
    Admin,
}

/// Result of an audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditResult {
    /// Action completed successfully
    Success,
    /// Action failed
    Failure,
    /// Action resulted in an error
    Error,
}

/// Global audit logger instance
static AUDIT_LOGGER: once_cell::sync::Lazy<AuditLogger> = once_cell::sync::Lazy::new(|| {
    AuditLogger::new("cloudshuttle-shared")
});

/// Get the global audit logger
pub fn global_audit_logger() -> &'static AuditLogger {
    &AUDIT_LOGGER
}

/// Convenience function for logging audit events
pub fn audit(event: AuditEvent) {
    global_audit_logger().log(event);
}

/// Convenience function for authentication events
pub fn audit_auth(action: &str, user_id: Option<&str>, result: AuditResult) {
    let mut event = AuditEvent::new(AuditEventType::Authentication, action)
        .with_result(result);

    if let Some(uid) = user_id {
        event = event.with_user_id(uid);
    }

    audit(event);
}

/// Convenience function for authorization events
pub fn audit_authz(action: &str, user_id: Option<&str>, resource: Option<&str>, result: AuditResult) {
    let mut event = AuditEvent::new(AuditEventType::Authorization, action)
        .with_result(result);

    if let Some(uid) = user_id {
        event = event.with_user_id(uid);
    }

    if let Some(res) = resource {
        event = event.with_resource_id(res);
    }

    audit(event);
}

/// Convenience function for data access events
pub fn audit_data_access(action: &str, user_id: Option<&str>, resource: Option<&str>, result: AuditResult) {
    let mut event = AuditEvent::new(AuditEventType::DataAccess, action)
        .with_result(result);

    if let Some(uid) = user_id {
        event = event.with_user_id(uid);
    }

    if let Some(res) = resource {
        event = event.with_resource_id(res);
    }

    audit(event);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_event_creation() {
        let event = AuditEvent::new(AuditEventType::Authentication, "login")
            .with_user_id("user123")
            .with_result(AuditResult::Success);

        assert_eq!(event.action, "login");
        assert_eq!(event.user_id, Some("user123".to_string()));
        assert!(matches!(event.result, AuditResult::Success));
    }

    #[test]
    fn test_audit_logger_creation() {
        let logger = AuditLogger::new("test-service");
        assert_eq!(logger.service_name, "test-service");
    }

    #[test]
    fn test_global_audit_logger() {
        let logger = global_audit_logger();
        assert_eq!(logger.service_name, "cloudshuttle-shared");
    }

    #[test]
    fn test_convenience_functions() {
        // These should not panic
        audit_auth("login", Some("user123"), AuditResult::Success);
        audit_authz("read", Some("user123"), Some("resource456"), AuditResult::Success);
        audit_data_access("query", Some("user123"), Some("table.users"), AuditResult::Success);
    }
}
