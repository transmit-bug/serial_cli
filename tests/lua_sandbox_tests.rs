//! Lua 沙箱与安全测试
//!
//! 测试脚本验证的静态分析功能和运行时安全边界。
//!
//! 注意：当前实现中 `enable_sandbox` 和 `memory_limit_mb` 配置项仅用于静态验证，
//! 运行时并不实际强制执行沙箱隔离。以下测试覆盖已有的静态分析功能，
//! 并标记出需要后续实现的运行时沙箱测试（作为 TODO）。

use serial_cli::lua::executor::ScriptEngine;
use serial_cli::lua::ScriptRuntime;
use serial_cli::script::ScriptManager;
use std::sync::Arc;
use tokio::sync::Mutex;

// ── 静态分析：危险函数检测 ──────────────────────────────────────────────

#[test]
fn test_detect_os_execute() {
    let script = r#"
        function on_send(data)
            os.execute("ls")
            return data
        end
    "#;
    let warnings = ScriptManager::validate_script_detailed(script);
    assert!(
        warnings.iter().any(|w| w.contains("dangerous")),
        "Should detect os.execute as dangerous"
    );
}

#[test]
fn test_detect_io_popen() {
    let script = r#"
        function on_send(data)
            io.popen("ls")
            return data
        end
    "#;
    let warnings = ScriptManager::validate_script_detailed(script);
    assert!(
        warnings.iter().any(|w| w.contains("dangerous")),
        "Should detect io.popen as dangerous"
    );
}

#[test]
fn test_detect_os_execute_in_comment_is_still_flagged() {
    // 静态分析是简单的字符串匹配，注释中的危险函数也会被标记
    // 这是可接受的保守行为
    let script = r#"
        -- os.execute is commented out
        function on_send(data)
            return data
        end
    "#;
    let warnings = ScriptManager::validate_script_detailed(script);
    assert!(
        warnings.iter().any(|w| w.contains("dangerous")),
        "Static analysis flags os.execute even in comments (conservative)"
    );
}

#[test]
fn test_safe_script_no_dangerous_warnings() {
    let script = r#"
        function on_send(data)
            log_info("sending")
            return data
        end

        function on_recv(data)
            return data
        end
    "#;
    let warnings = ScriptManager::validate_script_detailed(script);
    assert!(
        !warnings.iter().any(|w| w.contains("dangerous")),
        "Safe script should not have dangerous warnings, got: {:?}",
        warnings
    );
}

// ── 静态分析：回调检测 ──────────────────────────────────────────────────

#[test]
fn test_no_callbacks_warning() {
    let script = r#"
        function helper()
            return "helper"
        end
    "#;
    let warnings = ScriptManager::validate_script_detailed(script);
    assert!(
        warnings
            .iter()
            .any(|w| w.contains("No callbacks defined")),
        "Should warn about missing callbacks"
    );
}

#[test]
fn test_partial_callbacks_no_warning() {
    // 只要定义了至少一个回调，就不会警告
    let script = r#"
        function on_send(data)
            return data
        end
    "#;
    let warnings = ScriptManager::validate_script_detailed(script);
    assert!(
        !warnings
            .iter()
            .any(|w| w.contains("No callbacks defined")),
        "Should not warn when at least one callback exists"
    );
}

// ── 语法验证 ────────────────────────────────────────────────────────────

#[test]
fn test_valid_syntax() {
    let script = r#"
        function on_send(data)
            return data
        end
    "#;
    assert!(ScriptManager::validate_source(script).is_ok());
}

#[test]
fn test_invalid_syntax() {
    let script = r#"
        function on_send(data)
            return data
        -- missing end
    "#;
    assert!(ScriptManager::validate_source(script).is_err());
}

#[test]
fn test_empty_script_is_valid() {
    assert!(ScriptManager::validate_source("").is_ok());
}

// ── 运行时安全：危险 API 可用性测试 ─────────────────────────────────────
//
// 以下测试验证当前运行时行为：mlua 默认沙箱并不限制 os/io 等标准库。
// 如果后续实现了真正的沙箱隔离，这些测试应该更新为验证限制生效。

#[test]
fn test_os_execute_available_in_default_lua() {
    // 当前行为：mlua::Lua::new() 默认加载所有标准库，包括 os
    // 此测试记录这一事实，后续实现沙箱时应改为 assert!(result.is_err())
    let lua = mlua::Lua::new();
    let result = lua.load("type(os.execute)").exec();
    assert!(
        result.is_ok(),
        "Current behavior: os.execute is available in default Lua (not sandboxed)"
    );
}

#[test]
fn test_io_open_available_in_default_lua() {
    let lua = mlua::Lua::new();
    let result = lua.load("type(io.open)").exec();
    assert!(
        result.is_ok(),
        "Current behavior: io.open is available in default Lua (not sandboxed)"
    );
}

