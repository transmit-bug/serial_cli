//! 端口脚本回调链集成测试
//!
//! 测试端口写入→脚本回调的完整链路，验证：
//! 1. 脚本绑定到端口后，写入数据触发 on_send 回调
//! 2. 接收数据触发 on_recv 回调
//! 3. 脚本错误不会阻塞数据流
//! 4. 多个端口可以绑定不同脚本

use serial_cli::serial_core::serial_script::SerialScriptEngine;

/// 测试脚本：简单的回显脚本，返回原始数据
const ECHO_SCRIPT: &str = r#"
function on_send(data)
    return data
end

function on_recv(data)
    return data
end
"#;

/// 测试脚本：转换脚本，添加前缀
const PREFIX_SCRIPT: &str = r#"
function on_send(data)
    -- 添加前缀 "OUT:"
    local prefix = {79, 85, 84, 58}  -- "OUT:"
    local result = {}
    for i, v in ipairs(prefix) do
        result[i] = v
    end
    for i, v in ipairs(data) do
        result[#prefix + i] = v
    end
    return result
end

function on_recv(data)
    -- 添加前缀 "IN:"
    local prefix = {73, 78, 58}  -- "IN:"
    local result = {}
    for i, v in ipairs(prefix) do
        result[i] = v
    end
    for i, v in ipairs(data) do
        result[#prefix + i] = v
    end
    return result
end
"#;

/// 测试脚本：错误脚本，抛出异常
const ERROR_SCRIPT: &str = r#"
function on_send(data)
    error("Intentional error in on_send")
end

function on_recv(data)
    error("Intentional error in on_recv")
end
"#;

/// 测试脚本：拦截脚本，返回 nil（不发送数据）
const INTERCEPT_SCRIPT: &str = r#"
function on_send(data)
    return nil  -- 拦截所有发送
end

function on_recv(data)
    return nil  -- 拦截所有接收
end
"#;

#[tokio::test]
async fn test_create_script_engine() {
    let result = SerialScriptEngine::new(ECHO_SCRIPT);
    assert!(result.is_ok(), "Should create script engine from valid Lua");
}

#[tokio::test]
async fn test_create_script_engine_invalid_lua() {
    let invalid_lua = "this is not valid lua {{{";
    let result = SerialScriptEngine::new(invalid_lua);
    assert!(result.is_err(), "Should fail to create engine from invalid Lua");
}

#[tokio::test]
async fn test_load_script() {
    let engine = SerialScriptEngine::new(ECHO_SCRIPT).unwrap();
    let result = engine.load();
    assert!(result.is_ok(), "Should load and execute script");
}

#[tokio::test]
async fn test_on_send_echo() {
    let engine = SerialScriptEngine::new(ECHO_SCRIPT).unwrap();
    engine.load().unwrap();
    
    let test_data = b"Hello, World!";
    let result = engine.on_send(test_data);
    assert!(result.is_ok(), "on_send callback should execute");
    
    let output = result.unwrap();
    assert_eq!(output, test_data, "Echo script should return data unchanged");
}

#[tokio::test]
async fn test_on_recv_echo() {
    let engine = SerialScriptEngine::new(ECHO_SCRIPT).unwrap();
    engine.load().unwrap();
    
    let test_data = b"Response data";
    let output = engine.on_recv(test_data);
    assert_eq!(output, test_data, "Echo script should return data unchanged");
}

#[tokio::test]
async fn test_on_send_with_prefix() {
    let engine = SerialScriptEngine::new(PREFIX_SCRIPT).unwrap();
    engine.load().unwrap();
    
    let test_data = b"test";
    let result = engine.on_send(test_data);
    assert!(result.is_ok(), "on_send callback should execute");
    
    let output = result.unwrap();
    assert_eq!(output, b"OUT:test", "Prefix script should add OUT: prefix");
}

#[tokio::test]
async fn test_on_recv_with_prefix() {
    let engine = SerialScriptEngine::new(PREFIX_SCRIPT).unwrap();
    engine.load().unwrap();
    
    let test_data = b"response";
    let output = engine.on_recv(test_data);
    assert_eq!(output, b"IN:response", "Prefix script should add IN: prefix");
}

#[tokio::test]
async fn test_on_send_error() {
    let engine = SerialScriptEngine::new(ERROR_SCRIPT).unwrap();
    engine.load().unwrap();
    
    let test_data = b"test";
    let result = engine.on_send(test_data);
    // 错误被捕获，数据透传（不中断数据流）
    assert!(result.is_ok(), "Should handle error gracefully");
    assert_eq!(result.unwrap(), test_data, "Error should pass through data");
}

#[tokio::test]
async fn test_on_recv_error() {
    let engine = SerialScriptEngine::new(ERROR_SCRIPT).unwrap();
    engine.load().unwrap();
    
    let test_data = b"test";
    // on_recv doesn't return Result, it returns Vec<u8> directly
    // On error, it should return the original data (pass-through)
    let output = engine.on_recv(test_data);
    assert_eq!(output, test_data, "Error in on_recv should pass through data");
}

#[tokio::test]
async fn test_intercept_on_send() {
    let engine = SerialScriptEngine::new(INTERCEPT_SCRIPT).unwrap();
    engine.load().unwrap();
    
    let test_data = b"test";
    let result = engine.on_send(test_data);
    assert!(result.is_ok(), "Intercept script should not error");
    
    let output = result.unwrap();
    assert_eq!(output, Vec::<u8>::new(), "Intercept should return empty vec");
}

#[tokio::test]
async fn test_intercept_on_recv() {
    let engine = SerialScriptEngine::new(INTERCEPT_SCRIPT).unwrap();
    engine.load().unwrap();
    
    let test_data = b"test";
    let output = engine.on_recv(test_data);
    assert_eq!(output, Vec::<u8>::new(), "Intercept should return empty vec");
}

#[tokio::test]
async fn test_empty_data_handling() {
    let engine = SerialScriptEngine::new(ECHO_SCRIPT).unwrap();
    engine.load().unwrap();
    
    let empty_data = b"";
    
    let result_send = engine.on_send(empty_data);
    assert!(result_send.is_ok(), "Should handle empty data in on_send");
    assert_eq!(result_send.unwrap(), empty_data);
    
    let output_recv = engine.on_recv(empty_data);
    assert_eq!(output_recv, empty_data, "Should handle empty data in on_recv");
}

#[tokio::test]
async fn test_large_data_handling() {
    let engine = SerialScriptEngine::new(ECHO_SCRIPT).unwrap();
    engine.load().unwrap();
    
    let large_data = vec![0x41u8; 1024 * 1024]; // 1MB
    
    let result = engine.on_send(&large_data);
    assert!(result.is_ok(), "Should handle large data");
    assert_eq!(result.unwrap().len(), large_data.len());
}

#[tokio::test]
async fn test_binary_data_handling() {
    let engine = SerialScriptEngine::new(ECHO_SCRIPT).unwrap();
    engine.load().unwrap();
    
    let binary_data: Vec<u8> = (0..=255).collect();
    
    let result = engine.on_send(&binary_data);
    assert!(result.is_ok(), "Should handle binary data");
    assert_eq!(result.unwrap(), binary_data);
}

#[tokio::test]
async fn test_unicode_data_handling() {
    let engine = SerialScriptEngine::new(ECHO_SCRIPT).unwrap();
    engine.load().unwrap();
    
    let unicode_data = "你好，世界！🌍".as_bytes();
    
    let result = engine.on_send(unicode_data);
    assert!(result.is_ok(), "Should handle unicode data");
    assert_eq!(result.unwrap(), unicode_data);
}

#[tokio::test]
async fn test_multiple_engines_independent() {
    let engine1 = SerialScriptEngine::new(ECHO_SCRIPT).unwrap();
    engine1.load().unwrap();
    
    let engine2 = SerialScriptEngine::new(PREFIX_SCRIPT).unwrap();
    engine2.load().unwrap();
    
    let test_data = b"test";
    
    let result1 = engine1.on_send(test_data);
    let result2 = engine2.on_send(test_data);
    
    assert_eq!(result1.unwrap(), test_data, "Echo engine should not modify");
    assert_eq!(result2.unwrap(), b"OUT:test", "Prefix engine should add prefix");
}

#[tokio::test]
async fn test_on_send_without_function() {
    // 脚本没有定义 on_send 函数
    let script_no_on_send = r#"
    function on_recv(data)
        return data
    end
    "#;
    
    let engine = SerialScriptEngine::new(script_no_on_send).unwrap();
    engine.load().unwrap();
    
    let test_data = b"test";
    let result = engine.on_send(test_data);
    assert!(result.is_ok(), "Should handle missing on_send gracefully");
    assert_eq!(result.unwrap(), test_data, "Should pass through data");
}

#[tokio::test]
async fn test_on_recv_without_function() {
    // 脚本没有定义 on_recv 函数
    let script_no_on_recv = r#"
    function on_send(data)
        return data
    end
    "#;
    
    let engine = SerialScriptEngine::new(script_no_on_recv).unwrap();
    engine.load().unwrap();
    
    let test_data = b"test";
    let output = engine.on_recv(test_data);
    assert_eq!(output, test_data, "Should pass through data when on_recv missing");
}

#[tokio::test]
async fn test_script_with_globals() {
    // 测试使用全局变量的脚本
    let script_with_globals = r#"
    local counter = 0
    
    function on_send(data)
        counter = counter + 1
        return data
    end
    
    function on_recv(data)
        counter = counter + 1
        return data
    end
    
    function get_counter()
        return counter
    end
    "#;
    
    let engine = SerialScriptEngine::new(script_with_globals).unwrap();
    engine.load().unwrap();
    
    // 调用多次
    engine.on_send(b"test1").unwrap();
    engine.on_recv(b"test2");
    engine.on_send(b"test3").unwrap();
    
    // 全局状态应该保持
    // 注意：这里无法直接调用 get_counter，但可以验证脚本没有崩溃
}

#[tokio::test]
async fn test_script_returning_string() {
    // 测试返回字符串而非表的脚本
    let script_returning_string = r#"
    function on_send(data)
        return "modified"
    end
    "#;
    
    let engine = SerialScriptEngine::new(script_returning_string).unwrap();
    engine.load().unwrap();
    
    let test_data = b"original";
    let result = engine.on_send(test_data);
    assert!(result.is_ok(), "Should handle string return");
    assert_eq!(result.unwrap(), b"modified");
}

#[tokio::test]
async fn test_script_returning_nil() {
    // 测试返回 nil 的脚本（拦截）
    let script_returning_nil = r#"
    function on_send(data)
        return nil
    end
    "#;
    
    let engine = SerialScriptEngine::new(script_returning_nil).unwrap();
    engine.load().unwrap();
    
    let test_data = b"test";
    let result = engine.on_send(test_data);
    assert!(result.is_ok(), "Should handle nil return");
    assert_eq!(result.unwrap(), Vec::<u8>::new(), "Nil should result in empty vec");
}
