//! Server mode - persistent daemon for AI/automation workflows
//!
//! Provides a long-running server process with JSON-RPC 2.0 interface
//! for efficient serial port and protocol management.

pub mod listener;
pub mod rpc;
pub mod session;
pub mod state;

pub use session::{ServerSessionManager, ServerSessionMeta};
pub use state::{ConnectionContext, ServerConfig, ServerState};
