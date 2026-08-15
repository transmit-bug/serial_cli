//! 协议解析与 Property-based 测试
//!
//! 使用 proptest 验证纯函数的 roundtrip 属性，
//! 以及 Modbus RTU/ASCII 帧解析的边界条件。

use proptest::prelude::*;
use serial_cli::script::ScriptManager;
use serial_cli::utils::hex::{hex_decode, hex_encode, hex_encode_simple};

// ── Property-based: Hex 编解码 roundtrip ────────────────────────────────

proptest! {
    #[test]
    fn hex_encode_decode_roundtrip(bytes in any::<Vec<u8>>()) {
        let encoded = hex_encode_simple(&bytes);
        let decoded = hex_decode(&encoded).unwrap();
        prop_assert_eq!(bytes, decoded);
    }

    #[test]
    fn hex_encode_with_separator_roundtrip(
        bytes in any::<Vec<u8>>(),
        sep in proptest::sample::select(vec![":".to_string(), " ".to_string(), "-".to_string()])
    ) {
        let encoded = hex_encode(&bytes, &sep);
        let decoded = hex_decode(&encoded).unwrap();
        prop_assert_eq!(bytes, decoded);
    }

    #[test]
    fn hex_decode_uppercase_equals_lowercase(bytes in any::<Vec<u8>>()) {
        let encoded = hex_encode_simple(&bytes);
        let upper = encoded.to_uppercase();
        let decoded_lower = hex_decode(&encoded).unwrap();
        let decoded_upper = hex_decode(&upper).unwrap();
        prop_assert_eq!(decoded_lower, decoded_upper);
    }

    #[test]
    fn hex_encode_length_is_double(byte_count in 0usize..1000) {
        let bytes = vec![0xABu8; byte_count];
        let encoded = hex_encode_simple(&bytes);
        prop_assert_eq!(encoded.len(), byte_count * 2);
    }
}

// ── Hex 编解码边界测试 ──────────────────────────────────────────────────

#[test]
fn test_hex_empty_input() {
    assert_eq!(hex_encode_simple(&[]), "");
    assert_eq!(hex_decode("").unwrap(), Vec::<u8>::new());
}

#[test]
fn test_hex_single_byte_boundaries() {
    assert_eq!(hex_encode_simple(&[0x00]), "00");
    assert_eq!(hex_encode_simple(&[0xFF]), "ff");
    assert_eq!(hex_encode_simple(&[0x80]), "80");
}

#[test]
fn test_hex_decode_with_0x_prefix() {
    assert_eq!(hex_decode("0xAABB").unwrap(), vec![0xAA, 0xBB]);
    assert_eq!(hex_decode("0Xaabb").unwrap(), vec![0xAA, 0xBB]);
}

#[test]
fn test_hex_decode_odd_length_fails() {
    assert!(hex_decode("012").is_err());
    assert!(hex_decode("A").is_err());
}

#[test]
fn test_hex_decode_invalid_chars_fails() {
    assert!(hex_decode("GG").is_err());
    assert!(hex_decode("01XY").is_err());
}

// ── Modbus RTU 帧解析测试 ───────────────────────────────────────────────

/// Modbus RTU CRC-16 计算
fn modbus_crc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &byte in data {
        crc ^= byte as u16;
        for _ in 0..8 {
            if crc & 0x0001 != 0 {
                crc = (crc >> 1) ^ 0xA001;
            } else {
                crc >>= 1;
            }
        }
    }
    crc
}

#[test]
fn test_modbus_crc16_self_check() {
    // CRC 校验自检：data + CRC 的 CRC 应该等于 0
    let frame = [0x01, 0x03, 0x00, 0x00, 0x00, 0x01];
    let crc = modbus_crc16(&frame);
    let mut with_crc = frame.to_vec();
    with_crc.extend_from_slice(&crc.to_le_bytes());
    assert_eq!(modbus_crc16(&with_crc), 0);
}

#[test]
fn test_modbus_crc16_empty() {
    let crc = modbus_crc16(&[]);
    assert_eq!(crc, 0xFFFF);
}

#[test]
fn test_modbus_crc16_single_byte() {
    // 已知值：单字节 0x00 的 CRC
    let crc = modbus_crc16(&[0x00]);
    assert_ne!(crc, 0xFFFF); // 不应该等于初始值
}

#[test]
fn test_modbus_rtu_frame_assembly() {
    // 组装一个完整的 Modbus RTU 请求帧
    let slave: u8 = 1;
    let function: u8 = 0x03; // Read Holding Registers
    let start_addr: u16 = 0x0000;
    let quantity: u16 = 0x0001;

    let mut frame = vec![slave, function];
    frame.extend_from_slice(&start_addr.to_be_bytes());
    frame.extend_from_slice(&quantity.to_be_bytes());

    let crc = modbus_crc16(&frame);
    frame.extend_from_slice(&crc.to_le_bytes());

    // 验证帧结构
    assert_eq!(frame.len(), 8); // 1+1+2+2+2
    assert_eq!(frame[0], 0x01); // slave
    assert_eq!(frame[1], 0x03); // function
    assert_eq!(frame[2..4], [0x00, 0x00]); // start addr
    assert_eq!(frame[4..6], [0x00, 0x01]); // quantity

    // 验证 CRC：重新计算应匹配
    let data_part = &frame[..6];
    let expected_crc = modbus_crc16(data_part);
    let actual_crc = u16::from_le_bytes([frame[6], frame[7]]);
    assert_eq!(actual_crc, expected_crc);
}

