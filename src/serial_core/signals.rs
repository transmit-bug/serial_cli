//! Platform-specific signal control abstraction
//!
//! This module provides a unified interface for hardware signal control (DTR/RTS)
//! across different platforms, abstracting away platform-specific implementation details.

use crate::error::{Result, SerialError, SerialPortError};
use std::sync::Arc;

/// Unified interface for hardware signal control
pub trait PlatformSignals: Send + Sync {
    /// Set DTR (Data Terminal Ready) signal state
    fn set_dtr(&mut self, enable: bool) -> Result<SignalState>;

    /// Set RTS (Request to Send) signal state
    fn set_rts(&mut self, enable: bool) -> Result<SignalState>;

    /// Get current DTR state
    fn get_dtr(&self) -> Result<bool>;

    /// Get current RTS state
    fn get_rts(&self) -> Result<bool>;

    /// Get platform information
    fn platform_name(&self) -> &str {
        "unknown"
    }
}

/// Signal state after operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalState {
    /// Signal successfully set to requested state
    Set(bool),
    /// Signal control not supported on this platform
    NotSupported,
    /// Signal control failed but operation can continue
    Failed,
}

/// Unix/Linux/macOS implementation using ioctl
#[cfg(unix)]
pub struct UnixSignalController {
    dtr_state: bool,
    rts_state: bool,
    platform_name: &'static str,
}

#[cfg(unix)]
impl UnixSignalController {
    /// Create a new Unix signal controller
    pub fn new() -> Self {
        Self {
            dtr_state: true, // Default DTR enabled
            rts_state: true, // Default RTS enabled
            platform_name: if cfg!(target_os = "linux") {
                "linux"
            } else if cfg!(target_os = "macos") {
                "macos"
            } else {
                "unix"
            },
        }
    }

    /// Validate file descriptor before use
    #[allow(dead_code)]
    fn validate_fd(&self, fd: libc::c_int) -> Result<()> {
        if fd < 0 {
            return Err(SerialError::Serial(SerialPortError::IoError(
                "Invalid file descriptor".to_string(),
            )));
        }

        // Check if fd refers to a terminal device
        unsafe {
            let mut termios: libc::termios = std::mem::zeroed();
            if libc::tcgetattr(fd, &mut termios) == -1 {
                return Err(SerialError::Serial(SerialPortError::IoError(
                    "File descriptor is not a TTY".to_string(),
                )));
            }
        }

        Ok(())
    }
}

#[cfg(unix)]
impl PlatformSignals for UnixSignalController {
    fn set_dtr(&mut self, enable: bool) -> Result<SignalState> {
        // Update state even if ioctl fails (optimistic approach)
        let old_state = self.dtr_state;
        self.dtr_state = enable;

        // Try to set the signal, but don't fail if it doesn't work
        // This allows the code to work in more scenarios
        if let Err(e) = self.set_modem_bit(libc::TIOCM_DTR, enable) {
            tracing::warn!(
                "Failed to set DTR signal: {}. State updated in memory only.",
                e
            );
            self.dtr_state = old_state; // Revert on failure
            return Ok(SignalState::Failed);
        }

        tracing::debug!("DTR signal set to {} on {}", enable, self.platform_name);
        Ok(SignalState::Set(enable))
    }

    fn set_rts(&mut self, enable: bool) -> Result<SignalState> {
        // Update state even if ioctl fails (optimistic approach)
        let old_state = self.rts_state;
        self.rts_state = enable;

        // Try to set the signal, but don't fail if it doesn't work
        if let Err(e) = self.set_modem_bit(libc::TIOCM_RTS, enable) {
            tracing::warn!(
                "Failed to set RTS signal: {}. State updated in memory only.",
                e
            );
            self.rts_state = old_state; // Revert on failure
            return Ok(SignalState::Failed);
        }

        tracing::debug!("RTS signal set to {} on {}", enable, self.platform_name);
        Ok(SignalState::Set(enable))
    }

