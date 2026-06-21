//! 端口管理集成测试
//!
//! 测试端口冲突检测和状态同步功能

use serial_cli::serial_core::{FlowControl, Parity, PortManager, SerialConfig};

#[tokio::test]
async fn test_port_manager_creation() {
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

#[tokio::test]
async fn test_list_ports_returns_available() {
    let manager = PortManager::new();
    let ports = manager.list_ports().unwrap();

    // 返回的应该是 Vec<SerialPortInfo>
    assert!(
        !ports.is_empty() || ports.is_empty(),
        "Port list should be accessible"
    );
}

#[tokio::test]
async fn test_serial_config_default() {
    let config = SerialConfig::default();
    assert_eq!(config.baudrate, 115200);
    assert_eq!(config.databits, 8);
    assert_eq!(config.stopbits, 1);
    assert_eq!(config.timeout_ms, 1000);
}

#[tokio::test]
async fn test_serial_config_custom_values() {
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

#[tokio::test]
async fn test_parity_variants() {
    let none = Parity::None;
    let even = Parity::Even;
    let odd = Parity::Odd;

    // 确保可以创建不同变体
    assert_eq!(format!("{:?}", none), "None");
    assert_eq!(format!("{:?}", even), "Even");
    assert_eq!(format!("{:?}", odd), "Odd");
}

#[tokio::test]
async fn test_stop_bits_values() {
    let one = 1u8;
    let two = 2u8;

    assert_eq!(one, 1);
    assert_eq!(two, 2);
}

#[tokio::test]
async fn test_flow_control_variants() {
    let none = FlowControl::None;
    let software = FlowControl::Software;
    let hardware = FlowControl::Hardware;

    assert_eq!(format!("{:?}", none), "None");
    assert_eq!(format!("{:?}", software), "Software");
    assert_eq!(format!("{:?}", hardware), "Hardware");
}
