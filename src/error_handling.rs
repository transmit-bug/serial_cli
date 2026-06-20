//! Enhanced error handling and diagnostics
//!
//! This module provides enhanced error handling capabilities with better
//! diagnostics and user-friendly error messages.

use crate::error::{Result, SerialError};
use std::time::{SystemTime, UNIX_EPOCH};

/// Error context with additional information
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Error message
    pub message: String,
    /// Error code
    pub code: ErrorCode,
    /// Timestamp
    pub timestamp: u64,
    /// Additional context
    pub context: Vec<String>,
    /// Suggestions for resolution
    pub suggestions: Vec<String>,
}

/// Error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    /// Permission denied
    PermissionDenied,
    /// Port not found
    PortNotFound,
    /// Port busy
    PortBusy,
    /// Timeout
    Timeout,
    /// Invalid configuration
    InvalidConfig,
    /// Script error
    ScriptError,
    /// IO error
    IoError,
    /// Unknown error
    Unknown,
}

impl ErrorContext {
    /// Create new error context
    pub fn new(message: String, code: ErrorCode) -> Self {
        Self {
            message,
            code,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            context: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    /// Add context information
    pub fn add_context(mut self, context: String) -> Self {
        self.context.push(context);
        self
    }

    /// Add suggestion
    pub fn add_suggestion(mut self, suggestion: String) -> Self {
        self.suggestions.push(suggestion);
        self
    }

    /// Format error for display
    pub fn format(&self) -> String {
        let mut output = String::new();

        // Error header
        output.push_str(&format!("❌ Error: {}\n", self.message));

        // Error code
        output.push_str(&format!("   Code: {:?}\n", self.code));

        // Context
        if !self.context.is_empty() {
            output.push_str("   Context:\n");
            for ctx in &self.context {
                output.push_str(&format!("     - {}\n", ctx));
            }
        }

        // Suggestions
        if !self.suggestions.is_empty() {
            output.push_str("   Suggestions:\n");
            for suggestion in &self.suggestions {
                output.push_str(&format!("     💡 {}\n", suggestion));
            }
        }

        output
    }

    /// Create user-friendly error description
    pub fn description(&self) -> String {
        match self.code {
            ErrorCode::PermissionDenied => {
                format!("Permission denied: {}. Try running with elevated privileges or check file permissions.", self.message)
            }
            ErrorCode::PortNotFound => {
                format!("Port not found: {}. Check if the device is connected and the port name is correct.", self.message)
            }
            ErrorCode::PortBusy => {
                format!("Port busy: {}. Another application may be using this port. Close other applications and try again.", self.message)
            }
            ErrorCode::Timeout => {
                format!("Operation timeout: {}. The operation took too long to complete. Try increasing the timeout value.", self.message)
            }
            ErrorCode::InvalidConfig => {
                format!(
                    "Invalid configuration: {}. Check your configuration file and try again.",
                    self.message
                )
            }
            ErrorCode::ScriptError => {
                format!(
                    "Script error: {}. Check your Lua script for syntax or runtime errors.",
                    self.message
                )
            }
            ErrorCode::IoError => {
                format!(
                    "I/O error: {}. A system-level error occurred.",
                    self.message
                )
            }
            ErrorCode::Unknown => {
                format!(
                    "Unknown error: {}. An unexpected error occurred.",
                    self.message
                )
            }
        }
    }
}

/// Enhanced error handler
pub struct ErrorHandler {
    /// Verbose mode
    verbose: bool,
}

impl ErrorHandler {
    /// Create new error handler
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    /// Handle error with enhanced formatting
    pub fn handle_error(&self, error: &SerialError) -> ErrorContext {
        let (message, code) = self.classify_error(error);

        let mut context = ErrorContext::new(message, code);

        // Add standard context
        if self.verbose {
            context = context.add_context(format!("Error details: {:?}", error));
        }

        // Add suggestions based on error type
        context = self.add_suggestions(context, code);

        context
    }

    /// Classify error and get code
    fn classify_error(&self, error: &SerialError) -> (String, ErrorCode) {
        match error {
            SerialError::Serial(ref port_error) => {
                let error_str = port_error.to_string().to_lowercase();
                if error_str.contains("permission denied") || error_str.contains("access denied") {
                    (
                        format!("Permission denied: {}", port_error),
                        ErrorCode::PermissionDenied,
                    )
                } else if error_str.contains("not found") || error_str.contains("no such file") {
                    (
                        format!("Port not found: {}", port_error),
                        ErrorCode::PortNotFound,
                    )
                } else if error_str.contains("busy") || error_str.contains("in use") {
                    (format!("Port busy: {}", port_error), ErrorCode::PortBusy)
                } else {
                    (format!("Serial error: {}", port_error), ErrorCode::IoError)
                }
            }
            SerialError::Io(ref io_error) => {
                let error_str = io_error.to_string().to_lowercase();
                if error_str.contains("timeout") {
                    (
                        format!("Operation timeout: {}", io_error),
                        ErrorCode::Timeout,
                    )
                } else {
                    (format!("I/O error: {}", io_error), ErrorCode::IoError)
                }
            }
            SerialError::Config(ref config_error) => (
                format!("Configuration error: {}", config_error),
                ErrorCode::InvalidConfig,
            ),
            SerialError::Script(ref script_error) => (
                format!("Script error: {}", script_error),
                ErrorCode::ScriptError,
            ),
            _ => (format!("Error: {}", error), ErrorCode::Unknown),
        }
    }

    /// Add suggestions based on error code
    fn add_suggestions(&self, mut context: ErrorContext, code: ErrorCode) -> ErrorContext {
        match code {
            ErrorCode::PermissionDenied => {
                context = context.add_suggestion(
                    "Try running the command with sudo (Linux/macOS) or as Administrator (Windows)"
                        .to_string(),
                );
                context = context.add_suggestion(
                    "Check if your user has proper permissions for the serial port device"
                        .to_string(),
                );
                context = context.add_suggestion(
                    "On Linux, you may need to add your user to the 'dialout' or 'uucp' group"
                        .to_string(),
                );
            }
            ErrorCode::PortNotFound => {
                context =
                    context.add_suggestion("Check if the device is properly connected".to_string());
                context = context.add_suggestion(
                    "Verify the port name (e.g., COM1, /dev/ttyUSB0, /dev/tty.usbserial-*)"
                        .to_string(),
                );
                context = context.add_suggestion(
                    "Try running 'serial-cli list-ports' to see available ports".to_string(),
                );
            }
            ErrorCode::PortBusy => {
                context = context.add_suggestion(
                    "Close other applications that may be using this port".to_string(),
                );
                context = context.add_suggestion(
                    "Check if another instance of serial-cli is running".to_string(),
                );
                context = context.add_suggestion(
                    "On Linux, you can use 'lsof | grep tty' to find processes using the port"
                        .to_string(),
                );
            }
            ErrorCode::Timeout => {
                context = context
                    .add_suggestion("Increase the timeout value in configuration".to_string());
                context = context
                    .add_suggestion("Check if the device is responding correctly".to_string());
                context = context.add_suggestion(
                    "Verify the baud rate and other serial settings match the device".to_string(),
                );
            }
            ErrorCode::InvalidConfig => {
                context = context.add_suggestion(
                    "Check your configuration file (.serial-cli.toml) for syntax errors"
                        .to_string(),
                );
                context = context.add_suggestion(
                    "Try running 'serial-cli config reset' to restore default configuration"
                        .to_string(),
                );
                context = context.add_suggestion(
                    "Validate configuration values (e.g., baud rate must be a valid number)"
                        .to_string(),
                );
            }
            _ => {
                context = context
                    .add_suggestion("Check the error message for specific details".to_string());
                context = context.add_suggestion(
                    "Try running with --verbose flag for more information".to_string(),
                );
            }
        }

        context
    }
}

impl Default for ErrorHandler {
    fn default() -> Self {
        Self::new(false)
    }
}

/// Error recovery strategies
pub enum RecoveryStrategy {
    /// Retry the operation
    Retry { attempts: usize, delay_ms: u64 },
    /// Use fallback value - returns a special fallback error that caller can handle
    Fallback,
    /// Skip and continue - returns a special skip error that caller can handle
    Skip,
    /// Abort operation
    Abort,
}

/// Error recovery handler
pub struct RecoveryHandler {
    strategies: Vec<(ErrorCode, RecoveryStrategy)>,
}

impl RecoveryHandler {
    /// Create new recovery handler
    pub fn new() -> Self {
        Self {
            strategies: Vec::new(),
        }
    }

    /// Add recovery strategy for an error code
    pub fn add_strategy(&mut self, code: ErrorCode, strategy: RecoveryStrategy) -> &mut Self {
        self.strategies.push((code, strategy));
        self
    }

    /// Get recovery strategy for an error
    pub fn get_strategy(&self, code: ErrorCode) -> Option<&RecoveryStrategy> {
        self.strategies
            .iter()
            .find(|(c, _)| *c == code)
            .map(|(_, strategy)| strategy)
    }

    /// Attempt recovery from an error
    pub async fn recover<F, Fut, T>(&self, error: &SerialError, mut operation: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let handler = ErrorHandler::default();
        let (_, code) = handler.classify_error(error);

        if let Some(strategy) = self.get_strategy(code) {
            match strategy {
                RecoveryStrategy::Retry { attempts, delay_ms } => {
                    for i in 0..*attempts {
                        tokio::time::sleep(std::time::Duration::from_millis(*delay_ms)).await;
                        match operation().await {
                            Ok(result) => return Ok(result),
                            Err(e) if i == attempts - 1 => return Err(e),
                            Err(_) => continue,
                        }
                    }
                    // Should never reach here, but to satisfy the compiler
                    Err(SerialError::Serial(crate::error::SerialPortError::IoError(
                        "Recovery failed after all retry attempts".to_string(),
                    )))
                }
                RecoveryStrategy::Fallback => {
                    // Fallback strategy: Return a descriptive error that indicates fallback was used
                    // The caller can choose to handle this appropriately (e.g., use default value)
                    tracing::warn!("Fallback recovery strategy used for error: {:?}", error);
                    Err(SerialError::Serial(crate::error::SerialPortError::IoError(
                        format!("Operation failed - fallback mode activated for: {:?}", code),
                    )))
                }
                RecoveryStrategy::Skip => {
                    // Skip strategy: Return a descriptive error that indicates operation was skipped
                    tracing::info!(
                        "Skip recovery strategy used - operation skipped for error: {:?}",
                        error
                    );
                    Err(SerialError::Serial(crate::error::SerialPortError::IoError(
                        format!("Operation skipped for: {:?}", code),
                    )))
                }
                RecoveryStrategy::Abort => {
                    // Abort strategy: Return a descriptive error
                    tracing::error!(
                        "Abort recovery strategy used - operation aborted for error: {:?}",
                        error
                    );
                    Err(SerialError::Serial(crate::error::SerialPortError::IoError(
                        format!("Operation aborted for: {:?}", code),
                    )))
                }
            }
        } else {
            // No recovery strategy available
            Err(SerialError::Serial(crate::error::SerialPortError::IoError(
                format!("No recovery strategy available for error: {:?}", code),
            )))
        }
    }
}

impl Default for RecoveryHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_context_formatting() {
        let context = ErrorContext::new("Test error".to_string(), ErrorCode::PortNotFound)
            .add_context("Port: /dev/ttyUSB0".to_string())
            .add_suggestion("Check connection".to_string());

        let formatted = context.format();
        assert!(formatted.contains("Test error"));
        assert!(formatted.contains("PortNotFound"));
        assert!(formatted.contains("Check connection"));
    }

    #[test]
    fn test_error_handler_classification() {
        let handler = ErrorHandler::new(false);
        let serial_error = SerialError::Serial(crate::error::SerialPortError::permission_denied(
            "/dev/ttyUSB0",
            None,
        ));

        let context = handler.handle_error(&serial_error);
        assert_eq!(context.code, ErrorCode::PermissionDenied);
    }

    #[test]
    fn test_recovery_handler() {
        let mut recovery = RecoveryHandler::new();
        recovery.add_strategy(
            ErrorCode::Timeout,
            RecoveryStrategy::Retry {
                attempts: 3,
                delay_ms: 100,
            },
        );

        let strategy = recovery.get_strategy(ErrorCode::Timeout);
        assert!(strategy.is_some());

        if let Some(RecoveryStrategy::Retry { attempts, delay_ms }) = strategy {
            assert_eq!(*attempts, 3);
            assert_eq!(*delay_ms, 100);
        } else {
            panic!("Expected Retry strategy");
        }
    }

    #[test]
    fn test_recovery_handler_unknown_error() {
        let recovery = RecoveryHandler::new();
        let strategy = recovery.get_strategy(ErrorCode::PortNotFound);
        assert!(strategy.is_none());
    }

    #[test]
    fn test_recovery_handler_multiple_strategies() {
        let mut recovery = RecoveryHandler::new();
        recovery.add_strategy(
            ErrorCode::Timeout,
            RecoveryStrategy::Retry {
                attempts: 3,
                delay_ms: 100,
            },
        );
        recovery.add_strategy(ErrorCode::PortBusy, RecoveryStrategy::Abort);
        recovery.add_strategy(ErrorCode::PortNotFound, RecoveryStrategy::Skip);

        // Timeout should have Retry
        match recovery.get_strategy(ErrorCode::Timeout).unwrap() {
            RecoveryStrategy::Retry { attempts, delay_ms } => {
                assert_eq!(*attempts, 3);
                assert_eq!(*delay_ms, 100);
            }
            _ => panic!("Expected Retry for Timeout"),
        }

        // PortBusy should have Abort
        assert!(matches!(
            recovery.get_strategy(ErrorCode::PortBusy).unwrap(),
            RecoveryStrategy::Abort
        ));

        // PortNotFound should have Skip
        assert!(matches!(
            recovery.get_strategy(ErrorCode::PortNotFound).unwrap(),
            RecoveryStrategy::Skip
        ));
    }

    #[test]
    fn test_recovery_handler_fallback_strategy() {
        let mut recovery = RecoveryHandler::new();
        recovery.add_strategy(ErrorCode::ScriptError, RecoveryStrategy::Fallback);
        assert!(matches!(
            recovery.get_strategy(ErrorCode::ScriptError).unwrap(),
            RecoveryStrategy::Fallback
        ));
    }
}
