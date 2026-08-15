//! Extended protocol script tests (map #84).
//!
//! The 12 previously-unreferenced scripts in `scripts/protocols/` are now
//! loadable, smoke-tested, and documented in `docs/reference/protocols.md`.
//! These tests guarantee each script loads, carries `SCRIPT_META`, and its
//! framing functions behave for representative payloads.

use serial_cli::script::ScriptManager;

const EXTENDED_PROTOCOLS: [&str; 11] = [
    "can",
    "dlt645",
    "dmx512",
    "i2c_uart",
    "mqtt_serial",
    "nmea0183",
    "onewire",
    "pzem004t",
    "sdi12",
    "spi_uart",
    "temp_sensor",
];

fn engine(name: &str) -> serial_cli::serial_core::SerialScriptEngine {
    let manager = ScriptManager::new();
    manager.create_engine(name).unwrap_or_else(|e| panic!("{name} failed to load: {e}"))
}

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

/// All extended protocols load and are discoverable.
#[test]
fn all_extended_protocols_load_and_list() {
    let manager = ScriptManager::new();
    let listed = manager.list();
    for name in EXTENDED_PROTOCOLS {
        assert!(
            listed.iter().any(|s| s.name == name),
            "{name} missing from script list"
        );
        // Loading must succeed and produce a working engine.
        let _ = manager.create_engine(name).unwrap_or_else(|e| panic!("{name}: {e}"));
    }
}

// ── can (SLCAN/Lawicel) ─────────────────────────────────────────────────

#[test]
fn can_protocol_send_and_recv() {
    let eng = engine("can");
    // SLCAN command input: id_lo, id_hi, dlc, payload...
    let frame = eng.on_send(&[0x01, 0x02, 3, 0xAA, 0xBB, 0xCC]).unwrap();
    assert!(!frame.is_empty(), "can on_send returned empty frame");

    // on_recv parses raw SLCAN text lines ("t" + id + dlc + data + CR)
    let parsed = eng.on_recv(b"t01020304\r");
    assert!(!parsed.is_empty(), "can on_recv failed to parse SLCAN line");
}

// ── dlt645 (Chinese smart meter) ────────────────────────────────────────

#[test]
fn dlt645_protocol_frame_roundtrip() {
    let eng = engine("dlt645");
    // Valid request: 0x68 + 6 addr + 0x68 + ctrl + len + data
    let request = vec![0x68, 1, 2, 3, 4, 5, 6, 0x68, 0x13, 0x04, 0x01, 0x02, 0x03, 0x04];
    let frame = eng.on_send(&request).unwrap();
    assert!(frame.len() > request.len(), "dlt645 should add framing/checksum");

    let out = eng.on_recv(&frame);
    assert!(!out.is_empty(), "dlt645 on_recv failed to parse its own frame");
}

// ── dmx512 (stage lighting) ─────────────────────────────────────────────

#[test]
fn dmx512_protocol_roundtrip() {
    let eng = engine("dmx512");
    let channels: Vec<u8> = (1..=16).collect();
    let frame = eng.on_send(&channels).unwrap();
    assert!(frame.len() > channels.len(), "dmx512 should add framing");

    let out = eng.on_recv(&frame);
    assert!(!out.is_empty(), "dmx512 on_recv failed to parse its own frame");
}

// ── i2c_uart (bridge) ───────────────────────────────────────────────────

#[test]
fn i2c_uart_protocol_roundtrip() {
    let eng = engine("i2c_uart");
    let payload = vec![0x12, 0x34, 0x56, 0x78];
    let frame = eng.on_send(&payload).unwrap();
    assert!(!frame.is_empty());
    assert!(frame.len() > payload.len(), "i2c_uart should add framing");

    let out = eng.on_recv(&frame);
    assert_eq!(out, payload, "i2c_uart on_recv should strip framing + checksum");
}

// ── mqtt_serial (AT based) ──────────────────────────────────────────────

#[test]
fn mqtt_serial_at_termination() {
    let eng = engine("mqtt_serial");
    // AT commands get a trailing CR; plain text passes through.
    assert_eq!(eng.on_send(b"AT").unwrap(), b"AT\r");
    assert_eq!(eng.on_send(b"AT\r").unwrap(), b"AT\r");
    assert_eq!(eng.on_send(b"hello").unwrap(), b"hello");
}

#[test]
fn mqtt_serial_response_buffering() {
    // Partial (unmatched) data buffers to nil; a fresh engine sees "OK\r\n" complete.
    let eng = engine("mqtt_serial");
    assert!(eng.on_recv(b"CONN").is_empty());
    assert!(eng.on_recv(b"CONNECTED").is_empty());

    let eng2 = engine("mqtt_serial");
    let out = eng2.on_recv(b"OK\r\n");
    assert!(String::from_utf8_lossy(&out).contains("OK"));
}

