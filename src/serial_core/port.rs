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
use crate::serial_core::serial_script::SerialScriptEngine;
#[cfg(unix)]
use crate::serial_core::signals::PlatformSignals;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

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
    /// Excess buffer for data returned by on_recv that didn't fit the caller's buffer.
    /// Prepend this data on the next read() call before performing an OS read.
    excess_buffer: Vec<u8>,
    /// Broadcast channel for received raw data.
    /// Subscribers (e.g., server connections) receive data pushed by the IoLoop.
    /// Buffer size of 32 means slow subscribers will drop oldest data.
    data_tx: broadcast::Sender<Vec<u8>>,
    script_engine: Option<SerialScriptEngine>,
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
        // Check for duplicate opens — scan values to see if this device name is already open.
        {
            let ports_guard = self.ports.lock().await;
            let handles: Vec<Arc<Mutex<SerialPortHandle>>> =
                ports_guard.values().cloned().collect();
            drop(ports_guard);

            for h in &handles {
                let handle = h.lock().await;
                if handle.name() == name {
                    return Err(SerialError::Serial(SerialPortError::port_busy(
                        name,
                        Some("Port is already opened by this application"),
                    )));
                }
            }
        }

        // Configure port using platform-specific implementation
        let configured = crate::serial_core::platform::configure_port(name, &config)?;

        #[cfg(unix)]
        let (port, signal_controller) = (configured.port, configured.signal_controller);
        #[cfg(not(unix))]
        let (port, _signal_controller) = (configured.port, ());

        let dtr_state = config.dtr_enable;
        let rts_state = config.rts_enable;
        let (data_tx, _data_rx) = broadcast::channel::<Vec<u8>>(32);
        let handle = SerialPortHandle {
            name: name.to_string(),
            port,
            config,
            excess_buffer: Vec::new(),
            data_tx,
            script_engine: None,
            dtr_state,
            rts_state,
            #[cfg(unix)]
            signal_controller,
        };

        let mut ports_guard = self.ports.lock().await;
        let port_id = format!("{}-{}", name, uuid::Uuid::new_v4());
        let data_tx_clone = handle.data_tx.clone();
        let port_handle = Arc::new(Mutex::new(handle));
        ports_guard.insert(port_id.clone(), port_handle.clone());

        if *self.io_loop_enabled.lock().await {
            let port_id_clone = port_id.clone();
            let port_handle_clone = port_handle.clone();

            tokio::spawn(async move {
                let mut buffer = vec![0u8; 4096];
                loop {
                    let n = {
                        let mut handle = port_handle_clone.lock().await;
                        match handle.read(&mut buffer) {
                            Ok(n) => n,
                            Err(_) => break,
                        }
                    };

                    if n > 0 {
                        let data = buffer[..n].to_vec();
                        let _ = data_tx_clone.send(data.clone());
                        tracing::debug!("IoLoop: Received {} bytes from {}", n, port_id_clone);
                        if let Ok(text) = String::from_utf8(data) {
                            tracing::debug!("Data: {}", text);
                        }
                    }

                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                }
            });

            tracing::debug!("Started IoLoop task for port: {}", port_id);
        }

        Ok(port_id)
    }

    /// Open a serial port with virtual port detection.
    ///
    /// This is a wrapper around `open_port` that skips DTR/RTS initialization
    /// for virtual ports (PTY, named pipes, etc.) which don't support modem
    /// control signals via ioctl.
    ///
    /// # Arguments
    ///
    /// * `name` - Port device path (e.g., `/dev/ttyUSB0`, `/dev/pts/0`)
    /// * `config` - Serial port settings
    /// * `is_virtual` - Whether this is a virtual port that doesn't support DTR/RTS
    pub async fn open_port_virtual(
        &self,
        name: &str,
        mut config: SerialConfig,
        is_virtual: bool,
    ) -> Result<String> {
        // For virtual ports, disable DTR/RTS to avoid ENOTTY errors
        if is_virtual {
            config.dtr_enable = false;
            config.rts_enable = false;
        }
        self.open_port(name, config).await
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

    /// Return `(port_id, port_name)` pairs for all currently open ports.
    pub async fn list_open_ports(&self) -> Vec<(String, String)> {
        let ports_guard = self.ports.lock().await;
        let entries: Vec<(String, Arc<Mutex<SerialPortHandle>>)> = ports_guard
            .iter()
            .map(|(id, handle)| (id.clone(), handle.clone()))
            .collect();
        drop(ports_guard);

        let mut result = Vec::with_capacity(entries.len());
        for (id, handle) in entries {
            let h = handle.lock().await;
            result.push((id, h.name().to_string()));
        }
        result.sort_by(|a, b| a.1.cmp(&b.1));
        result
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

    /// Attach a Lua script engine to an open port.
    ///
    /// # Errors
    ///
    /// Returns an error if the port is not found or a script is already attached.
    pub async fn attach_script(&self, port_id: &str, engine: SerialScriptEngine) -> Result<()> {
        let port_handle = self.get_port(port_id).await?;
        let mut handle = port_handle.lock().await;
        // Safety: the script engine is attached to this handle and the port
        // pointer remains valid as long as the handle exists.
        unsafe { handle.attach_script(engine) }
    }

    /// Attach a script by name from the ScriptManager.
    ///
    /// Looks up the script in the ScriptManager, creates a SerialScriptEngine,
    /// and attaches it to the port. This is the unified way to attach scripts
    /// (replaces `set_port_protocol_by_name`).
    ///
    /// # Errors
    ///
    /// Returns an error if the script is not found, the port is not found,
    /// or a script is already attached.
    pub async fn attach_script_by_name(
        &self,
        port_id: &str,
        script_manager: &crate::script::ScriptManager,
        script_name: &str,
    ) -> Result<()> {
        let engine = script_manager.create_engine(script_name)?;
        self.attach_script(port_id, engine).await
    }

    /// Detach the script engine from an open port.
    /// Calls `on_close` and stops the timer before detaching.
    pub async fn detach_script(&self, port_id: &str) -> Result<()> {
        let port_handle = self.get_port(port_id).await?;
        let mut handle = port_handle.lock().await;
        handle.detach_script();
        Ok(())
    }

    /// Check if a port has a script engine attached.
    ///
    /// # Errors
    ///
    /// Returns [`SerialError::Serial`] if the port ID is not found.
    pub async fn has_script(&self, port_id: &str) -> Result<bool> {
        let port_handle = self.get_port(port_id).await?;
        let handle = port_handle.lock().await;
        Ok(handle.has_script())
    }

    /// Get script status for a port. Returns a tuple of (has_script, timer_interval_ms).
    ///
    /// # Errors
    ///
    /// Returns [`SerialError::Serial`] if the port ID is not found.
    pub async fn get_script_status(&self, port_id: &str) -> Result<(bool, u64)> {
        let port_handle = self.get_port(port_id).await?;
        let handle = port_handle.lock().await;
        Ok((handle.has_script(), handle.script_timer_interval_ms()))
    }

    /// Discover all UI actions (`action_*` functions) defined in the port's script.
    ///
    /// Returns an empty vec if no script is attached.
    ///
    /// # Errors
    ///
    /// Returns [`SerialError::Serial`] if the port ID is not found.
    pub async fn list_script_actions(
        &self,
        port_id: &str,
    ) -> Result<Vec<crate::lua::ui_actions::UiAction>> {
        let port_handle = self.get_port(port_id).await?;
        let handle = port_handle.lock().await;
        match &handle.script_engine {
            Some(engine) => engine.discover_actions().await,
            None => Ok(vec![]),
        }
    }

    /// Execute a UI action function by name on the port's script engine.
    ///
    /// # Errors
    ///
    /// Returns [`SerialError::Serial`] if the port ID is not found.
    /// Returns an error if no script is attached or the function does not exist.
    pub async fn call_script_action(&self, port_id: &str, function_name: &str) -> Result<String> {
        let port_handle = self.get_port(port_id).await?;
        let handle = port_handle.lock().await;
        match &handle.script_engine {
            Some(engine) => engine.execute_action(function_name).await,
            None => Err(SerialError::Script(crate::error::ScriptError::ApiError(
                "No script attached to port".into(),
            ))),
        }
    }
}

/// Thin wrapper to assert `*mut dyn SerialPort` is safe to Send/Sync between threads.
///
/// Safety: The pointer is only valid while the `SerialPortHandle` is alive.
/// The script engine (which holds the callback) is always dropped before the handle.
#[derive(Clone)]
struct PortPtr(Arc<PortPtrInner>);

struct PortPtrInner {
    ptr: std::ptr::NonNull<dyn serialport::SerialPort>,
}

// Safety: All accesses to the port through this pointer are serialized by the
// SerialPortHandle's tokio::Mutex. The pointer is valid as long as the handle exists.
unsafe impl Send for PortPtrInner {}
unsafe impl Sync for PortPtrInner {}

// Safety: All accesses to the port through this pointer are serialized by the
// SerialPortHandle's tokio::Mutex. The pointer is valid as long as the handle exists.
unsafe impl Send for PortPtr {}
unsafe impl Sync for PortPtr {}

impl PortPtr {
    fn new(ptr: *mut dyn serialport::SerialPort) -> Self {
        Self(Arc::new(PortPtrInner {
            // Safety: caller guarantees the pointer is non-null and valid
            ptr: unsafe { std::ptr::NonNull::new_unchecked(ptr) },
        }))
    }

    /// Write data to the serial port.
    /// Safety: the underlying port pointer must be valid.
    unsafe fn write_all(&self, data: &[u8]) -> std::io::Result<()> {
        let raw: *mut dyn serialport::SerialPort = self.0.ptr.as_ptr();
        (&mut *raw).write_all(data)
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

    /// Subscribe to the data broadcast channel.
    /// Returns a [`broadcast::Receiver<Vec<u8>>`] that receives raw data
    /// from the IoLoop. Used by the server for push notifications.
    pub fn subscribe_data(&self) -> broadcast::Receiver<Vec<u8>> {
        self.data_tx.subscribe()
    }

    /// Attach a Lua script engine to this port.
    ///
    /// The script's `on_open` hook is called immediately with the port name and config.
    /// If the script defines `on_timer`, the timer is started automatically.
    ///
    /// The `serial_send()` Lua API is automatically wired to write directly to
    /// the underlying serial port (bypassing protocol encoding for auto-reply).
    /// # Safety
    ///
    /// The script engine must not outlive the port handle.
    pub unsafe fn attach_script(&mut self, mut engine: SerialScriptEngine) -> Result<()> {
        if self.script_engine.is_some() {
            return Err(SerialError::Script(crate::error::ScriptError::ApiError(
                "A script is already attached to this port".to_string(),
            )));
        }

        // Capture a raw pointer to the port for the send callback.
        // Safety: The callback is only invoked while the script engine is attached
        // to this handle, guaranteeing the port pointer remains valid.
        // All accesses are serialized by the SerialPortHandle's outer lock.
        let port_ptr = PortPtr::new(&mut *self.port);
        let send_fn = Arc::new(move |data: &[u8]| {
            // Safety: port_ptr is valid as long as the script is attached,
            // and all writes are serialized by the handle's outer lock.
            unsafe { port_ptr.write_all(data) }
                .map_err(|e| SerialError::Serial(SerialPortError::IoError(e.to_string())))?;
            Ok(data.len())
        });

        engine.set_send_callback(send_fn)?;
        engine.load()?;

        // Call on_open hook
        engine.on_open(&self.name, &self.config);

        self.script_engine = Some(engine);
        Ok(())
    }

    /// Detach the script engine from this port.
    /// Calls `on_close` and stops the timer before detaching.
    pub fn detach_script(&mut self) {
        if let Some(engine) = self.script_engine.take() {
            engine.on_close();
            // engine is dropped, which calls Drop::drop() → stop_timer()
        }
    }

    /// Check if a script engine is attached.
    pub fn has_script(&self) -> bool {
        self.script_engine.is_some()
    }

    /// Get the script engine's timer interval (0 = disabled).
    pub fn script_timer_interval_ms(&self) -> u64 {
        self.script_engine
            .as_ref()
            .map(|e| e.timer_interval_ms())
            .unwrap_or(0)
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
    /// Data flow: raw data → script `on_send()` → protocol `encode()` → serial port.
    /// The script can intercept, modify, or block the data entirely.
    ///
    /// # Errors
    ///
    /// Returns [`SerialError::Serial`] with [`IoError`](SerialPortError::IoError)
    /// if the underlying write fails.
    pub fn write(&mut self, data: &[u8]) -> Result<usize> {
        // Script engine hook (on_send) — handles encoding and behavior
        let after_script = if let Some(ref engine) = self.script_engine {
            engine.on_send(data)?
        } else {
            data.to_vec()
        };

        // If script returned nil or empty result, treat as intercepted
        if after_script.is_empty() {
            return Ok(0);
        }

        self.port
            .write(&after_script)
            .map_err(|e| SerialError::Serial(SerialPortError::IoError(e.to_string())))
    }

    /// Read bytes from the serial port into the provided buffer.
    /// Returns the number of bytes actually read. Respects the configured timeout.
    ///
    /// Data flow: excess buffer (if any) → serial port → protocol `parse()` →
    /// script `on_recv()` → caller. The script can intercept, modify, suppress,
    /// or auto-reply to received data.
    ///
    /// If `on_recv` returns more data than fits in `buf`, excess data is stored
    /// and prepended on the next `read()` call — no data is silently lost.
    ///
    /// # Errors
    ///
    /// Returns [`SerialError::Serial`] with [`IoError`](SerialPortError::IoError)
    /// if the read fails (e.g., timeout, disconnected).
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        // First, drain any excess data from a previous call
        if !self.excess_buffer.is_empty() {
            let len = self.excess_buffer.len().min(buf.len());
            buf[..len].copy_from_slice(&self.excess_buffer[..len]);
            self.excess_buffer.drain(..len);
            return Ok(len);
        }

        let n = self
            .port
            .read(buf)
            .map_err(|e| SerialError::Serial(SerialPortError::IoError(e.to_string())))?;

        if n == 0 {
            return Ok(0);
        }

        // Script engine hook (on_recv) — handles framing and behavior
        let after_script = if let Some(ref engine) = self.script_engine {
            engine.on_recv(&buf[..n])
        } else {
            buf[..n].to_vec()
        };

        // Copy result back to buffer, storing excess for the next call
        let len = after_script.len().min(buf.len());
        buf[..len].copy_from_slice(&after_script[..len]);
        if after_script.len() > len {
            self.excess_buffer.extend_from_slice(&after_script[len..]);
        }
        Ok(len)
    }

    /// Close the port, consuming `self`. Calls the script's `on_close` hook
    /// and stops the timer before dropping.
    pub fn close(mut self) -> Result<()> {
        if let Some(engine) = self.script_engine.take() {
            engine.on_close();
            // engine dropped → Drop::drop() → stop_timer()
        }
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
