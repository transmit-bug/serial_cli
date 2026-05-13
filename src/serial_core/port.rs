//! Serial port management
//!
//! This module provides serial port discovery, configuration, and management.
//!
//! # Key types
//!
//! - [`PortManager`] — thread-safe manager that tracks open ports
//! - [`SerialConfig`] — port settings (baud rate, data bits, parity, etc.)
//! - [`SerialPortHandle`] — RAII handle for an open port with read/write access
//! - [`SerialPortInfo`] — metadata about an enumerated port

use crate::error::{Result, SerialError, SerialPortError};
#[cfg(unix)]
use crate::serial_core::signals::PlatformSignals;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

/// Thread-safe manager for discovering, opening, and tracking serial ports.
///
/// All operations are `async`-safe internally via `tokio::Mutex`.
/// The manager maintains a registry of open ports keyed by a unique ID
/// (port name + UUID). When IoLoop mode is enabled, a background task
/// is spawned for each opened port to continuously read incoming data.
#[derive(Clone)]
pub struct PortManager {
    ports: Arc<Mutex<HashMap<String, Arc<Mutex<SerialPortHandle>>>>>,
    io_loop_enabled: Arc<Mutex<bool>>,
}

impl Default for PortManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PortManager {
    /// Create a new port manager with IoLoop disabled by default.
    pub fn new() -> Self {
        Self {
            ports: Arc::new(Mutex::new(HashMap::new())),
            io_loop_enabled: Arc::new(Mutex::new(false)),
        }
    }

    /// Create a new port manager with IoLoop enabled.
    ///
    /// When IoLoop is active, every port opened via [`open_port`](Self::open_port)
    /// will have a background task that continuously reads incoming data.
    pub fn with_ioloop() -> Self {
        Self {
            ports: Arc::new(Mutex::new(HashMap::new())),
            io_loop_enabled: Arc::new(Mutex::new(true)),
        }
    }

    /// Enable or disable IoLoop mode.
    ///
    /// Changes affect only ports opened after this call. Already-open
    /// ports retain their original IoLoop state.
    pub async fn set_ioloop_enabled(&self, enabled: bool) {
        let mut ioloop = self.io_loop_enabled.lock().await;
        *ioloop = enabled;
    }

    /// Check if IoLoop mode is currently enabled.
    pub async fn is_ioloop_enabled(&self) -> bool {
        *self.io_loop_enabled.lock().await
    }
}

/// RAII handle for an open serial port.
///
/// Provides read/write access, signal control (DTR/RTS), and optional
/// protocol association. The underlying OS file descriptor is held by
/// the boxed [`serialport::SerialPort`] trait object.
///
/// # Platform notes
///
/// On Unix, DTR/RTS signals are controlled via `ioctl` on the raw file
/// descriptor. On Windows, signal control is not available and methods
/// return [`SignalState::NotSupported`](crate::serial_core::signals::SignalState::NotSupported).
pub struct SerialPortHandle {
    name: String,
    port: Box<dyn serialport::SerialPort>,
    config: SerialConfig,
    protocol: Option<String>,
    dtr_state: bool,
    rts_state: bool,
    #[cfg(unix)]
    signal_controller: crate::serial_core::signals::UnixSignalController,
}

/// Serial port configuration parameters.
///
/// All fields map directly to standard RS-232 settings.
/// Defaults to 115200 baud, 8 data bits, 1 stop bit, no parity, no flow control.
#[derive(Debug, Clone)]
pub struct SerialConfig {
    /// Baud rate (bits per second). Common values: 9600, 19200, 38400, 57600, 115200.
    pub baudrate: u32,
    /// Number of data bits per frame. Valid range: 5–8.
    pub databits: u8,
    /// Number of stop bits. Valid values: 1 or 2.
    pub stopbits: u8,
    /// Parity checking mode. See [`Parity`] for options.
    pub parity: Parity,
    /// Read timeout in milliseconds. `0` means non-blocking.
    pub timeout_ms: u64,
    /// Hardware/software flow control mode. See [`FlowControl`].
    pub flow_control: FlowControl,
    /// Assert DTR (Data Terminal Ready) signal on port open.
    pub dtr_enable: bool,
    /// Assert RTS (Request To Send) signal on port open.
    pub rts_enable: bool,
}

