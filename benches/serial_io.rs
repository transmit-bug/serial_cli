//! Serial I/O throughput benchmarks

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use serial_cli::script::ScriptManager;

/// Benchmark script execution throughput
fn bench_script_execution(c: &mut Criterion) {
    let manager = ScriptManager::new();

    let mut group = c.benchmark_group("script_execution");

    for protocol in ["at_command", "modbus_rtu", "modbus_ascii", "line"] {
        let engine = manager.create_engine(protocol).unwrap();

        group.bench_function(BenchmarkId::new(protocol, "on_send"), |b| {
            let input = vec![0x01, 0x03, 0x00, 0x00, 0x00, 0x0A];
            b.iter(|| {
                black_box(engine.on_send(black_box(&input)))
            })
        });

        group.bench_function(BenchmarkId::new(protocol, "on_recv"), |b| {
            let input = vec![0x01, 0x03, 0x00, 0x00, 0x00, 0x0A];
            let encoded = engine.on_send(&input).unwrap();
            b.iter(|| {
                black_box(engine.on_recv(black_box(&encoded)))
            })
        });
    }

    group.finish();
}

/// Benchmark large payload processing
fn bench_large_payload(c: &mut Criterion) {
    let manager = ScriptManager::new();
    let engine = manager.create_engine("modbus_rtu").unwrap();

    let mut group = c.benchmark_group("large_payload");

    for size in [64, 256, 1024, 4096] {
        let input: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();

        group.bench_function(BenchmarkId::new("on_send", size), |b| {
            b.iter(|| {
                black_box(engine.on_send(black_box(&input)))
            })
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_script_execution,
    bench_large_payload,
);
criterion_main!(benches);
