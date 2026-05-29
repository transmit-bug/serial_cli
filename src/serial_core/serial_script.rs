//! Serial port Lua script engine
//!
//! Provides a full Lua scripting environment attached to a serial port,
//! with lifecycle hooks for open/send/recv/close/timer events.
//!
//! Unlike the [`Protocol`](crate::protocol::Protocol) trait which only handles
//! encode/parse, the script engine gives users complete control over the
//! serial port lifecycle — auto-reply, heartbeat, conditional filtering, etc.

use crate::error::{Result, SerialError};
use mlua::Lua;
use std::sync::{Arc, Mutex};

/// Wrapper that asserts `mlua::Lua` is safe to Send between threads.
///
/// Safety: mlua::Lua contains raw `*mut lua_State` pointers which are not Send
/// by default. However, Lua is designed for single-threaded access — our
/// `Mutex` ensures exclusive access, making it safe to transfer ownership
/// between threads as long as only one thread accesses it at a time.
struct LuaSend(Lua);
unsafe impl Send for LuaSend {}
unsafe impl Sync for LuaSend {}

impl LuaSend {
    fn new(lua: Lua) -> Self {
        Self(lua)
    }

    fn inner(&self) -> &Lua {
        &self.0
    }
}

/// Lua script engine attached to a single serial port session.
///
/// Maintains a persistent Lua state across the port's lifetime, enabling
/// stateful scripts (counters, buffers, accumulated data, etc.).
pub struct SerialScriptEngine {
    /// Persistent Lua state — one instance per port session
    lua: Arc<Mutex<LuaSend>>,
    /// Script source (for hot-reload)
    script: String,
    /// Stored send callback — re-registered on hot-reload
    send_fn: Option<SendFn>,
    /// Timer interval in ms (0 = disabled)
    timer_interval_ms: u64,
    /// Timer task handle
    timer_task: Option<std::thread::JoinHandle<()>>,
    /// Whether the timer should stop
    timer_stop: Arc<Mutex<bool>>,
}

/// Type alias for the send function callback.
pub type SendFn = Arc<dyn Fn(&[u8]) -> Result<usize> + Send + Sync>;

impl SerialScriptEngine {
    /// Create a new script engine from Lua source code.
    ///
    /// # Errors
    /// Returns an error if the script fails syntax validation.
    pub fn new(script: &str) -> Result<Self> {
        let lua = Lua::new();

        // Validate syntax
        lua.load(script).exec().map_err(|e| {
            SerialError::Script(crate::error::ScriptError::ApiError(format!(
                "Invalid Lua script: {e}"
            )))
        })?;

        Ok(Self {
            lua: Arc::new(Mutex::new(LuaSend::new(lua))),
            script: script.to_string(),
            send_fn: None,
            timer_interval_ms: 0,
            timer_task: None,
            timer_stop: Arc::new(Mutex::new(false)),
        })
    }

    /// Initialize the Lua globals with our API functions.
    /// Delegates to [`ScriptRuntime`](crate::lua::runtime::ScriptRuntime) for unified tool registration.
    fn init_globals(lua_send: &LuaSend) -> Result<()> {
        crate::lua::runtime::ScriptRuntime::register_all(lua_send.inner())
    }

    /// Load and execute the script in the Lua state.
    pub fn load(&self) -> Result<()> {
        {
            let lua_guard = self.lua.lock().unwrap();
            let lua = lua_guard.inner();
            lua.load(&self.script).exec().map_err(|e| {
                SerialError::Script(crate::error::ScriptError::ApiError(format!(
                    "Script execution failed: {e}"
                )))
            })?;
        }
        Self::init_globals(&self.lua.lock().unwrap())
    }

    /// Register the serial_send callback in Lua globals.
    /// This must be called after `load()` and before using the engine.
    pub fn set_send_callback(&mut self, send_fn: SendFn) -> Result<()> {
        self.send_fn = Some(send_fn.clone());
        let lua_guard = self.lua.lock().unwrap();
        let lua = lua_guard.inner();
        let globals = lua.globals();

        globals.set(
            "serial_send",
            lua.create_function(move |_lua, data_table: mlua::Table| {
                let bytes = lua_table_to_bytes(&data_table)?;
                match send_fn(&bytes) {
                    Ok(n) => Ok(n as i64),
                    Err(e) => Err(mlua::Error::RuntimeError(format!(
                        "serial_send failed: {e}"
                    ))),
                }
            })?,
        )?;
        Ok(())
    }

