//! 脚本验证集成测试
//!
//! 测试脚本验证功能的完整流程

use serial_cli::script::ScriptManager;

#[tokio::test]
async fn test_validate_script_syntax_valid() {
    let valid_script = r#"
        -- Valid script
        function on_send(data)
            return data
        end

        function on_recv(data)
            return data
        end
    "#;

    let result = ScriptManager::validate_source(valid_script);
    assert!(result.is_ok(), "Valid script should pass validation");
}

#[tokio::test]
async fn test_validate_script_syntax_invalid() {
    let invalid_script = r#"
        -- Invalid script
        function on_send(data)
            return data
        -- Missing end
    "#;

    let result = ScriptManager::validate_source(invalid_script);
    assert!(result.is_err(), "Invalid script should fail validation");
}

#[tokio::test]
async fn test_validate_script_missing_callbacks() {
    let script_without_callbacks = r#"
        -- Script without required callbacks
        function helper()
            return "helper"
        end
    "#;

    let warnings = ScriptManager::validate_script_detailed(script_without_callbacks);
    assert!(
        warnings
            .iter()
            .any(|w: &String| w.contains("No callbacks defined")),
        "Should warn about no callbacks defined"
    );
}

#[tokio::test]
async fn test_validate_script_dangerous_functions() {
    let script_with_dangerous_calls = r#"
        -- Script with dangerous functions
        function on_send(data)
            os.execute("ls")
            return data
        end
    "#;

    let warnings = ScriptManager::validate_script_detailed(script_with_dangerous_calls);
    assert!(
        warnings
            .iter()
            .any(|w: &String| w.contains("dangerous function")),
        "Should warn about dangerous functions"
    );
}

#[tokio::test]
async fn test_validate_script_with_comments() {
    let script_with_comments = r#"
        --[[
        This is a multi-line comment
        with lots of text
        ]]

        --[[--
        Short comment
        --]]--

        function on_send(data)
            -- Single line comment
            return data
        end
    "#;

    let warnings = ScriptManager::validate_script_detailed(script_with_comments);
    // Should not warn about comments
    assert!(
        !warnings.iter().any(|w: &String| w.contains("comment")),
        "Should not warn about comments"
    );
}

#[tokio::test]
async fn test_validate_script_empty() {
    let empty_script = "";

    let result = ScriptManager::validate_source(empty_script);
    // Empty script is valid Lua
    assert!(result.is_ok(), "Empty script should be valid");
}

#[tokio::test]
async fn test_validate_script_with_print() {
    let script_with_print = r#"
        function on_send(data)
            print("Sending:", data)
            return data
        end
    "#;

    let warnings = ScriptManager::validate_script_detailed(script_with_print);
    // print is not dangerous
    assert!(
        !warnings.iter().any(|w: &String| w.contains("print")),
        "Should not warn about print"
    );
}

#[tokio::test]
async fn test_validate_script_multiple_issues() {
    let script_with_issues = r#"
        -- Script with multiple issues
        function on_send(data)
            os.execute("dangerous")
            io.popen("also dangerous")
            return data
        end
    "#;

    let warnings = ScriptManager::validate_script_detailed(script_with_issues);
    // Implementation generates one warning for dangerous functions
    assert!(
        warnings
            .iter()
            .any(|w: &String| w.contains("dangerous functions")),
        "Should warn about dangerous functions"
    );
}

#[tokio::test]
async fn test_validate_script_partial_callbacks() {
    // Implementation note: validate_script_detailed only warns if NO callbacks are defined.
    // If at least one callback exists, it doesn't warn about missing ones.
    let script_partial = r#"
        -- Script with only on_send
        function on_send(data)
            return data
        end
    "#;

    let warnings = ScriptManager::validate_script_detailed(script_partial);
    // Should NOT warn about missing on_recv (implementation doesn't check individual callbacks)
    assert!(
        !warnings.iter().any(|w: &String| w.contains("on_recv")),
        "Should not warn about missing on_recv when other callbacks exist"
    );
}

#[tokio::test]
async fn test_validate_script_complete_callbacks() {
    let script_complete = r#"
        -- Script with all callbacks
        function on_open(port)
            -- Initialize
        end

        function on_close(port)
            -- Cleanup
        end

        function on_send(data)
            return data
        end

        function on_recv(data)
            return data
        end
    "#;

    let warnings = ScriptManager::validate_script_detailed(script_complete);
    assert!(
        warnings.is_empty()
            || !warnings
                .iter()
                .any(|w: &String| w.contains("Missing required callback")),
        "Should not warn about missing callbacks when all are present"
    );
}
