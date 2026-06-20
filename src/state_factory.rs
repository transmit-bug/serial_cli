//! State container factory
//!
//! Provides shared factory functions for creating common state containers
//! (PortManager, ScriptManager) used across different parts of the application.

use crate::script::ScriptManager;
use crate::serial_core::PortManager;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Shared state container holding the core managers
#[derive(Clone)]
pub struct CoreManagers {
    /// Serial port manager
    pub port_manager: Arc<Mutex<PortManager>>,
    /// Script manager
    pub script_manager: Arc<Mutex<ScriptManager>>,
}

impl CoreManagers {
    /// Create a new set of core managers
    pub fn new() -> Self {
        Self {
            port_manager: Arc::new(Mutex::new(PortManager::new())),
            script_manager: Arc::new(Mutex::new(ScriptManager::new())),
        }
    }

    /// Create core managers with IoLoop enabled for the port manager
    pub fn with_ioloop() -> Self {
        Self {
            port_manager: Arc::new(Mutex::new(PortManager::with_ioloop())),
            script_manager: Arc::new(Mutex::new(ScriptManager::new())),
        }
    }
}

impl Default for CoreManagers {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_managers_creation() {
        let managers = CoreManagers::new();
        assert!(Arc::strong_count(&managers.port_manager) >= 1);
        assert!(Arc::strong_count(&managers.script_manager) >= 1);
    }

    #[test]
    fn test_core_managers_with_ioloop() {
        let managers = CoreManagers::with_ioloop();
        assert!(Arc::strong_count(&managers.port_manager) >= 1);
        assert!(Arc::strong_count(&managers.script_manager) >= 1);
    }

    #[test]
    fn test_core_managers_clone() {
        let managers1 = CoreManagers::new();
        let managers2 = managers1.clone();

        // Both should point to the same underlying managers
        assert!(Arc::ptr_eq(&managers1.port_manager, &managers2.port_manager));
        assert!(Arc::ptr_eq(&managers1.script_manager, &managers2.script_manager));
    }
}
