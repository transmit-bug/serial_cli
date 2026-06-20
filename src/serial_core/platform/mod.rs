//! Platform-specific serial port configuration
//!
//! This module abstracts platform-specific serial port setup, particularly
//! the DTR/RTS signal control which differs between Unix and Windows.

use crate::error::{Result, SerialError, SerialPortError};
use crate::serial_core::port::{FlowControl, Parity, SerialConfig};
use crate::serial_core::signals::PlatformSignals;
use std::time::Duration;

/// Result of platform-specific port configuration
pub struct ConfiguredPort {
    /// The configured serial port (boxed trait object)
    pub port: Box<dyn serialport::SerialPort>,
    /// Unix signal controller (only on Unix platforms)
    #[cfg(unix)]
    pub signal_controller: crate::serial_core::signals::UnixSignalController,
}

/// Configure a serial port with platform-specific settings
///
/// This function handles the platform-specific aspects of opening and configuring
/// a serial port, including:
/// - Setting baud rate, data bits, parity, stop bits, and flow control
/// - Configuring DTR/RTS signals (Unix-only via ioctl)
///
/// # Arguments
///
/// * `name` - Port device path (e.g., `/dev/ttyUSB0`, `COM1`)
/// * `config` - Serial port configuration
///
/// # Returns
///
/// A `ConfiguredPort` containing the opened and configured serial port
///
/// # Errors
///
/// Returns an error if the port cannot be opened or configured
pub fn configure_port(name: &str, config: &SerialConfig) -> Result<ConfiguredPort> {
    #[cfg(unix)]
    {
        configure_port_unix(name, config)
    }

    #[cfg(not(unix))]
    {
        configure_port_windows(name, config)
    }
}

/// Build a serial port builder from configuration
fn build_port(name: &str, config: &SerialConfig) -> serialport::SerialPortBuilder {
    serialport::new(name, config.baudrate)
        .timeout(Duration::from_millis(config.timeout_ms))
        .data_bits(match config.databits {
            5 => serialport::DataBits::Five,
            6 => serialport::DataBits::Six,
            7 => serialport::DataBits::Seven,
            8 => serialport::DataBits::Eight,
            _ => serialport::DataBits::Eight,
        })
        .parity(match config.parity {
            Parity::None => serialport::Parity::None,
            Parity::Odd => serialport::Parity::Odd,
            Parity::Even => serialport::Parity::Even,
        })
        .stop_bits(match config.stopbits {
            1 => serialport::StopBits::One,
            2 => serialport::StopBits::Two,
            _ => serialport::StopBits::One,
        })
        .flow_control(match config.flow_control {
            FlowControl::None => serialport::FlowControl::None,
            FlowControl::Software => serialport::FlowControl::Software,
            FlowControl::Hardware => serialport::FlowControl::Hardware,
        })
}

/// Map serialport errors to our error type
fn map_serial_error(e: serialport::Error, port_name: &str) -> SerialError {
    let error_msg = e.to_string();
    if error_msg.contains("permission denied")
        || error_msg.contains("Permission denied")
        || error_msg.contains("Access is denied")
    {
        SerialError::Serial(SerialPortError::permission_denied(
            port_name,
            Some("Try running as Administrator or check port permissions"),
        ))
    } else if error_msg.contains("not found")
        || error_msg.contains("No such file")
        || error_msg.contains("The system cannot find the file")
    {
        SerialError::Serial(SerialPortError::PortNotFound(port_name.to_string()))
    } else if error_msg.contains("busy")
        || error_msg.contains("Busy")
        || error_msg.contains("used by another application")
    {
        SerialError::Serial(SerialPortError::port_busy(
            port_name,
            Some("Port is in use by another application"),
        ))
    } else {
        SerialError::Serial(SerialPortError::IoError(format!(
            "Failed to open port {}: {}",
            port_name, e
        )))
    }
}

// Unix-specific implementation
#[cfg(unix)]
fn configure_port_unix(name: &str, config: &SerialConfig) -> Result<ConfiguredPort> {
    use std::os::unix::io::AsRawFd;

    let builder = build_port(name, config);

    // Open as TTYPort to get raw fd
    let tty = builder
        .open_native()
        .map_err(|e| map_serial_error(e, name))?;
    let fd = tty.as_raw_fd();

    // Set DTR/RTS via real ioctl
    set_dtr_on_fd(fd, config.dtr_enable);
    set_rts_on_fd(fd, config.rts_enable);

    // Create signal controller with correct initial state
    let mut signal_controller = crate::serial_core::signals::UnixSignalController::new();
    signal_controller.set_dtr(config.dtr_enable).ok();
    signal_controller.set_rts(config.rts_enable).ok();

    // Keep TTYPort directly — it implements serialport::SerialPort with blocking I/O
    let port: Box<dyn serialport::SerialPort> = Box::new(tty);

    Ok(ConfiguredPort {
        port,
        signal_controller,
    })
}

/// Set DTR signal on Unix file descriptor
#[cfg(unix)]
fn set_dtr_on_fd(fd: libc::c_int, enable: bool) {
    let result = unsafe {
        crate::serial_core::signals::UnixSignalController::set_modem_bit_on_fd(
            fd,
            libc::TIOCM_DTR,
            enable,
        )
    };
    if let Err(e) = result {
        tracing::warn!("Failed to set DTR on open: {}", e);
    }
}

/// Set RTS signal on Unix file descriptor
#[cfg(unix)]
fn set_rts_on_fd(fd: libc::c_int, enable: bool) {
    let result = unsafe {
        crate::serial_core::signals::UnixSignalController::set_modem_bit_on_fd(
            fd,
            libc::TIOCM_RTS,
            enable,
        )
    };
    if let Err(e) = result {
        tracing::warn!("Failed to set RTS on open: {}", e);
    }
}

// Windows-specific implementation
#[cfg(not(unix))]
fn configure_port_windows(name: &str, config: &SerialConfig) -> Result<ConfiguredPort> {
    let builder = build_port(name, config);

    let port = builder
        .open_native()
        .map_err(|e| map_serial_error(e, name))?;

    Ok(ConfiguredPort {
        port: Box::new(port),
    })
}
