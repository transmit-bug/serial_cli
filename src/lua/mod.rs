//! Lua runtime module
//!
//! This module provides Lua scripting integration.

pub mod bindings;
pub(crate) mod stdlib; // Used by CLI — will be unified into ScriptRuntime in Phase 2
pub mod engine;
pub mod executor;
pub mod runtime;

pub use bindings::LuaBindings;
pub use engine::LuaEngine;
pub use executor::ScriptEngine;
pub use runtime::ScriptRuntime;
