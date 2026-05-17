//! Lua API bindings
//!
//! This module provides the Rust API bindings for Lua scripts.
//!
//! # Async safety
//!
//! All serial APIs that need async access run in a separate thread with its own
//! tokio runtime to avoid panicking when Lua executes inside an existing tokio context
//! (the "cannot block current thread" problem).

use crate::error::{Result, SerialError};
use crate::lua::runtime::ScriptRuntime;
use crate::serial_core::PortManager;
use mlua::{Function, Lua, Value};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Lua API bindings
pub struct LuaBindings {
    lua: Lua,
    port_manager: Option<Arc<Mutex<PortManager>>>,
}

impl LuaBindings {
    /// Create new Lua bindings
    pub fn new() -> Result<Self> {
        let lua = Lua::new();
        Ok(Self {
            lua,
            port_manager: None,
        })
    }

    /// Register logging API via ScriptRuntime.
    pub fn register_log_api(&self) -> Result<()> {
        ScriptRuntime::register_log(&self.lua)
    }

    /// Register utility APIs via ScriptRuntime.
    /// This provides json_encode, json_encode_pretty, json_decode, sleep_ms.
    pub fn register_utility_apis(&self) -> Result<()> {
        ScriptRuntime::register_json(&self.lua)?;
        ScriptRuntime::register_time(&self.lua)?;
        Ok(())
    }