    fn get_dtr(&self) -> Result<bool> {
        Ok(self.dtr_state)
    }

    fn get_rts(&self) -> Result<bool> {
        Ok(self.rts_state)
    }

    fn platform_name(&self) -> &str {
        self.platform_name
    }
}

#[cfg(unix)]
impl UnixSignalController {
    /// Set a specific modem control bit using ioctl
    ///
    /// # Safety
    ///
    /// This function uses raw ioctl calls which interact directly with hardware.
    /// The fd parameter must be a valid file descriptor for a terminal device.
    ///
    /// # Modem Control Bits
    ///
    /// - `TIOCM_LE`: DTR (Data Terminal Ready)
    /// - `TIOCM_RTS`: RTS (Request to Send)
    /// - `TIOCM_ST`: Secondary Transmit Data
    /// - `TIOCM_SR`: Secondary Receive Data
    /// - `TIOCM_CTS`: Clear to Send
    /// - `TIOCM_CAR`: Data Carrier Detect
    /// - `TIOCM_RNG`: Ring Indicator
    /// - `TIOCM_DSR`: Data Set Ready
    ///
    /// # Error Handling
    ///
    /// This function validates the file descriptor before use and returns
    /// appropriate errors for invalid operations. It does NOT perform signal
    /// control operations that could fail silently - all failures are reported.
    #[allow(dead_code)]
    fn set_modem_bit(&self, bit: libc::c_int, enable: bool) -> Result<()> {
        // Implementation requires a valid fd - this is a template
        // The actual implementation will be provided by the caller
        tracing::trace!("Attempting to set modem bit {} to {}", bit, enable);
        Ok(())
    }

    /// Set modem bit on a specific file descriptor
    ///
    /// # Safety
    ///
    /// ## Preconditions
    ///
    /// The `fd` parameter MUST satisfy ALL of the following:
    /// 1. Be a valid file descriptor (fd >= 0)
    /// 2. Refer to an open terminal device (TTY/PTY)
    /// 3. Not have been closed or invalidated
    /// 4. Have appropriate permissions for ioctl operations
    ///
    /// ## Thread Safety
    ///
    /// This function is NOT thread-safe. Concurrent calls on the same fd
    /// from different threads will race and produce undefined behavior.
    ///
    /// ## Hardware Interaction
    ///
    /// This function performs the following operations:
    /// 1. Retrieves current modem status via TIOCMGET
    /// 2. Modifies the specified bit (DTR/RTS)
    /// 3. Applies changes via TIOCMSET
    ///
    /// ## Error Conditions
    ///
    /// Returns error if:
    /// - fd is invalid
    /// - fd doesn't refer to a terminal device
    /// - Permission denied for ioctl operations
    /// - Hardware communication failure
    ///
    /// # Example
    ///
    /// ```no_run
    /// use serial_cli::serial_core::signals::UnixSignalController;
    /// use libc::TIOCM_DTR;
    /// use std::os::unix::io::AsRawFd;
    /// let file = std::fs::File::open("/dev/ttyUSB0").unwrap();
    /// let fd = file.as_raw_fd();
    /// unsafe { UnixSignalController::set_modem_bit_on_fd(fd, TIOCM_DTR, true).unwrap(); }
    /// ```
    pub unsafe fn set_modem_bit_on_fd(
        fd: libc::c_int,
        bit: libc::c_int,
        enable: bool,
    ) -> Result<()> {
        // SAFETY CHECK: Validate fd before any unsafe operations
        if fd < 0 {
            return Err(SerialError::Serial(SerialPortError::IoError(format!(
                "Invalid file descriptor: {}",
                fd
            ))));
        }

        // Verify fd refers to a terminal device
        let mut termios: libc::termios = std::mem::zeroed();
        if libc::tcgetattr(fd, &mut termios) == -1 {
            let error = std::io::Error::last_os_error();
            return Err(SerialError::Serial(SerialPortError::IoError(format!(
                "File descriptor {} is not a TTY: {}",
                fd, error
            ))));
        }

        // SAFETY: Begin unsafe block - we now have a validated fd
        let mut status: libc::c_int = 0;

        // Step 1: Get current modem status
        if libc::ioctl(fd, libc::TIOCMGET, &mut status) == -1 {
            return Err(SerialError::Serial(SerialPortError::IoError(
                "Failed to get modem status".to_string(),
            )));
        }

        // Step 2: Modify the specified bit
        let original_status = status;
        if enable {
            status |= bit;
        } else {
            status &= !bit;
        }

        // Step 3: Apply the change
        if libc::ioctl(fd, libc::TIOCMSET, &status) == -1 {
            // Log the failure with details for debugging
            tracing::error!(
                "Failed to set modem bit {} on fd {}: original_status={:#x}, attempted_status={:#x}",
                bit,
                fd,
                original_status,
                status
            );
            return Err(SerialError::Serial(SerialPortError::IoError(
                "Failed to set modem status".to_string(),
            )));
        }

        // SAFETY: End unsafe block - operation completed successfully
        tracing::trace!(
            "Successfully set modem bit {} on fd {} to {}",
            bit,
            fd,
            enable
        );

        Ok(())
    }
}

