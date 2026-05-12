//! Lua API bindings
//!
//! This module provides the Rust API bindings for Lua scripts.

use crate::error::{Result, SerialError};
use crate::serial_core::PortManager;
use mlua::{Function, Lua, Value};
use std::cell::RefCell;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Lua API bindings
pub struct LuaBindings {
    lua: Lua,
    port_manager: Option<Arc<Mutex<PortManager>>>,
    runtime_handle: RefCell<Option<tokio::runtime::Handle>>,
}

impl LuaBindings {
    /// Create new Lua bindings
    pub fn new() -> Result<Self> {
        let lua = Lua::new();
        Ok(Self {
            lua,
            port_manager: None,
            runtime_handle: RefCell::new(None),
        })
    }

    /// Register logging API
    pub fn register_log_api(&self) -> Result<()> {
        let globals = self.lua.globals();

        // log.info
        let info = self.lua.create_function(|_, msg: String| {
            tracing::info!("[INFO] {}", msg);
            Ok(())
        })?;
        globals.set("log_info", info)?;

        // log.debug
        let debug = self.lua.create_function(|_, msg: String| {
            tracing::info!("[DEBUG] {}", msg);
            Ok(())
        })?;
        globals.set("log_debug", debug)?;

        // log.warn
        let warn = self.lua.create_function(|_, msg: String| {
            tracing::info!("[WARN] {}", msg);
            Ok(())
        })?;
        globals.set("log_warn", warn)?;

        // log.error
        let error = self.lua.create_function(|_, msg: String| {
            tracing::info!("[ERROR] {}", msg);
            Ok(())
        })?;
        globals.set("log_error", error)?;

        Ok(())
    }

    /// Register utility APIs
    pub fn register_utility_apis(&self) -> Result<()> {
        let globals = self.lua.globals();

        // json.encode - proper JSON serialization
        let encode = self.lua.create_function(|_, value: Value| {
            let json_value = Self::lua_value_to_json(value)?;
            serde_json::to_string(&json_value)
                .map_err(|e| mlua::Error::RuntimeError(format!("JSON encode error: {}", e)))
        })?;
        globals.set("json_encode", encode)?;

        // json.encode_pretty - pretty printed JSON
        let encode_pretty = self.lua.create_function(|_, value: Value| {
            let json_value = Self::lua_value_to_json(value)?;
            serde_json::to_string_pretty(&json_value)
                .map_err(|e| mlua::Error::RuntimeError(format!("JSON encode error: {}", e)))
        })?;
        globals.set("json_encode_pretty", encode_pretty)?;

        // json.decode - parse JSON string to Lua table
        let decode = self.lua.create_function(|lua, json_str: String| {
            let v: serde_json::Value = serde_json::from_str(&json_str)
                .map_err(|e| mlua::Error::RuntimeError(format!("JSON decode error: {}", e)))?;
            Self::json_to_lua_value(lua, v)
        })?;
        globals.set("json_decode", decode)?;

        // sleep
        let sleep = self.lua.create_function(|_, ms: u64| {
            std::thread::sleep(std::time::Duration::from_millis(ms));
            Ok(())
        })?;
        globals.set("sleep_ms", sleep)?;

        Ok(())
    }

    /// Convert Lua value to JSON value
    fn lua_value_to_json(value: Value) -> mlua::Result<serde_json::Value> {
        match value {
            Value::Nil => Ok(serde_json::Value::Null),
            Value::Boolean(b) => Ok(serde_json::Value::Bool(b)),
            Value::Integer(i) => Ok(serde_json::Value::Number(serde_json::Number::from(i))),
            Value::Number(n) => serde_json::Number::from_f64(n)
                .map(serde_json::Value::Number)
                .ok_or_else(|| mlua::Error::RuntimeError("Invalid float value".to_string())),
            Value::String(s) => Ok(serde_json::Value::String(s.to_str()?.to_string())),
            Value::Table(t) => {
                // Try to detect if it's an array or object
                // Check if it's a sequential array (1, 2, 3, ...)
                let len = t.len().unwrap_or(0) as usize;
                if len > 0 {
                    let mut arr = Vec::with_capacity(len);
                    for i in 1..=len as i64 {
                        match t.get(i) {
                            Ok(v) => arr.push(Self::lua_value_to_json(v)?),
                            Err(_) => arr.push(serde_json::Value::Null),
                        }
                    }
                    Ok(serde_json::Value::Array(arr))
                } else {
                    // Treat as object
                    let mut map = serde_json::Map::new();
                    let pairs: mlua::Result<Vec<(mlua::String, Value)>> = t.pairs().collect();
                    for pair in pairs.map_err(|e| mlua::Error::RuntimeError(e.to_string()))? {
                        let key = pair.0.to_str()?.to_string();
                        let value = Self::lua_value_to_json(pair.1)?;
                        map.insert(key, value);
                    }
                    Ok(serde_json::Value::Object(map))
                }
            }
            Value::LightUserData(_)
            | Value::Function(_)
            | Value::Thread(_)
            | Value::UserData(_)
            | Value::Error(_) => Ok(serde_json::Value::Null),
        }
    }

