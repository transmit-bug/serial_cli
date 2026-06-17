//! Script manager tests

use serial_cli::script::ScriptManager;

#[test]
fn test_manager_creation() {
    let manager = ScriptManager::new();
    // Should have built-in scripts
    let scripts = manager.list();
    assert!(!scripts.is_empty());
}

#[test]
fn test_load_valid_script() {
    let mut manager = ScriptManager::new();

    let path = std::path::PathBuf::from("tests/fixtures/protocols/test_valid.lua");
    let result = manager.load(&path);

    assert!(result.is_ok());
    let info = result.unwrap();
    assert_eq!(info.name, "test_valid");
}

#[test]
fn test_load_invalid_script() {
    let mut manager = ScriptManager::new();

    let path = std::path::PathBuf::from("tests/fixtures/protocols/test_syntax_error.lua");
    let result = manager.load(&path);

    assert!(result.is_err());
}

#[test]
fn test_validate_script() {
    let valid_path = std::path::PathBuf::from("tests/fixtures/protocols/test_valid.lua");
    let source = std::fs::read_to_string(&valid_path).unwrap();
    assert!(ScriptManager::validate_source(&source).is_ok());

    let error_path = std::path::PathBuf::from("tests/fixtures/protocols/test_syntax_error.lua");
    let source = std::fs::read_to_string(&error_path).unwrap();
    assert!(ScriptManager::validate_source(&source).is_err());
}

#[test]
fn test_list_scripts() {
    let mut manager = ScriptManager::new();

    let path = std::path::PathBuf::from("tests/fixtures/protocols/test_valid.lua");
    manager.load(&path).unwrap();

    // Check that script is tracked in manager
    assert!(manager.has("test_valid"));
    let scripts = manager.list();
    assert!(scripts.iter().any(|s| s.name == "test_valid"));
}
