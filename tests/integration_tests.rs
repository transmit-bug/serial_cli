//! Integration tests for platform-specific signal control
//!
//! These tests verify that the platform abstraction layer works correctly
//! and that signal control functions behave as expected across different platforms.

/// Macro to reduce platform-specific controller creation code duplication
macro_rules! get_controller {
    () => {{
        #[cfg(unix)]
        {
            serial_cli::serial_core::signals::UnixSignalController::new()
        }
        #[cfg(windows)]
        {
            serial_cli::serial_core::signals::WindowsSignalController::new()
        }
        #[cfg(not(any(unix, windows)))]
        {
            serial_cli::serial_core::signals::FallbackSignalController::new()
        }
    }};
}

#[cfg(test)]
mod signal_control_tests {
    use serial_cli::serial_core::signals::{PlatformSignals, SignalState};

    #[test]
    fn test_signal_controller_creation() {
        let controller = get_controller!();
        assert!(!controller.platform_name().is_empty());
    }

    #[test]
    fn test_dtr_state_management() {
        let mut controller = get_controller!();

        // Test initial state
        assert!(controller.get_dtr().unwrap());

        // Test state changes
        let result = controller.set_dtr(false);
        assert!(result.is_ok());

        match result.unwrap() {
            SignalState::Set(state) => assert!(!state),
            SignalState::NotSupported => {
                // Expected on platforms without hardware support
            }
            SignalState::Failed => {
                // Hardware control failed but state was updated
                assert!(!controller.get_dtr().unwrap());
            }
        }
    }

    #[test]
    fn test_rts_state_management() {
        let mut controller = get_controller!();

        // Test initial state
        assert!(controller.get_rts().unwrap());

        // Test state changes
        let result = controller.set_rts(false);
        assert!(result.is_ok());

        match result.unwrap() {
            SignalState::Set(state) => assert!(!state),
            SignalState::NotSupported => {
                // Expected on platforms without hardware support
            }
            SignalState::Failed => {
                // Hardware control failed but state was updated
                assert!(!controller.get_rts().unwrap());
            }
        }
    }

    #[test]
    fn test_signal_toggle() {
        let mut controller = get_controller!();

        // Toggle DTR
        for _ in 0..3 {
            let current = controller.get_dtr().unwrap();
            let result = controller.set_dtr(!current);
            assert!(result.is_ok());

            // Verify state changed
            assert_ne!(controller.get_dtr().unwrap(), current);
        }
    }

    #[test]
    fn test_concurrent_signal_operations() {
        use std::sync::Barrier;
        use std::sync::{Arc, Mutex};
        use std::thread;

        let controller = Arc::new(Mutex::new(get_controller!()));
        let barrier = Arc::new(Barrier::new(2));

        // Test concurrent DTR operations
        let controller1 = controller.clone();
        let barrier1 = barrier.clone();
        let handle1 = thread::spawn(move || {
            barrier1.wait();
            let mut ctrl = controller1.lock().unwrap();
            ctrl.set_dtr(true).is_ok()
        });

        let controller2 = controller.clone();
        let barrier2 = barrier.clone();
        let handle2 = thread::spawn(move || {
            barrier2.wait();
            let mut ctrl = controller2.lock().unwrap();
            ctrl.set_rts(false).is_ok()
        });

        let result1 = handle1.join().unwrap();
        let result2 = handle2.join().unwrap();

        // Both operations must complete successfully under mutex protection
        assert!(
            result1 && result2,
            "Both concurrent operations should succeed"
        );
    }

    #[test]
    fn test_error_recovery() {
        let mut controller = get_controller!();

        // Test that state changes are handled correctly
        let initial_dtr = controller.get_dtr().unwrap();
        let initial_rts = controller.get_rts().unwrap();

        // Test state transitions with proper error handling
        let dtr_result = controller.set_dtr(!initial_dtr);
        let rts_result = controller.set_rts(!initial_rts);

        // Verify operations succeeded and state changed
        assert!(dtr_result.is_ok(), "DTR state change should succeed");
        assert!(rts_result.is_ok(), "RTS state change should succeed");

        // Verify state actually changed
        assert_ne!(
            controller.get_dtr().unwrap(),
            initial_dtr,
            "DTR state should have changed"
        );
        assert_ne!(
            controller.get_rts().unwrap(),
            initial_rts,
            "RTS state should have changed"
        );
    }

    #[test]
    fn test_platform_specific_behavior() {
        let controller = get_controller!();

        // Test platform-specific behavior
        #[cfg(unix)]
        {
            assert!(matches!(
                controller.platform_name(),
                "linux" | "macos" | "unix"
            ));
        }

        #[cfg(windows)]
        {
            assert_eq!(controller.platform_name(), "windows");
        }

        #[cfg(not(any(unix, windows)))]
        {
            assert_eq!(controller.platform_name(), "fallback");
        }
    }
}

#[cfg(test)]
mod script_lifecycle_tests {
    use serial_cli::script::ScriptManager;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_script_load_unload() {
        let mut manager = ScriptManager::new();

        // Create a test script with a specific name
        let script = r#"
            -- Script: test_unload
            function on_recv(data)
                return data
            end
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(script.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        // Get the file name (without extension) as the script name
        let file_name = temp_file
            .path()
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        // Load the script
        let load_result = manager.load(temp_file.path());
        assert!(load_result.is_ok());
        assert!(manager.has(&file_name));

        // Unload the script
        let unload_result = manager.unload(&file_name);
        assert!(unload_result.is_ok());
        assert!(!manager.has(&file_name));
    }

    #[test]
    fn test_cannot_unload_built_in() {
        let mut manager = ScriptManager::new();

        let result = manager.unload("line");
        assert!(result.is_err());
        assert!(manager.has("line"));
    }
}