    /// Call `on_open(port_name, config_table)` if defined.
    pub fn on_open(&self, port_name: &str, config: &crate::serial_core::SerialConfig) {
        let lua_guard = self.lua.lock().unwrap();
        let lua = lua_guard.inner();
        let globals = lua.globals();
        let Ok(func) = globals.get::<_, mlua::Function>("on_open") else {
            return;
        };

        let config_table = lua.create_table().unwrap();
        config_table
            .set("baudrate", config.baudrate as i64)
            .unwrap();
        config_table
            .set("databits", config.databits as i64)
            .unwrap();
        config_table
            .set("stopbits", config.stopbits as i64)
            .unwrap();
        config_table
            .set("timeout_ms", config.timeout_ms as i64)
            .unwrap();

        if let Err(e) = func.call::<_, ()>((port_name, config_table)) {
            tracing::error!("[script] on_open error: {e}");
        }
    }

    /// Call `on_send(data)` if defined. Returns the processed data.
    /// If the function returns `nil`, the send is intercepted (empty Vec returned).
    /// If the function doesn't exist, data is passed through unchanged.
    pub fn on_send(&self, data: &[u8]) -> Result<Vec<u8>> {
        let lua_guard = self.lua.lock().unwrap();
        let lua = lua_guard.inner();
        let globals = lua.globals();
        let Ok(func) = globals.get::<_, mlua::Function>("on_send") else {
            return Ok(data.to_vec());
        };

        let data_table = bytes_to_lua_table(lua, data)?;
        let result = func.call::<_, mlua::Value>(data_table);

        // All data conversion must happen inside the lock scope
        match result {
            Ok(mlua::Value::Table(table)) => {
                Ok(lua_table_to_bytes(&table).unwrap_or_else(|_| data.to_vec()))
            }
            Ok(mlua::Value::Nil) => Ok(vec![]), // Intercept: don't send
            Ok(mlua::Value::String(s)) => Ok(s.as_bytes().to_vec()),
            _ => Ok(data.to_vec()), // Pass through
        }
    }

    /// Call `on_recv(data)` if defined. Returns the processed data.
    /// If the function returns `nil`, the data is suppressed (empty Vec returned).
    /// The callback may also call `serial_send()` for auto-reply.
    pub fn on_recv(&self, data: &[u8]) -> Vec<u8> {
        let lua_guard = self.lua.lock().unwrap();
        let lua = lua_guard.inner();
        let globals = lua.globals();
        let Ok(func) = globals.get::<_, mlua::Function>("on_recv") else {
            return data.to_vec();
        };

        let data_table = match bytes_to_lua_table(lua, data) {
            Ok(t) => t,
            Err(e) => {
                tracing::error!("[script] on_recv data conversion error: {e}");
                return data.to_vec();
            }
        };

        let result = func.call::<_, mlua::Value>(data_table);

        // All data conversion must happen inside the lock scope
        match result {
            Ok(mlua::Value::Table(table)) => lua_table_to_bytes(&table).unwrap_or(data.to_vec()),
            Ok(mlua::Value::Nil) => vec![], // Suppress data
            Ok(mlua::Value::String(s)) => s.as_bytes().to_vec(),
            _ => data.to_vec(), // Pass through
        }
    }

    /// Call `on_close()` if defined.
    pub fn on_close(&self) {
        let lua_guard = self.lua.lock().unwrap();
        let lua = lua_guard.inner();
        let globals = lua.globals();
        let Ok(func) = globals.get::<_, mlua::Function>("on_close") else {
            return;
        };

        if let Err(e) = func.call::<_, ()>(()) {
            tracing::error!("[script] on_close error: {e}");
        }
    }