    /// Convert JSON value to Lua value
    fn json_to_lua_value(lua: &Lua, value: serde_json::Value) -> mlua::Result<Value<'_>> {
        match value {
            serde_json::Value::Null => Ok(Value::Nil),
            serde_json::Value::Bool(b) => Ok(Value::Boolean(b)),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(Value::Integer(i))
                } else if let Some(f) = n.as_f64() {
                    Ok(Value::Number(f))
                } else {
                    Ok(Value::Nil)
                }
            }
            serde_json::Value::String(s) => Ok(Value::String(lua.create_string(&s)?)),
            serde_json::Value::Array(arr) => {
                let table = lua.create_table()?;
                for (i, v) in arr.into_iter().enumerate() {
                    table.set(i as i64 + 1, Self::json_to_lua_value(lua, v)?)?;
                }
                Ok(Value::Table(table))
            }
            serde_json::Value::Object(obj) => {
                let table = lua.create_table()?;
                for (k, v) in obj.into_iter() {
                    table.set(k, Self::json_to_lua_value(lua, v)?)?;
                }
                Ok(Value::Table(table))
            }
        }
    }

    /// Register all APIs
    pub fn register_all_apis(&self) -> Result<()> {
        // Existing APIs
        self.register_log_api()?;
        self.register_utility_apis()?;

        // New Serial APIs (only if port manager is initialized)
        if self.port_manager.is_some() {
            self.register_serial_open()?;
            self.register_serial_close()?;
            self.register_serial_send()?;
            self.register_serial_recv()?;
            self.register_serial_list()?;
        }

        // New Protocol APIs
        self.register_protocol_encode()?;
        self.register_protocol_decode()?;
        self.register_protocol_list()?;
        self.register_protocol_info()?;

        // New protocol management APIs
        self.register_protocol_load()?;
        self.register_protocol_unload()?;
        self.register_protocol_reload()?;
        self.register_protocol_validate()?;

        // Virtual serial port APIs
        self.register_virtual_create()?;
        self.register_virtual_stop()?;

        Ok(())
    }

    /// Execute a Lua script
    pub fn execute_script(&self, script: &str) -> Result<()> {
        self.lua.load(script).exec().map_err(SerialError::Lua)
    }

    /// Execute a Lua function (simplified - returns success/failure)
    pub fn execute_function(&self, func_name: &str, args: Vec<Value>) -> Result<()> {
        let globals = self.lua.globals();
        let func: Function = globals.get(func_name).map_err(SerialError::Lua)?;

        func.call(args).map(|_: Value| ()).map_err(SerialError::Lua)
    }

    /// Get a global value (simplified)
    pub fn get_global(&self, name: &str) -> Result<Value<'_>> {
        let globals = self.lua.globals();
        globals.get(name).map_err(SerialError::Lua)
    }

    /// Set a global value (simplified)
    pub fn set_global(&self, name: &str, value: Value) -> Result<()> {
        let globals = self.lua.globals();
        globals.set(name, value).map_err(SerialError::Lua)
    }

    /// Set the port manager
    pub fn set_port_manager(&mut self, pm: Arc<Mutex<PortManager>>) {
        self.port_manager = Some(pm);
    }

    /// Ensure runtime is initialized
    fn ensure_runtime(&self) -> Result<()> {
        if self.runtime_handle.borrow().is_none() {
            // Try to get current runtime handle, or create a new runtime
            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                *self.runtime_handle.borrow_mut() = Some(handle);
            } else {
                // No current runtime, create a new one and get its handle
                tokio::runtime::Runtime::new()
                    .map(|rt| {
                        *self.runtime_handle.borrow_mut() = Some(rt.handle().clone());
                        // Drop the runtime explicitly
                        drop(rt);
                    })
                    .map_err(|e| SerialError::Config(format!("Failed to create runtime: {}", e)))?;
            }
        }
        Ok(())
    }

    /// Register serial_open API
    pub fn register_serial_open(&self) -> Result<()> {
        self.ensure_runtime()?;

        let port_manager = self
            .port_manager
            .clone()
            .ok_or_else(|| SerialError::Config("PortManager not initialized".to_string()))?;

        let handle = self
            .runtime_handle
            .borrow()
            .as_ref()
            .ok_or_else(|| SerialError::Config("Runtime handle not initialized".to_string()))?
            .clone();

        let open = self.lua.create_function(
            move |_, (port_name, config_table): (String, mlua::Table)| {
                // Parse configuration table
                let baudrate: u32 = config_table.get("baudrate").unwrap_or(115200);
                let databits: u8 = config_table.get("data_bits").unwrap_or(8);
                let stopbits: u8 = config_table.get("stop_bits").unwrap_or(1);
                let parity_str: String = config_table.get("parity").unwrap_or("none".to_string());
                let timeout: u64 = config_table.get("timeout").unwrap_or(1000);
                let flow_control_str: String = config_table
                    .get("flow_control")
                    .unwrap_or("none".to_string());
                let dtr_enable: bool = config_table.get("dtr_enable").unwrap_or(true);
                let rts_enable: bool = config_table.get("rts_enable").unwrap_or(true);

                // Parse parity
                let parity = match parity_str.to_lowercase().as_str() {
                    "odd" => crate::serial_core::Parity::Odd,
                    "even" => crate::serial_core::Parity::Even,
                    _ => crate::serial_core::Parity::None,
                };

                // Parse flow control
                let flow_control = match flow_control_str.to_lowercase().as_str() {
                    "software" => crate::serial_core::FlowControl::Software,
                    "hardware" => crate::serial_core::FlowControl::Hardware,
                    _ => crate::serial_core::FlowControl::None,
                };

                let pm_guard = handle.block_on(port_manager.lock());
                let config = crate::serial_core::SerialConfig {
                    baudrate,
                    databits,
                    stopbits,
                    parity,
                    timeout_ms: timeout,
                    flow_control,
                    dtr_enable,
                    rts_enable,
                };

                let port_id = handle
                    .block_on(pm_guard.open_port(&port_name, config))
                    .map_err(|e: crate::error::SerialError| {
                        mlua::Error::RuntimeError(e.to_string())
                    })?;

                Ok(port_id)
            },
        )?;

        self.lua.globals().set("serial_open", open)?;
        Ok(())
    }

    /// Register serial_close API
    pub fn register_serial_close(&self) -> Result<()> {
        self.ensure_runtime()?;

        let port_manager = self
            .port_manager
            .clone()
            .ok_or_else(|| SerialError::Config("PortManager not initialized".to_string()))?;

        let handle = self
            .runtime_handle
            .borrow()
            .as_ref()
            .ok_or_else(|| SerialError::Config("Runtime handle not initialized".to_string()))?
            .clone();

        let close = self.lua.create_function(move |_, port_id: String| {
            let pm_guard = handle.block_on(port_manager.lock());

            handle
                .block_on(pm_guard.close_port(&port_id))
                .map_err(|e: crate::error::SerialError| mlua::Error::RuntimeError(e.to_string()))?;

            Ok(true)
        })?;

        self.lua.globals().set("serial_close", close)?;
        Ok(())
    }

    /// Register serial_send API
    pub fn register_serial_send(&self) -> Result<()> {
        self.ensure_runtime()?;

        let port_manager = self
            .port_manager
            .clone()
            .ok_or_else(|| SerialError::Config("PortManager not initialized".to_string()))?;

        let handle = self
            .runtime_handle
            .borrow()
            .as_ref()
            .ok_or_else(|| SerialError::Config("Runtime handle not initialized".to_string()))?
            .clone();

        let send = self
            .lua
            .create_function(move |_, (port_id, data): (String, String)| {
                let pm_guard = handle.block_on(port_manager.lock());

                let port_handle = handle.block_on(pm_guard.get_port(&port_id)).map_err(
                    |e: crate::error::SerialError| mlua::Error::RuntimeError(e.to_string()),
                )?;

                let mut port = handle.block_on(port_handle.lock());
                let bytes =
                    port.write(data.as_bytes())
                        .map_err(|e: crate::error::SerialError| {
                            mlua::Error::RuntimeError(e.to_string())
                        })?;

                Ok(bytes)
            })?;

        self.lua.globals().set("serial_send", send)?;
        Ok(())
    }

    /// Register serial_recv API
    pub fn register_serial_recv(&self) -> Result<()> {
        self.ensure_runtime()?;

        let port_manager = self
            .port_manager
            .clone()
            .ok_or_else(|| SerialError::Config("PortManager not initialized".to_string()))?;

        let handle = self
            .runtime_handle
            .borrow()
            .as_ref()
            .ok_or_else(|| SerialError::Config("Runtime handle not initialized".to_string()))?
            .clone();

        let recv = self
            .lua
            .create_function(move |_, (port_id, timeout_ms): (String, u64)| {
                let pm_guard = handle.block_on(port_manager.lock());

                let port_handle = handle.block_on(pm_guard.get_port(&port_id)).map_err(
                    |e: crate::error::SerialError| mlua::Error::RuntimeError(e.to_string()),
                )?;

                // Clone handle for the async block
                let rt_clone = handle.clone();

                // Wrap synchronous read in async block for timeout
                let read_future = async move {
                    // Move port_handle into the blocking task
                    let read_result = tokio::task::spawn_blocking(move || {
                        let mut port = rt_clone.block_on(port_handle.lock());
                        let mut buffer = vec![0u8; 4096];

                        let n = port.read(&mut buffer)?;

                        buffer.truncate(n);
                        Ok(String::from_utf8_lossy(&buffer).to_string())
                    })
                    .await
                    .map_err(|e| {
                        crate::error::SerialError::Serial(crate::error::SerialPortError::IoError(
                            e.to_string(),
                        ))
                    })?;

                    read_result
                };

                // Apply timeout
                let result =
                    tokio::time::timeout(std::time::Duration::from_millis(timeout_ms), read_future);

                let data = handle
                    .block_on(result)
                    .map_err(|_| mlua::Error::RuntimeError("Timeout".to_string()))?
                    .map_err(|e: crate::error::SerialError| {
                        mlua::Error::RuntimeError(e.to_string())
                    })?;

                Ok(data)
            })?;

        self.lua.globals().set("serial_recv", recv)?;
        Ok(())
    }

    /// Register serial_list API
    pub fn register_serial_list(&self) -> Result<()> {
        self.ensure_runtime()?;

        let port_manager = self
            .port_manager
            .clone()
            .ok_or_else(|| SerialError::Config("PortManager not initialized".to_string()))?;

        let handle = self
            .runtime_handle
            .borrow()
            .as_ref()
            .ok_or_else(|| SerialError::Config("Runtime handle not initialized".to_string()))?
            .clone();

        let list = self.lua.create_function(move |lua, ()| {
            let pm_guard = handle.block_on(port_manager.lock());

            let ports = pm_guard
                .list_ports()
                .map_err(|e: crate::error::SerialError| mlua::Error::RuntimeError(e.to_string()))?;

            // Convert to Lua table
            let result = lua.create_table()?;
            for (i, port) in ports.iter().enumerate() {
                let port_table = lua.create_table()?;
                port_table.set("port_name", port.port_name.clone())?;
                port_table.set("port_type", port.port_type.clone())?;
                result.set(i + 1, port_table)?;
            }

            Ok(result)
        })?;

        self.lua.globals().set("serial_list", list)?;
        Ok(())
    }

    /// Register virtual_create API
    pub fn register_virtual_create(&self) -> Result<()> {
        self.ensure_runtime()?;

        let create = self.lua.create_function(move |lua, (backend, monitor): (Option<String>, Option<bool>)| {
            use crate::serial_core::{VirtualConfig, VirtualSerialPair};
            use crate::serial_core::backends::BackendType;

            let backend_type = match backend.as_deref() {
                Some("pty") => BackendType::Pty,
                Some("namedpipe") => BackendType::NamedPipe,
                Some("socat") => BackendType::Socat,
                None => BackendType::detect(),
                Some(other) => {
                    return Err(mlua::Error::RuntimeError(format!(
                        "Unknown backend: {}. Available: pty, namedpipe, socat",
                        other
                    )))
                }
            };

            let config = VirtualConfig {
                backend: backend_type,
                monitor: monitor.unwrap_or(false),
                monitor_output: None,
                max_packets: 0,
                bridge_buffer_size: 8192,
            };

            // We need to spawn this in a runtime since create is async
            let rt = tokio::runtime::Runtime::new()
                .map_err(|e| mlua::Error::RuntimeError(format!("Failed to create runtime: {}", e)))?;

            let pair = rt.block_on(VirtualSerialPair::create(config))
                .map_err(|e| mlua::Error::RuntimeError(format!("Failed to create virtual pair: {}", e)))?;

            // Clone the values we need before creating the result table
            let id = pair.id.clone();
            let port_a = pair.port_a.clone();
            let port_b = pair.port_b.clone();
            let backend = format!("{:?}", pair.backend_type);
            let running = pair.is_running();

            // Return result as Lua table
            let result = lua.create_table()?;
            result.set("id", id)?;
            result.set("port_a", port_a)?;
            result.set("port_b", port_b)?;
            result.set("backend", backend)?;
            result.set("running", running)?;

            // Note: We're intentionally dropping the pair here to clean up resources
            // In a real implementation, you'd want to store it somewhere for later use
            tracing::warn!("Virtual pair created but not stored. Resources will be cleaned up immediately.");

            Ok(result)
        })?;

        self.lua.globals().set("virtual_create", create)?;
        Ok(())
    }

    /// Register virtual_stop API
    pub fn register_virtual_stop(&self) -> Result<()> {
        let stop = self.lua.create_function(move |_, _id: String| {
            // Note: This is a simplified implementation
            // In a real implementation, you'd need to manage virtual pair lifecycle
            tracing::warn!(
                "virtual_stop called but virtual pair management not implemented in Lua"
            );
            Ok(true)
        })?;

        self.lua.globals().set("virtual_stop", stop)?;
        Ok(())
    }

    /// Get the Lua instance
    pub fn lua(&self) -> &Lua {
        &self.lua
    }

    /// Register all built-in protocols (line, at_command, modbus_rtu, modbus_ascii)
    ///
    /// This is a convenience function for initializing the protocol registry with
    /// all standard protocols. It's typically called during application initialization
    /// or when setting up a new protocol registry for Lua scripts.
    ///
    /// # Example
    /// ```no_run
    /// use serial_cli::lua::LuaBindings;
    /// # async fn example() {
    /// let mut registry = serial_cli::protocol::ProtocolRegistry::new();
    /// LuaBindings::register_builtins(&mut registry).await;
    /// # }
    /// ```
    pub async fn register_builtins(registry: &mut crate::protocol::ProtocolRegistry) {
        use crate::protocol::built_in::{AtCommandProtocol, LineProtocol, ModbusProtocol};
        use crate::protocol::registry::SimpleProtocolFactory;
        use std::sync::Arc;

        registry
            .register(Arc::new(SimpleProtocolFactory::new(
                "line".to_string(),
                "Line-based protocol".to_string(),
                LineProtocol::new,
            )))
            .await;

        registry
            .register(Arc::new(SimpleProtocolFactory::new(
                "at_command".to_string(),
                "AT Command protocol".to_string(),
                AtCommandProtocol::new,
            )))
            .await;

        registry
            .register(Arc::new(SimpleProtocolFactory::new(
                "modbus_rtu".to_string(),
                "Modbus RTU protocol".to_string(),
                || ModbusProtocol::new(crate::protocol::built_in::modbus::ModbusMode::Rtu),
            )))
            .await;

        registry
            .register(Arc::new(SimpleProtocolFactory::new(
                "modbus_ascii".to_string(),
                "Modbus ASCII protocol".to_string(),
                || ModbusProtocol::new(crate::protocol::built_in::modbus::ModbusMode::Ascii),
            )))
            .await;
    }

    /// Register protocol_encode API
    pub fn register_protocol_encode(&self) -> Result<()> {
        let encode =
            self.lua
                .create_function(move |_, (protocol_name, data): (String, String)| {
                    // Simplified version without runtime creation
                    // For basic protocols, process according to their semantics
                    match protocol_name.as_str() {
                        "line" | "lines" => {
                            // Add newline only if not already present
                            if data.ends_with('\n') {
                                Ok(data)
                            } else {
                                Ok(data + "\n")
                            }
                        }
                        "at_command" => {
                            // Add CRLF only if not already present
                            if data.ends_with("\r\n") {
                                Ok(data)
                            } else {
                                Ok(data + "\r\n")
                            }
                        }
                        "modbus_rtu" => {
                            // Add Modbus CRC
                            let data_bytes = data.as_bytes();
                            let crc = Self::calculate_modbus_crc(data_bytes);
                            let mut result = data.clone();
                            result.push((crc & 0xFF) as u8 as char);
                            result.push(((crc >> 8) & 0xFF) as u8 as char);
                            Ok(result)
                        }
                        "modbus_ascii" => Ok(data.clone()), // Pass through for ASCII
                        _ => Ok(data),                      // Default: pass through
                    }
                })?;

        self.lua.globals().set("protocol_encode", encode)?;
        Ok(())
    }

    /// Calculate Modbus CRC (helper function)
    #[allow(dead_code)]
    fn calculate_modbus_crc(data: &[u8]) -> u16 {
        let mut crc: u16 = 0xFFFF;
        for &byte in data {
            crc ^= byte as u16;
            for _ in 0..8 {
                if crc & 0x0001 != 0 {
                    crc = (crc >> 1) ^ 0xA001;
                } else {
                    crc >>= 1;
                }
            }
        }
        crc
    }

    /// Register protocol_decode API
    pub fn register_protocol_decode(&self) -> Result<()> {
        let decode =
            self.lua
                .create_function(move |_, (protocol_name, data): (String, String)| {
                    // Simplified version without runtime creation
                    // For basic protocols, process according to their semantics
                    match protocol_name.as_str() {
                        "line" | "lines" => {
                            // For line protocol, return data as-is (parse doesn't trim)
                            // The protocol's parse() method just returns the data unchanged
                            Ok(data)
                        }
                        "at_command" => {
                            // For AT command protocol, return data as-is (parse doesn't trim)
                            // The protocol's parse() method just returns the data unchanged
                            Ok(data)
                        }
                        "modbus_rtu" | "modbus_ascii" => Ok(data.clone()), // Pass through for Modbus
                        _ => Ok(data),                                     // Default: pass through
                    }
                })?;

        self.lua.globals().set("protocol_decode", decode)?;
        Ok(())
    }

    /// Register protocol_list API
    pub fn register_protocol_list(&self) -> Result<()> {
        let list = self.lua.create_function(|lua, ()| {
            // Return static list of built-in protocols without creating runtime
            let result = lua.create_table()?;

            // Built-in protocols
            let builtins = [
                ("lines", "Line-based protocol (delimited by newlines)"),
                ("at_command", "AT Command protocol for modems"),
                ("modbus_rtu", "Modbus RTU protocol"),
                ("modbus_ascii", "Modbus ASCII protocol"),
            ];

            for (i, (name, description)) in builtins.iter().enumerate() {
                let proto_table = lua.create_table()?;
                proto_table.set("name", *name)?;
                proto_table.set("description", *description)?;
                proto_table.set("type", "built-in")?;
                result.set(i + 1, proto_table)?;
            }

            Ok(result)
        })?;

        self.lua.globals().set("protocol_list", list)?;
        Ok(())
    }

    /// Register protocol_info API
    pub fn register_protocol_info(&self) -> Result<()> {
        let info = self.lua.create_function(|lua, protocol_name: String| {
            // Return static information about built-in protocols without creating runtime
            let builtins = [
                ("lines", "Line-based protocol (delimited by newlines)"),
                ("at_command", "AT Command protocol for modems"),
                ("modbus_rtu", "Modbus RTU protocol"),
                ("modbus_ascii", "Modbus ASCII protocol"),
            ];

            let protocol = builtins
                .iter()
                .find(|(name, _)| *name == protocol_name)
                .ok_or_else(|| {
                    mlua::Error::RuntimeError(format!("Protocol not found: {}", protocol_name))
                })?;

            let result = lua.create_table()?;
            result.set("name", protocol.0)?;
            result.set("description", protocol.1)?;
            result.set("type", "built-in")?;
            Ok(result)
        })?;

        self.lua.globals().set("protocol_info", info)?;
        Ok(())
    }

    /// Register protocol_load API
    pub fn register_protocol_load(&self) -> Result<()> {
        use crate::protocol::ProtocolValidator;

        let load = self.lua.create_function(move |_lua, path: String| {
            let cm = crate::config::ConfigManager::load_with_fallback();

            // Validate the path exists
            let path_obj = std::path::PathBuf::from(&path);
            if !path_obj.exists() {
                return Ok((false, format!("File not found: {}", path)));
            }

            // Validate the script
            if let Err(e) = ProtocolValidator::validate_script(&path_obj) {
                return Ok((false, format!("Validation failed: {}", e)));
            }

            // Derive protocol name from filename stem
            let proto_name = path_obj
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            // Check it's not a built-in
            if crate::protocol::built_in::is_builtin_protocol(&proto_name) {
                return Ok((
                    false,
                    format!(
                        "Cannot load: '{}' is a reserved built-in protocol name",
                        proto_name
                    ),
                ));
            }

            // If already loaded, reload it instead
            if cm.get_custom_protocol(&proto_name).is_some() {
                match cm.update_custom_protocol(proto_name.clone(), path_obj.clone()) {
                    Ok(_) => {
                        let _ = cm.save(None);
                        Ok((
                            true,
                            format!("Protocol reloaded: {} (from {})", proto_name, path),
                        ))
                    }
                    Err(e) => Ok((false, format!("Failed to reload protocol: {}", e))),
                }
            } else {
                // Add to config
                match cm.add_custom_protocol(proto_name.clone(), path_obj.clone()) {
                    Ok(_) => {
                        let _ = cm.save(None);
                        Ok((
                            true,
                            format!("Protocol loaded: {} (from {})", proto_name, path),
                        ))
                    }
                    Err(e) => Ok((false, format!("Failed to load protocol: {}", e))),
                }
            }
        })?;
        self.lua.globals().set("protocol_load", load)?;
        Ok(())
    }

    /// Register protocol_unload API
    pub fn register_protocol_unload(&self) -> Result<()> {
        let unload = self.lua.create_function(move |_, name: String| {
            // Check it's not a built-in
            if crate::protocol::built_in::is_builtin_protocol(&name) {
                return Ok((false, format!("Cannot unload built-in protocol: {}", name)));
            }

            let cm = crate::config::ConfigManager::load_with_fallback();

            match cm.remove_custom_protocol(&name) {
                Ok(_) => {
                    let _ = cm.save(None);
                    Ok((true, format!("Protocol unloaded: {}", name)))
                }
                Err(e) => Ok((false, format!("Failed to unload protocol: {}", e))),
            }
        })?;
        self.lua.globals().set("protocol_unload", unload)?;
        Ok(())
    }

    /// Register protocol_reload API
    pub fn register_protocol_reload(&self) -> Result<()> {
        use crate::protocol::ProtocolValidator;

        let reload = self.lua.create_function(move |_, name: String| {
            let cm = crate::config::ConfigManager::load_with_fallback();

            // Get existing protocol path
            let existing = cm.get_custom_protocol(&name);
            let Some(proto) = existing else {
                return Ok((false, format!("Custom protocol not found: {}", name)));
            };

            let script_path = proto.path.clone();

            // Validate the script still exists
            if !script_path.exists() {
                return Ok((
                    false,
                    format!("Script file not found: {}", script_path.display()),
                ));
            }

            if let Err(e) = ProtocolValidator::validate_script(&script_path) {
                return Ok((false, format!("Script validation failed: {}", e)));
            }

            // Update in config
            match cm.update_custom_protocol(name.clone(), script_path.clone()) {
                Ok(_) => {
                    let _ = cm.save(None);
                    Ok((true, format!("Protocol reloaded: {}", name)))
                }
                Err(e) => Ok((false, format!("Failed to reload protocol: {}", e))),
            }
        })?;
        self.lua.globals().set("protocol_reload", reload)?;
        Ok(())
    }

    /// Register protocol_validate API
    pub fn register_protocol_validate(&self) -> Result<()> {
        let validate = self.lua.create_function(|_lua, path: String| {
            use crate::protocol::ProtocolValidator;

            // Validate the path exists
            let path_obj = std::path::PathBuf::from(&path);
            if !path_obj.exists() {
                // Return (false, error_message) instead of throwing error
                return Ok((false, format!("File not found: {}", path)));
            }

            // Validate the script
            match ProtocolValidator::validate_script(&path_obj) {
                Ok(_) => Ok((true, "Validation successful".to_string())),
                Err(e) => Ok((false, format!("Validation failed: {}", e))),
            }
        })?;
        self.lua.globals().set("protocol_validate", validate)?;
        Ok(())
    }
}

