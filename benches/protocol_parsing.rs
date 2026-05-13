//! Protocol encoding/decoding benchmarks

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use serial_cli::protocol::built_in::modbus::ModbusMode;
use serial_cli::protocol::built_in::{AtCommandProtocol, LineProtocol, ModbusProtocol};
use serial_cli::protocol::Protocol;

mod common;

/// Benchmark AT command parsing
fn bench_at_command_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("at_command_parsing");

    // Parse OK response
    group.bench_function("ok_response", |b| {
        let mut protocol = AtCommandProtocol::new();
        let data = b"OK\r\n";

        b.iter(|| black_box(protocol.parse(black_box(data)).unwrap()));
    });

    // Parse ERROR response
    group.bench_function("error_response", |b| {
        let mut protocol = AtCommandProtocol::new();
        let data = b"ERROR\r\n";

        b.iter(|| {
            let result = protocol.parse(black_box(data));
            assert!(result.is_err() || result.is_ok()); // May error, that's OK for benchmark
        });
    });

    // Parse multi-line response
    group.bench_function("multiline_response", |b| {
        let mut protocol = AtCommandProtocol::new();
        let data = b"+CWLAP: (4,\"MyNetwork\")\r\n+CWLAP: (3,\"OtherNetwork\")\r\nOK\r\n";

        b.iter(|| black_box(protocol.parse(black_box(data)).unwrap()));
    });

    group.finish();
}

/// Benchmark AT command encoding
fn bench_at_command_encoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("at_command_encoding");

    for size in [8, 16, 32, 64].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut protocol = AtCommandProtocol::new();
            let data = vec![b'A'; size];

            b.iter(|| black_box(protocol.encode(black_box(&data)).unwrap()));
        });
    }

    group.finish();
}

/// Benchmark Modbus RTU encoding
fn bench_modbus_rtu_encoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("modbus_rtu_encoding");

    // Read holding registers (function 0x03)
    group.bench_function("read_holding_registers", |b| {
        let protocol = ModbusProtocol::new(ModbusMode::Rtu);
        let data = [0x00, 0x00, 0x00, 0x0A]; // start_addr=0x0000, quantity=10

        b.iter(|| {
            black_box(protocol.encode_request(black_box(1), black_box(0x03), black_box(&data)))
                .unwrap()
        });
    });

    // Write single register (function 0x06)
    group.bench_function("write_single_register", |b| {
        let protocol = ModbusProtocol::new(ModbusMode::Rtu);
        let data = [0x00, 0x00, 0x00, 0x01]; // start_addr=0x0000, value=0x0001

        b.iter(|| {
            black_box(protocol.encode_request(black_box(1), black_box(0x06), black_box(&data)))
                .unwrap()
        });
    });

    // Variable register count
    for count in [1, 10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("variable_registers", count),
            count,
            |b, &count| {
                let protocol = ModbusProtocol::new(ModbusMode::Rtu);
                let data = [0x00, 0x00, (count >> 8) as u8, (count & 0xFF) as u8]; // start_addr=0x0000, quantity=count

                b.iter(|| {
                    black_box(protocol.encode_request(
                        black_box(1),
                        black_box(0x03),
                        black_box(&data),
                    ))
                    .unwrap()
                });
            },
        );
    }

    group.finish();
}

/// Benchmark Modbus RTU decoding
fn bench_modbus_rtu_decoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("modbus_rtu_decoding");

    // Parse normal response
    group.bench_function("normal_response", |b| {
        let mut protocol = ModbusProtocol::new(ModbusMode::Rtu);
        // Create response: [slave_id, function_code, byte_count, data... , crc_lo, crc_hi]
        let mut response = vec![1, 0x03, 20]; // 20 bytes = 10 registers * 2
        response.extend_from_slice(&[0u8; 20]);
        // Calculate and append CRC
        let crc_data = &response.clone();
        let crc = calculate_crc_for_modbus(crc_data);
        response.extend_from_slice(&crc.to_le_bytes());

        b.iter(|| black_box(protocol.parse_response(black_box(&response))));
    });

    group.finish();
}

