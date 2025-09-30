//! Service-specific error handling traits

use async_trait::async_trait;

/// Trait for service-specific errors
#[async_trait]
pub trait ServiceError: std::error::Error + Send + Sync {
    /// Get the error code for this error type
    fn error_code(&self) -> &'static str;

    /// Get the HTTP status code for this error
    fn http_status(&self) -> http::StatusCode;

    /// Get a user-friendly message for this error
    fn user_message(&self) -> String;

    /// Get additional context for logging/debugging
    fn context(&self) -> Option<serde_json::Value> {
        None
    }

    /// Check if this error should trigger alerting
    fn should_alert(&self) -> bool {
        false
    }

    /// Get the error category for grouping/monitoring
    fn category(&self) -> &'static str {
        "general"
    }
}

/// Helper macro for implementing ServiceError
#[macro_export]
macro_rules! impl_service_error {
    ($error_type:ty, $code:expr, $status:expr, $category:expr) => {
        impl $crate::ServiceError for $error_type {
            fn error_code(&self) -> &'static str {
                $code
            }

            fn http_status(&self) -> http::StatusCode {
                $status
            }

            fn user_message(&self) -> String {
                self.to_string()
            }

            fn category(&self) -> &'static str {
                $category
            }
        }
    };
}

/// Extension trait for Result types with ServiceError
pub trait ServiceResultExt<T> {
    fn with_context<F>(self, f: F) -> Self
    where
        F: FnOnce() -> String;

    fn with_error_code(self, code: &'static str) -> Self;
}

impl<T, E> ServiceResultExt<T> for Result<T, E>
where
    E: ServiceError,
{
    fn with_context<F>(self, _f: F) -> Self
    where
        F: FnOnce() -> String,
    {
        // For now, just return self. In a real implementation,
        // you might want to wrap the error with additional context
        self
    }

    fn with_error_code(self, _code: &'static str) -> Self {
        // For now, just return self. In a real implementation,
        // you might want to create a new error with the specified code
        self
    }
}

/// Error metrics collector for monitoring
pub struct ErrorMetrics {
    pub total_errors: std::sync::atomic::AtomicU64,
    pub errors_by_category: std::collections::HashMap<String, std::sync::atomic::AtomicU64>,
}

impl ErrorMetrics {
    pub fn new() -> Self {
        Self {
            total_errors: std::sync::atomic::AtomicU64::new(0),
            errors_by_category: std::collections::HashMap::new(),
        }
    }

    pub fn record_error<E: ServiceError>(&mut self, error: &E) {
        self.total_errors.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let category = error.category().to_string();
        self.errors_by_category
            .entry(category)
            .or_insert_with(|| std::sync::atomic::AtomicU64::new(0))
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn get_total_errors(&self) -> u64 {
        self.total_errors.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn get_errors_by_category(&self, category: &str) -> u64 {
        self.errors_by_category
            .get(category)
            .map(|counter| counter.load(std::sync::atomic::Ordering::Relaxed))
            .unwrap_or(0)
    }
}

/// Error boundary for isolating error handling
pub struct ErrorBoundary<T> {
    result: Result<T, Box<dyn ServiceError>>,
}

impl<T> ErrorBoundary<T> {
    pub fn new(result: Result<T, Box<dyn ServiceError>>) -> Self {
        Self { result }
    }

    pub fn map_err<E, F>(self, f: F) -> Result<T, E>
    where
        F: FnOnce(Box<dyn ServiceError>) -> E,
        E: ServiceError,
    {
        self.result.map_err(f)
    }

    pub fn unwrap_or_else<F>(self, f: F) -> T
    where
        F: FnOnce(Box<dyn ServiceError>) -> T,
    {
        match self.result {
            Ok(value) => value,
            Err(error) => f(error),
        }
    }

    pub fn log_error(&self) {
        if let Err(error) = &self.result {
            tracing::error!(
                error_code = error.error_code(),
                category = error.category(),
                "Service error in boundary: {}",
                error
            );
        }
    }
}