impl Default for LuaBindings {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bindings_creation() {
        let mut bindings = LuaBindings::new().unwrap();
        let pm = Arc::new(Mutex::new(PortManager::new()));
        bindings.set_port_manager(pm);
        assert!(bindings.register_all_apis().is_ok());
    }

    #[test]
    fn test_simple_script() {
        let bindings = LuaBindings::new().unwrap();

        let script = r#"
            local x = 10
            local y = 20
        "#;

        assert!(bindings.execute_script(script).is_ok());
    }

    #[test]
    fn test_script_with_print() {
        let bindings = LuaBindings::new().unwrap();

        let script = r#"
            print("Hello from Lua!")
        "#;

        assert!(bindings.execute_script(script).is_ok());
    }

    #[test]
    fn test_log_api() {
        let bindings = LuaBindings::new().unwrap();
        bindings.register_log_api().unwrap();

        let script = r#"
            log_info("Test info message")
        "#;

        assert!(bindings.execute_script(script).is_ok());
    }

    #[test]
    fn test_utility_apis() {
        let bindings = LuaBindings::new().unwrap();
        bindings.register_utility_apis().unwrap();

        let script = r#"
            local json_str = json_encode({test = "value"})
            assert(type(json_str) == "string", "Expected string")
            assert(string.find(json_str, "test") ~= nil, "Expected 'test' in JSON")
        "#;

        assert!(bindings.execute_script(script).is_ok());
    }