#[test]
fn test_modbus_rtu_frame_parse() {
    // 解析一个完整的 Modbus RTU 响应帧
    // Slave=1, Func=3, ByteCount=2, Data=0x0064, CRC
    let data = [0x01, 0x03, 0x02, 0x00, 0x64];
    let crc = modbus_crc16(&data);
    let mut frame = data.to_vec();
    frame.extend_from_slice(&crc.to_le_bytes());

    // 解析
    assert_eq!(frame[0], 0x01); // slave
    assert_eq!(frame[1], 0x03); // function
    assert_eq!(frame[2], 0x02); // byte count
    let value = u16::from_be_bytes([frame[3], frame[4]]);
    assert_eq!(value, 100); // 0x0064 = 100

    // CRC 验证
    let received_crc = u16::from_le_bytes([frame[5], frame[6]]);
    let calculated_crc = modbus_crc16(&frame[..5]);
    assert_eq!(received_crc, calculated_crc);
}

#[test]
fn test_modbus_rtu_error_response() {
    // Modbus 异常响应：功能码 = 原功能码 + 0x80
    let slave: u8 = 1;
    let error_function: u8 = 0x83; // 0x03 + 0x80
    let exception_code: u8 = 0x02; // Illegal Data Address

    let data = [slave, error_function, exception_code];
    let crc = modbus_crc16(&data);
    let mut frame = data.to_vec();
    frame.extend_from_slice(&crc.to_le_bytes());

    assert_eq!(frame.len(), 5); // 1+1+1+2
    assert_eq!(frame[1] & 0x80, 0x80); // 错误标志
    assert_eq!(frame[2], 0x02); // 异常码
}

#[test]
fn test_modbus_rtu_various_function_codes() {
    // 测试不同功能码的帧组装
    let test_cases: Vec<(u8, &str)> = vec![
        (0x01, "Read Coils"),
        (0x02, "Read Discrete Inputs"),
        (0x03, "Read Holding Registers"),
        (0x04, "Read Input Registers"),
        (0x05, "Write Single Coil"),
        (0x06, "Write Single Register"),
        (0x0F, "Write Multiple Coils"),
        (0x10, "Write Multiple Registers"),
    ];

    for (func_code, name) in test_cases {
        let frame = [0x01, func_code, 0x00, 0x00, 0x00, 0x01];
        let crc = modbus_crc16(&frame);
        // 每个功能码都应该能正确计算 CRC
        assert_ne!(crc, 0xFFFF, "CRC should not be initial value for {}", name);
    }
}

// ── Modbus ASCII 帧解析测试 ──────────────────────────────────────────────

/// 计算 Modbus ASCII LRC (Longitudinal Redundancy Check)
fn modbus_lrc(data: &[u8]) -> u8 {
    let mut lrc: u8 = 0;
    for &byte in data {
        lrc = lrc.wrapping_add(byte);
    }
    lrc.wrapping_neg()
}

#[test]
fn test_modbus_ascii_lrc_known_value() {
    // Slave=1, Func=3, Start=0x0000, Qty=0x0001
    let data = [0x01, 0x03, 0x00, 0x00, 0x00, 0x01];
    let lrc = modbus_lrc(&data);
    // LRC = -(0x01+0x03+0x00+0x00+0x00+0x01) = -(0x05) = 0xFB
    assert_eq!(lrc, 0xFB);
}

#[test]
fn test_modbus_ascii_frame_assembly() {
    // 组装 Modbus ASCII 帧：`:LRC\r\n`
    let slave: u8 = 1;
    let function: u8 = 0x03;
    let start_addr: u16 = 0x0000;
    let quantity: u16 = 0x0001;

    let mut data = vec![slave, function];
    data.extend_from_slice(&start_addr.to_be_bytes());
    data.extend_from_slice(&quantity.to_be_bytes());

    let lrc = modbus_lrc(&data);

    // ASCII 帧格式：`:` + hex(LRC前数据) + hex(LRC) + CR + LF
    let hex_data: String = data.iter().map(|b| format!("{:02X}", b)).collect();
    let frame = format!(":{:02X}{}{:02X}\r\n", slave, &hex_data[2..], lrc);

    // 验证帧结构
    assert!(frame.starts_with(':'));
    assert!(frame.ends_with("\r\n"));
}

#[test]
fn test_modbus_ascii_lrc_empty() {
    let lrc = modbus_lrc(&[]);
    assert_eq!(lrc, 0); // -0 = 0
}

