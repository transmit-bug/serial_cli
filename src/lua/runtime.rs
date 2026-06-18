//! Unified Lua script runtime
//!
//! Provides a single place to register all tool functions on any `mlua::Lua` instance.
//! This eliminates the duplication across [`SerialScriptEngine`](crate::serial_core::serial_script::SerialScriptEngine),
//! [`LuaBindings`], and [`LuaStdLib`].
//!
//! # Performance Optimization
//!
//! This module includes a thread-local Lua state pool for reusing `Lua` instances,
//! significantly reducing overhead in repeated executions.
//!
//! # Usage
//!
//! ```ignore
//! use serial_cli::lua::runtime::ScriptRuntime;
//! use mlua::Lua;
//!
//! let lua = Lua::new();
//! ScriptRuntime::register_all(&lua)?;
//! lua.load("log_info('hello')").exec()?;
//! ```

use crate::error::Result;
use mlua::{Lua, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Thread-local Lua state pool for reusing Lua instances
///
/// Creating a new `Lua` instance is expensive. This pool allows reusing instances
/// within the same thread, significantly reducing overhead.
pub struct LuaStatePool {
    pool: RefCell<Vec<Lua>>,
    max_size: usize,
}

impl LuaStatePool {
    /// Create a new pool with the specified maximum size
    pub fn new(max_size: usize) -> Self {
        Self {
            pool: RefCell::new(Vec::with_capacity(max_size)),
            max_size,
        }
    }

    /// Acquire a Lua instance from the pool, or create a new one if empty
    pub fn acquire(&self) -> Lua {
        let mut pool = self.pool.borrow_mut();
        pool.pop().unwrap_or_else(Lua::new)
    }

    /// Release a Lua instance back to the pool
    pub fn release(&self, lua: Lua) {
        let mut pool = self.pool.borrow_mut();
        if pool.len() < self.max_size {
            pool.push(lua);
        }
        // If pool is full, just drop the Lua instance
    }

    /// Get the current number of available instances in the pool
    pub fn available(&self) -> usize {
        self.pool.borrow().len()
    }
}

impl Default for LuaStatePool {
    fn default() -> Self {
        Self::new(10) // Default pool size
    }
}

// Thread-local storage for Lua state pools
// Each thread maintains its own pool of Lua instances.
thread_local! {
    static LUA_POOL: LuaStatePool = LuaStatePool::new(10);
}

/// Acquire a Lua instance from the thread-local pool
pub fn acquire_lua() -> Lua {
    LUA_POOL.with(|pool| pool.acquire())
}

/// Release a Lua instance back to the thread-local pool
pub fn release_lua(lua: Lua) {
    LUA_POOL.with(|pool| pool.release(lua))
}

/// Script cache for avoiding redundant parsing and execution
///
/// Caches the result of script execution based on script content hash.
/// This is useful for scripts that are executed repeatedly with the same input.
#[derive(Clone)]
pub struct ScriptCache {
    cache: Arc<Mutex<HashMap<String, bool>>>,
}

impl ScriptCache {
    /// Create a new empty cache
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Check if a script has been executed before
    pub fn contains(&self, script: &str) -> bool {
        let hash = self.compute_hash(script);
        self.cache.lock().unwrap().contains_key(&hash)
    }

    /// Mark a script as executed
    pub fn mark_executed(&self, script: &str) {
        let hash = self.compute_hash(script);
        self.cache.lock().unwrap().insert(hash, true);
    }

    /// Clear the cache
    pub fn clear(&self) {
        self.cache.lock().unwrap().clear();
    }

    /// Get the number of cached scripts
    pub fn len(&self) -> usize {
        self.cache.lock().unwrap().len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.lock().unwrap().is_empty()
    }

    /// Compute a hash of the script content
    fn compute_hash(&self, source: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        source.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

impl Default for ScriptCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Unified Lua runtime for registering all tool functions.
///
/// All methods operate on a borrowed `&Lua` — no ownership or wrapper state.
/// This makes it composable: register on any Lua instance at any time.
pub struct ScriptRuntime;

impl ScriptRuntime {
    /// Register ALL tool functions on a Lua instance.
    ///
    /// This is the one-stop call for setting up a Lua environment with
    /// all Serial CLI APIs: logging, JSON, hex, string/bytes conversion,
    /// and time utilities.
    pub fn register_all(lua: &Lua) -> Result<()> {
        Self::register_log(lua)?;
        Self::register_json(lua)?;
        Self::register_hex(lua)?;
        Self::register_string_bytes(lua)?;
        Self::register_time(lua)?;
        Ok(())
    }

    /// Register logging functions: `log_info`, `log_debug`, `log_warn`, `log_error`.
    pub fn register_log(lua: &Lua) -> Result<()> {
        let globals = lua.globals();

        globals.set(
            "log_info",
            lua.create_function(|_, msg: String| {
                tracing::info!("[script] {}", msg);
                Ok(())
            })?,
        )?;
        globals.set(
            "log_debug",
            lua.create_function(|_, msg: String| {
                tracing::debug!("[script] {}", msg);
                Ok(())
            })?,
        )?;
        globals.set(
            "log_warn",
            lua.create_function(|_, msg: String| {
                tracing::warn!("[script] {}", msg);
                Ok(())
            })?,
        )?;
        globals.set(
            "log_error",
            lua.create_function(|_, msg: String| {
                tracing::error!("[script] {}", msg);
                Ok(())
            })?,
        )?;

        Ok(())
    }

    /// Register JSON functions: `json_encode`, `json_encode_pretty`, `json_decode`.
    pub fn register_json(lua: &Lua) -> Result<()> {
        let globals = lua.globals();

        globals.set(
            "json_encode",
            lua.create_function(|_, value: Value| {
                let json_value = lua_value_to_json(value)?;
                serde_json::to_string(&json_value)
                    .map_err(|e| mlua::Error::RuntimeError(format!("JSON encode error: {}", e)))
            })?,
        )?;
        globals.set(
            "json_encode_pretty",
            lua.create_function(|_, value: Value| {
                let json_value = lua_value_to_json(value)?;
                serde_json::to_string_pretty(&json_value)
                    .map_err(|e| mlua::Error::RuntimeError(format!("JSON encode error: {}", e)))
            })?,
        )?;
        globals.set(
            "json_decode",
            lua.create_function(|lua, json_str: String| {
                let v: serde_json::Value = serde_json::from_str(&json_str)
                    .map_err(|e| mlua::Error::RuntimeError(format!("JSON decode error: {}", e)))?;
                json_to_lua_value(lua, v)
            })?,
        )?;

        Ok(())
    }

    /// Register hex functions: `hex_encode`, `hex_decode`, `hex_to_bytes`.
    pub fn register_hex(lua: &Lua) -> Result<()> {
        let globals = lua.globals();

        globals.set(
            "hex_encode",
            lua.create_function(|_, data_table: mlua::Table| {
                let bytes = lua_table_to_bytes(&data_table)?;
                Ok(bytes
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<String>())
            })?,
        )?;

        globals.set(
            "hex_decode",
            lua.create_function(|lua, hex: String| {
                let bytes = Self::hex_decode(&hex)?;
                let table = lua.create_table()?;
                for (i, &byte) in bytes.iter().enumerate() {
                    table.set(i + 1, byte)?;
                }
                Ok(table)
            })?,
        )?;

        globals.set(
            "hex_to_bytes",
            lua.create_function(|lua, hex: String| {
                let bytes = Self::hex_decode(&hex)?;
                let result = lua.create_table()?;
                for (i, byte) in bytes.iter().enumerate() {
                    result.set(i + 1, *byte)?;
                }
                Ok(result)
            })?,
        )?;

        Ok(())
    }

    /// Register string/bytes conversion: `bytes_to_string`, `string_to_bytes`, `bytes_to_hex`.
    pub fn register_string_bytes(lua: &Lua) -> Result<()> {
        let globals = lua.globals();

        globals.set(
            "bytes_to_string",
            lua.create_function(|_, data_table: mlua::Table| {
                let bytes = lua_table_to_bytes(&data_table)?;
                Ok(String::from_utf8_lossy(&bytes).to_string())
            })?,
        )?;

        globals.set(
            "string_to_bytes",
            lua.create_function(|lua, s: String| {
                let table = lua.create_table()?;
                for (i, byte) in s.bytes().enumerate() {
                    table.set(i + 1, byte)?;
                }
                Ok(table)
            })?,
        )?;

        globals.set(
            "bytes_to_hex",
            lua.create_function(|_, bytes: Value| {
                let bytes_vec = match bytes {
                    Value::String(s) => s.to_str().unwrap().as_bytes().to_vec(),
                    Value::Table(t) => {
                        let mut vec = Vec::new();
                        for pair in t.pairs::<usize, u8>() {
                            let (_, byte) = pair.map_err(|e| {
                                mlua::Error::RuntimeError(format!("table iteration error: {}", e))
                            })?;
                            vec.push(byte);
                        }
                        vec
                    }
                    _ => {
                        return Err(mlua::Error::RuntimeError(
                            "Expected string or table".to_string(),
                        ))
                    }
                };
                Ok(bytes_vec
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<String>())
            })?,
        )?;

        Ok(())
    }

    /// Register time functions: `sleep_ms`, `time_now`.
    pub fn register_time(lua: &Lua) -> Result<()> {
        let globals = lua.globals();

        globals.set(
            "sleep_ms",
            lua.create_function(|_, ms: u64| {
                std::thread::sleep(std::time::Duration::from_millis(ms));
                Ok(())
            })?,
        )?;

        globals.set(
            "time_now",
            lua.create_function(|_, _: ()| {
                Ok(std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0))
            })?,
        )?;

        Ok(())
    }

    /// Decode a hex string to bytes (shared helper).
    fn hex_decode(hex: &str) -> mlua::Result<Vec<u8>> {
        let hex = hex.replace([' ', ':', '-'], "");
        if hex.is_empty() {
            return Ok(vec![]);
        }
        if !hex.len().is_multiple_of(2) {
            return Err(mlua::Error::RuntimeError(
                "Hex string must have even length".to_string(),
            ));
        }
        if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(mlua::Error::RuntimeError(
                "Invalid hex string: contains non-hex characters".to_string(),
            ));
        }
        let bytes: Vec<u8> = (0..hex.len())
            .step_by(2)
            .filter_map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
            .collect();
        Ok(bytes)
    }
}

// ── Shared helper functions ──────────────────────────────────────────────

/// Convert a Lua table (1-indexed array of bytes) to a Vec<u8>.
pub fn lua_table_to_bytes(table: &mlua::Table) -> mlua::Result<Vec<u8>> {
    let len = table.len().unwrap_or(0) as usize;
    let mut bytes = Vec::with_capacity(len);
    for i in 1..=len {
        let byte: u8 = table.get(i).unwrap_or(0);
        bytes.push(byte);
    }
    Ok(bytes)
}

/// Convert a byte slice to a Lua table (1-indexed array).
pub fn bytes_to_lua_table<'lua>(lua: &'lua Lua, data: &[u8]) -> mlua::Result<mlua::Table<'lua>> {
    let table = lua.create_table()?;
    for (i, &byte) in data.iter().enumerate() {
        table.set(i + 1, byte)?;
    }
    Ok(table)
}

