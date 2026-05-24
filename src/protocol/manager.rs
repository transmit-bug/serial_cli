//! Protocol manager for custom protocol lifecycle
//!
//! Manages loading, unloading, reloading, and persistence of custom protocols.

use crate::error::{ProtocolError, Result, SerialError};
use crate::protocol::loader::ProtocolLoader;
use crate::protocol::validator::ProtocolValidator;
use crate::protocol::{ProtocolInfo, ProtocolRegistry};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

/// Custom protocol metadata tracked by the [`ProtocolManager`].
#[derive(Debug, Clone)]
pub struct CustomProtocol {
    /// Protocol name.
    pub name: String,
    /// Filesystem path to the Lua script.
    pub script_path: PathBuf,
    /// Time the protocol was loaded (or last reloaded).
    pub loaded_at: std::time::SystemTime,
    /// Reload counter (incremented on each hot-reload).
    pub version: u64,
}

/// Manages the lifecycle of custom Lua protocols: loading from disk,
/// registering into the [`ProtocolRegistry`], unloading, reloading, and
/// optional hot-reload watching.
pub struct ProtocolManager {
    registry: Arc<Mutex<ProtocolRegistry>>,
    custom_protocols: HashMap<String, CustomProtocol>,
    /// Hot-reload watcher task handle
    watcher_task: Option<JoinHandle<()>>,
    /// Whether hot-reload is enabled
    hot_reload_enabled: Arc<Mutex<bool>>,
}

impl ProtocolManager {
    /// Create a new manager with the given registry. Hot-reload is disabled by default.
    pub fn new(registry: Arc<Mutex<ProtocolRegistry>>) -> Self {
        Self {
            registry,
            custom_protocols: HashMap::new(),
            watcher_task: None,
            hot_reload_enabled: Arc::new(Mutex::new(false)),
        }
    }

    /// Load a protocol from a Lua script file, register it in the registry,
    /// and track it as a [`CustomProtocol`].
    ///
    /// # Errors
    ///
    /// Propagates errors from [`ProtocolLoader`] (validation, file I/O)
    /// and factory creation.
    pub async fn load_protocol(&mut self, path: &Path) -> Result<ProtocolInfo> {
        // Load the protocol
        let loaded = ProtocolLoader::load_from_file(path)?;

        // Create factory
        let factory = ProtocolLoader::create_factory(&loaded)?;

        // Store metadata
        let custom = CustomProtocol {
            name: loaded.name.clone(),
            script_path: loaded.script_path.clone(),
            loaded_at: loaded.loaded_at,
            version: 1,
        };

        // Register to protocol registry
        let mut registry = self.registry.lock().await;
        registry.register(factory).await;
        drop(registry);

        // Store in custom protocols tracking
        self.custom_protocols.insert(loaded.name.clone(), custom);

        tracing::info!(
            "Successfully loaded and registered protocol: {}",
            loaded.name
        );

        Ok(ProtocolInfo {
            name: loaded.name,
            description: "Custom Lua protocol".to_string(),
        })
    }

    /// Unload a previously loaded custom protocol from both the registry and
    /// the internal tracking map.
    ///
    /// # Errors
    ///
    /// Returns [`SerialError::Protocol`] if the protocol is not found.
    pub async fn unload_protocol(&mut self, name: &str) -> Result<()> {
        // Check if protocol exists
        if !self.custom_protocols.contains_key(name) {
            return Err(SerialError::Protocol(ProtocolError::NotFound(
                name.to_string(),
            )));
        }

        // Remove from metadata
        self.custom_protocols.remove(name);

        // Remove from registry
        let mut registry = self.registry.lock().await;
        registry.unregister(name).await?;

        tracing::info!("Successfully unloaded protocol: {}", name);

        Ok(())
    }

    /// Reload a custom protocol from its original file path. Unloads the
    /// existing version first, then loads fresh from disk.
    ///
    /// # Errors
    ///
    /// Returns [`SerialError::Protocol`] if the protocol is not found or
    /// the reloaded script fails validation.
    pub async fn reload_protocol(&mut self, name: &str) -> Result<()> {
        // Get existing metadata
        let script_path = self
            .custom_protocols
            .get(name)
            .ok_or_else(|| SerialError::Protocol(ProtocolError::NotFound(name.to_string())))?
            .script_path
            .clone();

        // Unload existing
        self.unload_protocol(name).await?;

        // Reload from file
        self.load_protocol(&script_path).await?;

        Ok(())
    }

