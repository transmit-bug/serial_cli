//! Lua script execution benchmarks

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use serial_cli::lua::LuaEngine;

/// Benchmark script execution overhead
fn bench_script_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("script_overhead");

    // Empty script baseline
    group.bench_function("empty", |b| {
        b.iter(|| {
            let engine = LuaEngine::new().unwrap();
            let lua = engine.lua();
            black_box(lua.load("").exec()).unwrap()
        });
    });

    // Simple script
    group.bench_function("simple", |b| {
        b.iter(|| {
            let engine = LuaEngine::new().unwrap();
            let lua = engine.lua();
            black_box(lua.load("return 1 + 1").exec()).unwrap()
        });
    });

    // Complex calculation
    group.bench_function("complex", |b| {
        let script = r#"
            local function fibonacci(n)
                if n <= 1 then return n end
                return fibonacci(n - 1) + fibonacci(n - 2)
            end
            return fibonacci(10)
        "#;

        b.iter(|| {
            let engine = LuaEngine::new().unwrap();
            let lua = engine.lua();
            black_box(lua.load(script).exec()).unwrap()
        });
    });

    group.finish();
}

/// Benchmark data transformation in Lua
fn bench_data_transformation(c: &mut Criterion) {
    let mut group = c.benchmark_group("data_transformation");

    for size in [64usize, 256, 1024].iter() {
        group.bench_with_input(BenchmarkId::new("hex_encode", size), size, |b, &size| {
            b.iter(|| {
                let engine = LuaEngine::new().unwrap();
                let lua = engine.lua();

                // Create simple data that's safe to pass to Lua
                let data_str: String = (0..size)
                    .map(|i| {
                        // Use only safe ASCII characters
                        char::from_u32(((i % 26) + 65) as u32).unwrap()
                    })
                    .collect();

                let script = format!(
                    r#"
                    local data = "{}"
                    local hex = ""
                    for i = 1, #data do
                        local byte = string.byte(data, i)
                        hex = hex .. string.format("%02X", byte)
                    end
                    return hex
                "#,
                    data_str
                );

                black_box(lua.load(&script).exec()).unwrap()
            });
        });
    }

    group.finish();
}

/// Benchmark callback overhead
fn bench_callback_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("callback_overhead");

    // Data callback
    group.bench_function("on_data", |b| {
        let script = r#"
            function on_data(data)
                -- Process incoming data
                return #data
            end
        "#;

        b.iter(|| {
            let engine = LuaEngine::new().unwrap();
            let lua = engine.lua();
            lua.load(script).exec().unwrap();
            // Actually call the callback
            black_box(lua.load("return on_data('test data')").exec()).unwrap()
        });
    });

    // Error callback
    group.bench_function("on_error", |b| {
        let script = r#"
            function on_error(err)
                -- Handle error
                return err
            end
        "#;

        b.iter(|| {
            let engine = LuaEngine::new().unwrap();
            let lua = engine.lua();
            lua.load(script).exec().unwrap();
            // Actually call the callback
            black_box(lua.load("return on_error('test error')").exec()).unwrap()
        });
    });

    group.finish();
}

/// Benchmark script loading time
fn bench_script_loading(c: &mut Criterion) {
    let mut group = c.benchmark_group("script_loading");

    // Small script
    group.bench_function("small", |b| {
        let script = "return 1 + 1";

        b.iter(|| {
            let engine = LuaEngine::new().unwrap();
            let lua = engine.lua();
            black_box(lua.load(black_box(script)).exec()).unwrap()
        });
    });

    // Medium script
    group.bench_function("medium", |b| {
        let script = r#"
            local function helper1(x) return x * 2 end
            local function helper2(x) return x + 10 end
            return helper2(helper1(5))
        "#;

        b.iter(|| {
            let engine = LuaEngine::new().unwrap();
            let lua = engine.lua();
            black_box(lua.load(black_box(script)).exec()).unwrap()
        });
    });

    // Large script
    group.bench_function("large", |b| {
        let script: String = (0..100)
            .map(|i| format!("local func{} = function() return {} end\n", i, i))
            .collect();

        b.iter(|| {
            let engine = LuaEngine::new().unwrap();
            let lua = engine.lua();
            black_box(lua.load(black_box(&script)).exec()).unwrap()
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_script_overhead,
    bench_data_transformation,
    bench_callback_overhead,
    bench_script_loading
);
criterion_main!(benches);