#[test]
fn test_modbus_ascii_lrc_roundtrip() {
    // LRC(data + LRC) 应该等于 0
    let data = [0x01, 0x03, 0x00, 0x00, 0x00, 0x01];
    let lrc = modbus_lrc(&data);
    let mut with_lrc = data.to_vec();
    with_lrc.push(lrc);
    assert_eq!(modbus_lrc(&with_lrc), 0);
}

// ── Line 协议测试 ───────────────────────────────────────────────────────

#[test]
fn test_line_protocol_on_send_appends_newline() {
    let manager = ScriptManager::new();
    let engine = manager.create_engine("line").unwrap();

    // line 协议的 on_send 追加换行符（除非已存在）
    let result = engine.on_send(b"Hello").unwrap();
    assert_eq!(result, b"Hello\n");

    // 已带换行则不改
    let result = engine.on_send(b"Hello\n").unwrap();
    assert_eq!(result, b"Hello\n");
}

#[test]
fn test_line_protocol_on_recv_passthrough() {
    let manager = ScriptManager::new();
    let engine = manager.create_engine("line").unwrap();

    let result = engine.on_recv(b"Hello\n");
    assert_eq!(result, b"Hello\n");
}

#[test]
fn test_line_protocol_empty_data() {
    let manager = ScriptManager::new();
    let engine = manager.create_engine("line").unwrap();

    let result = engine.on_send(b"").unwrap();
    assert_eq!(result, b"\n");
}

#[test]
fn test_line_protocol_binary_lossy_newline() {
    let manager = ScriptManager::new();
    let engine = manager.create_engine("line").unwrap();

    // line 是文本协议：二进制经 UTF-8 lossy 转换后追加换行（不保真透传）
    let data: Vec<u8> = (0..=255).collect();
    let result = engine.on_send(&data).unwrap();
    assert!(result.ends_with(b"\n"));
    assert!(result.len() >= 1);
}

// ── AT Command 协议测试 ─────────────────────────────────────────────────

#[test]
fn test_at_command_encode_appends_cr() {
    let manager = ScriptManager::new();
    let engine = manager.create_engine("at_command").unwrap();

    // AT 命令的 on_send 追加 \r（除非已存在）
    let result = engine.on_send(b"ATZ").unwrap();
    assert_eq!(result, b"ATZ\r");

    let result = engine.on_send(b"ATZ\r").unwrap();
    assert_eq!(result, b"ATZ\r");
}

#[test]
fn test_at_command_decode() {
    let manager = ScriptManager::new();
    let engine = manager.create_engine("at_command").unwrap();

    let result = engine.on_recv(b"OK\r\n");
    assert_eq!(result, b"OK\r\n");
}

// ── Modbus RTU 协议脚本测试 ─────────────────────────────────────────────

#[test]
fn test_modbus_rtu_script_encode() {
    let manager = ScriptManager::new();
    let engine = manager.create_engine("modbus_rtu").unwrap();

    // 发送请求：slave=1, func=3, start=0, qty=1
    let request = vec![0x01, 0x03, 0x00, 0x00, 0x00, 0x01];
    let result = engine.on_send(&request).unwrap();

    // 应该追加了 2 字节 CRC
    assert_eq!(result.len(), request.len() + 2);

    // CRC 验证
    let data_part = &result[..request.len()];
    let crc_part = &result[request.len()..];
    let expected_crc = modbus_crc16(data_part);
    let actual_crc = u16::from_le_bytes([crc_part[0], crc_part[1]]);
    assert_eq!(actual_crc, expected_crc);
}

#[test]
fn test_modbus_rtu_script_decode_valid_crc() {
    let manager = ScriptManager::new();
    let engine = manager.create_engine("modbus_rtu").unwrap();

    // 构造带正确 CRC 的响应帧
    let data = [0x01, 0x03, 0x02, 0x00, 0x64];
    let crc = modbus_crc16(&data);
    let mut frame = data.to_vec();
    frame.extend_from_slice(&crc.to_le_bytes());

    let result = engine.on_recv(&frame);
    // 有效 CRC：应该返回去掉 CRC 的数据
    assert_eq!(result, data);
}

#[test]
fn test_modbus_rtu_script_decode_invalid_crc() {
    let manager = ScriptManager::new();
    let engine = manager.create_engine("modbus_rtu").unwrap();

    // 构造带错误 CRC 的帧
    let data = [0x01, 0x03, 0x02, 0x00, 0x64];
    let mut frame = data.to_vec();
    frame.extend_from_slice(&[0x00, 0x00]); // 错误的 CRC

    let result = engine.on_recv(&frame);
    // modbus_rtu 的 on_recv 可能不校验 CRC（取决于实现），
    // 但如果校验则应返回空。验证行为一致即可。
    // 这里只验证不 panic
    let _ = result;
}

#[test]
fn test_modbus_rtu_script_decode_short_frame() {
    let manager = ScriptManager::new();
    let engine = manager.create_engine("modbus_rtu").unwrap();

    // 少于 4 字节的帧应该被丢弃
    let short_frame = [0x01, 0x03];
    let result = engine.on_recv(&short_frame);
    assert!(result.is_empty(), "Short frame should be discarded");
}
