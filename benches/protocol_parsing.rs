//! Protocol encoding/decoding benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serial_cli::script::ScriptManager;

/// Benchmark AT command encoding/decoding
fn bench_at_command_roundtrip(c: &mut Criterion) {
    let manager = ScriptManager::new();
    let engine = manager.create_engine("at_command").unwrap();

    c.bench_function("at_command_roundtrip", |b| {
        b.iter(|| {
            let input = b"AT+CSQ".to_vec();
            let encoded = engine.on_send(&input).unwrap();
            let decoded = engine.on_recv(black_box(&encoded));
            black_box(decoded)
        })
    });
}

/// Benchmark Modbus RTU encoding/decoding
fn bench_modbus_rtu_roundtrip(c: &mut Criterion) {
    let manager = ScriptManager::new();
    let engine = manager.create_engine("modbus_rtu").unwrap();

    c.bench_function("modbus_rtu_roundtrip", |b| {
        b.iter(|| {
            let input = vec![0x01, 0x03, 0x00, 0x00, 0x00, 0x0A];
            let encoded = engine.on_send(black_box(&input)).unwrap();
            let decoded = engine.on_recv(black_box(&encoded));
            black_box(decoded)
        })
    });
}

/// Benchmark Modbus ASCII encoding/decoding
fn bench_modbus_ascii_roundtrip(c: &mut Criterion) {
    let manager = ScriptManager::new();
    let engine = manager.create_engine("modbus_ascii").unwrap();

    c.bench_function("modbus_ascii_roundtrip", |b| {
        b.iter(|| {
            let input = vec![0x01, 0x03, 0x00, 0x00, 0x00, 0x0A];
            let encoded = engine.on_send(black_box(&input)).unwrap();
            let decoded = engine.on_recv(black_box(&encoded));
            black_box(decoded)
        })
    });
}

/// Benchmark Line protocol encoding/decoding
fn bench_line_roundtrip(c: &mut Criterion) {
    let manager = ScriptManager::new();
    let engine = manager.create_engine("line").unwrap();

    c.bench_function("line_roundtrip", |b| {
        b.iter(|| {
            let input = b"Hello, World!".to_vec();
            let encoded = engine.on_send(&input).unwrap();
            let decoded = engine.on_recv(black_box(&encoded));
            black_box(decoded)
        })
    });
}

criterion_group!(
    benches,
    bench_at_command_roundtrip,
    bench_modbus_rtu_roundtrip,
    bench_modbus_ascii_roundtrip,
    bench_line_roundtrip,
);
criterion_main!(benches);
