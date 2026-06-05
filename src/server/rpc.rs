//! JSON-RPC 2.0 dispatcher
//!
//! Handles incoming JSON-RPC requests and dispatches to appropriate method handlers.
//! Uses typed parameter structs for automatic serde deserialization.

use crate::serial_core::SerialConfig;
use crate::server::session::ServerSessionManager;
use crate::server::state::{ConnectionContext, ServerState};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::SystemTime;

// ── Typed parameter structs ──────────────────────────────────────────────

#[derive(Deserialize)]
struct PortOpenParams {
    port: String,
    #[serde(default = "default_baudrate")]
    baudrate: u32,
    protocol: Option<String>,
}

fn default_baudrate() -> u32 {
    115200
}

#[derive(Deserialize)]
struct PortCloseParams {
    connection_id: String,
}

#[derive(Deserialize)]
struct PortSendParams {
    connection_id: String,
    data: String,
}

#[derive(Deserialize)]
struct PortRecvParams {
    connection_id: String,
    #[serde(default = "default_timeout")]
    timeout: u64,
}

fn default_timeout() -> u64 {
    1000
}

#[derive(Deserialize)]
struct ProtocolLoadParams {
    path: String,
    #[serde(default)]
    #[allow(dead_code)]
    name: Option<String>,
}

#[derive(Deserialize)]
struct ProtocolUnloadParams {
    name: String,
}

// ── JSON-RPC envelope types ─────────────────────────────────────────────

/// JSON-RPC 2.0 request
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Option<Value>,
    id: Value,
}

/// JSON-RPC 2.0 response
#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: &'static str,
    result: Option<Value>,
    error: Option<JsonRpcError>,
    id: Value,
}

/// JSON-RPC 2.0 error
#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

/// Structured error returned by method handlers: (code, message, optional data).
type MethodError = (i32, String, Option<Value>);

/// JSON-RPC dispatcher
pub struct RpcDispatcher {
    state: ServerState,
}

impl RpcDispatcher {
    /// Create a new RPC dispatcher
    pub fn new(state: ServerState) -> Self {
        Self { state }
    }

    /// Handle incoming JSON-RPC request.
    /// Appends `\n` to every response so clients can use line-based framing
    /// instead of tracking brace nesting on persistent connections.
    pub async fn handle_request(&self, request: &str) -> String {
        let req = match serde_json::from_str::<JsonRpcRequest>(request) {
            Ok(req) => req,
            Err(_) => {
                return Self::serialize_response(Self::error_resp(
                    -32700,
                    "Parse error",
                    None,
                    &Value::Null,
                ));
            }
        };

        if req.jsonrpc != "2.0" {
            return Self::serialize_response(Self::error_resp(
                -32600,
                "Invalid Request",
                None,
                &req.id,
            ));
        }

        self.state
            .total_requests
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let result: Result<Value, MethodError> = match req.method.as_str() {
            "port_list" => self.port_list(req.params).await,
            "port_open" => self.port_open(req.params).await,
            "port_close" => self.port_close(req.params).await,
            "port_send" => self.port_send(req.params).await,
            "port_recv" => self.port_recv(req.params).await,
            "port_subscribe" => self.port_subscribe(req.params).await,
            "port_unsubscribe" => self.port_unsubscribe(req.params).await,
            "protocol_list" => self.protocol_list(req.params).await,
            "protocol_load" => self.protocol_load(req.params).await,
            "protocol_unload" => self.protocol_unload(req.params).await,
            "connection_list" => self.connection_list(req.params).await,
            "server_stats" => self.server_stats(req.params).await,
            _ => Err((-32601, "Method not found".to_string(), None)),
        };

        if result.is_err() {
            self.state
                .total_errors
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }

        let response = match result {
            Ok(value) => Self::success_resp(value, &req.id),
            Err((code, message, data)) => Self::error_resp(code, &message, data, &req.id),
        };
        Self::serialize_response(response)
    }

