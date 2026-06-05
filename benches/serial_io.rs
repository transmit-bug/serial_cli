//! Serial I/O throughput benchmarks
//!
//! Measures buffer copy, protocol encode/decode round-trips,
//! and raw CRC/LRC computation speed.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use serial_cli::protocol::built_in::modbus::ModbusMode;
use serial_cli::protocol::built_in::ModbusProtocol;
use serial_cli::protocol::Protocol;

/// Benchmark buffer copy throughput at various sizes
fn bench_buffer_copy(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer_copy");

    for size in [64, 256, 1024, 4096, 16384] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            let data = vec![0u8; size];
            b.iter(|| black_box(black_box(&data).clone()));
        });
    }

    group.finish();
}

/// Benchmark full Modbus RTU encode + parse round-trip
fn bench_modbus_rtu_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("modbus_rtu_roundtrip");

    let payload = vec![0x01, 0x03, 0x00, 0x00, 0x00, 0x0A];

    group.bench_function("encode_and_parse", |b| {
        b.iter(|| {
            let mut proto = ModbusProtocol::new(ModbusMode::Rtu);
            let encoded = proto.encode(black_box(&payload)).unwrap();
            black_box(proto.parse(black_box(&encoded)).unwrap())
        });
    });

    group.finish();
}

/// Benchmark Modbus ASCII round-trip (more expensive due to hex encoding)
fn bench_modbus_ascii_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("modbus_ascii_roundtrip");

    let payload = vec![0x01, 0x03, 0x00, 0x00, 0x00, 0x0A];

    group.bench_function("encode_and_parse", |b| {
        b.iter(|| {
            let mut proto = ModbusProtocol::new(ModbusMode::Ascii);
            let encoded = proto.encode(black_box(&payload)).unwrap();
            black_box(proto.parse(black_box(&encoded)).unwrap())
        });
    });

    group.finish();
}

/// Benchmark standalone CRC16 computation
fn bench_modbus_crc16(c: &mut Criterion) {
    let mut group = c.benchmark_group("modbus_crc16");

    let data: Vec<u8> = (0..256).map(|i| i as u8).collect();

    group.bench_function("compute", |b| {
        b.iter(|| black_box(ModbusProtocol::calculate_crc(black_box(&data))));
    });

    group.finish();
}

/// Benchmark standalone LRC computation
fn bench_modbus_lrc(c: &mut Criterion) {
    let mut group = c.benchmark_group("modbus_lrc");

    let data: Vec<u8> = (0..256).map(|i| i as u8).collect();

    group.bench_function("compute", |b| {
        b.iter(|| black_box(ModbusProtocol::calculate_lrc(black_box(&data))));
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_buffer_copy,
    bench_modbus_rtu_roundtrip,
    bench_modbus_ascii_roundtrip,
    bench_modbus_crc16,
    bench_modbus_lrc,
);
criterion_main!(benches);
