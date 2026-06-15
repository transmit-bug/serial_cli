//! CommandService — shared orchestration logic for all command surfaces.
//!
//! Provides the business logic that CLI, JSON-RPC, and Tauri command handlers
//! delegate to. Each surface adapter creates a CommandService with its own
//! manager references and calls these methods.

use crate::error::Result;
use crate::script::{ScriptInfo, ScriptManager};
use crate::serial_core::PortManager;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Shared command orchestration.
///
/// Holds references to the core managers and exposes methods that
/// compose them. Surface adapters (CLI, RPC, Tauri) delegate to
/// these methods instead of implementing the logic themselves.
pub struct CommandService {
    pub port_manager: Arc<Mutex<PortManager>>,
    pub script_manager: Arc<Mutex<ScriptManager>>,
}

impl CommandService {
    /// Create a new CommandService with the given managers.
    pub fn new(
        port_manager: Arc<Mutex<PortManager>>,
        script_manager: Arc<Mutex<ScriptManager>>,
    ) -> Self {
        Self {
            port_manager,
            script_manager,
        }
    }

    // ── Script operations ──────────────────────────────────────────

    /// List all scripts (built-in + custom).
    pub async fn list_scripts(&self) -> Vec<ScriptInfo> {
        let manager = self.script_manager.lock().await;
        manager.list()
    }

    /// Load a custom script from a Lua file.
    pub async fn load_script(&self, path: &Path) -> Result<ScriptInfo> {
        let mut manager = self.script_manager.lock().await;
        manager.load(path)
    }

    /// Unload a custom script by name.
    pub async fn unload_script(&self, name: &str) -> Result<()> {
        let mut manager = self.script_manager.lock().await;
        manager.unload(name)
    }

    /// Reload a custom script from its original file path.
    pub async fn reload_script(&self, name: &str) -> Result<()> {
        let mut manager = self.script_manager.lock().await;
        manager.reload(name)
    }

    /// Attach a script to an open port by name.
    pub async fn attach_script(&self, port_id: &str, script_name: &str) -> Result<()> {
        let manager = self.script_manager.lock().await;
        let pm = self.port_manager.lock().await;
        pm.attach_script_by_name(port_id, &manager, script_name).await
    }

    /// Detach a script from an open port.
    pub async fn detach_script(&self, port_id: &str) -> Result<()> {
        let pm = self.port_manager.lock().await;
        pm.detach_script(port_id).await
    }

    /// Check if a port has a script attached.
    pub async fn has_script(&self, port_id: &str) -> Result<bool> {
        let pm = self.port_manager.lock().await;
        pm.has_script(port_id).await
    }

    // ── Port operations ────────────────────────────────────────────

    /// List all available serial ports.
    pub async fn list_ports(&self) -> Result<Vec<crate::serial_core::SerialPortInfo>> {
        let pm = self.port_manager.lock().await;
        pm.list_ports()
    }

    /// Open a serial port with default configuration.
    pub async fn open_port(&self, port_name: &str) -> Result<String> {
        let pm = self.port_manager.lock().await;
        pm.open_port(port_name, crate::serial_core::SerialConfig::default()).await
    }

    /// Close a serial port by ID.
    pub async fn close_port(&self, port_id: &str) -> Result<()> {
        let pm = self.port_manager.lock().await;
        pm.close_port(port_id).await
    }
}

impl Clone for CommandService {
    fn clone(&self) -> Self {
        Self {
            port_manager: Arc::clone(&self.port_manager),
            script_manager: Arc::clone(&self.script_manager),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    fn create_service() -> CommandService {
        let pm = Arc::new(Mutex::new(PortManager::new()));
        let sm = Arc::new(Mutex::new(ScriptManager::new()));
        CommandService::new(pm, sm)
    }

    #[tokio::test]
    async fn test_list_scripts_includes_built_ins() {
        let service = create_service();
        let scripts = service.list_scripts().await;

        assert!(!scripts.is_empty());
        let names: Vec<&str> = scripts.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"line"));
        assert!(names.contains(&"at_command"));
        assert!(names.contains(&"modbus_rtu"));
    }

    #[tokio::test]
    async fn test_load_and_unload_custom_script() {
        let dir = std::env::temp_dir().join("serial_cli_cmd_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("cmd_test.lua");
        std::fs::write(&path, "function on_recv(data) return data end").unwrap();

        let service = create_service();

        // Load
        let info = service.load_script(&path).await.unwrap();
        assert_eq!(info.name, "cmd_test");
        assert!(!info.built_in);

        // Should appear in list
        let scripts = service.list_scripts().await;
        let names: Vec<&str> = scripts.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"cmd_test"));

        // Unload
        service.unload_script("cmd_test").await.unwrap();
        let scripts = service.list_scripts().await;
        let names: Vec<&str> = scripts.iter().map(|s| s.name.as_str()).collect();
        assert!(!names.contains(&"cmd_test"));

        // Cleanup
        std::fs::remove_file(&path).ok();
        std::fs::remove_dir(&dir).ok();
    }

    #[tokio::test]
    async fn test_cannot_unload_built_in() {
        let service = create_service();
        let result = service.unload_script("line").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_clone_shares_state() {
        let service1 = create_service();
        let service2 = service1.clone();

        // Both should see the same built-in scripts
        let scripts1 = service1.list_scripts().await;
        let scripts2 = service2.list_scripts().await;
        assert_eq!(scripts1.len(), scripts2.len());
    }
}
