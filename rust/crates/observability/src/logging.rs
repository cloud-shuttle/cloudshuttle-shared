//! Structured logging utilities

use tracing::{info, warn, error, debug, Level};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Log an operation start
#[macro_export]
macro_rules! log_operation_start {
    ($operation:expr) => {
        info!(operation = $operation, "Starting operation");
    };
    ($operation:expr, $($field:tt)*) => {
        info!(operation = $operation, $($field)*, "Starting operation");
    };
}

/// Log an operation completion
#[macro_export]
macro_rules! log_operation_complete {
    ($operation:expr) => {
        info!(operation = $operation, "Operation completed");
    };
    ($operation:expr, $($field:tt)*) => {
        info!(operation = $operation, $($field)*, "Operation completed");
    };
}

/// Log performance metrics
#[macro_export]
macro_rules! log_performance {
    ($operation:expr, $duration_ms:expr) => {
        info!(operation = $operation, duration_ms = $duration_ms, "Operation performance");
    };
    ($operation:expr, $duration_ms:expr, $($field:tt)*) => {
        info!(operation = $operation, duration_ms = $duration_ms, $($field)*, "Operation performance");
    };
}

/// Tracing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    pub service_name: String,
    pub level: String,
    pub format: LogFormat,
    pub enable_json: bool,
    pub enable_file_logging: bool,
    pub log_file_path: Option<String>,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            service_name: "cloudshuttle-service".to_string(),
            level: "INFO".to_string(),
            format: LogFormat::default(),
            enable_json: false,
            enable_file_logging: false,
            log_file_path: None,
        }
    }
}

/// Log format options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    Compact,
    Pretty,
    Json,
}

impl Default for LogFormat {
    fn default() -> Self {
        Self::Compact
    }
}

impl FromStr for LogFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "compact" => Ok(Self::Compact),
            "pretty" => Ok(Self::Pretty),
            "json" => Ok(Self::Json),
            _ => Err(format!("Unknown log format: {}", s)),
        }
    }
}

/// Initialize tracing with the given configuration
pub fn init_tracing(service_name: &str, default_level: Level) -> Result<(), Box<dyn std::error::Error>> {
    init_tracing_with_config(TracingConfig {
        service_name: service_name.to_string(),
        level: default_level.to_string(),
        ..TracingConfig::default()
    })
}

/// Initialize tracing with full configuration
pub fn init_tracing_with_config(config: TracingConfig) -> Result<(), Box<dyn std::error::Error>> {
    // Create filter from environment or config
    let filter = EnvFilter::try_from_env("RUST_LOG")
        .unwrap_or_else(|_| {
            EnvFilter::new(format!("{}={}", config.service_name, config.level))
        });

    // Create formatter based on configuration
    let subscriber = match config.format {
        LogFormat::Compact => tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().with_target(false).with_thread_ids(false).with_thread_names(false).compact()),
        LogFormat::Pretty => tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().with_target(false).with_thread_ids(false).with_thread_names(false).pretty()),
        LogFormat::Json => tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().with_target(false).with_thread_ids(false).with_thread_names(false).json()),
    };

    // Initialize the subscriber
    subscriber.init();

    info!(
        service = %config.service_name,
        level = %config.level,
        format = ?config.format,
        "Tracing initialized"
    );

    Ok(())
}

/// Get the current tracing level
pub fn current_level() -> Option<Level> {
    // This is a simplified implementation
    // In a real implementation, you'd get this from the current subscriber
    Some(Level::INFO)
}

/// Context-aware logging
pub struct Logger {
    context: std::collections::HashMap<String, serde_json::Value>,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            context: std::collections::HashMap::new(),
        }
    }

    pub fn with_context(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.context.insert(key.into(), json_value);
        }
        self
    }

    pub fn with_request_id(self, request_id: impl Into<String>) -> Self {
        self.with_context("request_id", request_id.into())
    }

    pub fn with_user_id(self, user_id: impl Into<String>) -> Self {
        self.with_context("user_id", user_id.into())
    }

    pub fn with_tenant_id(self, tenant_id: impl Into<String>) -> Self {
        self.with_context("tenant_id", tenant_id.into())
    }

    pub fn info(&self, message: &str) {
        let context_str = self.context_string();
        if context_str.is_empty() {
            info!("{}", message);
        } else {
            info!("{} {}", message, context_str);
        }
    }

    pub fn warn(&self, message: &str) {
        let context_str = self.context_string();
        if context_str.is_empty() {
            warn!("{}", message);
        } else {
            warn!("{} {}", message, context_str);
        }
    }

    pub fn error(&self, message: &str) {
        let context_str = self.context_string();
        if context_str.is_empty() {
            error!("{}", message);
        } else {
            error!("{} {}", message, context_str);
        }
    }

    pub fn debug(&self, message: &str) {
        let context_str = self.context_string();
        if context_str.is_empty() {
            debug!("{}", message);
        } else {
            debug!("{} {}", message, context_str);
        }
    }

    fn context_string(&self) -> String {
        if self.context.is_empty() {
            String::new()
        } else {
            format!("context={:?}", self.context)
        }
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

/// Log level utilities
pub struct LogLevel;

impl LogLevel {
    pub fn from_str(s: &str) -> Option<Level> {
        match s.to_uppercase().as_str() {
            "TRACE" => Some(Level::TRACE),
            "DEBUG" => Some(Level::DEBUG),
            "INFO" => Some(Level::INFO),
            "WARN" => Some(Level::WARN),
            "ERROR" => Some(Level::ERROR),
            _ => None,
        }
    }

    pub fn to_string(level: Level) -> String {
        match level {
            Level::TRACE => "TRACE",
            Level::DEBUG => "DEBUG",
            Level::INFO => "INFO",
            Level::WARN => "WARN",
            Level::ERROR => "ERROR",
        }.to_string()
    }
}

/// Log sampling for high-frequency logs
pub struct LogSampler {
    interval: std::time::Duration,
    last_log: std::sync::Mutex<std::time::Instant>,
}

impl LogSampler {
    pub fn new(interval: std::time::Duration) -> Self {
        Self {
            interval,
            last_log: std::sync::Mutex::new(std::time::Instant::now() - interval),
        }
    }

    pub fn should_log(&self) -> bool {
        let mut last_log = self.last_log.lock().unwrap();
        let now = std::time::Instant::now();

        if now.duration_since(*last_log) >= self.interval {
            *last_log = now;
            true
        } else {
            false
        }
    }

    pub fn sample_info(&self, message: &str) {
        if self.should_log() {
            info!("{}", message);
        }
    }

    pub fn sample_debug(&self, message: &str) {
        if self.should_log() {
            debug!("{}", message);
        }
    }
}

/// Performance logging
pub struct PerformanceLogger {
    operation: String,
    start_time: std::time::Instant,
}

impl PerformanceLogger {
    pub fn start(operation: impl Into<String>) -> Self {
        let operation = operation.into();
        log_operation_start!(operation);
        Self {
            operation,
            start_time: std::time::Instant::now(),
        }
    }

    pub fn log_duration(self) {
        let duration = self.start_time.elapsed();
        log_performance!(self.operation, duration.as_millis() as u64);
    }
}

impl Drop for PerformanceLogger {
    fn drop(&mut self) {
        let duration = self.start_time.elapsed();
        info!(
            operation = %self.operation,
            duration_ms = duration.as_millis(),
            "Operation completed"
        );
    }
}
