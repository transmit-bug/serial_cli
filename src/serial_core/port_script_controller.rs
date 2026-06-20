//! Port-Script binding controller
//!
//! This module provides a high-level abstraction for managing the lifecycle
//! of a script attached to a serial port. It encapsulates the complexity of
//! callback registration, timer management, and state transitions.
//!
//! # Overview
//!
//! The `PortScriptController` manages:
//! - Script engine creation and loading
//! - Send callback registration (wired to the port's write method)
//! - Timer lifecycle (if the script defines `on_timer`)
//! - State tracking (attached/detached)
//!
//! # Usage
//!
//! ```ignore
//! let controller = PortScriptController::new();
//! controller.attach_script(&mut port_handle, script_engine)?;
//! // ... use port ...
//! controller.detach_script(&mut port_handle);
//! ```

use crate::error::{Result, SerialError};
use crate::serial_core::serial_script::SerialScriptEngine;
use std::sync::Arc;

/// Controller for managing script attachment to a serial port.
///
/// This type encapsulates the lifecycle of a script attached to a port,
/// including callback registration, timer management, and state transitions.
pub struct PortScriptController {
    /// The attached script engine (if any)
    engine: Option<SerialScriptEngine>,
    /// Whether the script has been loaded
    loaded: bool,
    /// Timer interval in milliseconds (0 = disabled)
    timer_interval_ms: u64,
}

impl PortScriptController {
    /// Create a new controller with no script attached.
    pub fn new() -> Self {
        Self {
            engine: None,
            loaded: false,
            timer_interval_ms: 0,
        }
    }

    /// Attach a script engine to this controller.
    ///
    /// This method:
    /// 1. Stores the script engine
    /// 2. Marks the script as not yet loaded
    ///
    /// # Errors
    ///
    /// Returns an error if a script is already attached.
    pub fn attach_engine(&mut self, engine: SerialScriptEngine) -> Result<()> {
        if self.engine.is_some() {
            return Err(SerialError::Script(crate::error::ScriptError::ApiError(
                "A script is already attached".to_string(),
            )));
        }

        self.timer_interval_ms = engine.timer_interval_ms();
        self.engine = Some(engine);
        self.loaded = false;

        Ok(())
    }

    /// Load the script and initialize callbacks.
    ///
    /// This method:
    /// 1. Calls `engine.load()` to compile and execute the script
    /// 2. Registers the send callback (if provided)
    /// 3. Calls `on_open` hook
    /// 4. Marks the script as loaded
    ///
    /// # Arguments
    ///
    /// * `port_name` - Name of the port (passed to `on_open`)
    /// * `config` - Serial port configuration (passed to `on_open`)
    /// * `send_callback` - Optional callback for sending data (wired to `serial_send`)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No script is attached
    /// - Script is already loaded
    /// - Script loading fails
    pub fn load_script(
        &mut self,
        port_name: &str,
        config: &crate::serial_core::SerialConfig,
        send_callback: Option<Arc<dyn Fn(&[u8]) -> Result<usize> + Send + Sync>>,
    ) -> Result<()> {
        let engine = self.engine.as_mut().ok_or_else(|| {
            SerialError::Script(crate::error::ScriptError::ApiError(
                "No script attached".to_string(),
            ))
        })?;

        if self.loaded {
            return Err(SerialError::Script(crate::error::ScriptError::ApiError(
                "Script already loaded".to_string(),
            )));
        }

        // Register send callback if provided
        if let Some(callback) = send_callback {
            engine.set_send_callback(callback)?;
        }

        // Load the script
        engine.load()?;

        // Call on_open hook
        engine.on_open(port_name, config);

        self.loaded = true;

        Ok(())
    }

    /// Detach the script and clean up resources.
    ///
    /// This method:
    /// 1. Calls `on_close` hook (if script was loaded)
    /// 2. Stops the timer (if running)
    /// 3. Removes the script engine
    ///
    /// This is a no-op if no script is attached.
    pub fn detach_script(&mut self) {
        if let Some(mut engine) = self.engine.take() {
            if self.loaded {
                engine.on_close();
            }
            // Timer is stopped automatically when engine is dropped
        }

        self.loaded = false;
        self.timer_interval_ms = 0;
    }

    /// Check if a script is attached.
    pub fn is_attached(&self) -> bool {
        self.engine.is_some()
    }

    /// Check if the script is loaded and ready.
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    /// Get the timer interval in milliseconds (0 = disabled).
    pub fn timer_interval_ms(&self) -> u64 {
        self.timer_interval_ms
    }

    /// Process incoming data through the script's `on_recv` callback.
    ///
    /// Returns the processed data, or the original data if no script is attached
    /// or the callback is not defined.
    pub fn on_recv(&self, data: &[u8]) -> Vec<u8> {
        match &self.engine {
            Some(engine) if self.loaded => engine.on_recv(data),
            _ => data.to_vec(),
        }
    }

    /// Process outgoing data through the script's `on_send` callback.
    ///
    /// Returns the processed data, or an error if the callback fails.
    /// If no script is attached, returns the original data unchanged.
    pub fn on_send(&self, data: &[u8]) -> Result<Vec<u8>> {
        match &self.engine {
            Some(engine) if self.loaded => engine.on_send(data),
            _ => Ok(data.to_vec()),
        }
    }

    /// Start the script's timer (if defined).
    ///
    /// This is typically called automatically after `load_script`.
    /// The timer will call `on_timer` at the specified interval.
    pub fn start_timer(&mut self) {
        if let Some(engine) = self.engine.as_mut() {
            if self.timer_interval_ms > 0 {
                engine.start_timer();
            }
        }
    }

    /// Stop the script's timer (if running).
    ///
    /// This is called automatically when the script is detached.
    pub fn stop_timer(&mut self) {
        if let Some(engine) = self.engine.as_mut() {
            engine.stop_timer();
        }
    }
}

impl Default for PortScriptController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_controller_creation() {
        let controller = PortScriptController::new();
        assert!(!controller.is_attached());
        assert!(!controller.is_loaded());
        assert_eq!(controller.timer_interval_ms(), 0);
    }

    #[test]
    fn test_attach_engine() {
        let mut controller = PortScriptController::new();
        let engine = SerialScriptEngine::new("function on_recv(d) return d end").unwrap();

        assert!(controller.attach_engine(engine).is_ok());
        assert!(controller.is_attached());
        assert!(!controller.is_loaded());
    }

    #[test]
    fn test_attach_engine_already_attached() {
        let mut controller = PortScriptController::new();
        let engine1 = SerialScriptEngine::new("function on_recv(d) return d end").unwrap();
        let engine2 = SerialScriptEngine::new("function on_recv(d) return d end").unwrap();

        controller.attach_engine(engine1).unwrap();
        let result = controller.attach_engine(engine2);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("already attached"));
    }

    #[test]
    fn test_detach_script() {
        let mut controller = PortScriptController::new();
        let engine = SerialScriptEngine::new("function on_recv(d) return d end").unwrap();

        controller.attach_engine(engine).unwrap();
        assert!(controller.is_attached());

        controller.detach_script();
        assert!(!controller.is_attached());
    }

    #[test]
    fn test_on_recv_no_script() {
        let controller = PortScriptController::new();
        let data = b"hello";

        let result = controller.on_recv(data);
        assert_eq!(result, data);
    }

    #[test]
    fn test_on_send_no_script() {
        let controller = PortScriptController::new();
        let data = b"hello";

        let result = controller.on_send(data).unwrap();
        assert_eq!(result, data);
    }
}
