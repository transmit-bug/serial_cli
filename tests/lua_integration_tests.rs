//! Integration tests for Lua APIs

use serial_cli::lua::executor::ScriptEngine;
use serial_cli::lua::ScriptRuntime;
use std::sync::Arc;
use tokio::sync::Mutex;

#[test]
fn test_serial_api_integration() {
    let mut engine = ScriptEngine::new().unwrap();
    let port_manager = Arc::new(Mutex::new(engine.port_manager().clone()));
    engine.bindings.set_port_manager(port_manager);
    ScriptRuntime::register_all(engine.bindings.lua()).unwrap();
    engine.bindings.register_all_apis().unwrap();

    let script = r#"
        local ports = serial_list()
        assert(type(ports) == "table", "serial_list should return table")
    "#;

    assert!(engine.bindings.execute_script(script).is_ok());
}

#[test]
fn test_protocol_api_integration() {
    let mut engine = ScriptEngine::new().unwrap();
    let port_manager = Arc::new(Mutex::new(engine.port_manager().clone()));
    engine.bindings.set_port_manager(port_manager);
    ScriptRuntime::register_all(engine.bindings.lua()).unwrap();
    engine.bindings.register_all_apis().unwrap();

    let script = r#"
        local protocols = protocol_list()
        assert(type(protocols) == "table")

        -- Verify minimum expected protocols (more may exist)
        assert(#protocols >= 4, "Should have at least 4 built-in protocols")

        local encoded = protocol_encode("line", "test")
        assert(type(encoded) == "string")
        assert(string.sub(encoded, -1) == "\n")

        local decoded = protocol_decode("line", "test\n")
        assert(type(decoded) == "string")
    "#;

    assert!(engine.bindings.execute_script(script).is_ok());
}

#[test]
fn test_conversion_api_integration() {
    let lua = mlua::Lua::new();
    ScriptRuntime::register_all(&lua).unwrap();

    let script = r#"
        local bytes = hex_to_bytes("010203")
        assert(bytes[1] == 1)
        assert(bytes[2] == 2)
        assert(bytes[3] == 3)

        local hex = bytes_to_hex({1, 2, 3})
        assert(hex == "010203")

        local str = bytes_to_string({72, 101, 108, 108, 111})
        assert(str == "Hello")

        local bytes2 = string_to_bytes("ABC")
        assert(bytes2[1] == 65)
        assert(bytes2[2] == 66)
        assert(bytes2[3] == 67)
    "#;

    lua.load(script).exec().unwrap();
}

#[test]
fn test_end_to_end_modbus_workflow() {
    let mut engine = ScriptEngine::new().unwrap();
    let port_manager = Arc::new(Mutex::new(engine.port_manager().clone()));
    engine.bindings.set_port_manager(port_manager);
    ScriptRuntime::register_all(engine.bindings.lua()).unwrap();
    engine.bindings.register_all_apis().unwrap();

    let script = r#"
        -- Test line protocol roundtrip
        local original = "Hello, World!"
        local encoded = protocol_encode('line', original)
        local decoded = protocol_decode('line', encoded)
        -- Line protocol adds newline if not present
        assert(decoded == original .. "\n", "Line protocol roundtrip failed")

        -- Test with data that already has newline
        local with_newline = "Test\n"
        local encoded2 = protocol_encode('line', with_newline)
        local decoded2 = protocol_decode('line', encoded2)
        assert(decoded2 == with_newline, "Line protocol with existing newline failed")

        -- Test Modbus encoding (binary protocols use hex-encoded strings)
        local pdu_hex = "0103000000" .. string.format("%02x", 10)
        local modbus_encoded = protocol_encode('modbus_rtu', pdu_hex)
        assert(type(modbus_encoded) == "string")
        assert(string.len(modbus_encoded) > string.len(pdu_hex), "Encoded data should include CRC")

        -- Modbus round-trip: encode then decode should recover original PDU
        local modbus_decoded = protocol_decode('modbus_rtu', modbus_encoded)
        assert(modbus_decoded == pdu_hex, "Modbus round-trip failed")

        -- Test AT command protocol
        local at_cmd = "ATZ"
        local at_encoded = protocol_encode('at_command', at_cmd)
        local at_decoded = protocol_decode('at_command', at_encoded)
        -- AT command adds \r\n
        assert(at_decoded == "ATZ\r\n", "AT command roundtrip failed")
    "#;

    match engine.bindings.execute_script(script) {
        Ok(_) => {}
        Err(e) => panic!("Script execution failed: {:?}", e),
    }
}

#[test]
fn test_protocol_load_validate() {
    let mut engine = ScriptEngine::new().unwrap();
    let port_manager = Arc::new(Mutex::new(engine.port_manager().clone()));
    engine.bindings.set_port_manager(port_manager);
    ScriptRuntime::register_all(engine.bindings.lua()).unwrap();
    engine.bindings.register_all_apis().unwrap();

    // Test loading a valid protocol
    let script = r#"
        local ok, err = protocol_load("tests/fixtures/protocols/test_valid.lua")
        assert(ok, err)
    "#;

    assert!(engine.bindings.execute_script(script).is_ok());
}
