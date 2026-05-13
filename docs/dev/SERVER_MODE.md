# Server Mode 守护进程设计方案

**Version**: 0.5.0-dev
**Status**: Implemented
**Author**: Claude + Pony
**Date**: 2026-05-10

---

## 📋 目录

1. [架构概览](#架构概览)
2. [核心设计](#核心设计)
3. [实现细节](#实现细节)
4. [API 规范](#api-规范)
5. [安全考虑](#安全考虑)
6. [实施计划](#实施计划)
7. [测试策略](#测试策略)

---

## 架构概览

### 设计目标

基于项目定位 **"optimized for AI/automation workflows"**，设计一个轻量级守护进程模式，实现：

1. **持久化连接管理** - 避免频繁的 open/close 开销
2. **协议状态持久化** - 协议热重载、动态加载在守护进程中生效
3. **AI 友好 API** - JSON RPC 2.0 over Unix Socket (低延迟)
4. **多客户端支持** - 多个 AI Agent 可以共享连接
5. **最小侵入性** - 复用现有架构，不破坏现有功能

### 与现有模式对比

| 特性 | CLI 模式 | 交互式模式 | **Server Mode** | Tauri GUI |
|------|----------|-----------|----------------|-----------|
| 启动方式 | `serial-cli <cmd>` | `serial-cli interactive` | `serial-cli server start` | GUI 应用 |
| 连接生命周期 | 单次命令 | 会话级（单用户） | **守护进程级（多调用）** | 应用级 |
| 协议持久化 | ❌ | ✅ 会话级 | **✅ ✅ 全局级** | ✅ 应用级 |
| 适用场景 | 脚本、CI/CD | 人工调试 | **AI/自动化** | 交互式 GUI |
| IPC 机制 | - | - | **Unix Socket / TCP** | Tauri commands |
| 延迟 | 50-200ms | 1-5ms | **1-5ms** | 1-5ms |

### 架构图

```
┌─────────────────────────────────────────────────────────────┐
│                    Serial CLI 使用模式                       │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  1. CLI Mode (现有)                                          │
│     $ serial-cli port send /dev/ttyUSB0 "AT"                 │
│     └─→ 无状态，每次 open/close                              │
│                                                              │
│  2. Interactive Mode (现有)                                  │
│     $ serial-cli interactive                                 │
│     └─→ 单用户 REPL，会话级状态                              │
│                                                              │
│  3. ⭐ Server Mode (新增)                                    │
│     $ serial-cli server start                               │
│     └─→ 守护进程 + Unix Socket                               │
│         $ serial-cli call port_open '{...}'                 │
│         $ serial-cli call port_send '{...}'                 │
│                                                              │
│  4. Tauri GUI (现有)                                        │
│     $ open GUI                                               │
│     └─→ 长连接 + 事件系统                                     │
│                                                              │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                   Server Mode 内部架构                       │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────┐      ┌──────────────┐                    │
│  │ AI Agent /   │─────▶│ Unix Socket  │                    │
│  │ Automation   │      │ Listener     │                    │
│  └──────────────┘      └───────┬──────┘                    │
│         ▲                      │                           │
│         │                      ▼                           │
│         │              ┌───────────────┐                   │
│         │              │ JSON RPC 2.0   │                   │
│         │              │ Dispatcher     │                   │
│         │              └───────┬───────┘                   │
│         │                      │                           │
│         │                      ▼                           │
│         │              ┌───────────────┐                   │
│         │              │ ServerState   │                   │
│         │              │ ┌───────────┐ │                   │
│         │              │ │PortManager│ │ (复用现有)        │
│         │              │ └───────────┘ │                   │
│         │              │ ┌───────────┐ │                   │
│         │              │ │Protocol   │ │ (复用现有)        │
│         │              │ │Manager    │ │                   │
│         │              │ └───────────┘ │                   │
│         │              │ ┌───────────┐ │                   │
│         │              │ │Connection │ │ (新增 - 连接池)   │
│         │              │ │Pool       │ │                   │
│         │              │ └───────────┘ │                   │
│         │              └───────┬───────┘                   │
│         │                      │                           │
│         │                      ▼                           │
│         │              ┌───────────────┐                   │
│         └──────────────▶│ Serial Ports  │                   │
│                        │ / Protocols   │                   │
│                        └───────────────┘                   │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## 核心设计

### 1. 复用现有架构

**已有组件（直接复用）**：

```rust
// ✅ src/serial_core/port.rs - PortManager, SerialConfig
// ✅ src/protocol/manager.rs - ProtocolManager (load/unload/reload)
// ✅ src/protocol/registry.rs - ProtocolRegistry
// ✅ src/cli/json.rs - JsonFormatter (已有完善 JSON 支持)
// ✅ src/cli/sniff_session.rs - Session 管理经验

// 🆕 新增组件
// src/server/mod.rs - Server mode 主模块
// src/server/session.rs - ServerSession 管理
// src/server/rpc.rs - JSON RPC 2.0 dispatcher
// src/server/connection_pool.rs - 连接池
```

**核心设计原则**：

1. **不破坏现有功能** - Server Mode 作为独立的子命令
2. **共享核心代码** - PortManager、ProtocolManager 在所有模式下共享
3. **参考 SniffDaemon** - 复用 spawn + session file 管理机制
4. **参考 Tauri** - 复用 AppState 模式，但不依赖 GUI

### 2. 进程模型

```bash
# 启动守护进程
$ serial-cli server start [--socket-path PATH] [--port PORT]

# 输出：
# ✓ Server started
#   Socket: /tmp/serial-cli.sock
#   PID: 12345
#   Log: ~/.cache/serial_cli/server.log

# 后台运行，可以通过以下方式交互：
# 1. CLI call 命令
$ serial-cli call port_open '{"port": "/dev/ttyUSB0"}'

# 2. 直接发送 JSON 到 socket
$ echo '{"jsonrpc": "2.0", "method": "port_list", ...}' | socat - /tmp/serial-cli.sock

# 3. 使用客户端 SDK (Python/TypeScript)
python_client.port_open("/dev/ttyUSB0", protocol="modbus_rtu")
```

### 3. Session 管理（参考 SniffDaemon）

```rust
// src/server/session.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSessionMeta {
    pub pid: u32,
    pub socket_path: PathBuf,
    pub started_at: u64,
    pub log_path: PathBuf,
    pub max_connections: usize,
}

// Session file: ~/.cache/serial_cli/server_session.json
// - 记录 PID、socket 路径
// - 启动时检查进程是否存活
// - stop 命令发送 SIGTERM
// - status 命令查询 session 文件
```

---

## 实现细节

### Phase 1: 核心 MVP (2-3 天)

#### 1.1 命令接口

```rust
// src/cli/args.rs

#[derive(Subcommand)]
enum Commands {
    // ... 现有命令 ...

    /// Server mode (守护进程)
    Server {
        #[command(subcommand)]
        server_command: ServerCommand,
    },
}

#[derive(Subcommand)]
enum ServerCommand {
    /// Start server daemon
    Start {
        /// Unix socket path (default: /tmp/serial-cli.sock)
        #[arg(long)]
        socket_path: Option<String>,

        /// TCP port (alternative to Unix socket)
        #[arg(long)]
        port: Option<u16>,

        /// Log file path
        #[arg(long)]
        log: Option<String>,

        /// Max concurrent connections
        #[arg(long, default_value = "10")]
        max_connections: usize,
    },

    /// Stop server daemon
    Stop,

    /// Server status
    Status,

    /// (Internal) Server daemon entry point
    #[command(hide = true)]
    Daemon {
        #[arg(long)]
        socket_path: Option<String>,

        #[arg(long)]
        port: Option<u16>,
    },
}

/// Call RPC method (for AI/automation)
Call {
    /// RPC method name (e.g., port_open, port_send)
    method: String,

    /// JSON arguments (e.g., '{"port": "/dev/ttyUSB0"}')
    #[arg(value_name = "JSON_ARGS")]
    args: String,

    /// Use stdin for args (useful for piping)
    #[arg(long)]
    stdin: bool,
}
```

#### 1.2 Server State

```rust
// src/server/state.rs

use serial_cli::protocol::{ProtocolManager, ProtocolRegistry};
use serial_cli::serial_core::PortManager;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

/// Server global state (similar to Tauri's AppState)
#[derive(Clone)]
pub struct ServerState {
    /// Port manager (shared with CLI)
    pub port_manager: Arc<Mutex<PortManager>>,

    /// Protocol registry (shared with CLI)
    pub protocol_registry: Arc<Mutex<ProtocolRegistry>>,

    /// Protocol manager (shared with CLI)
    pub protocol_manager: Arc<Mutex<ProtocolManager>>,

    /// Active connections (port_id -> ConnectionContext)
    pub connections: Arc<RwLock<HashMap<String, ConnectionContext>>>,

    /// Server config
    pub config: ServerConfig,
}

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub socket_path: Option<PathBuf>,
    pub tcp_port: Option<u16>,
    pub max_connections: usize,
    pub log_path: PathBuf,
}

/// Connection context (per-connection state)
#[derive(Clone)]
pub struct ConnectionContext {
    pub connection_id: String,
    pub port_id: Option<String>,
    pub protocol_name: Option<String>,
    pub created_at: SystemTime,
    pub last_activity: SystemTime,
}

impl ServerState {
    pub async fn new(config: ServerConfig) -> Self {
        let protocol_registry = Arc::new(Mutex::new(ProtocolRegistry::new()));
        let protocol_manager =
            Arc::new(Mutex::new(ProtocolManager::new(protocol_registry.clone())));

        Self {
            port_manager: Arc::new(Mutex::new(PortManager::new())),
            protocol_registry,
            protocol_manager,
            connections: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }
}
```

#### 1.3 JSON RPC Dispatcher

```rust
// src/server/rpc.rs

use jsonrpc::{Server, Response};
use serde_json::Value;

pub struct RpcDispatcher {
    state: ServerState,
}

impl RpcDispatcher {
    pub fn new(state: ServerState) -> Self {
        Self { state }
    }

    pub async fn handle_request(&self, request: &str) -> String {
        // Parse JSON RPC 2.0 request
        let req: JsonRpcRequest = match serde_json::from_str(request) {
            Ok(req) => req,
            Err(e) => return self.error_response(-32700, "Parse error", None),
        };

        // Dispatch to method handler
        let result = match req.method.as_str() {
            "port_list" => self.port_list(req.params).await,
            "port_open" => self.port_open(req.params).await,
            "port_close" => self.port_close(req.params).await,
            "port_send" => self.port_send(req.params).await,
            "port_recv" => self.port_recv(req.params).await,
            "protocol_list" => self.protocol_list(req.params).await,
            "protocol_load" => self.protocol_load(req.params).await,
            "protocol_unload" => self.protocol_unload(req.params).await,
            "connection_list" => self.connection_list(req.params).await,
            _ => self.error_response(-32601, "Method not found", None),
        };

        // Format JSON RPC 2.0 response
        serde_json::to_string(&result).unwrap_or_default()
    }

    // Method handlers...

    async fn port_open(&self, params: Option<Value>) -> JsonRpcResponse {
        let params = match params {
            Some(p) => p,
            None => return self.error_response(-32602, "Missing params", None),
        };

        // Parse parameters
        let port = params.get("port").and_then(|v| v.as_str());
        let baudrate = params.get("baudrate").and_then(|v| v.as_u64()).unwrap_or(115200);
        let protocol = params.get("protocol").and_then(|v| v.as_str());

        if port.is_none() {
            return self.error_response(-32602, "Missing 'port' parameter", None);
        }

        // Open port
        let config = SerialConfig {
            baudrate: baudrate as u32,
            ..Default::default()
        };

        let port_id = match self.state.port_manager.lock().await.open_port(port.unwrap(), config).await {
            Ok(id) => id,
            Err(e) => return self.error_response(-32603, &format!("Failed to open port: {}", e), None),
        };

        // Store connection context
        let ctx = ConnectionContext {
            connection_id: port_id.clone(),
            port_id: Some(port_id.clone()),
            protocol_name: protocol.map(|s| s.to_string()),
            created_at: SystemTime::now(),
            last_activity: SystemTime::now(),
        };

        self.state.connections.write().await.insert(port_id.clone(), ctx);

        // Return success
        self.success_response(json!({
            "connection_id": port_id,
            "port": port,
            "protocol": protocol,
        }))
    }
}
```

#### 1.4 Unix Socket Listener

```rust
// src/server/listener.rs

use tokio::net::UnixListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub async fn run_socket_server(state: ServerState, socket_path: PathBuf) -> Result<()> {
    // Remove existing socket file
    if socket_path.exists() {
        std::fs::remove_file(&socket_path)?;
    }

    let listener = UnixListener::bind(&socket_path)?;
    let rpc = RpcDispatcher::new(state);

    println!("✓ Server listening on: {}", socket_path.display());

    loop {
        match listener.accept().await {
            Ok((mut stream, _addr)) => {
                let rpc = rpc.clone();

                tokio::spawn(async move {
                    let mut buf = vec![0u8; 8192];

                    loop {
                        match stream.read(&mut buf).await {
                            Ok(0) => break, // Client disconnected
                            Ok(n) => {
                                let request = String::from_utf8_lossy(&buf[..n]);
                                let response = rpc.handle_request(&request).await;

                                if let Err(e) = stream.write_all(response.as_bytes()).await {
                                    eprintln!("Failed to send response: {}", e);
                                    break;
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to read from socket: {}", e);
                                break;
                            }
                        }
                    }
                });
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }
}
```

### Phase 2: 增强功能 (可选)

#### 2.1 连接池管理

```rust
// src/server/connection_pool.rs

pub struct ConnectionPool {
    connections: HashMap<String, ConnectionContext>,
    max_connections: usize,
    idle_timeout: Duration,
}

impl ConnectionPool {
    /// Cleanup idle connections
    pub async fn cleanup_idle(&mut self) {
        let now = SystemTime::now();
        self.connections.retain(|_, ctx| {
            now.duration_since(ctx.last_activity)
                .map(|d| d < self.idle_timeout)
                .unwrap_or(false)
        });
    }

    /// Get connection stats
    pub fn stats(&self) -> ConnectionStats {
        ConnectionStats {
            active: self.connections.len(),
            max: self.max_connections,
        }
    }
}
```

#### 2.2 事件系统（实时数据流）

```rust
// WebSocket support for real-time data streaming

pub async fn port_subscribe(port_id: String) -> impl Stream<Item = DataEvent> {
    // Subscribe to data received events
    // Similar to Tauri's event system
}
```

---

## API 规范

### JSON RPC 2.0 接口

#### Request Format

```json
{
  "jsonrpc": "2.0",
  "method": "port_open",
  "params": {
    "port": "/dev/ttyUSB0",
    "baudrate": 115200,
    "protocol": "modbus_rtu"
  },
  "id": 1
}
```

#### Response Format

```json
{
  "jsonrpc": "2.0",
  "result": {
    "connection_id": "conn_123456",
    "port": "/dev/ttyUSB0",
    "protocol": "modbus_rtu"
  },
  "id": 1
}
```

### 核心 API 方法

| 方法 | 参数 | 返回值 | 描述 |
|------|------|--------|------|
| `port_list` | - | `[{port_name, port_type}]` | 列出可用端口 |
| `port_open` | `{port, baudrate?, protocol?}` | `{connection_id, port, protocol}` | 打开端口 |
| `port_close` | `{connection_id}` | `{success}` | 关闭端口 |
| `port_send` | `{connection_id, data (base64)}` | `{bytes_sent}` | 发送数据 |
| `port_recv` | `{connection_id, timeout?}` | `{data (base64)}` | 接收数据 |
| `protocol_list` | - | `[{name, description}]` | 列出协议 |
| `protocol_load` | `{path, name?}` | `{name, loaded_at}` | 加载协议 |
| `protocol_unload` | `{name}` | `{success}` | 卸载协议 |
| `connection_list` | - | `[{connection_id, port, protocol, created_at}]` | 列出活跃连接 |

### CLI 调用示例

```bash
# 1. 列出端口
$ serial-cli call port_list '{}'
# → {"jsonrpc": "2.0", "result": {"ports": [...]}, "id": null}

# 2. 打开端口（使用 protocol）
$ serial-cli call port_open '{
    "port": "/dev/ttyUSB0",
    "baudrate": 115200,
    "protocol": "modbus_rtu"
}'
# → {"jsonrpc": "2.0", "result": {"connection_id": "conn_123", ...}, "id": null}

# 3. 发送数据（数据会被 protocol encode）
$ serial-cli call port_send '{
    "connection_id": "conn_123",
    "data": "ATEST"
}'
# → {"jsonrpc": "2.0", "result": {"bytes_sent": 7}, "id": null}

# 4. 接收数据（数据会被 protocol parse）
$ serial-cli call port_recv '{
    "connection_id": "conn_123",
    "timeout": 1000
}'
# → {"jsonrpc": "2.0", "result": {"data": "OK", ...}, "id": null}

# 5. 关闭连接
$ serial-cli call port_close '{
    "connection_id": "conn_123"
}'
```

---

## 安全考虑

### 1. Unix Socket 权限

```rust
// 设置 socket 文件权限为 user-only
use std::os::unix::fs::PermissionsExt;

let mut perms = std::fs::metadata(&socket_path)?.permissions();
perms.set_mode(0o600); // user read/write only
std::fs::set_permissions(&socket_path, perms)?;
```

### 2. 连接限制

```rust
// 限制最大并发连接数
if state.connections.len() >= state.config.max_connections {
    return error_response(-32000, "Max connections reached", None);
}
```

### 3. 超时清理

```rust
// 清理空闲连接
tokio::spawn(async move {
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
        state.cleanup_idle_connections().await;
    }
});
```

### 4. 日志隔离

```rust
// 服务器日志独立文件
let log_path = dirs::BaseDirs::new()?
    .cache_dir()
    .join("serial_cli")
    .join("server.log");
```

---

## 实施计划

### Milestone 1: 核心 MVP (3-4 天)

- [ ] Day 1: 基础架构
  - [ ] 添加 `server` 子命令到 `args.rs`
  - [ ] 实现 `ServerState` 和 `ConnectionContext`
  - [ ] 实现 session 管理类（参考 `sniff_session.rs`）
  - [ ] 添加 `server start/stop/status` 命令

- [ ] Day 2: JSON RPC Dispatcher
  - [ ] 实现 `RpcDispatcher` 基础框架
  - [ ] 实现 `port_list` 和 `port_open` 方法
  - [ ] 实现 Unix Socket listener
  - [ ] 实现 `call` CLI 命令（客户端）

- [ ] Day 3: 核心方法实现
  - [ ] 实现 `port_close`, `port_send`, `port_recv`
  - [ ] 实现 `protocol_list`, `protocol_load`, `protocol_unload`
  - [ ] 实现 `connection_list`
  - [ ] 错误处理和 JSON 格式化

- [ ] Day 4: 测试和文档
  - [ ] 单元测试（mock PortManager）
  - [ ] 集成测试（真实串口，使用虚拟端口）
  - [ ] 使用文档（docs/ai/SERVER_MODE.md）
  - [ ] 示例脚本（examples/server_demo.sh）

### Milestone 2: 增强功能 (可选，2-3 天)

- [ ] 连接池管理（超时清理）
- [ ] WebSocket 支持（实时数据流）
- [ ] 性能优化（连接复用、批量操作）
- [ ] Python/TypeScript SDK

### Milestone 3: AI 集成 (可选，2 天)

- [ ] OpenAPI/Swagger 文档生成
- [ ] LangChain Tool 集成
- [ ] Structured Output 支持

---

## 测试策略

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rpc_dispatcher_port_list() {
        let state = ServerState::test_new().await;
        let dispatcher = RpcDispatcher::new(state);

        let request = r#"{"jsonrpc": "2.0", "method": "port_list", "id": 1}"#;
        let response = dispatcher.handle_request(request).await;

        assert!(response.contains("\"result\""));
    }

    #[tokio::test]
    async fn test_port_open_with_protocol() {
        let state = ServerState::test_new().await;
        let dispatcher = RpcDispatcher::new(state);

        // Use virtual port for testing
        let vport = create_virtual_port().await;

        let request = json!({
            "jsonrpc": "2.0",
            "method": "port_open",
            "params": {
                "port": vport.path,
                "protocol": "line"
            },
            "id": 1
        }).to_string();

        let response = dispatcher.handle_request(&request).await;
        let json: Value = serde_json::from_str(&response).unwrap();

        assert_eq!(json["result"]["port"], vport.path);
        assert_eq!(json["result"]["protocol"], "line");
    }
}
```

### 集成测试

```bash
# tests/server_integration_test.sh

#!/bin/bash

set -e

# 1. Start server
serial-cli server start --socket-path /tmp/test-server.sock &
SERVER_PID=$!
sleep 1

# 2. Test port list
RESULT=$(echo '{"jsonrpc":"2.0","method":"port_list","id":1}' | socat - /tmp/test-server.sock)
echo "port_list: $RESULT"

# 3. Test port open
RESULT=$(echo '{"jsonrpc":"2.0","method":"port_open","params":{"port":"/dev/ttyUSB0"},"id":1}' | socat - /tmp/test-server.sock)
echo "port_open: $RESULT"

# 4. Stop server
serial-cli server stop
```

### 性能测试

```rust
#[tokio::test]
async fn bench_concurrent_calls() {
    let state = ServerState::new().await;
    let dispatcher = RpcDispatcher::new(state);

    let start = Instant::now();

    // Spawn 100 concurrent calls
    let tasks: Vec<_> = (0..100)
        .map(|_| {
            let dispatcher = dispatcher.clone();
            tokio::spawn(async move {
                dispatcher.handle_request(r#"{"jsonrpc":"2.0","method":"port_list","id":1}"#).await
            })
        })
        .collect();

    for task in tasks {
        task.await.unwrap();
    }

    let duration = start.elapsed();
    assert!(duration.as_millis() < 1000, "Should complete in < 1s");
}
```

---

## 成功指标

### 功能指标

- ✅ Server 可以启动/停止/查询状态
- ✅ 支持 10+ 并发连接
- ✅ 支持 8+ 核心协议操作
- ✅ 协议在守护进程中持久化（load 一次，全局可用）
- ✅ 延迟 < 5ms (vs CLI 50-200ms)

### 质量指标

- ✅ 测试覆盖率 > 80%
- ✅ 无内存泄漏（运行 24h 稳定）
- ✅ 错误处理完善（所有错误返回 JSON）
- ✅ 文档完整（API 文档 + 示例）

### AI 友好度指标

- ✅ JSON RPC 2.0 标准接口
- ✅ 结构化输出（易于解析）
- ✅ 错误信息包含建议（AI 可以理解）
- ✅ 提供 Python/TypeScript SDK

---

## 参考资料

- **现有实现**：
  - `src/cli/sniff_session.rs` - Session 管理模式
  - `src-tauri/src/state/app_state.rs` - AppState 模式
  - `src/cli/json.rs` - JSON 输出格式

- **外部依赖**：
  - `jsonrpc` crate - JSON RPC 2.0 实现
  - `tokio::net::UnixListener` - Unix Socket 支持
  - `serde_json` - JSON 序列化

- **协议标准**：
  - JSON RPC 2.0 Specification: https://www.jsonrpc.org/specification

---

## 总结

这个 Server Mode 设计：

1. ✅ **最小侵入性** - 不破坏现有功能，作为独立子命令
2. ✅ **复用现有架构** - PortManager、ProtocolManager 直接复用
3. ✅ **参考成熟模式** - SniffDaemon + Tauri AppState
4. ✅ **AI 友好** - JSON RPC 2.0 标准接口
5. ✅ **实现成本低** - 核心功能 3-4 天完成
6. ✅ **扩展性好** - 支持后续添加 WebSocket、SDK 等

**建议优先级**：🔥 高（符合项目定位 "optimized for AI/automation workflows"）