    fn success_resp(result: Value, id: &Value) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0",
            result: Some(result),
            error: None,
            id: id.clone(),
        }
    }

    fn error_resp(code: i32, message: &str, data: Option<Value>, id: &Value) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0",
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.to_string(),
                data,
            }),
            id: id.clone(),
        }
    }

    fn serialize_response(resp: JsonRpcResponse) -> String {
        format!("{}\n", serde_json::to_string(&resp).unwrap_or_default())
    }

    // === Method handlers ===
    // Each returns Result<Value, MethodError> where MethodError = (code, message, data)

    /// List available serial ports
    async fn port_list(&self, params: Option<Value>) -> Result<Value, MethodError> {
        let _ = params;

        let manager = self.state.port_manager.lock().await;
        let ports = manager
            .list_ports()
            .map_err(|e| (-32603, e.to_string(), None))?;

        let port_list: Vec<Value> = ports
            .into_iter()
            .map(|p| {
                serde_json::json!({
                    "port_name": p.port_name,
                    "port_type": format!("{:?}", p.port_type),
                })
            })
            .collect();

        Ok(serde_json::json!({ "ports": port_list }))
    }

    /// Open a serial port
    async fn port_open(&self, params: Option<Value>) -> Result<Value, MethodError> {
        let params: PortOpenParams = parse_params(params)?;

        if self.state.is_max_connections_reached().await {
            return Err((-32000, "Max connections reached".to_string(), None));
        }

        let config = SerialConfig {
            baudrate: params.baudrate,
            ..Default::default()
        };

        let port_id = self
            .state
            .port_manager
            .lock()
            .await
            .open_port(&params.port, config)
            .await
            .map_err(|e| (-32603, e.to_string(), None))?;

        // Attach protocol instance to the port handle if one was requested
        if let Some(ref proto_name) = params.protocol {
            let registry = self.state.protocol_registry.lock().await;
            let port_manager = self.state.port_manager.lock().await;
            if let Err(e) = port_manager
                .set_port_protocol_by_name(&port_id, &registry, proto_name)
                .await
            {
                tracing::warn!(
                    "Failed to attach protocol '{}' to port {}: {}",
                    proto_name,
                    port_id,
                    e
                );
            }
        }

        let connection_id = port_id.clone();
        let ctx = ConnectionContext {
            connection_id: connection_id.clone(),
            port_id: Some(port_id.clone()),
            protocol_name: params.protocol.clone(),
            created_at: SystemTime::now(),
            last_activity: SystemTime::now(),
            subscribed: false,
        };

        self.state
            .add_connection(ctx)
            .await
            .map_err(|e| (-32603, e.to_string(), None))?;

        Ok(serde_json::json!({
            "connection_id": connection_id,
            "port": params.port,
            "protocol": params.protocol,
        }))
    }

    /// Close a serial port
    async fn port_close(&self, params: Option<Value>) -> Result<Value, MethodError> {
        let params: PortCloseParams = parse_params(params)?;

        let ctx = self
            .state
            .remove_connection(&params.connection_id)
            .await
            .ok_or((-32603, "Connection not found".to_string(), None))?;

        if let Some(port_id) = ctx.port_id {
            self.state
                .port_manager
                .lock()
                .await
                .close_port(&port_id)
                .await
                .map_err(|e| (-32603, e.to_string(), None))?;
        }

        Ok(serde_json::json!({
            "success": true,
            "connection_id": params.connection_id,
        }))
    }

    /// Send data to a port
    async fn port_send(&self, params: Option<Value>) -> Result<Value, MethodError> {
        let params: PortSendParams = parse_params(params)?;

        let port_id = {
            let connections = self.state.connections.read().await;
            let ctx = connections.get(&params.connection_id).ok_or((
                -32603,
                "Connection not found".to_string(),
                None,
            ))?;
            ctx.port_id
                .clone()
                .ok_or((-32603, "No port associated".to_string(), None))?
        };

        let data = if let Some(hex_str) = params.data.strip_prefix("hex:") {
            crate::cli::commands::parsers::parse_hex_string(hex_str)
                .map_err(|e| (-32602, e.to_string(), None))?
        } else {
            params.data.as_bytes().to_vec()
        };

        let port_handle = self
            .state
            .port_manager
            .lock()
            .await
            .get_port(&port_id)
            .await
            .map_err(|e| (-32603, e.to_string(), None))?;

        let mut handle = port_handle.lock().await;
        let bytes_sent = handle
            .write(&data)
            .map_err(|e| (-32603, e.to_string(), None))?;

        self.state.update_activity(&params.connection_id).await;

        Ok(serde_json::json!({
            "bytes_sent": bytes_sent,
        }))
    }

    /// Receive data from a port
    async fn port_recv(&self, params: Option<Value>) -> Result<Value, MethodError> {
        let params: PortRecvParams = parse_params(params)?;

        let port_id = {
            let connections = self.state.connections.read().await;
            let ctx = connections.get(&params.connection_id).ok_or((
                -32603,
                "Connection not found".to_string(),
                None,
            ))?;
            ctx.port_id
                .clone()
                .ok_or((-32603, "No port associated".to_string(), None))?
        };

        let port_handle = self
            .state
            .port_manager
            .lock()
            .await
            .get_port(&port_id)
            .await
            .map_err(|e| (-32603, e.to_string(), None))?;

        let timeout_duration = std::time::Duration::from_millis(params.timeout);

        let read_result = tokio::time::timeout(
            timeout_duration,
            tokio::task::spawn_blocking(move || {
                let mut buffer = vec![0u8; 4096];
                let mut handle = port_handle.blocking_lock();
                let result = handle.read(&mut buffer);
                (result, buffer)
            }),
        )
        .await;

        let (data_hex, bytes_read, timed_out) = match read_result {
            Ok(Ok((Ok(n), buffer))) => {
                let n = n.min(buffer.len());
                (hex_encode(&buffer[..n]), n, false)
            }
            Ok(Ok((Err(e), _))) => return Err((-32603, e.to_string(), None)),
            Ok(Err(e)) => return Err((-32603, format!("Task join error: {:?}", e), None)),
            Err(_) => {
                return Ok(serde_json::json!({
                    "data": "",
                    "bytes_read": 0,
                    "timeout": true,
                }));
            }
        };

        self.state.update_activity(&params.connection_id).await;

        Ok(serde_json::json!({
            "data": data_hex,
            "bytes_read": bytes_read,
            "timeout": timed_out,
        }))
    }

    /// Subscribe to data push notifications for a port
    async fn port_subscribe(&self, params: Option<Value>) -> Result<Value, MethodError> {
        #[derive(Deserialize)]
        struct Params {
            connection_id: String,
        }
        let params: Params = parse_params(params)?;

        let mut connections = self.state.connections.write().await;
        let ctx = connections.get_mut(&params.connection_id).ok_or((
            -32603,
            "Connection not found".to_string(),
            None,
        ))?;

        if ctx.subscribed {
            return Ok(serde_json::json!({
                "subscribed": true,
                "already_subscribed": true,
            }));
        }

        ctx.subscribed = true;
        Ok(serde_json::json!({
            "subscribed": true,
        }))
    }

    /// Unsubscribe from data push notifications
    async fn port_unsubscribe(&self, params: Option<Value>) -> Result<Value, MethodError> {
        #[derive(Deserialize)]
        struct Params {
            connection_id: String,
        }
        let params: Params = parse_params(params)?;

        let mut connections = self.state.connections.write().await;
        let ctx = connections.get_mut(&params.connection_id).ok_or((
            -32603,
            "Connection not found".to_string(),
            None,
        ))?;

        ctx.subscribed = false;
        Ok(serde_json::json!({
            "subscribed": false,
        }))
    }

    /// List available protocols
    async fn protocol_list(&self, params: Option<Value>) -> Result<Value, MethodError> {
        let _ = params;

        let registry = self.state.protocol_registry.lock().await;
        let protocols = registry.list_protocols().await;

        let protocol_list: Vec<Value> = protocols
            .into_iter()
            .map(|p| {
                serde_json::json!({
                    "name": p.name,
                    "description": p.description,
                })
            })
            .collect();

        Ok(serde_json::json!({ "protocols": protocol_list }))
    }

    /// Load a custom protocol
    async fn protocol_load(&self, params: Option<Value>) -> Result<Value, MethodError> {
        let params: ProtocolLoadParams = parse_params(params)?;

        let path_buf = std::path::PathBuf::from(&params.path);
        let mut manager = self.state.protocol_manager.lock().await;

        let info = manager
            .load_protocol(&path_buf)
            .await
            .map_err(|e| (-32603, e.to_string(), None))?;

        Ok(serde_json::json!({
            "name": info.name,
            "description": info.description,
        }))
    }

    /// Unload a custom protocol
    async fn protocol_unload(&self, params: Option<Value>) -> Result<Value, MethodError> {
        let params: ProtocolUnloadParams = parse_params(params)?;

        let mut manager = self.state.protocol_manager.lock().await;
        manager
            .unload_protocol(&params.name)
            .await
            .map_err(|e| (-32603, e.to_string(), None))?;

        Ok(serde_json::json!({
            "success": true,
            "name": params.name,
        }))
    }

    /// List active connections
    async fn connection_list(&self, params: Option<Value>) -> Result<Value, MethodError> {
        let _ = params;

        let connections = self.state.connections.read().await;
        let mut connection_list = Vec::new();

        for ctx in connections.values() {
            connection_list.push(serde_json::json!({
                "connection_id": ctx.connection_id,
                "port_id": ctx.port_id,
                "protocol": ctx.protocol_name,
                "created_at": format_timestamp(ctx.created_at),
            }));
        }

        Ok(serde_json::json!({
            "connections": connection_list,
        }))
    }

    /// Get server statistics
    async fn server_stats(&self, params: Option<Value>) -> Result<Value, MethodError> {
        let _ = params;

        let stats = self.state.connection_stats().await;
        let total_requests = self
            .state
            .total_requests
            .load(std::sync::atomic::Ordering::Relaxed);
        let total_errors = self
            .state
            .total_errors
            .load(std::sync::atomic::Ordering::Relaxed);
        let started_at = ServerSessionManager::current_timestamp();

        Ok(serde_json::json!({
            "connections": stats,
            "max_connections": self.state.config.max_connections,
            "total_requests": total_requests,
            "total_errors": total_errors,
            "started_at": started_at,
        }))
    }
}

