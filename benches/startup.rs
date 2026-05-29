//! Application startup benchmarks
//!
//! Measures cold/warm config load, Lua engine initialization,
//! and Lua protocol script loading times.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serial_cli::config::ConfigManager;
use serial_cli::lua::LuaEngine;
use serial_cli::protocol::loader::ProtocolLoader;
use std::env;
use std::fs::File;
use std::io::Write;

/// Benchmark cold ConfigManager initialization (no config file)
fn bench_config_cold_start(c: &mut Criterion) {
    c.bench_function("config_cold_start", |b| {
        b.iter(|| black_box(ConfigManager::new()))
    });
}

/// Benchmark warm ConfigManager initialization (with fallback defaults)
fn bench_config_warm_start(c: &mut Criterion) {
    c.bench_function("config_warm_start", |b| {
        b.iter(|| black_box(ConfigManager::load_with_fallback()))
    });
}

/// Benchmark Lua engine initialization
fn bench_lua_engine_init(c: &mut Criterion) {
    c.bench_function("lua_engine_init", |b| {
        b.iter(|| {
            let _engine = black_box(LuaEngine::new().unwrap());
        })
    });
}

/// Benchmark loading a Lua protocol script
fn bench_protocol_load(c: &mut Criterion) {
    let script_content = r#"
        -- Protocol: benchmark_proto
        function on_frame(data)
            return data
        end

        function on_encode(data)
            return data
        end
    "#;

    c.bench_function("protocol_load_lua_script", |b| {
        b.iter(|| {
            let temp_dir = env::temp_dir();
            let temp_file_path =
                temp_dir.join(format!("benchmark_proto_{}.lua", std::process::id()));

            let mut file = File::create(&temp_file_path).expect("create temp file");
            file.write_all(script_content.as_bytes()).expect("write temp file");

            let _loaded = black_box(ProtocolLoader::load_from_file(&temp_file_path).unwrap());

            let _ = std::fs::remove_file(&temp_file_path);
        })
    });
}

criterion_group!(
    benches,
    bench_config_cold_start,
    bench_config_warm_start,
    bench_lua_engine_init,
    bench_protocol_load,
);
criterion_main!(benches);
