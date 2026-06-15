//! Unified script system
//!
//! Manages Lua scripts that define port lifecycle callbacks.
//! Replaces the former `protocol/` module for custom protocol handling.

pub mod built_in;
pub mod manager;

pub use manager::ScriptManager;

/// Metadata about a registered script.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ScriptInfo {
    /// Script name (e.g., `"modbus_rtu"`, `"my_custom"`).
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Whether this is a built-in script.
    pub built_in: bool,
}
