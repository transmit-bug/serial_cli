//! Concurrency benchmarks
//!
//! Measures parallel task spawning, concurrent Lua engine initialization,
//! and concurrent config loading overhead.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use serial_cli::config::ConfigManager;
use serial_cli::lua::LuaEngine;
use tokio::task::JoinSet;

/// Run async benchmark code on a separate thread with its own runtime.
fn run_async_benchmark<F, Fut>(f: F)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    let handle = std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("create benchmark runtime");
        rt.block_on(f())
    });
    handle.join().expect("benchmark thread panicked");
}

/// Benchmark concurrent buffer allocations with async task spawning
fn bench_concurrent_buffer_ops(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_buffer_ops");

    for num_tasks in [2, 4, 8] {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_tasks),
            &num_tasks,
            |b, &n| {
                b.iter(|| {
                    run_async_benchmark(move || async move {
                        let mut join_set = JoinSet::new();
                        for _ in 0..n {
                            join_set.spawn(async {
                                let _buffer = vec![0u8; 4096];
                                tokio::task::yield_now().await;
                            });
                        }
                        while let Some(res) = join_set.join_next().await {
                            res.expect("task join failed");
                        }
                    })
                })
            },
        );
    }

    group.finish();
}

/// Benchmark concurrent Lua engine initialization
fn bench_concurrent_lua_init(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_lua_init");

    for num_tasks in [2, 4] {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_tasks),
            &num_tasks,
            |b, &n| {
                b.iter(|| {
                    run_async_benchmark(move || async move {
                        let mut join_set = JoinSet::new();
                        for _ in 0..n {
                            join_set.spawn(async {
                                let _engine = LuaEngine::new().expect("Lua engine init failed");
                            });
                        }
                        while let Some(res) = join_set.join_next().await {
                            res.expect("task join failed");
                        }
                    })
                })
            },
        );
    }

    group.finish();
}

/// Benchmark concurrent ConfigManager initialization
fn bench_concurrent_config_load(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_config_load");

    for num_tasks in [2, 4, 8] {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_tasks),
            &num_tasks,
            |b, &n| {
                b.iter(|| {
                    run_async_benchmark(move || async move {
                        let mut join_set = JoinSet::new();
                        for _ in 0..n {
                            join_set.spawn(async {
                                let _manager = ConfigManager::new();
                            });
                        }
                        while let Some(res) = join_set.join_next().await {
                            res.expect("task join failed");
                        }
                    })
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_concurrent_buffer_ops,
    bench_concurrent_lua_init,
    bench_concurrent_config_load,
);
criterion_main!(benches);