    /// List all registered protocols, marking custom ones with a `(custom)` suffix.
    pub async fn list_protocols(&self) -> Vec<ProtocolInfo> {
        let registry = self.registry.lock().await;
        let mut protocols = registry.list_protocols().await;

        // Mark custom protocols
        for proto in protocols.iter_mut() {
            if self.custom_protocols.contains_key(&proto.name) {
                proto.description = format!("{} (custom)", proto.description);
            }
        }

        protocols
    }

    /// Look up custom protocol metadata by name.
    pub fn get_custom_protocol(&self, name: &str) -> Option<&CustomProtocol> {
        self.custom_protocols.get(name)
    }

    /// Get the number of loaded custom protocols.
    pub fn custom_protocols_len(&self) -> usize {
        self.custom_protocols.len()
    }

    /// Validate a protocol script without loading or registering it.
    pub fn validate_protocol(path: &Path) -> Result<()> {
        ProtocolValidator::validate_script(path)?;
        Ok(())
    }

    /// Enable hot-reload for protocol scripts.
    ///
    /// Spawns a background task that watches the directories of all currently
    /// loaded (and future) custom protocol scripts via [`ProtocolWatcher`],
    /// automatically reloading them when the file changes on disk.
    pub async fn enable_hot_reload(&mut self) -> Result<()> {
        if *self.hot_reload_enabled.lock().await {
            tracing::warn!("Hot-reload is already enabled");
            return Ok(());
        }

        // Create the file watcher
        let mut watcher = crate::protocol::ProtocolWatcher::new()?;

        // Watch directories of all currently loaded custom protocols
        for custom in self.custom_protocols.values() {
            if let Err(e) = watcher.watch(&custom.script_path) {
                tracing::warn!(
                    "Failed to watch protocol script {}: {}",
                    custom.script_path.display(),
                    e
                );
            }
        }

        // Take the reload event receiver
        let mut reload_rx = watcher
            .reload_events()
            .ok_or_else(|| SerialError::Protocol(ProtocolError::InvalidState(
                "Failed to get reload event receiver".to_string(),
            )))?;

        // Clone data the background task needs
        let registry = self.registry.clone();
        let custom_protocols: Arc<Mutex<HashMap<String, CustomProtocol>>> =
            Arc::new(Mutex::new(self.custom_protocols.clone()));

        // Spawn the background reload task
        let task = tokio::spawn(async move {
            while let Some(path) = reload_rx.recv().await {
                let _file_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown");

                tracing::info!("Protocol script changed: {}", path.display());

                // Look up the protocol name from the path
                let name = {
                    let customs = custom_protocols.lock().await;
                    customs
                        .iter()
                        .find(|(_, v)| v.script_path == path)
                        .map(|(k, _)| k.clone())
                };

                let Some(name) = name else {
                    tracing::debug!("Changed file is not a tracked protocol: {}", path.display());
                    continue;
                };

                // Reload the protocol
                let loader_result = ProtocolLoader::load_from_file(&path);
                match loader_result {
                    Ok(loaded) => match ProtocolLoader::create_factory(&loaded) {
                        Ok(factory) => {
                            let mut reg = registry.lock().await;
                            if let Err(e) = reg.unregister(&name).await {
                                tracing::debug!("Unregister old version: {}", e);
                            }
                            reg.register(factory).await;
                            drop(reg);

                            // Update version
                            let mut customs = custom_protocols.lock().await;
                            if let Some(custom) = customs.get_mut(&name) {
                                custom.version += 1;
                                custom.loaded_at = std::time::SystemTime::now();
                            }

                            tracing::info!("Hot-reloaded protocol: {}", name);
                        }
                        Err(e) => {
                            tracing::error!("Failed to create factory for {}: {}", name, e);
                        }
                    },
                    Err(e) => {
                        tracing::error!("Failed to reload protocol from {}: {}", path.display(), e);
                    }
                }
            }
        });

        self.watcher_task = Some(task);
        *self.hot_reload_enabled.lock().await = true;

        tracing::info!("Protocol hot-reload enabled (watching {} protocols)", self.custom_protocols.len());
        Ok(())
    }

    /// Disable hot-reload for protocol scripts
    pub async fn disable_hot_reload(&mut self) -> Result<()> {
        *self.hot_reload_enabled.lock().await = false;
        if let Some(task) = self.watcher_task.take() {
            task.abort();
        }
        tracing::info!("Protocol hot-reload disabled");
        Ok(())
    }

    /// Check if hot-reload is currently enabled
    pub async fn is_hot_reload_enabled(&self) -> bool {
        *self.hot_reload_enabled.lock().await
    }
}