/// Convert Lua value to JSON value.
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
            // Collect all key-value pairs first (pairs() consumes the table).
            let collected: Vec<(Value, Value)> = t
                .pairs()
                .collect::<mlua::Result<Vec<(Value, Value)>>>()
                .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;

            let has_string_keys = collected
                .iter()
                .any(|(k, _)| !matches!(k, Value::Integer(_)));
            let len = collected.len();

            if len > 0 && !has_string_keys {
                // Pure array table: sort by integer key and collect values.
                let mut entries: Vec<(i64, Value)> = collected
                    .into_iter()
                    .filter_map(|(k, v)| {
                        if let Value::Integer(i) = k {
                            Some((i, v))
                        } else {
                            None
                        }
                    })
                    .collect();
                entries.sort_by_key(|(i, _)| *i);
                let arr: Vec<serde_json::Value> = entries
                    .into_iter()
                    .map(|(_, v)| lua_value_to_json(v))
                    .collect::<mlua::Result<_>>()?;
                Ok(serde_json::Value::Array(arr))
            } else {
                // Object table
                let mut map = serde_json::Map::new();
                for (key, val) in collected {
                    if let Value::String(s) = key {
                        let k = s.to_str()?.to_string();
                        let v = lua_value_to_json(val)?;
                        map.insert(k, v);
                    }
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

/// Convert JSON value to Lua value.
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
                table.set(i as i64 + 1, json_to_lua_value(lua, v)?)?;
            }
            Ok(Value::Table(table))
        }
        serde_json::Value::Object(obj) => {
            let table = lua.create_table()?;
            for (k, v) in obj.into_iter() {
                table.set(k, json_to_lua_value(lua, v)?)?;
            }
            Ok(Value::Table(table))
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_all() {
        let lua = Lua::new();
        ScriptRuntime::register_all(&lua).unwrap();

        let script = r#"
            log_info("test")
            local hex = hex_encode({0xDE, 0xAD})
            assert(hex == "dead", "Expected 'dead', got '" .. hex .. "'")
            local json = json_encode({name = "test"})
            assert(type(json) == "string")
            local bytes = string_to_bytes("AB")
            assert(bytes[1] == 0x41)
            assert(bytes[2] == 0x42)
            local now = time_now()
            assert(now > 0)
        "#;
        lua.load(script).exec().unwrap();
    }

    #[test]
    fn test_json_roundtrip() {
        let lua = Lua::new();
        ScriptRuntime::register_all(&lua).unwrap();

        let script = r#"
            local original = {
                name = "test",
                value = 42,
                nested = {x = 10, y = 20},
                items = {1, 2, 3}
            }
            local encoded = json_encode(original)
            local decoded = json_decode(encoded)
            assert(decoded.name == original.name, "name mismatch")
            assert(decoded.value == original.value, "value mismatch")
            assert(decoded.nested.x == original.nested.x, "nested.x mismatch")
            assert(decoded.items[1] == original.items[1], "items[1] mismatch")
        "#;
        lua.load(script).exec().unwrap();
    }

    #[test]
    fn test_hex_roundtrip() {
        let lua = Lua::new();
        ScriptRuntime::register_all(&lua).unwrap();

        let script = r#"
            local original = {0xDE, 0xAD, 0xBE, 0xEF}
            local hex = hex_encode(original)
            assert(hex == "deadbeef", "Expected 'deadbeef', got '" .. hex .. "'")
            local decoded = hex_decode(hex)
            assert(decoded[1] == 0xDE)
            assert(decoded[2] == 0xAD)
            assert(decoded[3] == 0xBE)
            assert(decoded[4] == 0xEF)
        "#;
        lua.load(script).exec().unwrap();
    }

    #[test]
    fn test_bytes_string_roundtrip() {
        let lua = Lua::new();
        ScriptRuntime::register_all(&lua).unwrap();

        let script = r#"
            local original = "Hello, World!"
            local bytes = string_to_bytes(original)
            local restored = bytes_to_string(bytes)
            assert(restored == original, "Roundtrip failed: " .. restored)
        "#;
        lua.load(script).exec().unwrap();
    }

    #[test]
    fn test_hex_to_bytes() {
        let lua = Lua::new();
        ScriptRuntime::register_all(&lua).unwrap();

        let script = r#"
            local bytes = hex_to_bytes("010203")
            assert(type(bytes) == "table")
            assert(bytes[1] == 1)
            assert(bytes[2] == 2)
            assert(bytes[3] == 3)
        "#;
        lua.load(script).exec().unwrap();
    }

    #[test]
    fn test_lua_state_pool() {
        let pool = LuaStatePool::new(5);
        assert_eq!(pool.available(), 0);

        // Acquire creates new instance
        let lua1 = pool.acquire();
        assert_eq!(pool.available(), 0);

        // Release returns to pool
        pool.release(lua1);
        assert_eq!(pool.available(), 1);

        // Acquire reuses from pool
        let lua2 = pool.acquire();
        assert_eq!(pool.available(), 0);

        pool.release(lua2);
    }

    #[test]
    fn test_acquire_release_helpers() {
        // Test thread-local helpers
        let lua = acquire_lua();
        ScriptRuntime::register_all(&lua).unwrap();

        let script = "log_info('pool test')";
        lua.load(script).exec().unwrap();

        release_lua(lua);
    }

    #[test]
    fn test_script_cache() {
        let cache = ScriptCache::new();
        assert!(cache.is_empty());

        let script = "print('hello')";
        assert!(!cache.contains(script));

        cache.mark_executed(script);
        assert!(cache.contains(script));
        assert_eq!(cache.len(), 1);

        cache.clear();
        assert!(!cache.contains(script));
        assert!(cache.is_empty());
    }
}
