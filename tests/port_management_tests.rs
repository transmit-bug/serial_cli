//! 端口管理集成测试
//!
//! 测试端口配置和管理功能

use serial_cli::serial_core::{FlowControl, Parity, PortManager, SerialConfig};

#[test]
fn test_port_manager_creation() {
    let manager = PortManager::new();
    let ports = manager.list_ports();
    // 可能没有实际端口，但调用不应失败
    assert!(ports.is_ok(), "PortManager should be created successfully");
}

#[tokio::test]
async fn test_open_nonexistent_port() {
    let manager = PortManager::new();
    let config = SerialConfig::default();

    let result = manager.open_port("/dev/nonexistent", config).await;
    assert!(result.is_err(), "Opening nonexistent port should fail");
}

#[tokio::test]
async fn test_close_unopened_port() {
    let manager = PortManager::new();

    let result = manager.close_port("/dev/ttyUSB0").await;
    assert!(result.is_err(), "Closing unopened port should fail");
}

#[test]
fn test_serial_config_default() {
    let config = SerialConfig::default();
    assert_eq!(config.baudrate, 115200);
    assert_eq!(config.databits, 8);
    assert_eq!(config.stopbits, 1);
    assert_eq!(config.timeout_ms, 1000);
}

#[test]
fn test_serial_config_custom_values() {
    let config = SerialConfig {
        baudrate: 9600,
        databits: 7,
        stopbits: 2,
        parity: Parity::Even,
        timeout_ms: 5000,
        flow_control: FlowControl::Software,
        dtr_enable: true,
        rts_enable: false,
    };

    assert_eq!(config.baudrate, 9600);
    assert_eq!(config.databits, 7);
    assert_eq!(config.stopbits, 2);
    assert_eq!(config.timeout_ms, 5000);
}

#[test]
fn test_parity_debug_roundtrip() {
    // 配置文件中 parity 用字符串 "none"/"even"/"odd" 表示
    // 测试 Debug 输出与预期一致（用于日志和诊断）
    let cases = vec![
        (Parity::None, "None"),
        (Parity::Even, "Even"),
        (Parity::Odd, "Odd"),
    ];
    for (parity, expected) in cases {
        assert_eq!(format!("{:?}", parity), expected);
    }
}

#[test]
fn test_flow_control_debug_roundtrip() {
    let cases = vec![
        (FlowControl::None, "None"),
        (FlowControl::Software, "Software"),
        (FlowControl::Hardware, "Hardware"),
    ];
    for (fc, expected) in cases {
        assert_eq!(format!("{:?}", fc), expected);
    }
}

#[test]
fn test_serial_config_clone() {
    let config = SerialConfig {
        baudrate: 9600,
        databits: 7,
        stopbits: 2,
        parity: Parity::Even,
        timeout_ms: 5000,
        flow_control: FlowControl::Hardware,
        dtr_enable: true,
        rts_enable: false,
    };

    let cloned = config.clone();
    assert_eq!(cloned.baudrate, config.baudrate);
    assert_eq!(format!("{:?}", cloned.parity), format!("{:?}", config.parity));
}
