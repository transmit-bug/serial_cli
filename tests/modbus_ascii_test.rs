//! Modbus ASCII protocol integration tests
//!
//! 验证 Modbus ASCII 协议的编解码功能

use serial_cli::serial_core::serial_script::SerialScriptEngine;

/// 测试 Modbus ASCII 协议的 on_send 回调（编码）
#[tokio::test]
async fn test_modbus_ascii_on_send() {
    let lua_source = include_str!("../src/script/built_in/modbus_ascii.lua");

    let engine = SerialScriptEngine::new(lua_source).unwrap();
    engine.load().unwrap();

    // 测试数据：地址 0x01，功能码 0x03，起始地址 0x0000，寄存器数量 0x0001
    let test_data = vec![0x01, 0x03, 0x00, 0x00, 0x00, 0x01];

    let encoded = engine.on_send(&test_data).unwrap();

    // 验证结果不为空
    assert!(!encoded.is_empty());

    // 验证以 ':' 开头 (0x3A)
    assert_eq!(encoded[0], 0x3A);

    // 验证以 CR LF 结尾 (0x0D 0x0A)
    assert_eq!(encoded[encoded.len() - 2], 0x0D);
    assert_eq!(encoded[encoded.len() - 1], 0x0A);

    // 将结果转换为字符串以便检查
    let result_str: String = encoded.iter().map(|&b| b as char).collect();

    // 验证格式：:[HEX_DATA][LRC_HEX]\r\n
    assert!(result_str.starts_with(':'));
    assert!(result_str.ends_with("\r\n"));
}

/// 测试 Modbus ASCII 协议的 on_recv 回调（解码）
#[tokio::test]
async fn test_modbus_ascii_on_recv() {
    let lua_source = include_str!("../src/script/built_in/modbus_ascii.lua");

    let engine = SerialScriptEngine::new(lua_source).unwrap();
    engine.load().unwrap();

    // 构造一个 Modbus ASCII 响应帧
    // 地址 0x01，功能码 0x03，字节数 0x02，数据 0x00 0x0A
    let data_bytes = vec![0x01, 0x03, 0x02, 0x00, 0x0A];

    // 计算 LRC
    let lrc_sum: u8 = data_bytes.iter().fold(0u8, |acc, &x| acc.wrapping_add(x));
    let lrc = ((256u16 - (lrc_sum as u16 % 256)) % 256) as u8;

    // 构造完整帧：:[DATA][LRC]\r\n
    let mut frame_bytes = data_bytes.clone();
    frame_bytes.push(lrc);

    let hex_str = format!(
        ":{}\r\n",
        frame_bytes
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join("")
    );
    let ascii_bytes: Vec<u8> = hex_str.bytes().collect();

    let decoded = engine.on_recv(&ascii_bytes);

    // 验证解码结果与原始数据匹配
    assert_eq!(decoded, data_bytes);
}

/// 测试 Modbus ASCII 协议的往返编码（发送后接收应该能还原）
#[tokio::test]
async fn test_modbus_ascii_roundtrip() {
    let lua_source = include_str!("../src/script/built_in/modbus_ascii.lua");

    let engine = SerialScriptEngine::new(lua_source).unwrap();
    engine.load().unwrap();

    // 原始数据
    let original_data = vec![0x01, 0x03, 0x00, 0x00, 0x00, 0x0A];

    // 编码
    let encoded = engine.on_send(&original_data).unwrap();

    // 解码
    let decoded = engine.on_recv(&encoded);

    // 验证往返一致
    assert_eq!(decoded, original_data);
}

/// 测试 LRC 校验失败的情况
#[tokio::test]
async fn test_modbus_ascii_lrc_failure() {
    let lua_source = include_str!("../src/script/built_in/modbus_ascii.lua");

    let engine = SerialScriptEngine::new(lua_source).unwrap();
    engine.load().unwrap();

    // 构造一个错误的 Modbus ASCII 帧（LRC 错误）
    // 数据：01 03 00 00 00 01，正确的 LRC 应该是 FB，这里使用 FF（错误）
    let invalid_frame = b":010300000001FF\r\n";
    let ascii_bytes: Vec<u8> = invalid_frame.to_vec();

    let result = engine.on_recv(&ascii_bytes);

    // LRC 校验失败应该返回空数组
    assert!(result.is_empty());
}