    /// Start the timer if `on_timer()` is defined and returns a positive interval.
    pub fn start_timer(&mut self) {
        // Check if on_timer exists
        let has_timer = {
            let lua_guard = self.lua.lock().unwrap();
            let lua = lua_guard.inner();
            let globals = lua.globals();
            #[allow(clippy::let_and_return)]
            let has = globals.get::<_, mlua::Function>("on_timer").is_ok();
            has
        };
        if !has_timer {
            tracing::debug!("[script] on_timer not defined, timer disabled");
            return;
        }

        // Check initial interval
        let interval = self.check_timer_interval();
        if interval == 0 {
            tracing::debug!("[script] on_timer returned 0, timer disabled");
            return;
        }
        self.timer_interval_ms = interval;

        let lua_clone = self.lua.clone();
        let stop_flag = self.timer_stop.clone();
        *stop_flag.lock().unwrap() = false;

        self.timer_task = Some(std::thread::spawn(move || {
            let mut interval_ms = interval;

            loop {
                std::thread::sleep(std::time::Duration::from_millis(interval_ms));

                // Check stop flag
                if *stop_flag.lock().unwrap() {
                    break;
                }

                let result = {
                    let lua_guard = lua_clone.lock().unwrap();
                    let lua = lua_guard.inner();
                    let globals = lua.globals();
                    let Ok(func) = globals.get::<_, mlua::Function>("on_timer") else {
                        break;
                    };
                    // Convert to owned value inside the lock scope
                    let timer_result = match func.call::<_, mlua::Value>(()) {
                        Ok(mlua::Value::Integer(ms)) if ms > 0 => Some(ms as u64),
                        Ok(mlua::Value::Number(ms)) if ms > 0.0 => Some(ms as u64),
                        Ok(mlua::Value::Nil) => {
                            tracing::debug!("[script] on_timer returned nil, stopping timer");
                            None
                        }
                        Err(e) => {
                            tracing::error!("[script] on_timer error: {e}");
                            None
                        }
                        _ => Some(interval_ms),
                    };
                    timer_result
                };
                // Lock is released here

                match result {
                    Some(ms) if ms > 0 => {
                        interval_ms = ms;
                    }
                    None => break,
                    _ => {}
                }
                if *stop_flag.lock().unwrap() {
                    break;
                }
            }
            tracing::debug!("[script] timer thread stopped");
        }));
    }

    /// Check the timer interval from on_timer() without running the full loop.
    fn check_timer_interval(&self) -> u64 {
        let lua_guard = self.lua.lock().unwrap();
        let lua = lua_guard.inner();
        let globals = lua.globals();
        let Ok(func) = globals.get::<_, mlua::Function>("on_timer") else {
            return 0;
        };

        let result = func.call::<_, mlua::Value>(());
        // result is still in scope; all borrows tied to lua_guard
        // Convert to owned value before end of scope
        match result {
            Ok(mlua::Value::Integer(ms)) if ms > 0 => ms as u64,
            Ok(mlua::Value::Number(ms)) if ms > 0.0 => ms as u64,
            _ => 0,
        }
    }

    /// Stop the timer task.
    pub fn stop_timer(&mut self) {
        *self.timer_stop.lock().unwrap() = true;
        if let Some(task) = self.timer_task.take() {
            let _ = task.join();
        }
        self.timer_interval_ms = 0;
    }

    /// Reload the script from source (hot-reload).
    pub fn reload(&mut self, new_script: &str) -> Result<()> {
        self.stop_timer();

        let lua = Lua::new();
        lua.load(new_script).exec().map_err(|e| {
            SerialError::Script(crate::error::ScriptError::ApiError(format!(
                "Invalid Lua script: {e}"
            )))
        })?;

        *self.lua.lock().unwrap() = LuaSend::new(lua);
        self.script = new_script.to_string();

        self.load()?;

        // Re-register the send callback on the new Lua instance
        if let Some(ref send_fn) = self.send_fn.clone() {
            self.set_send_callback(send_fn.clone())?;
        }

        Ok(())
    }

    /// Get the timer interval in ms (0 = disabled).
    pub fn timer_interval_ms(&self) -> u64 {
        self.timer_interval_ms
    }

    /// Check if the script defines a given callback.
    pub fn has_callback(&self, name: &str) -> bool {
        let lua_guard = self.lua.lock().unwrap();
        let has = lua_guard
            .inner()
            .globals()
            .get::<_, mlua::Function>(name)
            .is_ok();
        has
    }

    /// Discover all UI actions in the script
    ///
    /// Scans for functions with the `action_` prefix and returns their metadata.
    pub async fn discover_actions(&self) -> Result<Vec<crate::lua::ui_actions::UiAction>> {
        let lua_guard = self.lua.lock().unwrap();
        crate::lua::ui_actions::discover_actions(lua_guard.inner())
    }

    /// Execute a UI action function by name.
    ///
    /// Calls the specified Lua function and returns its result as a string.
    pub async fn execute_action(&self, function_name: &str) -> Result<String> {
        let lua_guard = self.lua.lock().unwrap();
        crate::lua::ui_actions::execute_action_string(lua_guard.inner(), function_name)
    }
}

