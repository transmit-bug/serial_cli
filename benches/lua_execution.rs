//! Lua execution benchmarks

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use serial_cli::script::ScriptManager;

/// Benchmark script loading and validation
fn bench_script_load(c: &mut Criterion) {
    let manager = ScriptManager::new();

    c.bench_function("script_load_builtin", |b| {
        b.iter(|| {
            let engine = manager.create_engine(black_box("modbus_rtu")).unwrap();
            black_box(engine)
        })
    });
}

/// Benchmark script validation
fn bench_script_validation(c: &mut Criterion) {
    let manager = ScriptManager::new();

    c.bench_function("script_validate_source", |b| {
        let source = r#"
            function on_send(data)
                return data
            end
            function on_recv(data)
                return data
            end
        "#;
        b.iter(|| black_box(ScriptManager::validate_source(black_box(source))))
    });
}

/// Benchmark script list operations
fn bench_script_list(c: &mut Criterion) {
    let manager = ScriptManager::new();

    c.bench_function("script_list", |b| b.iter(|| black_box(manager.list())));
}

/// Benchmark script get_source
fn bench_script_get_source(c: &mut Criterion) {
    let manager = ScriptManager::new();

    let mut group = c.benchmark_group("script_get_source");

    for protocol in ["at_command", "modbus_rtu", "modbus_ascii", "line"] {
        group.bench_function(BenchmarkId::new(protocol, "get_source"), |b| {
            b.iter(|| black_box(manager.get_source(black_box(protocol))))
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_script_load,
    bench_script_validation,
    bench_script_list,
    bench_script_get_source,
);
criterion_main!(benches);
