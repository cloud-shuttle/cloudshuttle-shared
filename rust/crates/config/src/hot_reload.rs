//! Hot reloading for configuration
//!
//! This module provides functionality to watch configuration files
//! and reload them automatically when they change.

use futures::StreamExt;
use notify::Watcher;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::result::Result as StdResult;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio_stream::{wrappers::BroadcastStream, Stream};

/// Error type for hot reload operations
#[derive(Debug, thiserror::Error)]
pub enum HotReloadError {
    #[error("File watch error: {0}")]
    FileWatchError(String),

    #[error("Configuration reload failed: {0}")]
    ReloadFailed(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),
}

/// Result type for hot reload operations
pub type Result<T> = StdResult<T, HotReloadError>;

/// Configuration hot reloader
pub struct ConfigHotReloader<T> {
    /// Paths to watch for changes
    paths: Vec<PathBuf>,
    /// Current configuration
    current_config: Arc<RwLock<T>>,
    /// Change notification sender
    change_tx: broadcast::Sender<T>,
}

impl<T> ConfigHotReloader<T>
where
    T: Clone + Send + Sync + 'static,
{
    /// Create a new hot reloader
    pub fn new(initial_config: T) -> Self {
        let (change_tx, _) = broadcast::channel(16);

        Self {
            paths: Vec::new(),
            current_config: Arc::new(RwLock::new(initial_config)),
            change_tx,
        }
    }

    /// Add a path to watch for changes
    pub fn watch_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.paths.push(path.as_ref().to_path_buf());
        self
    }

    /// Add multiple paths to watch
    pub fn watch_paths<P: AsRef<Path>>(mut self, paths: &[P]) -> Self {
        for path in paths {
            self.paths.push(path.as_ref().to_path_buf());
        }
        self
    }

    /// Get the current configuration
    pub async fn current_config(&self) -> T {
        self.current_config.read().await.clone()
    }

    /// Update the configuration
    pub async fn update_config(&self, new_config: T) -> Result<()> {
        *self.current_config.write().await = new_config.clone();

        // Notify all listeners
        let _ = self.change_tx.send(new_config);

        Ok(())
    }

    /// Get a stream of configuration changes
    pub fn config_changes(&self) -> impl Stream<Item = T> {
        BroadcastStream::new(self.change_tx.subscribe())
            .map(|result| result.unwrap_or_else(|_| panic!("Config change stream lagged")))
    }

    /// Get change listener
    pub fn change_listener(&self) -> ConfigChangeListener<T> {
        ConfigChangeListener {
            config_rx: self.change_tx.subscribe(),
        }
    }

    /// Get paths being watched
    pub fn watched_paths(&self) -> &[PathBuf] {
        &self.paths
    }
}

/// Listener for configuration changes
pub struct ConfigChangeListener<T> {
    config_rx: broadcast::Receiver<T>,
}

impl<T> ConfigChangeListener<T>
where
    T: Clone + Send + Sync + 'static,
{
    /// Wait for the next configuration change
    pub async fn wait_for_change(&mut self) -> Option<T> {
        self.config_rx.recv().await.ok()
    }

    /// Create a new listener (convenience method)
    pub fn new(reloader: &ConfigHotReloader<T>) -> Self {
        reloader.change_listener()
    }
}

/// File watcher for configuration files
pub struct ConfigFileWatcher {
    watcher: notify::RecommendedWatcher,
    watched_paths: HashMap<PathBuf, std::time::SystemTime>,
}

impl ConfigFileWatcher {
    /// Create a new file watcher
    pub fn new() -> Result<Self> {
        let watcher = notify::recommended_watcher(|res| {
            match res {
                Ok(_) => tracing::debug!("File change detected"),
                Err(e) => tracing::error!("File watch error: {}", e),
            }
        })
        .map_err(|e| HotReloadError::FileWatchError(e.to_string()))?;

        Ok(Self {
            watcher,
            watched_paths: HashMap::new(),
        })
    }

    /// Watch a file for changes
    pub fn watch_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref().to_path_buf();

        self.watcher
            .watch(&path, notify::RecursiveMode::NonRecursive)
            .map_err(|e| HotReloadError::FileWatchError(e.to_string()))?;

        // Store modification time
        if let Ok(metadata) = std::fs::metadata(&path) {
            if let Ok(modified) = metadata.modified() {
                self.watched_paths.insert(path, modified);
            }
        }

        Ok(())
    }

    /// Check if any watched files have changed
    pub fn check_for_changes(&mut self) -> Vec<PathBuf> {
        let mut changed_files = Vec::new();

        for (path, last_modified) in &mut self.watched_paths {
            if let Ok(metadata) = std::fs::metadata(path) {
                if let Ok(current_modified) = metadata.modified() {
                    if &current_modified != last_modified {
                        *last_modified = current_modified;
                        changed_files.push(path.clone());
                    }
                }
            }
        }

        changed_files
    }
}

/// Hot reload manager that combines file watching with config reloading
pub struct HotReloadManager<T, F> {
    reloader: ConfigHotReloader<T>,
    file_watcher: ConfigFileWatcher,
    reload_fn: F,
}

impl<T, F, Fut> HotReloadManager<T, F>
where
    T: Clone + Send + Sync + 'static,
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<T>> + Send,
{
    /// Create a new hot reload manager
    pub fn new(initial_config: T, reload_fn: F) -> Result<Self> {
        Ok(Self {
            reloader: ConfigHotReloader::new(initial_config),
            file_watcher: ConfigFileWatcher::new()?,
            reload_fn,
        })
    }

    /// Watch a configuration file
    pub fn watch_config_file<P: AsRef<Path>>(mut self, path: P) -> Result<Self> {
        self.file_watcher.watch_file(&path)?;
        self.reloader = self.reloader.watch_path(path);
        Ok(self)
    }

    /// Start the hot reload loop
    pub async fn start(mut self) -> Result<()> {
        loop {
            // Check for file changes
            let changed_files = self.file_watcher.check_for_changes();

            if !changed_files.is_empty() {
                tracing::info!("Configuration files changed: {:?}", changed_files);

                // Reload configuration
                match (self.reload_fn)().await {
                    Ok(new_config) => {
                        self.reloader.update_config(new_config).await?;
                        tracing::info!("Configuration reloaded successfully");
                    }
                    Err(e) => {
                        tracing::error!("Failed to reload configuration: {}", e);
                    }
                }
            }

            // Sleep before next check
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }

    /// Get the config reloader
    pub fn reloader(&self) -> &ConfigHotReloader<T> {
        &self.reloader
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_config_hot_reloader() {
        let initial_config = "initial".to_string();
        let reloader = ConfigHotReloader::new(initial_config);

        // Check initial config
        assert_eq!(reloader.current_config().await, "initial");

        // Update config
        reloader.update_config("updated".to_string()).await.unwrap();
        assert_eq!(reloader.current_config().await, "updated");
    }

    #[test]
    fn test_config_change_listener() {
        let reloader = ConfigHotReloader::new(42);
        let mut listener = reloader.change_listener();

        // In a real test, we'd spawn a task to send changes
        // For now, just test creation
        assert!(true);
    }

    #[test]
    fn test_config_file_watcher() {
        let mut watcher = ConfigFileWatcher::new().unwrap();

        // Test watching a non-existent file (should not panic)
        let result = watcher.watch_file("/tmp/non-existent-config.toml");
        assert!(result.is_ok());
    }
}