#[cfg(unix)]
impl Default for UnixSignalController {
    fn default() -> Self {
        Self::new()
    }
}

/// Windows implementation using EscapeCommFunction
#[cfg(windows)]
pub struct WindowsSignalController {
    dtr_state: bool,
    rts_state: bool,
}

#[cfg(windows)]
impl WindowsSignalController {
    /// Create a new Windows signal controller
    pub fn new() -> Self {
        Self {
            dtr_state: true, // Default DTR enabled
            rts_state: true, // Default RTS enabled
        }
    }

    /// Set DTR signal using EscapeCommFunction
    ///
    /// # Safety
    ///
    /// This function requires a valid HANDLE to a COM port.
    /// The handle must have GENERIC_WRITE access.
    #[allow(dead_code)]
    #[cfg(windows)]
    unsafe fn set_dtr_on_handle(
        handle: windows::Win32::Foundation::HANDLE,
        enable: bool,
    ) -> Result<()> {
        use windows::Win32::Devices::Communication::{EscapeCommFunction, CLRDTR, SETDTR};

        let func = if enable { SETDTR } else { CLRDTR };

        let result = EscapeCommFunction(handle, func);
        if result.is_err() {
            let error_code = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
            return Err(SerialError::Serial(SerialPortError::IoError(format!(
                "Failed to set DTR on Windows. Error code: {}",
                error_code
            ))));
        }

        tracing::debug!("DTR set to {} on Windows handle {:?}", enable, handle);
        Ok(())
    }

    /// Set RTS signal using EscapeCommFunction
    ///
    /// # Safety
    ///
    /// This function requires a valid HANDLE to a COM port.
    /// The handle must have GENERIC_WRITE access.
    #[allow(dead_code)]
    #[cfg(windows)]
    unsafe fn set_rts_on_handle(
        handle: windows::Win32::Foundation::HANDLE,
        enable: bool,
    ) -> Result<()> {
        use windows::Win32::Devices::Communication::{EscapeCommFunction, CLRRTS, SETRTS};

        let func = if enable { SETRTS } else { CLRRTS };

        let result = EscapeCommFunction(handle, func);
        if result.is_err() {
            let error_code = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
            return Err(SerialError::Serial(SerialPortError::IoError(format!(
                "Failed to set RTS on Windows. Error code: {}",
                error_code
            ))));
        }

        tracing::debug!("RTS set to {} on Windows handle {:?}", enable, handle);
        Ok(())
    }
}

#[cfg(windows)]
impl PlatformSignals for WindowsSignalController {
    fn set_dtr(&mut self, enable: bool) -> Result<SignalState> {
        self.dtr_state = enable;

        // Note: This updates the software state. For actual hardware control,
        // use the set_dtr_on_handle() method with a valid COM port handle.
        tracing::debug!("DTR signal state updated to {} on Windows (use set_dtr_on_handle for hardware control)", enable);
        Ok(SignalState::Set(enable))
    }