    #[test]
    fn test_json_encode() {
        let bindings = LuaBindings::new().unwrap();
        bindings.register_utility_apis().unwrap();

        let script = r#"
            -- Test simple table
            local result = json_encode({name = "test", value = 42})
            assert(type(result) == "string", "Expected string")

            -- Test array
            local arr = json_encode({1, 2, 3})
            assert(type(arr) == "string", "Expected array string")

            -- Test nested table
            local nested = json_encode({data = {x = 10, y = 20}})
            assert(type(nested) == "string", "Expected nested string")
        "#;

        assert!(bindings.execute_script(script).is_ok());
    }

    #[test]
    fn test_json_encode_pretty() {
        let bindings = LuaBindings::new().unwrap();
        bindings.register_utility_apis().unwrap();

        let script = r#"
            local compact = json_encode({test = "value"})
            local pretty = json_encode_pretty({test = "value"})

            assert(string.len(pretty) > string.len(compact), "Pretty should be longer")
            assert(string.find(pretty, "\n") ~= nil, "Pretty should contain newlines")
        "#;

        assert!(bindings.execute_script(script).is_ok());
    }

    #[test]
    fn test_json_decode() {
        let bindings = LuaBindings::new().unwrap();
        bindings.register_utility_apis().unwrap();

        let script = r#"
            local obj = json_decode('{"name": "test", "value": 42}')
            assert(type(obj) == "table", "Expected table")
            assert(obj.name == "test", "Expected name='test'")
            assert(obj.value == 42, "Expected value=42")

            local arr = json_decode('[1, 2, 3]')
            assert(type(arr) == "table", "Expected array table")
            assert(arr[1] == 1, "Expected arr[1]=1")
            assert(arr[2] == 2, "Expected arr[2]=2")
        "#;

        assert!(bindings.execute_script(script).is_ok());
    }