impl Drop for SerialScriptEngine {
    fn drop(&mut self) {
        self.stop_timer();
    }
}

/// Convert a Lua table (1-indexed array of bytes) to a Vec<u8>.
fn lua_table_to_bytes(table: &mlua::Table) -> mlua::Result<Vec<u8>> {
    let len = table.len().unwrap_or(0) as usize;
    let mut bytes = Vec::with_capacity(len);
    for i in 1..=len {
        let byte: u8 = table.get(i).unwrap_or(0);
        bytes.push(byte);
    }
    Ok(bytes)
}

/// Convert a byte slice to a Lua table (1-indexed array).
fn bytes_to_lua_table<'lua>(lua: &'lua Lua, data: &[u8]) -> mlua::Result<mlua::Table<'lua>> {
    let table = lua.create_table()?;
    for (i, &byte) in data.iter().enumerate() {
        table.set(i + 1, byte)?;
    }
    Ok(table)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation_valid_script() {
        let script = r#"
            function on_open(port, config)
                log_info("Opened: " .. port)
            end
            function on_send(data)
                return data
            end
            function on_recv(data)
                return data
            end
            function on_close()
                log_info("Closed")
            end
        "#;
        let engine = SerialScriptEngine::new(script);
        assert!(engine.is_ok());
    }

    #[test]
    fn test_engine_creation_invalid_script() {
        let script = "this is not valid lua {{{";
        let engine = SerialScriptEngine::new(script);
        assert!(engine.is_err());
    }

    #[test]
    fn test_on_send_passthrough() {
        let script = r#"
            function on_send(data)
                return data
            end
        "#;
        let engine = SerialScriptEngine::new(script).unwrap();
        engine.load().unwrap();

        let data = vec![0x01, 0x02, 0x03];
        let result = engine.on_send(&data).unwrap();
        assert_eq!(result, data);
    }

    #[test]
    fn test_on_send_intercept() {
        let script = r#"
            function on_send(data)
                return nil  -- block all sends
            end
        "#;
        let engine = SerialScriptEngine::new(script).unwrap();
        engine.load().unwrap();

        let data = vec![0x01, 0x02, 0x03];
        let result = engine.on_send(&data).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_on_send_modify() {
        let script = r#"
            function on_send(data)
                -- Append 0x0A
                table.insert(data, 0x0A)
                return data
            end
        "#;
        let engine = SerialScriptEngine::new(script).unwrap();
        engine.load().unwrap();

        let data = vec![0x41, 0x42];
        let result = engine.on_send(&data).unwrap();
        assert_eq!(result, vec![0x41, 0x42, 0x0A]);
    }

    #[test]
    fn test_on_recv_suppress() {
        let script = r#"
            function on_recv(data)
                return nil  -- suppress all received data
            end
        "#;
        let engine = SerialScriptEngine::new(script).unwrap();
        engine.load().unwrap();

        let data = vec![0x01, 0x02];
        let result = engine.on_recv(&data);
        assert!(result.is_empty());
    }

    #[test]
    fn test_no_callback_passthrough() {
        let script = "-- empty script";
        let engine = SerialScriptEngine::new(script).unwrap();
        engine.load().unwrap();

        let data = vec![0x01, 0x02, 0x03];
        // No on_send defined → passthrough
        let result = engine.on_send(&data).unwrap();
        assert_eq!(result, data);
        // No on_recv defined → passthrough
        let result = engine.on_recv(&data);
        assert_eq!(result, data);
    }

    #[test]
    fn test_bytes_to_string_roundtrip() {
        let script = r#"
            function on_recv(data)
                local str = bytes_to_string(data)
                return string_to_bytes(string.upper(str))
            end
        "#;
        let engine = SerialScriptEngine::new(script).unwrap();
        engine.load().unwrap();

        let data = b"hello".to_vec();
        let result = engine.on_recv(&data);
        assert_eq!(result, b"HELLO".to_vec());
    }

    #[test]
    fn test_hex_encode_decode() {
        let script = r#"
            function on_recv(data)
                local hex = hex_encode(data)
                return hex_decode(hex)
            end
        "#;
        let engine = SerialScriptEngine::new(script).unwrap();
        engine.load().unwrap();

        let data = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let result = engine.on_recv(&data);
        assert_eq!(result, data);
    }
}