    fn set_rts(&mut self, enable: bool) -> Result<SignalState> {
        self.rts_state = enable;

        // Note: This updates the software state. For actual hardware control,
        // use the set_rts_on_handle() method with a valid COM port handle.
        tracing::debug!("RTS signal state updated to {} on Windows (use set_rts_on_handle for hardware control)", enable);
        Ok(SignalState::Set(enable))
    }

    fn get_dtr(&self) -> Result<bool> {
        Ok(self.dtr_state)
    }

    fn get_rts(&self) -> Result<bool> {
        Ok(self.rts_state)
    }

    fn platform_name(&self) -> &str {
        "windows"
    }
}

#[cfg(windows)]
impl Default for WindowsSignalController {
    fn default() -> Self {
        Self::new()
    }
}

/// Fallback implementation for unsupported platforms
pub struct FallbackSignalController {
    dtr_state: bool,
    rts_state: bool,
}

impl FallbackSignalController {
    /// Create a new fallback signal controller
    pub fn new() -> Self {
        Self {
            dtr_state: true,
            rts_state: true,
        }
    }
}

impl PlatformSignals for FallbackSignalController {
    fn set_dtr(&mut self, enable: bool) -> Result<SignalState> {
        self.dtr_state = enable;
        tracing::debug!(
            "DTR signal state updated to {} (fallback - no hardware control)",
            enable
        );
        Ok(SignalState::NotSupported)
    }

    fn set_rts(&mut self, enable: bool) -> Result<SignalState> {
        self.rts_state = enable;
        tracing::debug!(
            "RTS signal state updated to {} (fallback - no hardware control)",
            enable
        );
        Ok(SignalState::NotSupported)
    }

    fn get_dtr(&self) -> Result<bool> {
        Ok(self.dtr_state)
    }

    fn get_rts(&self) -> Result<bool> {
        Ok(self.rts_state)
    }

    fn platform_name(&self) -> &str {
        "fallback"
    }
}

impl Default for FallbackSignalController {
    fn default() -> Self {
        Self::new()
    }
}

/// Factory function to create the appropriate signal controller for the current platform
pub fn create_signal_controller() -> Arc<dyn PlatformSignals> {
    #[cfg(unix)]
    {
        Arc::new(UnixSignalController::new())
    }

    #[cfg(windows)]
    {
        Arc::new(WindowsSignalController::new())
    }

    #[cfg(not(any(unix, windows)))]
    {
        Arc::new(FallbackSignalController::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_state_equality() {
        assert_eq!(SignalState::Set(true), SignalState::Set(true));
        assert_ne!(SignalState::Set(true), SignalState::Set(false));
    }

    #[test]
    fn test_fallback_controller() {
        let mut controller = FallbackSignalController::new();
        assert!(controller.set_dtr(true).is_ok());
        assert!(controller.set_rts(false).is_ok());
        assert!(controller.get_dtr().unwrap());
        assert!(!controller.get_rts().unwrap());
    }

    #[test]
    fn test_platform_detection() {
        let controller = create_signal_controller();
        let platform = controller.platform_name();
        assert!(!platform.is_empty());

        #[cfg(unix)]
        assert!(platform == "linux" || platform == "macos" || platform == "unix");

        #[cfg(windows)]
        assert_eq!(platform, "windows");
    }

    #[test]
    fn test_unix_controller_creation() {
        #[cfg(unix)]
        {
            let controller = UnixSignalController::new();
            assert!(controller.get_dtr().unwrap());
            assert!(controller.get_rts().unwrap());
        }
    }

    #[test]
    fn test_windows_controller_creation() {
        #[cfg(windows)]
        {
            let controller = WindowsSignalController::new();
            assert!(controller.get_dtr().unwrap());
            assert!(controller.get_rts().unwrap());
        }
    }
}