/// Flow control mechanism for serial communication.
///
/// Flow control prevents buffer overflow when the receiver cannot keep up.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlowControl {
    /// No flow control. Data is sent continuously regardless of receiver state.
    None,
    /// Software flow control using XON/XOFF characters (Ctrl-Q / Ctrl-S).
    Software,
    /// Hardware flow control using RTS/CTS signal lines.
    Hardware,
}

/// Parity checking mode for error detection.
#[derive(Debug, Clone, Copy)]
pub enum Parity {
    /// No parity bit. No single-bit error detection.
    None,
    /// Odd parity — the parity bit ensures an odd number of 1-bits in the frame.
    Odd,
    /// Even parity — the parity bit ensures an even number of 1-bits in the frame.
    Even,
}

impl Default for SerialConfig {
    fn default() -> Self {
        Self {
            baudrate: 115200,
            databits: 8,
            stopbits: 1,
            parity: Parity::None,
            timeout_ms: 1000,
            flow_control: FlowControl::None,
            dtr_enable: true,
            rts_enable: true,
        }
    }
}

impl PortManager {
    /// Enumerate all serial ports available on the system.
    ///
    /// Returns a list of [`SerialPortInfo`] with port name, type, and
    /// platform-specific metadata (friendly name on Windows).
    ///
    /// # Errors
    ///
    /// Returns [`SerialError::Serial`] if the platform's enumeration API fails.
    pub fn list_ports(&self) -> Result<Vec<SerialPortInfo>> {
        tokio_serial::available_ports()
            .map_err(|e| SerialError::Serial(SerialPortError::IoError(e.to_string())))
            .map(|ports| {
                ports
                    .into_iter()
                    .map(|p| {
                        #[cfg_attr(not(target_os = "windows"), allow(unused_mut))]
                        let mut info = SerialPortInfo {
                            port_name: p.port_name.clone(),
                            port_type: format!("{:?}", p.port_type),
                            friendly_name: None,
                            hardware_id: None,
                            manufacturer: None,
                            com_number: None,
                        };

                        #[cfg(target_os = "windows")]
                        {
                            if let Some(com_str) = p.port_name.strip_prefix("COM") {
                                if let Ok(num) = com_str.parse::<u32>() {
                                    info.com_number = Some(num);
                                }
                            }
                            if info.com_number.is_some() {
                                info.friendly_name = Some(format!("Serial Port {}", p.port_name));
                            }
                        }

                        info
                    })
                    .collect()
            })
    }

    /// Open a serial port with the given configuration.
    ///
    /// On Unix, opens the TTY device and sets DTR/RTS via `ioctl`.
    /// If IoLoop is enabled, spawns a background read task.
    /// Returns a unique port ID (`<name>-<uuid>`) for later reference.
    ///
    /// # Arguments
    ///
    /// * `name` - Port device path (e.g., `/dev/ttyUSB0`, `COM1`)
    /// * `config` - Serial port settings
    ///
    /// # Errors
    ///
    /// Returns [`SerialError::Serial`] with [`PortNotFound`](SerialPortError::PortNotFound),
    /// [`PermissionDenied`](SerialPortError::PermissionDeniedWithHelp),
    /// [`PortBusy`](SerialPortError::PortBusyWithHelp), or
    /// [`IoError`](SerialPortError::IoError) depending on the underlying OS error.
    pub async fn open_port(&self, name: &str, config: SerialConfig) -> Result<String> {
        let ports_guard = self.ports.lock().await;
        if ports_guard.contains_key(name) {
            return Err(SerialError::Serial(SerialPortError::port_busy(
                name,
                Some("Port is already opened by this application"),
            )));
        }
        drop(ports_guard);

        // Open via serialport::TTYPort on Unix to get raw fd for DTR/RTS
        #[cfg(unix)]
        let (port, signal_controller) = {
            use std::os::unix::io::AsRawFd;

            let builder = serialport::new(name, config.baudrate)
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
                });

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