    #[test]
    fn test_json_roundtrip() {
        let bindings = LuaBindings::new().unwrap();
        bindings.register_utility_apis().unwrap();

        let script = r#"
            local original = {
                name = "test",
                value = 42,
                nested = {x = 10, y = 20},
                items = {1, 2, 3}
            }

            local encoded = json_encode(original)
            local decoded = json_decode(encoded)

            assert(decoded.name == original.name, "Roundtrip name mismatch")
            assert(decoded.value == original.value, "Roundtrip value mismatch")
            assert(decoded.nested.x == original.nested.x, "Roundtrip nested.x mismatch")
            assert(decoded.items[1] == original.items[1], "Roundtrip items[1] mismatch")
        "#;

        assert!(bindings.execute_script(script).is_ok());
    }

    #[test]
    fn test_serial_open_lua() {
        let mut bindings = LuaBindings::new().unwrap();
        let pm = Arc::new(Mutex::new(PortManager::new()));
        bindings.set_port_manager(pm);
        bindings.register_serial_open().unwrap();

        let script = r#"
            local ok, result = pcall(serial_open, "/dev/ttyUSB0", {baudrate = 115200})
            -- ok should be false (port doesn't exist)
            assert(ok == false, "Expected ok to be false but got " .. tostring(ok))

            -- result should be an error (can be string, userdata, or other type)
            -- The important thing is that pcall caught the error
            assert(result ~= nil, "Expected result to not be nil")

            -- Convert result to string to verify it's an error
            local result_str = tostring(result)
            assert(type(result_str) == "string", "Expected result_str to be string")
            assert(string.find(result_str, "Serial") ~= nil or
                   string.find(result_str, "not found") ~= nil,
                   "Expected error message to contain 'Serial' or 'not found', got: " .. result_str)
        "#;

        bindings.execute_script(script).unwrap();
    }

    #[test]
    fn test_serial_close_lua() {
        let mut bindings = LuaBindings::new().unwrap();
        let pm = Arc::new(Mutex::new(PortManager::new()));
        bindings.set_port_manager(pm);
        bindings.register_serial_close().unwrap();

        let script = r#"
            local ok, result = pcall(serial_close, "nonexistent-port")
            -- ok should be false (port doesn't exist)
            assert(ok == false, "Expected ok to be false but got " .. tostring(ok))

            -- result should be an error
            assert(result ~= nil, "Expected result to not be nil")

            -- Convert result to string to verify it's an error
            local result_str = tostring(result)
            assert(type(result_str) == "string", "Expected result_str to be string")
        "#;

        bindings.execute_script(script).unwrap();
    }

    #[test]
    fn test_serial_send_lua() {
        let mut bindings = LuaBindings::new().unwrap();
        let pm = Arc::new(Mutex::new(PortManager::new()));
        bindings.set_port_manager(pm);
        bindings.register_serial_send().unwrap();

        let script = r#"
            local ok, result = pcall(serial_send, "test-port", "Hello")
            -- Will fail (port doesn't exist) but tests the API
            assert(ok == false, "Expected ok to be false but got " .. tostring(ok))
            assert(result ~= nil, "Expected result to not be nil")
        "#;

        assert!(bindings.execute_script(script).is_ok());
    }

    #[test]
    fn test_serial_recv_lua() {
        let mut bindings = LuaBindings::new().unwrap();
        let pm = Arc::new(Mutex::new(PortManager::new()));
        bindings.set_port_manager(pm);
        bindings.register_serial_recv().unwrap();

        let script = r#"
            local ok, result = pcall(serial_recv, "test-port", 1000)
            -- Will fail but tests the API
            assert(ok == false, "Expected ok to be false but got " .. tostring(ok))
            assert(result ~= nil, "Expected result to not be nil")
        "#;

        assert!(bindings.execute_script(script).is_ok());
    }

    #[test]
    fn test_serial_list_lua() {
        let mut bindings = LuaBindings::new().unwrap();
        let pm = Arc::new(Mutex::new(PortManager::new()));
        bindings.set_port_manager(pm);
        bindings.register_serial_list().unwrap();

        let script = r#"
            local ports = serial_list()
            assert(type(ports) == "table", "Expected ports to be a table")
        "#;

        assert!(bindings.execute_script(script).is_ok());
    }

    #[test]
    fn test_protocol_encode_lua() {
        let bindings = LuaBindings::new().unwrap();
        bindings.register_protocol_encode().unwrap();

        let script = r#"
            -- Test lines protocol
            local encoded = protocol_encode("lines", "Hello")
            assert(type(encoded) == "string", "Expected string output")
            assert(string.sub(encoded, -1) == "\n", "Expected newline at end")

            -- Test at_command protocol
            local encoded_at = protocol_encode("at_command", "ATZ")
            assert(type(encoded_at) == "string", "Expected string output for AT command")
            assert(string.sub(encoded_at, -2) == "\r\n", "Expected CRLF at end")

            -- Test modbus_rtu protocol (pass-through)
            local encoded_modbus = protocol_encode("modbus_rtu", "\x01\x03\x00\x00\x00\x01")
            assert(type(encoded_modbus) == "string", "Expected string output for Modbus")
        "#;

        assert!(bindings.execute_script(script).is_ok());
    }

    #[test]
    fn test_protocol_encode_invalid_protocol() {
        let bindings = LuaBindings::new().unwrap();
        bindings.register_protocol_encode().unwrap();

        let script = r#"
            -- Invalid protocol should pass through data (simplified implementation)
            local result = protocol_encode("invalid_protocol", "test")
            assert(type(result) == "string", "Expected string output even for invalid protocol")
            assert(result == "test", "Expected pass-through for unknown protocol")
        "#;

        assert!(bindings.execute_script(script).is_ok());
    }

    #[test]
    fn test_protocol_decode_lua() {
        let bindings = LuaBindings::new().unwrap();
        bindings.register_protocol_decode().unwrap();

        let script = r#"
            -- Test line protocol (returns data as-is, matching actual protocol behavior)
            local decoded = protocol_decode("line", "Hello\n")
            assert(type(decoded) == "string")
            assert(decoded == "Hello\n", "Expected data to be returned as-is")

            -- Test lines protocol (alias for line)
            local decoded_lines = protocol_decode("lines", "World\n")
            assert(type(decoded_lines) == "string")
            assert(decoded_lines == "World\n", "Expected data to be returned as-is")

            -- Test at_command protocol (returns data as-is, matching actual protocol behavior)
            local decoded_at = protocol_decode("at_command", "ATZ\r\n")
            assert(type(decoded_at) == "string")
            assert(decoded_at == "ATZ\r\n", "Expected data to be returned as-is")
        "#;

        assert!(bindings.execute_script(script).is_ok());
    }

    #[test]
    fn test_protocol_decode_invalid_protocol() {
        let bindings = LuaBindings::new().unwrap();
        bindings.register_protocol_decode().unwrap();

        let script = r#"
            -- Invalid protocol should pass through data (simplified implementation)
            local result = protocol_decode("invalid_protocol", "test\n")
            assert(type(result) == "string", "Expected string output even for invalid protocol")
            assert(result == "test\n", "Expected pass-through for unknown protocol")
        "#;

        assert!(bindings.execute_script(script).is_ok());
    }

    #[test]
    fn test_protocol_list_lua() {
        let bindings = LuaBindings::new().unwrap();
        bindings.register_protocol_list().unwrap();

        let script = r#"
            local protocols = protocol_list()
            assert(type(protocols) == "table", "Expected protocols to be a table")
            -- Should have at least line, at_command, modbus_rtu, modbus_ascii
            assert(#protocols >= 4, "Expected at least 4 protocols, got " .. #protocols)
        "#;

        assert!(bindings.execute_script(script).is_ok());
    }

    #[test]
    fn test_protocol_info_lua() {
        let bindings = LuaBindings::new().unwrap();
        bindings.register_protocol_info().unwrap();

        let script = r#"
            local info = protocol_info("lines")
            assert(type(info) == "table")
            assert(info.name == "lines")
            assert(type(info.description) == "string")
            assert(info.type == "built-in")
        "#;

        assert!(bindings.execute_script(script).is_ok());
    }

    #[test]
    fn test_virtual_create_lua() {
        let bindings = LuaBindings::new().unwrap();
        bindings.register_virtual_create().unwrap();

        // Note: This test may not work on all systems due to PTY requirements
        // We'll just verify the API exists
        let script = r#"
            -- Test that the function exists
            assert(type(virtual_create) == "function", "virtual_create should be a function")
        "#;

        assert!(bindings.execute_script(script).is_ok());
    }

    #[test]
    fn test_virtual_stop_lua() {
        let bindings = LuaBindings::new().unwrap();
        bindings.register_virtual_stop().unwrap();

        let script = r#"
            -- Test that the function exists
            assert(type(virtual_stop) == "function", "virtual_stop should be a function")
        "#;

        assert!(bindings.execute_script(script).is_ok());
    }
}