// ── nmea0183 (GPS/marine) ───────────────────────────────────────────────

#[test]
fn nmea0183_checksum_append_and_parse() {
    let eng = engine("nmea0183");
    let sentence = b"$GPGGA,123519,4807.038,N,01131.000,E,1,08";
    let framed = eng.on_send(sentence).unwrap();
    let text = String::from_utf8_lossy(&framed);
    assert!(text.starts_with('$'), "nmea on_send should keep $ prefix");
    assert!(text.ends_with("\r\n"), "nmea on_send should terminate with CRLF");
    assert!(text.contains('*'), "nmea on_send should append *CS");

    let out = eng.on_recv(&framed);
    assert!(
        String::from_utf8_lossy(&out).contains("$GPGGA,123519"),
        "nmea on_recv should recover the sentence"
    );
}

// ── onewire (DS18B20 bridge) ────────────────────────────────────────────

#[test]
fn onewire_protocol_roundtrip() {
    let eng = engine("onewire");
    let cmd = vec![0x44, 0xAA, 0xBB]; // convert + payload
    let frame = eng.on_send(&cmd).unwrap();
    assert!(!frame.is_empty());
    assert!(frame.len() > cmd.len(), "onewire should add framing");

    let out = eng.on_recv(&frame);
    assert_eq!(out, cmd, "onewire on_recv should strip LEN + checksum");
}

// ── pzem004t (Modbus RTU variant) ───────────────────────────────────────

#[test]
fn pzem004t_append_crc_via_lib() {
    let eng = engine("pzem004t");
    // PZEM uses the shared modbus_rtu_lib (dedup, #83) — validates require().
    let request = vec![0x01, 0x03, 0x00, 0x00, 0x00, 0x0A];
    let frame = eng.on_send(&request).unwrap();
    assert_eq!(frame.len(), request.len() + 2);

    let data_part = &frame[..request.len()];
    let crc_part = &frame[request.len()..];
    assert_eq!(u16::from_le_bytes([crc_part[0], crc_part[1]]), modbus_crc16(data_part));
}

#[test]
fn pzem004t_decode_valid_crc() {
    let eng = engine("pzem004t");
    let data = [0x01, 0x03, 0x02, 0x00, 0x64];
    let mut frame = data.to_vec();
    frame.extend_from_slice(&modbus_crc16(&data).to_le_bytes());
    assert_eq!(eng.on_recv(&frame), data);
}

// ── sdi12 (environmental sensor) ────────────────────────────────────────

#[test]
fn sdi12_command_terminator() {
    let eng = engine("sdi12");
    assert_eq!(eng.on_send(b"M0").unwrap(), b"M0!");
    assert_eq!(eng.on_send(b"M0!").unwrap(), b"M0!");
    assert_eq!(eng.on_send(b"R0").unwrap(), b"R0!");
}

#[test]
fn sdi12_response_buffering() {
    // Partial data (no CRLF yet) buffers to nil; the script keeps the residue.
    let eng = engine("sdi12");
    assert!(eng.on_recv(b"+0.0+1.2").is_empty());

    // Fresh engine: a complete CRLF-terminated response is returned.
    let eng2 = engine("sdi12");
    let out = eng2.on_recv(b"+0.0+1.2\r\n");
    assert_eq!(out, b"+0.0+1.2\r\n");
}

// ── spi_uart (bridge) ───────────────────────────────────────────────────

#[test]
fn spi_uart_protocol_roundtrip() {
    let eng = engine("spi_uart");
    let payload = vec![0xDE, 0xAD, 0xBE, 0xEF];
    let frame = eng.on_send(&payload).unwrap();
    assert!(!frame.is_empty());
    assert!(frame.len() > payload.len(), "spi_uart should add framing");

    let out = eng.on_recv(&frame);
    assert_eq!(out, payload, "spi_uart on_recv should strip framing + checksum");
}

// ── temp_sensor (Modbus RTU driver) ─────────────────────────────────────

#[test]
fn temp_sensor_append_crc_via_lib() {
    let eng = engine("temp_sensor");
    let request = vec![0x01, 0x03, 0x00, 0x00, 0x00, 0x01];
    let frame = eng.on_send(&request).unwrap();
    assert_eq!(frame.len(), request.len() + 2);

    let data_part = &frame[..request.len()];
    let crc_part = &frame[request.len()..];
    assert_eq!(u16::from_le_bytes([crc_part[0], crc_part[1]]), modbus_crc16(data_part));
}

#[test]
fn temp_sensor_decode_valid_response() {
    let eng = engine("temp_sensor");
    let data = [0x01, 0x03, 0x02, 0x00, 0x64];
    let mut frame = data.to_vec();
    frame.extend_from_slice(&modbus_crc16(&data).to_le_bytes());
    let out = eng.on_recv(&frame);
    assert!(!out.is_empty(), "temp_sensor on_recv failed on valid response");
}