            (port, signal_controller)
        };

        #[cfg(not(unix))]
        let (port, _signal_controller) = {
            // On Windows, use serialport::new().open_native() to get COMPort
            let builder = serialport::new(name, config.baudrate)
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
                });

            let port = builder
                .open_native()
                .map_err(|e| map_serial_error(e, name))?;
            (Box::new(port), ())
        };

        let dtr_state = config.dtr_enable;
        let rts_state = config.rts_enable;
        let handle = SerialPortHandle {
            name: name.to_string(),
            port,
            config,
            protocol: None,
            dtr_state,
            rts_state,
            #[cfg(unix)]
            signal_controller,
        };

        let mut ports_guard = self.ports.lock().await;
        let port_id = format!("{}-{}", name, uuid::Uuid::new_v4());
        let port_handle = Arc::new(Mutex::new(handle));
        ports_guard.insert(port_id.clone(), port_handle.clone());

        if *self.io_loop_enabled.lock().await {
            let port_id_clone = port_id.clone();
            let port_handle_clone = port_handle.clone();

            tokio::spawn(async move {
                let mut buffer = vec![0u8; 4096];
                loop {
                    let mut handle = port_handle_clone.lock().await;
                    match handle.read(&mut buffer) {
                        Ok(n) if n > 0 => {
                            let data = buffer[..n].to_vec();
                            tracing::debug!("IoLoop: Received {} bytes from {}", n, port_id_clone);
                            if let Ok(text) = String::from_utf8(data.clone()) {
                                tracing::debug!("Data: {}", text);
                            }
                        }
                        Ok(_) => {}
                        Err(_) => break,
                    }
                    drop(handle);
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                }
            });

            tracing::debug!("Started IoLoop task for port: {}", port_id);
        }

        Ok(port_id)
    }

    /// Close a serial port by its unique ID and remove it from the registry.
    ///
    /// # Errors
    ///
    /// Returns [`SerialError::Serial`] with [`PortNotFound`](SerialPortError::PortNotFound)
    /// if the port ID does not exist in the registry.
    pub async fn close_port(&self, port_id: &str) -> Result<()> {
        let mut ports_guard = self.ports.lock().await;
        ports_guard.remove(port_id).ok_or_else(|| {
            SerialError::Serial(SerialPortError::PortNotFound(port_id.to_string()))
        })?;
        Ok(())
    }

    /// Retrieve the [`SerialPortHandle`] for a given port ID.
    ///
    /// # Errors
    ///
    /// Returns [`SerialError::Serial`] with [`PortNotFound`](SerialPortError::PortNotFound)
    /// if the port ID is not registered.
    pub async fn get_port(&self, port_id: &str) -> Result<Arc<Mutex<SerialPortHandle>>> {
        let ports_guard = self.ports.lock().await;
        ports_guard
            .get(port_id)
            .cloned()
            .ok_or_else(|| SerialError::Serial(SerialPortError::PortNotFound(port_id.to_string())))
    }

    /// Associate a protocol name with an open port.
    ///
    /// # Errors
    ///
    /// Returns [`SerialError::Serial`] if the port ID is not found.
    pub async fn set_port_protocol(
        &self,
        port_id: &str,
        protocol_name: Option<String>,
    ) -> Result<()> {
        let port_handle = self.get_port(port_id).await?;
        let mut handle = port_handle.lock().await;
        handle.set_protocol(protocol_name);
        Ok(())
    }

    /// Get the protocol name associated with a port.
    ///
    /// # Errors
    ///
    /// Returns [`SerialError::Serial`] if the port ID is not found.
    pub async fn get_port_protocol(&self, port_id: &str) -> Result<Option<String>> {
        let port_handle = self.get_port(port_id).await?;
        let handle = port_handle.lock().await;
        Ok(handle.protocol().map(|s| s.to_string()))
    }

    /// Set the DTR (Data Terminal Ready) signal for a port.
    ///
    /// # Errors
    ///
    /// Returns [`SerialError::Serial`] if the port is not found or the
    /// platform does not support DTR control.
    pub async fn set_dtr(&self, port_id: &str, enable: bool) -> Result<()> {
        let port_handle = self.get_port(port_id).await?;
        let mut handle = port_handle.lock().await;
        handle.set_dtr(enable)?;
        Ok(())
    }

    /// Set the RTS (Request To Send) signal for a port.
    ///
    /// # Errors
    ///
    /// Returns [`SerialError::Serial`] if the port is not found or the
    /// platform does not support RTS control.
    pub async fn set_rts(&self, port_id: &str, enable: bool) -> Result<()> {
        let port_handle = self.get_port(port_id).await?;
        let mut handle = port_handle.lock().await;
        handle.set_rts(enable)?;
        Ok(())
    }

    /// Get the current DTR signal state for a port.
    ///
    /// # Errors
    ///
    /// Returns [`SerialError::Serial`] if the port ID is not found.
    pub async fn get_dtr(&self, port_id: &str) -> Result<bool> {
        let port_handle = self.get_port(port_id).await?;
        let handle = port_handle.lock().await;
        Ok(handle.dtr_enabled())
    }

    /// Get the current RTS signal state for a port.
    ///
    /// # Errors
    ///
    /// Returns [`SerialError::Serial`] if the port ID is not found.
    pub async fn get_rts(&self, port_id: &str) -> Result<bool> {
        let port_handle = self.get_port(port_id).await?;
        let handle = port_handle.lock().await;
        Ok(handle.rts_enabled())
    }
}

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

