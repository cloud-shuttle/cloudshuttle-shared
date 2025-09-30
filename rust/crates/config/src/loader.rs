//! Configuration loader with environment and file support

use config::{Config as ConfigBuilder, ConfigError, Environment, File};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::path::Path;
use validator::Validate;

/// Configuration loader
pub struct ConfigLoader {
    service_name: String,
    env_prefix: Option<String>,
    config_files: Vec<String>,
    overrides: HashMap<String, String>,
}

impl ConfigLoader {
    /// Create a new config loader for a service
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            env_prefix: None,
            config_files: vec!["config/default".to_string()],
            overrides: HashMap::new(),
        }
    }

    /// Set environment variable prefix
    pub fn with_env_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.env_prefix = Some(prefix.into());
        self
    }

    /// Add a configuration file
    pub fn with_config_file(mut self, file: impl Into<String>) -> Self {
        self.config_files.push(file.into());
        self
    }

    /// Add environment-specific config file
    pub fn with_env_config_file(self, environment: &str) -> Self {
        let file = format!("config/{}", environment);
        self.with_config_file(file)
    }

    /// Override a configuration value
    pub fn with_override(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.overrides.insert(key.into(), value.into());
        self
    }

    /// Load configuration from all sources
    pub fn load<T: DeserializeOwned + Validate>(&self) -> Result<T, ConfigError> {
        let mut builder = ConfigBuilder::builder();

        // Add configuration files
        for file in &self.config_files {
            if Path::new(file).exists() {
                builder = builder.add_source(File::with_name(file));
                tracing::debug!("Loaded config file: {}", file);
            } else {
                tracing::debug!("Config file not found: {}", file);
            }
        }

        // Add environment variables
        if let Some(prefix) = &self.env_prefix {
            builder = builder.add_source(Environment::with_prefix(prefix).separator("_"));
        } else {
            builder = builder.add_source(Environment::default());
        }

        // Add overrides
        for (key, value) in &self.overrides {
            builder = builder.set_override(key, value.clone())?;
        }

        // Build the configuration
        let config = builder.build()?;

        // Deserialize and validate
        let settings: T = config.try_deserialize().map_err(|e| {
            ConfigError::Message(format!("Deserialization error: {}", e))
        })?;

        // Validate the configuration
        settings.validate().map_err(|e| {
            ConfigError::Message(format!("Validation error: {}", e))
        })?;

        tracing::info!("Configuration loaded for service: {}", self.service_name);
        Ok(settings)
    }

    /// Load configuration without validation (for partial configs)
    pub fn load_unvalidated<T: DeserializeOwned>(&self) -> Result<T, ConfigError> {
        let mut builder = ConfigBuilder::builder();

        // Add configuration files
        for file in &self.config_files {
            if Path::new(file).exists() {
                builder = builder.add_source(File::with_name(file));
            }
        }

        // Add environment variables
        if let Some(prefix) = &self.env_prefix {
            builder = builder.add_source(Environment::with_prefix(prefix).separator("_"));
        } else {
            builder = builder.add_source(Environment::default());
        }

        // Add overrides
        for (key, value) in &self.overrides {
            builder = builder.set_override(key, value.clone())?;
        }

        let config = builder.build()?;
        let settings: T = config.try_deserialize().map_err(|e| {
            ConfigError::Message(format!("Deserialization error: {}", e))
        })?;

        Ok(settings)
    }
}

/// Environment detection
pub struct Environment;

impl Environment {
    /// Get current environment (development, staging, production)
    pub fn current() -> String {
        std::env::var("ENVIRONMENT")
            .or_else(|_| std::env::var("ENV"))
            .unwrap_or_else(|_| "development".to_string())
    }

    /// Check if running in development
    pub fn is_development() -> bool {
        Self::current() == "development"
    }

    /// Check if running in production
    pub fn is_production() -> bool {
        Self::current() == "production"
    }

    /// Check if running in staging
    pub fn is_staging() -> bool {
        Self::current() == "staging"
    }
}

/// Configuration file watcher for hot reloading
pub struct ConfigWatcher {
    paths: Vec<String>,
    callback: Box<dyn Fn() + Send + Sync>,
}

impl ConfigWatcher {
    pub fn new<F>(callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        Self {
            paths: vec!["config".to_string()],
            callback: Box::new(callback),
        }
    }

    pub fn watch_path(mut self, path: impl Into<String>) -> Self {
        self.paths.push(path.into());
        self
    }

    pub async fn start_watching(self) -> Result<(), Box<dyn std::error::Error>> {
        // In a real implementation, this would use notify crate
        // to watch for file changes and call the callback
        tracing::info!("Config watcher started (stub implementation)");
        Ok(())
    }
}

/// Configuration template renderer
pub struct ConfigTemplate {
    template: String,
    variables: HashMap<String, String>,
}

impl ConfigTemplate {
    pub fn new(template: impl Into<String>) -> Self {
        Self {
            template: template.into(),
            variables: HashMap::new(),
        }
    }

    pub fn with_variable(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.variables.insert(key.into(), value.into());
        self
    }

    pub fn render(&self) -> String {
        let mut result = self.template.clone();

        for (key, value) in &self.variables {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        result
    }
}

/// Configuration schema validator
pub struct SchemaValidator;

impl SchemaValidator {
    pub fn validate_against_schema<T: serde::Serialize>(
        config: &T,
        schema_path: &str,
    ) -> Result<(), ConfigError> {
        // In a real implementation, this would validate against JSON schema
        // For now, just return success
        tracing::debug!("Schema validation stub for {}", schema_path);
        Ok(())
    }
}
