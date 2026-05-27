# Benchmark Commands

Performance benchmarking tools for measuring and tracking serial CLI performance across releases.

## Command

```
serial-cli benchmark <subcommand>
```

## Subcommands

### `benchmark run [category]`

Execute benchmarks for a specified category and print results to stdout.

```
serial-cli benchmark run <category> [options]
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `category` | Benchmark category (see table below) |

**Options:**

| Flag | Default | Description |
|------|---------|-------------|
| `--iterations <N>` | `100` | Number of measured iterations per benchmark |
| `--output <file>` | *(none)* | Save JSON report to the given path |

**Categories:**

| Category | Description | Key benchmarks |
|----------|-------------|----------------|
| `all` | Every benchmark across all categories | Full suite |
| `serial-io` | Serial I/O throughput and pipeline latency | Buffer copy at various sizes (64--16384 bytes), Modbus RTU/ASCII encode-decode round-trip |
| `virtual-port` | Virtual port creation and bridging | PTY virtual port creation timing (platform-dependent) |
| `protocol` | Protocol parsing and checksum speed | Modbus RTU/ASCII parse and encode, CRC16 and LRC computation |
| `startup` | Application initialization time | Cold/warm start, protocol loading from Lua scripts, LuaJIT engine init |
| `memory` | Allocation overhead and footprint | Buffer allocation at various sizes, ConfigManager and LuaEngine memory footprint |
| `concurrency` | Concurrent task execution overhead | Parallel buffer ops, Lua engine init, and config load at 2/4/8 task counts |

### `benchmark compare <baseline> <current>`

Load two JSON benchmark reports and print a per-benchmark regression/improvement summary.

```
serial-cli benchmark compare <baseline.json> <current.json>
```

Comparison results are tagged as:

- **[REGRESSION]** -- current run is more than 5% slower than baseline
- **[IMPROVEMENT]** -- current run is more than 5% faster than baseline
- **[NO CHANGE]** -- within the 5% threshold

Results are sorted by regression severity (worst regressions first).

### `benchmark list`

Print all available benchmark categories with usage examples.

```
serial-cli benchmark list
```

## Output Format

### Console Summary

Each benchmark prints its name, category, total elapsed time, and a per-iteration metric:

- Timing benchmarks show `ns/iter` (nanoseconds per iteration).
- Throughput benchmarks show `MB/s` (megabytes per second).

At the end of a run, a grouped summary is printed by category.

### JSON Report

When `--output <file>` is specified, results are serialized as a `BenchmarkReport`:

```json
{
  "timestamp": "2026-04-25T10:30:00.000000000Z",
  "results": [
    {
      "name": "modbus_rtu_roundtrip",
      "category": "SerialIo",
      "iterations": 100,
      "elapsed_ns": 15000000,
      "bytes_processed": 600
    }
  ]
}
```

Each `BenchmarkResult` provides:

- `avg_ns_per_iteration()` -- mean time per iteration.
- `throughput_bytes_per_sec()` -- bytes/second when `bytes_processed` is set.

## Example Workflow: Track Performance Across Releases

1. **Run benchmarks on the current release and save the baseline:**

   ```bash
   serial-cli benchmark run all --iterations 500 --output v1.0.0-baseline.json
   ```

2. **After making changes, run the same benchmarks:**

   ```bash
   serial-cli benchmark run all --iterations 500 --output v1.1.0-candidate.json
   ```

3. **Compare the two results:**

   ```bash
   serial-cli benchmark compare v1.0.0-baseline.json v1.1.0-candidate.json
   ```

   Example output:

   ```
   === Benchmark Comparison ===

     modbus_rtu_parse: [IMPROVEMENT] (-12.3%)
       Baseline: 45.20 ns/iter -> Current: 39.64 ns/iter

     cold_start_no_config: [NO CHANGE] (+1.2%)
       Baseline: 120.50 ns/iter -> Current: 121.95 ns/iter

     concurrent_config_load_8: [REGRESSION] (+8.7%)
       Baseline: 350.00 ns/iter -> Current: 380.45 ns/iter

   Summary:
     Regressions: 1
     Improvements: 1
     Unchanged: 1
   ```

4. **Investigate regressions** by re-running the affected category with more iterations for tighter confidence:

   ```bash
   serial-cli benchmark run concurrency --iterations 2000 --output concurrency-deep.json
   ```

## Tips

- Use `--iterations` higher than the default (100) for noisy benchmarks or CI pipelines.
- Run `benchmark list` for quick reference to available categories.
- Virtual port benchmarks require PTY support and are skipped on platforms where it is unavailable.
- Memory benchmarks measure allocation overhead only; for heap profiling use external tools such as `heaptrack` or `valgrind`.
- Concurrency benchmarks may show diminishing returns at higher task counts due to thread contention.