fn map_serial_error(e: serialport::Error, name: &str) -> SerialError {
    let error_msg = e.to_string();
    if error_msg.contains("permission denied")
        || error_msg.contains("Permission denied")
        || error_msg.contains("Access is denied")
    {
        SerialError::Serial(SerialPortError::permission_denied(
            name,
            Some("Try running as Administrator or check port permissions"),
        ))
    } else if error_msg.contains("not found")
        || error_msg.contains("No such file")
        || error_msg.contains("The system cannot find the file")
    {
        SerialError::Serial(SerialPortError::PortNotFound(name.to_string()))
    } else if error_msg.contains("busy")
        || error_msg.contains("Busy")
        || error_msg.contains("used by another application")
    {
        SerialError::Serial(SerialPortError::port_busy(
            name,
            Some("Close other applications using this port or try a different port"),
        ))
    } else {
        SerialError::Serial(SerialPortError::IoError(error_msg))
    }
}

impl SerialPortHandle {
    /// Get the port device name (e.g., `/dev/ttyUSB0`).
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the port's active [`SerialConfig`].
    pub fn config(&self) -> &SerialConfig {
        &self.config
    }

    /// Get the protocol name associated with this port, if one has been set.
    pub fn protocol(&self) -> Option<&str> {
        self.protocol.as_deref()
    }

    /// Set or clear the protocol association for this port.
    pub fn set_protocol(&mut self, protocol_name: Option<String>) {
        self.protocol = protocol_name;
    }

    /// Set the DTR signal state. Reverts on platform control failure to keep
    /// in-memory state consistent with the actual hardware state.
    ///
    /// # Errors
    ///
    /// Returns an error if the platform signal control fails.
    pub fn set_dtr(&mut self, enable: bool) -> Result<()> {
        if enable != self.dtr_state {
            let old_state = self.dtr_state;
            self.dtr_state = enable;

            #[cfg(unix)]
            let result = self.signal_controller.set_dtr(enable);

            #[cfg(not(unix))]
            let result: Result<crate::serial_core::signals::SignalState> = {
                self.dtr_state = enable;
                Ok(crate::serial_core::signals::SignalState::NotSupported)
            };

            match result {
                Ok(crate::serial_core::signals::SignalState::Set(_)) => {
                    tracing::info!("DTR signal set to: {} for port {}", enable, self.name);
                }
                Ok(_) => {
                    tracing::warn!(
                        "DTR signal control not available on this platform for port {}",
                        self.name
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to set DTR signal for port {}: {}. State updated in memory only.",
                        self.name,
                        e
                    );
                    self.dtr_state = old_state;
                }
            }
        }
        Ok(())
    }