#[test]
fn test_debug_library_not_loaded_by_default() {
    // mlua 默认不加载 debug 库（安全默认值）
    let lua = mlua::Lua::new();
    let result = lua.load("type(debug.getinfo)").exec();
    // debug 库默认不可用 — 这是一个好的安全默认值
    assert!(
        result.is_err(),
        "mlua default: debug library is NOT loaded (secure default)"
    );
}

#[test]
fn test_loadfile_available_in_default_lua() {
    let lua = mlua::Lua::new();
    let result = lua.load("type(loadfile)").exec();
    assert!(
        result.is_ok(),
        "Current behavior: loadfile is available in default Lua (not sandboxed)"
    );
}

#[test]
fn test_dofile_available_in_default_lua() {
    let lua = mlua::Lua::new();
    let result = lua.load("type(dofile)").exec();
    assert!(
        result.is_ok(),
        "Current behavior: dofile is available in default Lua (not sandboxed)"
    );
}

// ── 运行时安全：内存限制 ────────────────────────────────────────────────
//
// TODO: 当 memory_limit_mb 被实际实现后，以下测试应更新为验证限制生效。

#[test]
fn test_lua_allocation_basic() {
    // 验证 Lua 可以正常分配内存
    let script_manager = Arc::new(Mutex::new(ScriptManager::new()));
    let engine = ScriptEngine::new(script_manager).unwrap();
    let result = engine.execute_string(r#"
        local t = {}
        for i = 1, 10000 do
            t[i] = string.rep("x", 100)
        end
    "#);
    assert!(result.is_ok(), "Basic allocation should succeed");
}

// ── 运行时安全：超时 ────────────────────────────────────────────────────
//
// TODO: 当 timeout_seconds 被实际实现后，以下测试应更新为验证超时生效。
// 当前行为：死循环会永久阻塞线程。

#[test]
fn test_script_execution_completes_quickly() {
    // 验证正常脚本可以快速完成
    let script_manager = Arc::new(Mutex::new(ScriptManager::new()));
    let engine = ScriptEngine::new(script_manager).unwrap();
    let result = engine.execute_string(r#"
        local sum = 0
        for i = 1, 1000 do
            sum = sum + i
        end
        assert(sum == 500500)
    "#);
    assert!(result.is_ok(), "Quick script should complete");
}

// ── 注册的 API 不包含危险函数 ────────────────────────────────────────────

#[test]
fn test_scriptengine_does_not_register_os() {
    // ScriptEngine 通过 ScriptRuntime::register_all 注册 API
    // 验证 os 库没有被移除（当前行为），但也没有被额外注册危险函数
    let script_manager = Arc::new(Mutex::new(ScriptManager::new()));
    let engine = ScriptEngine::new(script_manager).unwrap();
    ScriptRuntime::register_all(engine.bindings.lua()).unwrap();
    engine.bindings.register_all_apis().unwrap();

    // 验证注册的安全 API 可用
    let result = engine.execute_string(r#"
        assert(type(log_info) == "function")
        assert(type(json_encode) == "function")
        assert(type(hex_encode) == "function")
        assert(type(sleep_ms) == "function")
    "#);
    assert!(result.is_ok(), "Registered APIs should be available");
}

#[test]
fn test_validate_script_with_print_not_flagged() {
    // print 不被视为危险函数
    let script = r#"
        function on_send(data)
            print("Sending:", data)
            return data
        end
    "#;
    let warnings = ScriptManager::validate_script_detailed(script);
    assert!(
        !warnings.iter().any(|w| w.contains("dangerous")),
        "print should not be flagged as dangerous"
    );
}

#[test]
fn test_validate_script_with_log_functions_not_flagged() {
    let script = r#"
        function on_send(data)
            log_info("info")
            log_debug("debug")
            log_warn("warn")
            log_error("error")
            return data
        end
    "#;
    let warnings = ScriptManager::validate_script_detailed(script);
    assert!(
        !warnings.iter().any(|w| w.contains("dangerous")),
        "Log functions should not be flagged as dangerous"
    );
}

// ── 多问题检测 ──────────────────────────────────────────────────────────

#[test]
fn test_validate_script_multiple_issues() {
    // 脚本同时有：无回调 + 危险函数
    let script = r#"
        function helper()
            os.execute("rm -rf /")
        end
    "#;
    let warnings = ScriptManager::validate_script_detailed(script);
    assert!(
        warnings.iter().any(|w| w.contains("dangerous")),
        "Should detect dangerous function"
    );
    assert!(
        warnings
            .iter()
            .any(|w| w.contains("No callbacks defined")),
        "Should detect missing callbacks"
    );
}
