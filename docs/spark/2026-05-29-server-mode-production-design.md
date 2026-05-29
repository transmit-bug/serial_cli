# Server Mode 生产就绪 + 依赖重构 + 功能增强设计

**Version**: 0.6.0
**Status**: Proposed
**Date**: 2026-05-29

---

## 目录

1. [架构总览](#1-架构总览)
2. [依赖替换 (Layer 1)](#2-依赖替换-layer-1)
3. [Bug 修复 (Layer 2)](#3-bug-修复-layer-2)
4. [新增功能 (Layer 3)](#4-新增功能-layer-3)
5. [实施顺序](#5-实施顺序)
6. [Cargo.toml 变更](#6-cargotoml-变更)
7. [测试策略](#7-测试策略)

---

## 1. 架构总览

### 1.1 背景

Phase 1 MVP 已实现基本的 server start/stop/status/call 和 10 个 RPC 方法。但存在以下缺口：

- **消息帧 bug**：`listener.rs` 的 `stream.read()` 不保证消息边界
- **无优雅关闭**：SIGTERM 直接杀进程，端口/Lua/会话未清理
- **手动 JSON-RPC**：`rpc.rs` 600+ 行手动实现，Cargo.toml 有 `jsonrpc = "0.20"` 但未使用
- **平台特定进程检查**：`libc::kill` + 20+ 行 Windows API
- **伪守护进程**：`Command::spawn` 不是真正的 Unix daemon
- **数据推送缺失**：AI Agent 只能轮询 `port_recv`，无法高效等待响应
- **日志未写入文件**：`log_path` 配置了但没有 file appender
- **配置硬编码**：`max_connections`、`idle_timeout_secs` 写死在 `run_daemon()`
- **Socket 权限未设置**：任何进程都能连接 `/tmp/serial-cli.sock`

### 1.2 变更分层

```
┌─────────────────────────────────────────────────────────┐
│  Layer 3: 新增功能 (F1-F3)                              │
│  F1. 数据到达推送  F2. 日志监控  F3. 配置与认证          │
├─────────────────────────────────────────────────────────┤
│  Layer 2: Bug 修复 (R1-R2)                              │
│  R1. 消息帧     R2. 优雅关闭                             │
├─────────────────────────────────────────────────────────┤
│  Layer 1: 依赖替换 (D1-D4)                              │
│  D1. jsonrpc  D2. sysinfo  D3. daemonize  D4. tokio-util│
└─────────────────────────────────────────────────────────┘
```

实施顺序：Layer 1 → Layer 2 → Layer 3。每步独立可测试、可回退。

---

## 2. 依赖替换 (Layer 1)

### D1. jsonrpc crate 替代手动 JSON-RPC

**文件**: `src/server/rpc.rs`

**当前**：600+ 行手动 `JsonRpcRequest`/`JsonRpcResponse`/`JsonRpcError` 结构体 + `handle_request` 字符串匹配分发。Cargo.toml 已有 `jsonrpc = "0.20"` 但未使用。

**替换为**：`jsonrpc` crate 的 `JsonRpcServer` + `add_method` 注册。该 crate 提供 transport-agnostic 的 JSON-RPC 2.0 实现，不依赖 HTTP/WebSocket，完美适配我们的 Unix socket + LinesCodec 场景。

**示例对比**：

```rust
// BEFORE (current manual implementation)
async fn port_open(&self, params: Option<Value>) -> Result<Value, MethodError> {
    let params = params.ok_or(...)?;
    let port = params.get("port").and_then(|v| v.as_str()).ok_or(...)?;
    // ... 40+ lines of manual param parsing
}

// AFTER (jsonrpc crate)
let mut server = jsonrpc::ServerBuilder::new(io_handler);
server.add_method("port_open", |params: PortOpenParams| {
    // typed params, automatic JSON deserialization
});
```

**效果**：
- 自动 JSON-RPC 2.0 规范合规（batch requests、notifications、error codes）
- 类型安全的参数解析（serde deserialization）
- ID 匹配、版本校验、错误序列化
- 与任何 transport 兼容（Unix socket、TCP、stdin/stdout）

**10 个 RPC 方法映射**：

| 当前方法 | jsonrpsee 方法 | 参数结构体 | 返回结构体 |
|----------|---------------|-----------|-----------|
| `port_list` | `port::list` | `()` | `PortListResult` |
| `port_open` | `port::open` | `PortOpenParams` | `PortOpenResult` |
| `port_close` | `port::close` | `PortCloseParams` | `PortCloseResult` |
| `port_send` | `port::send` | `PortSendParams` | `PortSendResult` |
| `port_recv` | `port::recv` | `PortRecvParams` | `PortRecvResult` |
| `protocol_list` | `protocol::list` | `()` | `ProtocolListResult` |
| `protocol_load` | `protocol::load` | `ProtocolLoadParams` | `ProtocolLoadResult` |
| `protocol_unload` | `protocol::unload` | `ProtocolUnloadParams` | `ProtocolUnloadResult` |
| `connection_list` | `connection::list` | `()` | `ConnectionListResult` |
| `server_stats` | `server::stats` | `()` | `ServerStatsResult` |

### D2. sysinfo 替代进程检查

**文件**: `src/server/session.rs`

**当前**：
```rust
#[cfg(unix)]
pub fn is_process_running(pid: u32) -> bool {
    unsafe { libc::kill(pid as libc::pid_t, 0) == 0 }
}
// + 20 lines of Windows OpenProcess/TerminateProcess
```

**替换为**：
```rust
use sysinfo::{System, ProcessRefreshKind, RefreshKind};

pub fn is_process_running(pid: u32) -> bool {
    let sys = System::new_with_specifics(
        RefreshKind::nothing().with_processes(ProcessRefreshKind::nothing())
    );
    sys.process(sysinfo::Pid::from_u32(pid)).is_some()
}

pub fn stop_process(pid: u32) -> Result<()> {
    let sys = System::new_with_specifics(
        RefreshKind::nothing().with_processes(ProcessRefreshKind::nothing())
    );
    sys.process(sysinfo::Pid::from_u32(pid))
        .ok_or_else(|| SerialError::Io(...))?
        .kill();
    Ok(())
}
```

**效果**：移除所有 `#[cfg(unix)]` / `#[cfg(windows)]` 分支，移除 `unsafe`。

### D3. daemonize 替代 Command::spawn

**文件**: `src/cli/commands/server.rs` — `start_server()` 函数

**当前**：
```rust
let current_exe = std::env::current_exe()?;
let mut child = Command::new(&current_exe)
    .args(&["server", "daemon", "--socket-path", ...])
    .stdout(std::process::Stdio::piped())
    .stderr(std::process::Stdio::piped())
    .spawn()?;
std::thread::sleep(Duration::from_millis(500));
match child.try_wait() { ... }
```

**替换为**：
```rust
use daemonize::Daemonize;

let pid_file = ServerSessionManager::session_dir()?.join("server.pid");
let daemonize = Daemonize::new()
    .pid_file(&pid_file)
    .working_directory("/")
    .stdout(stdout_file.clone())
    .stderr(stderr_file.clone());

match daemonize.start() {
    Ok(_) => { /* daemon context: run_daemon() */ }
    Err(e) => return Err(SerialError::Io(...)),
}
```

**注意**：`daemonize.start()` 在父进程返回后，子进程继续执行。因此需要将 `run_daemon()` 放在 `daemonize.start()` 的成功分支中。

**不再需要**：`ServerCommand::Daemon` 隐藏子命令（它是 `Command::spawn(self)` 的自调用机制）。daemonize crate 处理了 fork/setsid 等。

### D4. tokio-util 消息帧

**文件**: `src/server/listener.rs`

**当前**：
```rust
match stream.read(&mut buf).await {
    Ok(n) => {
        let request = String::from_utf8_lossy(&buf[..n]);
        let response = rpc.handle_request(&request).await;
```

一个 10KB 的 JSON-RPC 请求可能被分成两个 TCP segment，导致第一次 `read()` 只拿到部分 JSON，`serde_json::from_str` 失败。

**替换为**：
```rust
use tokio_util::codec::{Framed, LinesCodec};

let mut framed = Framed::new(stream, LinesCodec::new());
while let Some(Ok(line)) = framed.next().await {
    let response = rpc.handle_request(&line).await;
    framed.send(response).await?;
}
```

`LinesCodec` 自动累积字节直到遇到 `\n`，保证每次返回完整一行。这与 `rpc.rs` 的 `serialize_response` 已追加 `\n` 的写入约定一致。

---

## 3. Bug 修复 (Layer 2)

### R1. 消息帧修复

与 D4 合并实施。`LinesCodec` 解决了消息截断和粘连问题。

### R2. 优雅关闭

**文件**: `src/server/listener.rs`, `src/cli/commands/server.rs`

**当前**：`run_socket_server` 是无限 `loop {}`，无法响应 SIGTERM。

**实现**：

```rust
use tokio_util::sync::CancellationToken;

pub async fn run_socket_server(state: ServerState, socket_path: PathBuf) -> Result<()> {
    let token = CancellationToken::new();
    let token_clone = token.clone();

    // Spawn signal handler
    tokio::spawn(async move {
        let mut sigterm = tokio::signal::unix::signal(
            tokio::signal::unix::SignalKind::terminate()
        ).expect("failed to create signal handler");
        sigterm.recv().await;
        tracing::info!("Received SIGTERM, initiating graceful shutdown...");
        token_clone.cancel();
    });

    // ... setup listener ...

    loop {
        tokio::select! {
            _ = token.cancelled() => {
                tracing::info!("Shutdown signal received, stopping accept loop");
                break;
            }
            result = listener.accept() => {
                match result {
                    Ok((stream, _)) => { /* spawn handler */ }
                    Err(e) => tracing::error!("Accept error: {}", e),
                }
            }
        }
    }

    // Cleanup phase
    tracing::info!("Shutting down: closing all ports, clearing session...");
    state.cleanup_all().await;  // new method: close all ports, stop all Lua timers
    ServerSessionManager::clear_session()?;
    if socket_path.exists() {
        std::fs::remove_file(&socket_path)?;
    }
    Ok(())
}
```

**`ServerState::cleanup_all()` 新方法的职责**：
1. 遍历所有 open ports，调用 `PortManager::close_port()`
2. 停止所有 Lua timer 线程（通过 `SerialScriptEngine::stop_timer()`）
3. 释放所有连接上下文

**stop_server 改进**：
- 当前：发送 SIGTERM → 等 500ms → 检查进程
- 改进后：发送 SIGTERM → 等待 pidfile 消失（daemonize 的 pid_file 在进程退出时自动删除）→ 确认关闭

---

## 4. 新增功能 (Layer 3)

### F1. 数据到达推送

**问题**：当前 `port_recv` 是拉模式（polling），AI Agent 需要循环调用等待设备响应，效率极低且浪费 CPU。

**方案**：基于 JSON-RPC Notification 的推送机制。

#### 4.1.1 IoLoop 改造

**文件**: `src/serial_core/port.rs` — `open_port` 中 spawn 的 background task

当前 IoLoop 读取数据后只做了 logging：
```rust
tokio::spawn(async move {
    let mut buffer = vec![0u8; 4096];
    loop {
        let n = { /* read from port */ };
        if n > 0 {
            let data = buffer[..n].to_vec();
            tracing::debug!("IoLoop: Received {} bytes from {}", n, port_id);
            // data is discarded after logging
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
});
```

改造为广播模式：
```rust
use tokio::sync::broadcast;

// In SerialPortHandle:
pub data_broadcast: broadcast::Sender<Vec<u8>>,

// In IoLoop:
if n > 0 {
    let data = buffer[..n].to_vec();
    let _ = data_tx.send(data);  // broadcast to all subscribers
    // existing frame_buffer logic continues unchanged
}
```

#### 4.1.2 订阅 RPC 方法

新增 `port_subscribe` 方法：

```json
// Request
{"jsonrpc": "2.0", "method": "port_subscribe", "params": {"connection_id": "conn_123"}, "id": 1}

// Response
{"jsonrpc": "2.0", "result": {"subscribed": true}, "id": 1}
```

订阅后，当该端口收到数据时，通过同一 Unix socket 连接推送 notification：

```json
// Notification (no "id" field - JSON-RPC 2.0 spec)
{"jsonrpc": "2.0", "method": "data_received", "params": {
    "connection_id": "conn_123",
    "data": "41542b4f4b",
    "bytes": 5
}}
```

#### 4.1.3 连接级别广播分发

每个 Unix socket 连接在 `tokio::spawn` 的任务中持有一个 `broadcast::Receiver`。handler loop 需要同时监听客户端请求和广播数据：

```rust
tokio::spawn(async move {
    let mut framed = Framed::new(stream, LinesCodec::new());
    let mut data_rx = data_broadcast.subscribe();

    loop {
        tokio::select! {
            // Client request
            Some(Ok(line)) = framed.next() => {
                let response = rpc.handle_request(&line).await;
                framed.send(response).await?;
            }
            // Data push notification
            Ok(data) = data_rx.recv() => {
                let notification = format!(
                    r#"{{"jsonrpc":"2.0","method":"data_received","params":{{"connection_id":"{}","data":"{}","bytes":{}}}}}"#,
                    connection_id,
                    hex::encode(&data),
                    data.len()
                );
                framed.send(notification).await?;
            }
        }
    }
});
```

#### 4.1.4 无订阅则丢弃

如果没有客户端订阅某个端口（`broadcast::Sender::receiver_count() == 0`），广播数据被自动丢弃，不影响现有 `port_recv` 拉模式。

`port_recv` 和推送是两条独立路径：
- `port_recv`：继续走 `frame_buffer` → `protocol parse` → `on_recv` → 返回
- 推送：IoLoop raw data → `broadcast` → `notification` → socket write

#### 4.1.5 数据流图

```
Serial Device ──▶ SerialPortHandle.read()
                       │
                       ├─▶ broadcast::send(raw data) ──▶ subscribed connections
                       │                                     │
                       │                                     ▼
                       │                            stream.write_all(notification)
                       │
                       └─▶ frame_buffer (protocol parse)
                                │
                                ▼
                         port_recv (pull mode, existing)
```

### F2. 日志与监控

#### 4.2.1 日志文件写入

**文件**: `src/cli/commands/server.rs` — `run_daemon()` 初始化

```rust
use tracing_subscriber::fmt::MakeWriter;

let log_file = OpenOptions::new()
    .create(true)
    .append(true)
    .open(&config.log_path)?;

let file_layer = tracing_subscriber::fmt::layer()
    .with_writer(Arc::new(Mutex::new(log_file)))
    .with_ansi(false)
    .json();  // structured logging for machine parsing

let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
    .unwrap_or_else(|_| "info".into());

tracing_subscriber::registry()
    .with(env_filter)
    .with(file_layer)
    .init();
```

**日志格式**（JSON）：
```json
{"timestamp":"2026-05-29T10:30:00Z","level":"INFO","target":"serial_cli::server::rpc","message":"RPC request","method":"port_open","connection":"192.168.1.1","duration_ms":2}
```

#### 4.2.2 RPC 请求审计

在 `RpcDispatcher::handle_request` 中记录：

```rust
async fn handle_request(&self, request: &str) -> String {
    let start = std::time::Instant::now();
    let method = extract_method(request);  // parse method name from JSON

    let response = /* dispatch logic */;

    let elapsed = start.elapsed();
    tracing::info!(
        method = method,
        duration_ms = elapsed.as_millis(),
        success = response.contains("result"),
        "RPC request completed"
    );

    response
}
```

#### 4.2.3 监控指标增强

`server_stats` RPC 方法返回扩展指标：

```json
{
  "connections": {"active": 3, "max": 10},
  "uptime_secs": 3600,
  "total_requests": 1523,
  "total_errors": 2,
  "avg_latency_ms": 1.2,
  "ports_open": ["/dev/ttyUSB0", "/dev/ttyUSB1"]
}
```

需要在 `ServerState` 中添加原子计数器：

```rust
use std::sync::atomic::{AtomicU64, Ordering};

pub struct ServerState {
    // ... existing fields ...
    pub total_requests: Arc<AtomicU64>,
    pub total_errors: Arc<AtomicU64>,
    pub total_latency_ms: Arc<AtomicU64>,
}
```

### F3. 配置与认证

#### 4.3.1 配置文件

**路径**: `~/.config/serial-cli/server.toml` 或 `--config` 参数指定

```toml
[server]
socket_path = "/tmp/serial-cli.sock"
socket_mode = 0o600        # Unix socket 权限 (default)
tcp_port = null            # 可选：TCP 端口（替代 Unix socket）
max_connections = 20
idle_timeout_secs = 300
log_path = "~/.cache/serial_cli/server.log"
log_level = "info"
```

**加载逻辑**：
```rust
impl ServerConfig {
    pub fn load(cli_path: Option<&Path>) -> Result<Self> {
        let path = cli_path
            .or_else(|| find_config_file())
            .ok_or_else(|| /* use defaults */)?;

        let content = fs::read_to_string(&path)?;
        let config: ServerConfigFile = toml::from_str(&content)?;
        Ok(config.into())
    }
}
```

配置文件字段覆盖默认值，CLI 参数覆盖配置文件。

优先级：CLI 参数 > 配置文件 > 默认值

#### 4.3.2 Unix Socket 权限

创建 socket 后设置权限：

```rust
let listener = UnixListener::bind(&socket_path)?;

// Set socket permissions
let mode = config.socket_mode;  // default 0o600
let mut perms = std::fs::metadata(&socket_path)?.permissions();
perms.set_mode(mode);
std::fs::set_permissions(&socket_path, perms)?;

tracing::info!("Socket created at {} with mode 0o{:o}", socket_path.display(), mode);
```

#### 4.3.3 硬编码消除

`run_daemon()` 中的硬编码值全部从 `ServerConfig` 读取：

| 硬编码值 | 当前 | 改进后 |
|----------|------|--------|
| `max_connections` | `10` | `config.max_connections` |
| `idle_timeout_secs` | `300` | `config.idle_timeout_secs` |
| `log_path` | `PathBuf::from("/tmp/...")` | `config.log_path` |

---

## 5. 实施顺序

```
Step 1: 新增 Cargo.toml 依赖
         │
         ▼
Step 2: D2 sysinfo (session.rs) — 最小改动，跨平台统一
         │
         ▼
Step 3: D3 daemonize (server.rs start_server) — 替换进程派生
         │
         ▼
Step 4: D4 + R1 tokio-util LinesCodec (listener.rs) — 消息帧修复
         │
         ▼
Step 5: D1 jsonrpc 替换 (rpc.rs) — 最大改动，600+ 行 → ~200 行
         │
         ▼
Step 6: R2 优雅关闭 (listener.rs + server.rs) — CancellationToken + signal handler
         │
         ▼
Step 7: F1 数据推送 (port.rs IoLoop + listener.rs handler + rpc.rs subscribe)
         │
         ▼
Step 8: F2 日志监控 (server.rs init + rpc.rs audit + state.rs counters)
         │
         ▼
Step 9: F3 配置认证 (server.rs config loading + socket permissions)
```

---

## 6. Cargo.toml 变更

```toml
# Add:
tokio-util = { version = "0.7", features = ["codec", "rt"] }
sysinfo = "0.32"
daemonize = "0.5"

# Existing (now wired up):
jsonrpc = "0.20"  # was unused, now powers RPC dispatch
```

---

## 7. 测试策略

### 7.1 单元测试

- `rpc.rs`：每个 jsonrpc handler 方法的参数验证和返回格式
- `session.rs`：`sysinfo` 进程检查 mock
- `listener.rs`：`LinesCodec` 帧分割测试（长消息、多消息合并、部分消息）
- `config.rs`：配置加载、优先级覆盖、TOML 解析错误

### 7.2 集成测试

- 完整生命周期：start → subscribe → send/recv → notification received → stop
- 优雅关闭：start → open port → SIGTERM → port closed, session cleared
- 消息帧：发送 10KB JSON-RPC 请求，验证正确解析
- 多客户端：两个客户端同时订阅同一端口，都收到通知

### 7.3 性能测试

- 对比修复前后的 RPC 延迟（`jsonrpsee` vs manual）
- 广播推送延迟：从串口接收到数据到客户端收到 notification 的时间
- 长时间运行测试：24h 无内存泄漏