/// Helper function to calculate Modbus CRC16 (matching the protocol implementation)
fn calculate_crc_for_modbus(data: &[u8]) -> u16 {
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

/// Benchmark Modbus ASCII encoding
fn bench_modbus_ascii_encoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("modbus_ascii_encoding");

    // Read holding registers (function 0x03)
    group.bench_function("read_holding_registers", |b| {
        let protocol = ModbusProtocol::new(ModbusMode::Ascii);
        let data = [0x00, 0x00, 0x00, 0x0A]; // start_addr=0x0000, quantity=10

        b.iter(|| {
            black_box(protocol.encode_request(black_box(1), black_box(0x03), black_box(&data)))
                .unwrap()
        });
    });

    // Write single register (function 0x06)
    group.bench_function("write_single_register", |b| {
        let protocol = ModbusProtocol::new(ModbusMode::Ascii);
        let data = [0x00, 0x00, 0x00, 0x01]; // start_addr=0x0000, value=0x0001

        b.iter(|| {
            black_box(protocol.encode_request(black_box(1), black_box(0x06), black_box(&data)))
                .unwrap()
        });
    });

    // Variable register count
    for count in [1, 10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("variable_registers", count),
            count,
            |b, &count| {
                let protocol = ModbusProtocol::new(ModbusMode::Ascii);
                let data = [0x00, 0x00, (count >> 8) as u8, (count & 0xFF) as u8];

                b.iter(|| {
                    black_box(protocol.encode_request(
                        black_box(1),
                        black_box(0x03),
                        black_box(&data),
                    ))
                    .unwrap()
                });
            },
        );
    }

    // Protocol trait encode benchmark
    for size in [4, 10, 20, 40].iter() {
        group.bench_with_input(BenchmarkId::new("encode", size), size, |b, &size| {
            let mut protocol = ModbusProtocol::new(ModbusMode::Ascii);
            let data = vec![0xABu8; size];

            b.iter(|| black_box(protocol.encode(black_box(&data)).unwrap()));
        });
    }

    group.finish();
}

/// Benchmark Modbus ASCII decoding
fn bench_modbus_ascii_decoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("modbus_ascii_decoding");

    // Parse normal response via parse_response
    group.bench_function("normal_response", |b| {
        let mut protocol = ModbusProtocol::new(ModbusMode::Ascii);
        // Create payload: slave_id + func_code + data
        let mut payload = vec![1, 0x03, 20];
        payload.extend_from_slice(&[0u8; 20]);

        // Use encode to build a valid ASCII frame
        let ascii_frame = protocol.encode(&payload).unwrap();

        b.iter(|| black_box(protocol.parse(black_box(&ascii_frame)).unwrap()));
    });

    // Parse via Protocol trait (encode then parse back)
    group.bench_function("round_trip", |b| {
        let mut protocol = ModbusProtocol::new(ModbusMode::Ascii);
        let data = vec![0x01, 0x03, 0x02, 0x00, 0x0A];
        let encoded = protocol.encode(&data).unwrap();

        b.iter(|| black_box(protocol.parse(black_box(&encoded)).unwrap()));
    });

    group.finish();
}

/// Benchmark Line protocol framing
fn bench_line_protocol_framing(c: &mut Criterion) {
    let mut group = c.benchmark_group("line_protocol_framing");

    // Single line
    group.bench_function("single_line", |b| {
        let mut protocol = LineProtocol::new();
        let data = b"Hello, World!\n";

        b.iter(|| black_box(protocol.parse(black_box(data)).unwrap()));
    });

    // Multiple lines
    for line_count in [2, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::new("multiline", line_count),
            line_count,
            |b, &line_count| {
                let mut protocol = LineProtocol::new();
                let data: String = (0..line_count).map(|i| format!("Line {}\n", i)).collect();
                let data = data.as_bytes();

                b.iter(|| black_box(protocol.parse(black_box(data)).unwrap()));
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_at_command_parsing,
    bench_at_command_encoding,
    bench_modbus_rtu_encoding,
    bench_modbus_rtu_decoding,
    bench_modbus_ascii_encoding,
    bench_modbus_ascii_decoding,
    bench_line_protocol_framing
);
criterion_main!(benches);
