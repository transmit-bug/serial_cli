//! Backend factory for creating virtual backends
//!
//! This module provides the BackendFactory which handles backend selection
//! with the following priority:
//! 1. CLI flag (explicit override)
//! 2. Config file setting
//! 3. Platform auto-detection

use crate::config::ConfigManager;
use crate::error::{Result, SerialError};
#[cfg(unix)]
use crate::serial_core::backends::PtyBackend;
use crate::serial_core::backends::{BackendType, SocatBackend, VirtualBackend};
use std::sync::Arc;

/// Backend factory for creating virtual backends
pub struct BackendFactory {
    config: Arc<ConfigManager>,
}

impl BackendFactory {
    /// Create a new backend factory
    pub fn new(config: Arc<ConfigManager>) -> Self {
        Self { config }
    }

    /// Create a backend instance with priority chain:
    /// 1. CLI flag (explicit)
    /// 2. Config file setting
    /// 3. Auto-detection
    pub async fn create_backend(
        &self,
        cli_override: Option<BackendType>,
    ) -> Result<Box<dyn VirtualBackend>> {
        let backend_type = self.resolve_backend_type(cli_override).await?;
        self.instantiate_backend(backend_type).await
    }

    /// Resolve backend type from priority chain
    async fn resolve_backend_type(&self, cli_override: Option<BackendType>) -> Result<BackendType> {
        // Priority 1: CLI flag
        if let Some(cli_type) = cli_override {
            tracing::debug!("Using CLI override for backend: {:?}", cli_type);
            return self.validate_backend_available(cli_type).await;
        }

        // Priority 2: Config file
        let config_type = self.config.get_virtual_backend_type();
        if config_type != BackendType::Auto {
            tracing::debug!("Using config setting for backend: {:?}", config_type);
            return self.validate_backend_available(config_type).await;
        }

        // Priority 3: Auto-detection
        let detected = BackendType::detect();
        tracing::debug!("Using auto-detected backend: {:?}", detected);
        self.validate_backend_available(detected).await
    }

    /// Validate that the selected backend is available on this system
    async fn validate_backend_available(&self, backend: BackendType) -> Result<BackendType> {
        match backend {
            BackendType::Pty => {
                #[cfg(unix)]
                return Ok(BackendType::Pty);

                #[cfg(not(unix))]
                return Err(SerialError::UnsupportedBackend(
                    "PTY backend is only available on Unix/macOS".to_string(),
                ));
            }
            BackendType::NamedPipe => {
                #[cfg(windows)]
                return Ok(BackendType::NamedPipe);

                #[cfg(not(windows))]
                return Err(SerialError::UnsupportedBackend(
                    "NamedPipe backend is only available on Windows".to_string(),
                ));
            }
            BackendType::Socat => {
                // Check if socat binary exists
                if !SocatBackend::check_available().await {
                    return Err(SerialError::MissingDependency(
                        "socat".to_string(),
                        "Install with: apt install socat | brew install socat".to_string(),
                    ));
                }
                Ok(BackendType::Socat)
            }
            BackendType::Auto => unreachable!("Auto should be resolved before validation"),
        }
    }

    /// Instantiate the concrete backend
    async fn instantiate_backend(&self, backend: BackendType) -> Result<Box<dyn VirtualBackend>> {
        let boxed: Box<dyn VirtualBackend> = match backend {
            #[cfg(unix)]
            BackendType::Pty => Box::new(PtyBackend::new()?),
            #[cfg(not(unix))]
            BackendType::Pty => {
                return Err(SerialError::UnsupportedBackend(
                    "PTY backend is not available on this platform".to_string(),
                ))
            }
            #[cfg(windows)]
            BackendType::NamedPipe => {
                use crate::serial_core::backends::NamedPipeBackend;
                Box::new(NamedPipeBackend::new()?)
            }
            #[cfg(not(windows))]
            BackendType::NamedPipe => {
                return Err(SerialError::UnsupportedBackend(
                    "NamedPipe backend is not available on this platform".to_string(),
                ))
            }
            BackendType::Socat => Box::new(SocatBackend::new()?),
            BackendType::Auto => {
                unreachable!("Auto should be resolved before instantiation")
            }
        };
        Ok(boxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_factory_create_backend_auto() {
        // This test requires a ConfigManager, which we can't easily create here
        // In a real test, you'd set up a test config
    }

    #[test]
    fn test_backend_type_resolution() {
        // Test that BackendType::detect() returns the correct type
        let detected = BackendType::detect();
        #[cfg(windows)]
        assert_eq!(detected, BackendType::NamedPipe);
        #[cfg(unix)]
        assert_eq!(detected, BackendType::Pty);
    }
}