/// Deserialize params from Option<Value> into a typed struct.
fn parse_params<T: for<'de> Deserialize<'de>>(params: Option<Value>) -> Result<T, MethodError> {
    let params = params.ok_or((-32602, "Missing params".to_string(), None))?;
    serde_json::from_value(params).map_err(|e| (-32602, format!("Invalid params: {}", e), None))
}

/// Encode bytes as hex string
fn hex_encode(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join("")
}

/// Format SystemTime as ISO 8601 string
fn format_timestamp(time: SystemTime) -> String {
    use std::time::UNIX_EPOCH;
    let duration = time.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();
    use chrono::{DateTime, Utc};
    let dt = DateTime::<Utc>::from_timestamp(secs as i64, 0).unwrap_or_default();
    dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::state::ServerConfig;

    #[test]
    fn test_hex_encode() {
        assert_eq!(hex_encode(b"ABC"), "414243");
        assert_eq!(hex_encode(b"\x00\xff"), "00ff");
    }

    #[test]
    fn test_hex_decode() {
        assert_eq!(
            crate::cli::commands::parsers::parse_hex_string("414243").unwrap(),
            b"ABC"
        );
        assert_eq!(
            crate::cli::commands::parsers::parse_hex_string("00ff").unwrap(),
            b"\x00\xff"
        );
    }

    #[tokio::test]
    async fn test_jsonrpc_parse_error() {
        let config = ServerConfig::default();
        let state = ServerState::new(config).await;
        let dispatcher = RpcDispatcher::new(state);

        let response = dispatcher.handle_request("invalid json{{{").await;
        assert!(response.contains(r#""code":-32700"#));
        assert!(response.contains("Parse error"));
        assert!(response.ends_with('\n'), "Response should end with newline");
    }

    #[tokio::test]
    async fn test_jsonrpc_invalid_version() {
        let config = ServerConfig::default();
        let state = ServerState::new(config).await;
        let dispatcher = RpcDispatcher::new(state);

        let request = r#"{"jsonrpc":"1.0","method":"port_list","id":1}"#;
        let response = dispatcher.handle_request(request).await;
        assert!(response.contains(r#""code":-32600"#));
        assert!(response.contains("Invalid Request"));
        assert!(response.ends_with('\n'), "Response should end with newline");
    }

    #[tokio::test]
    async fn test_jsonrpc_method_not_found() {
        let config = ServerConfig::default();
        let state = ServerState::new(config).await;
        let dispatcher = RpcDispatcher::new(state);

        let request = r#"{"jsonrpc":"2.0","method":"nonexistent_method","id":1}"#;
        let response = dispatcher.handle_request(request).await;
        assert!(response.contains(r#""code":-32601"#));
        assert!(response.contains("Method not found"));
    }

    #[tokio::test]
    async fn test_jsonrpc_port_list_format() {
        let config = ServerConfig::default();
        let state = ServerState::new(config).await;
        let dispatcher = RpcDispatcher::new(state);

        let request = r#"{"jsonrpc":"2.0","method":"port_list","params":{},"id":1}"#;
        let response = dispatcher.handle_request(request).await;

        eprintln!("Response: {}", response);

        assert!(response.contains(r#""jsonrpc":"2.0""#));
        assert!(response.contains(r#""id":1"#));
        assert!(response.ends_with('\n'), "Response should end with newline");

        let json_str = response.trim_end();
        let _: serde_json::Value =
            serde_json::from_str(json_str).expect("Response should be valid JSON");
    }

    #[tokio::test]
    async fn test_jsonrpc_response_preserves_id() {
        let config = ServerConfig::default();
        let state = ServerState::new(config).await;
        let dispatcher = RpcDispatcher::new(state);

        let request = r#"{"jsonrpc":"2.0","method":"port_list","params":{},"id":"test-id-123"}"#;
        let response = dispatcher.handle_request(request).await;

        assert!(response.contains(r#""id":"test-id-123""#));
    }

    #[tokio::test]
    async fn test_typed_params_port_open_defaults() {
        // Verify that PortOpenParams deserializes with defaults
        let params: PortOpenParams = serde_json::from_value(serde_json::json!({
            "port": "/dev/ttyUSB0"
        }))
        .unwrap();
        assert_eq!(params.port, "/dev/ttyUSB0");
        assert_eq!(params.baudrate, 115200);
        assert!(params.protocol.is_none());
    }

    #[tokio::test]
    async fn test_typed_params_port_recv_defaults() {
        let params: PortRecvParams = serde_json::from_value(serde_json::json!({
            "connection_id": "test"
        }))
        .unwrap();
        assert_eq!(params.connection_id, "test");
        assert_eq!(params.timeout, 1000);
    }
}
