//! Unified script system
//!
//! Manages Lua scripts that define port lifecycle callbacks.
//! Replaces the former `protocol/` module for custom protocol handling.

pub mod built_in;
pub mod manager;

pub use manager::{ScriptManager, ScriptStatistics};

/// Optional metadata embedded in a Lua script via the `SCRIPT_META` table.
///
/// Scripts declare this as a global Lua table:
/// ```lua
/// SCRIPT_META = {
///     name = "my_protocol",
///     version = "1.0.0",
///     description = "My custom protocol",
///     data_format = "binary",
/// }
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScriptMeta {
    /// Unique identifier (e.g., `"modbus_rtu"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Semver version string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Human-readable description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Author name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// License identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    /// Homepage URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    /// Tags for search/categorization.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// Data format: `"binary"` or `"text"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_format: Option<String>,
    /// Minimum complete frame size in bytes (for binary protocols).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_frame_size: Option<u64>,
}

/// Metadata about a registered script.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ScriptInfo {
    /// Script name (e.g., `"modbus_rtu"`, `"my_custom"`).
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Whether this is a built-in script.
    pub built_in: bool,
    /// Optional embedded metadata from `SCRIPT_META`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<ScriptMeta>,
}