    /// Register all APIs
    pub fn register_all_apis(&self) -> Result<()> {
        // Tool functions (log, json, hex, string/bytes, time)
        ScriptRuntime::register_all(&self.lua)?;

        // Serial APIs (only if port manager is initialized)
        if self.port_manager.is_some() {
            self.register_serial_open()?;
            self.register_serial_close()?;
            self.register_serial_send()?;
            self.register_serial_recv()?;
            self.register_serial_list()?;
        }

        // Protocol APIs
        self.register_protocol_encode()?;
        self.register_protocol_decode()?;
        self.register_protocol_list()?;
        self.register_protocol_info()?;

        // Protocol management APIs
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

    /// Discover all UI actions in the Lua state
    ///
    /// Scans for functions with the `action_` prefix and returns their metadata.
    pub fn discover_actions(&self) -> Result<Vec<crate::lua::ui_actions::UiAction>> {
        crate::lua::ui_actions::discover_actions(&self.lua)
    }

    /// Execute a UI action function by name
    ///
    /// Calls the specified Lua function and returns its result as a string.
    pub fn execute_action(&self, function_name: &str) -> Result<String> {
        crate::lua::ui_actions::execute_action_string(&self.lua, function_name)
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

    // ── Serial API registrations ──────────────────────────────────────────

    /// Register serial_open API
    pub fn register_serial_open(&self) -> Result<()> {
        let port_manager = self
            .port_manager
            .clone()
            .ok_or_else(|| SerialError::Config("PortManager not initialized".to_string()))?;

        let open = self.lua.create_function(
            move |_, (port_name, config_table): (String, mlua::Table)| {
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

                let parity = match parity_str.to_lowercase().as_str() {
                    "odd" => crate::serial_core::Parity::Odd,
                    "even" => crate::serial_core::Parity::Even,
                    _ => crate::serial_core::Parity::None,
                };

                let flow_control = match flow_control_str.to_lowercase().as_str() {
                    "software" => crate::serial_core::FlowControl::Software,
                    "hardware" => crate::serial_core::FlowControl::Hardware,
                    _ => crate::serial_core::FlowControl::None,
                };

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

                let pm = port_manager.clone();
                run_in_separate_runtime(|| async move {
                    let pm_guard = pm.lock().await;
                    pm_guard.open_port(&port_name, config).await
                })
            },
        )?;

        self.lua.globals().set("serial_open", open)?;
        Ok(())
    }

    /// Register serial_close API
    pub fn register_serial_close(&self) -> Result<()> {
        let port_manager = self
            .port_manager
            .clone()
            .ok_or_else(|| SerialError::Config("PortManager not initialized".to_string()))?;

        let close = self.lua.create_function(move |_, port_id: String| {
            let pm = port_manager.clone();
            run_in_separate_runtime(|| async move {
                let pm_guard = pm.lock().await;
                pm_guard.close_port(&port_id).await
            })
        })?;

        self.lua.globals().set("serial_close", close)?;
        Ok(())
    }

    /// Register serial_send API
    pub fn register_serial_send(&self) -> Result<()> {
        let port_manager = self
            .port_manager
            .clone()
            .ok_or_else(|| SerialError::Config("PortManager not initialized".to_string()))?;

        let send = self
            .lua
            .create_function(move |_, (port_id, data): (String, String)| {
                let pm = port_manager.clone();
                run_in_separate_runtime(|| async move {
                    let pm_guard = pm.lock().await;
                    let port_handle = pm_guard.get_port(&port_id).await?;
                    let mut port = port_handle.lock().await;
                    port.write(data.as_bytes())
                })
            })?;

        self.lua.globals().set("serial_send", send)?;
        Ok(())
    }

    /// Register serial_recv API
    pub fn register_serial_recv(&self) -> Result<()> {
        let port_manager = self
            .port_manager
            .clone()
            .ok_or_else(|| SerialError::Config("PortManager not initialized".to_string()))?;

        let recv = self
            .lua
            .create_function(move |_, (port_id, _timeout_ms): (String, u64)| {
                let pm = port_manager.clone();
                run_in_separate_runtime(move || async move {
                    let pm_guard = pm.lock().await;
                    let port_handle = pm_guard.get_port(&port_id).await?;

                    // Use spawn_blocking for the synchronous read
                    let read_result = tokio::task::spawn_blocking(move || {
                        let mut port = port_handle.blocking_lock();
                        let mut buffer = vec![0u8; 4096];
                        let n = port.read(&mut buffer)?;
                        buffer.truncate(n);
                        Ok::<_, crate::error::SerialError>(
                            String::from_utf8_lossy(&buffer).to_string(),
                        )
                    })
                    .await
                    .map_err(|e| {
                        crate::error::SerialError::Serial(crate::error::SerialPortError::IoError(
                            e.to_string(),
                        ))
                    })??;

                    Ok(read_result)
                })
            })?;

        self.lua.globals().set("serial_recv", recv)?;
        Ok(())
    }

    /// Register serial_list API
    pub fn register_serial_list(&self) -> Result<()> {
        let port_manager = self
            .port_manager
            .clone()
            .ok_or_else(|| SerialError::Config("PortManager not initialized".to_string()))?;

        let list = self.lua.create_function(move |lua, ()| {
            let pm = port_manager.clone();
            let ports = run_in_separate_runtime(|| async move {
                let pm_guard = pm.lock().await;
                pm_guard.list_ports()
            })
            .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;

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

    // ── Virtual port API registrations ────────────────────────────────────

    /// Register virtual_create API
    pub fn register_virtual_create(&self) -> Result<()> {
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

            let pair = run_in_separate_runtime(|| VirtualSerialPair::create(config))
                .map_err(|e| mlua::Error::RuntimeError(format!("Failed to create virtual pair: {}", e)))?;

            let id = pair.id.clone();
            let port_a = pair.port_a.clone();
            let port_b = pair.port_b.clone();
            let backend = format!("{:?}", pair.backend_type);
            let running = pair.is_running();

            let result = lua.create_table()?;
            result.set("id", id)?;
            result.set("port_a", port_a)?;
            result.set("port_b", port_b)?;
            result.set("backend", backend)?;
            result.set("running", running)?;

            tracing::warn!("Virtual pair created but not stored. Resources will be cleaned up immediately.");

            Ok(result)
        })?;

        self.lua.globals().set("virtual_create", create)?;
        Ok(())
    }

    /// Register virtual_stop API
    pub fn register_virtual_stop(&self) -> Result<()> {
        let stop = self.lua.create_function(move |_, _id: String| {
            tracing::warn!(
                "virtual_stop called but virtual pair management not implemented in Lua"
            );
            Ok(true)
        })?;

        self.lua.globals().set("virtual_stop", stop)?;
        Ok(())
    }

    // ── Protocol API registrations ────────────────────────────────────────

    /// Register all built-in protocols
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

    /// Get the Lua instance
    pub fn lua(&self) -> &Lua {
        &self.lua
    }

    // Protocol encode/decode/load/unload/validate registrations
    // (unchanged from before — they are synchronous, no async issues)

    /// Register protocol_encode API
    pub fn register_protocol_encode(&self) -> Result<()> {
        let encode =
            self.lua
                .create_function(move |_, (protocol_name, data): (String, String)| {
                    match protocol_name.as_str() {
                        "line" | "lines" => {
                            if data.ends_with('\n') {
                                Ok(data)
                            } else {
                                Ok(data + "\n")
                            }
                        }
                        "at_command" => {
                            if data.ends_with("\r\n") {
                                Ok(data)
                            } else {
                                Ok(data + "\r\n")
                            }
                        }
                        "modbus_rtu" => {
                            let data_bytes = data.as_bytes();
                            let crc = Self::calculate_modbus_crc(data_bytes);
                            let mut result = data.clone();
                            result.push((crc & 0xFF) as u8 as char);
                            result.push(((crc >> 8) & 0xFF) as u8 as char);
                            Ok(result)
                        }
                        "modbus_ascii" => Ok(data.clone()),
                        _ => Ok(data),
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
                    match protocol_name.as_str() {
                        "line" | "lines" => Ok(data),
                        "at_command" => Ok(data),
                        "modbus_rtu" | "modbus_ascii" => Ok(data.clone()),
                        _ => Ok(data),
                    }
                })?;

        self.lua.globals().set("protocol_decode", decode)?;
        Ok(())
    }

    /// Register protocol_list API
    pub fn register_protocol_list(&self) -> Result<()> {
        let list = self.lua.create_function(|lua, ()| {
            let result = lua.create_table()?;
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

            let path_obj = std::path::PathBuf::from(&path);
            if !path_obj.exists() {
                return Ok((false, format!("File not found: {}", path)));
            }

            if let Err(e) = ProtocolValidator::validate_script(&path_obj) {
                return Ok((false, format!("Validation failed: {}", e)));
            }

            let proto_name = path_obj
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            if crate::protocol::built_in::is_builtin_protocol(&proto_name) {
                return Ok((
                    false,
                    format!(
                        "Cannot load: '{}' is a reserved built-in protocol name",
                        proto_name
                    ),
                ));
            }

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

            let existing = cm.get_custom_protocol(&name);
            let Some(proto) = existing else {
                return Ok((false, format!("Custom protocol not found: {}", name)));
            };

            let script_path = proto.path.clone();

            if !script_path.exists() {
                return Ok((
                    false,
                    format!("Script file not found: {}", script_path.display()),
                ));
            }

            if let Err(e) = ProtocolValidator::validate_script(&script_path) {
                return Ok((false, format!("Script validation failed: {}", e)));
            }

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

            let path_obj = std::path::PathBuf::from(&path);
            if !path_obj.exists() {
                return Ok((false, format!("File not found: {}", path)));
            }

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

// ── Async helper ──────────────────────────────────────────────────────────

/// Run an async operation in a separate thread with its own tokio runtime.
///
/// This avoids the "cannot start a runtime from within a runtime context" panic
/// that occurs when `Handle::block_on` is called from inside an existing tokio
/// runtime (e.g., when Lua callbacks execute inside a Tauri async handler).
fn run_in_separate_runtime<F, Fut, T>(f: F) -> mlua::Result<T>
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<T>> + Send + 'static,
    T: Send + 'static,
{
    let join_handle = std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .expect("create separate runtime for Lua serial API");
        rt.block_on(f())
    });
    join_handle
        .join()
        .map_err(|_| mlua::Error::RuntimeError("serial API operation panicked".to_string()))?
        .map_err(|e| mlua::Error::RuntimeError(e.to_string()))
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serial_core::PortManager;

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
            local result = json_encode({name = "test", value = 42})
            assert(type(result) == "string", "Expected string")

            local arr = json_encode({1, 2, 3})
            assert(type(arr) == "string", "Expected array string")

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
            assert(ok == false, "Expected ok to be false but got " .. tostring(ok))
            assert(result ~= nil, "Expected result to not be nil")
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
            assert(ok == false, "Expected ok to be false but got " .. tostring(ok))
            assert(result ~= nil, "Expected result to not be nil")
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
            local encoded = protocol_encode("lines", "Hello")
            assert(type(encoded) == "string", "Expected string output")
            assert(string.sub(encoded, -1) == "\n", "Expected newline at end")

            local encoded_at = protocol_encode("at_command", "ATZ")
            assert(type(encoded_at) == "string", "Expected string output for AT command")
            assert(string.sub(encoded_at, -2) == "\r\n", "Expected CRLF at end")

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
            local decoded = protocol_decode("line", "Hello\n")
            assert(type(decoded) == "string")
            assert(decoded == "Hello\n", "Expected data to be returned as-is")

            local decoded_lines = protocol_decode("lines", "World\n")
            assert(type(decoded_lines) == "string")
            assert(decoded_lines == "World\n", "Expected data to be returned as-is")

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

        let script = r#"
            assert(type(virtual_create) == "function", "virtual_create should be a function")
        "#;

        assert!(bindings.execute_script(script).is_ok());
    }

    #[test]
    fn test_virtual_stop_lua() {
        let bindings = LuaBindings::new().unwrap();
        bindings.register_virtual_stop().unwrap();

        let script = r#"
            assert(type(virtual_stop) == "function", "virtual_stop should be a function")
        "#;

        assert!(bindings.execute_script(script).is_ok());
    }
}