    /// Set the RTS signal state. Reverts on platform control failure.
    ///
    /// # Errors
    ///
    /// Returns an error if the platform signal control fails.
    pub fn set_rts(&mut self, enable: bool) -> Result<()> {
        if enable != self.rts_state {
            let old_state = self.rts_state;
            self.rts_state = enable;

            #[cfg(unix)]
            let result = self.signal_controller.set_rts(enable);

            #[cfg(not(unix))]
            let result: Result<crate::serial_core::signals::SignalState> = {
                self.rts_state = enable;
                Ok(crate::serial_core::signals::SignalState::NotSupported)
            };

            match result {
                Ok(crate::serial_core::signals::SignalState::Set(_)) => {
                    tracing::info!("RTS signal set to: {} for port {}", enable, self.name);
                }
                Ok(_) => {
                    tracing::warn!(
                        "RTS signal control not available on this platform for port {}",
                        self.name
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to set RTS signal for port {}: {}. State updated in memory only.",
                        self.name,
                        e
                    );
                    self.rts_state = old_state;
                }
            }
        }
        Ok(())
    }

    /// Check whether DTR is currently asserted.
    pub fn dtr_enabled(&self) -> bool {
        self.dtr_state
    }

    /// Check whether RTS is currently asserted.
    pub fn rts_enabled(&self) -> bool {
        self.rts_state
    }

    /// Write raw bytes to the serial port. Returns the number of bytes written.
    ///
    /// # Errors
    ///
    /// Returns [`SerialError::Serial`] with [`IoError`](SerialPortError::IoError)
    /// if the underlying write fails.
    pub fn write(&mut self, data: &[u8]) -> Result<usize> {
        self.port
            .write(data)
            .map_err(|e| SerialError::Serial(SerialPortError::IoError(e.to_string())))
    }

    /// Read bytes from the serial port into the provided buffer.
    /// Returns the number of bytes actually read. Respects the configured timeout.
    ///
    /// # Errors
    ///
    /// Returns [`SerialError::Serial`] with [`IoError`](SerialPortError::IoError)
    /// if the read fails (e.g., timeout, disconnected).
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.port
            .read(buf)
            .map_err(|e| SerialError::Serial(SerialPortError::IoError(e.to_string())))
    }

    /// Close the port, consuming `self`. This is a no-op since the port
    /// is dropped when the handle goes out of scope.
    pub fn close(self) -> Result<()> {
        Ok(())
    }
}

/// Metadata about an enumerated serial port.
///
/// Fields are platform-dependent — on Windows, `com_number` and `friendly_name`
/// are populated automatically from the device manager.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SerialPortInfo {
    /// Device path (e.g., `/dev/ttyUSB0`, `COM1`).
    pub port_name: String,
    /// Port type as reported by the OS (`Usb`, `Pci`, `Bluetooth`, etc.).
    pub port_type: String,
    /// Human-readable device name from the OS device manager (Windows only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub friendly_name: Option<String>,
    /// Hardware identifier string (not yet populated).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hardware_id: Option<String>,
    /// Device manufacturer string (not yet populated).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manufacturer: Option<String>,
    /// COM port number extracted from the name (e.g., `COM3` → `3`). Windows only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub com_number: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = SerialConfig::default();
        assert_eq!(config.baudrate, 115200);
        assert_eq!(config.databits, 8);
        assert_eq!(config.stopbits, 1);
    }

    #[test]
    fn test_list_ports() {
        let manager = PortManager::new();
        let ports = manager.list_ports();
        assert!(ports.is_ok());
    }

    #[tokio::test]
    async fn test_port_manager_creation() {
        let manager = PortManager::new();
        let result = manager
            .open_port("NONEXISTENT", SerialConfig::default())
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_close_nonexistent_port() {
        let manager = PortManager::new();
        let result = manager.close_port("nonexistent_id").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_config_custom_baudrate() {
        let config = SerialConfig {
            baudrate: 9600,
            ..Default::default()
        };
        assert_eq!(config.baudrate, 9600);
    }

    #[test]
    fn test_config_all_fields() {
        let config = SerialConfig {
            baudrate: 57600,
            databits: 7,
            parity: Parity::Even,
            ..Default::default()
        };
        assert_eq!(config.baudrate, 57600);
        assert_eq!(config.databits, 7);
        assert!(matches!(config.parity, Parity::Even));
    }
}
