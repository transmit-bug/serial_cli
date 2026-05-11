//! JSON-RPC 2.0 dispatcher
//!
//! Handles incoming JSON-RPC requests and dispatches to appropriate method handlers.

use crate::serial_core::SerialConfig;
use crate::server::state::{ConnectionContext, ServerState};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::SystemTime;

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

/// JSON-RPC dispatcher
pub struct RpcDispatcher {
    state: ServerState,
}

impl RpcDispatcher {
    /// Create a new RPC dispatcher
    pub fn new(state: ServerState) -> Self {
        Self { state }
    }

    /// Handle incoming JSON-RPC request
    pub async fn handle_request(&self, request: &str) -> String {
        // Parse JSON-RPC request
        let req = match serde_json::from_str::<JsonRpcRequest>(request) {
            Ok(req) => req,
            Err(_) => {
                let resp = self.error_response(-32700, "Parse error", None);
                return serde_json::to_string(&resp.unwrap()).unwrap_or_default();
            }
        };

        // Validate JSON-RPC version
        if req.jsonrpc != "2.0" {
            let response = self.error_response_with_id(-32600, "Invalid Request", None, &req.id);
            return serde_json::to_string(&response).unwrap_or_default();
        }

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
            "server_stats" => self.server_stats(req.params).await,
            _ => self.error_response(-32601, "Method not found", None),
        };

        // Add request ID to response and serialize
        match result {
            Ok(mut resp) => {
                resp.id = req.id;
                serde_json::to_string(&resp).unwrap_or_default()
            }
            Err(err_resp_str) => {
                // Parse the error string back to JsonRpcResponse
                // This is a workaround since we're returning String from error_response
                serde_json::to_string(&err_resp_str).unwrap_or_default()
            }
        }
    }

    /// Create success response
    fn success_response(&self, result: Value) -> Result<JsonRpcResponse, String> {
        Ok(JsonRpcResponse {
            jsonrpc: "2.0",
            result: Some(result),
            error: None,
            id: Value::Null,
        })
    }

    /// Create error response
    fn error_response(&self, code: i32, message: &str, data: Option<&str>) -> Result<JsonRpcResponse, String> {
        Ok(JsonRpcResponse {
            jsonrpc: "2.0",
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.to_string(),
                data: data.map(|d| Value::String(d.to_string())),
            }),
            id: Value::Null,
        })
    }

    /// Create error response with custom ID
    fn error_response_with_id(&self, code: i32, message: &str, data: Option<&str>, id: &Value) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0",
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.to_string(),
                data: data.map(|d| Value::String(d.to_string())),
            }),
            id: id.clone(),
        }
    }

    // === Method handlers ===

    /// List available serial ports
    async fn port_list(&self, params: Option<Value>) -> Result<JsonRpcResponse, String> {
        let _ = params; // Not used

        let manager = self.state.port_manager.lock().await;
        let ports = manager.list_ports().map_err(|e| e.to_string())?;

        let port_list: Vec<Value> = ports
            .into_iter()
            .map(|p| {
                serde_json::json!({
                    "port_name": p.port_name,
                    "port_type": format!("{:?}", p.port_type),
                })
            })
            .collect();

        self.success_response(serde_json::json!({ "ports": port_list }))
    }

    /// Open a serial port
    async fn port_open(&self, params: Option<Value>) -> Result<JsonRpcResponse, String> {
        let params = params.ok_or("Missing params")?;

        let port = params
            .get("port")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'port' parameter")?;

        let baudrate = params
            .get("baudrate")
            .and_then(|v| v.as_u64())
            .unwrap_or(115200) as u32;

        let protocol = params.get("protocol").and_then(|v| v.as_str());

        // Check max connections
        if self.state.is_max_connections_reached().await {
            return self.error_response(-32000, "Max connections reached", None);
        }

        // Open port
        let config = SerialConfig {
            baudrate,
            ..Default::default()
        };

        let port_id = self
            .state
            .port_manager
            .lock()
            .await
            .open_port(port, config)
            .await
            .map_err(|e| e.to_string())?;

        // Create connection context
        let connection_id = port_id.clone();
        let ctx = ConnectionContext {
            connection_id: connection_id.clone(),
            port_id: Some(port_id.clone()),
            protocol_name: protocol.map(|s| s.to_string()),
            created_at: SystemTime::now(),
            last_activity: SystemTime::now(),
        };

        // Store connection
        self.state.add_connection(ctx).await.map_err(|e| e.to_string())?;

        // Set protocol if specified
        // TODO: Implement protocol attachment to port
        // if let Some(proto_name) = protocol {
        //     // Protocol will be used when sending/receiving data
        // }

        self.success_response(serde_json::json!({
            "connection_id": connection_id,
            "port": port,
            "protocol": protocol,
        }))
    }

    /// Close a serial port
    async fn port_close(&self, params: Option<Value>) -> Result<JsonRpcResponse, String> {
        let params = params.ok_or("Missing params")?;

        let connection_id = params
            .get("connection_id")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'connection_id' parameter")?;

        // Remove from connections
        let ctx = self
            .state
            .remove_connection(connection_id)
            .await
            .ok_or("Connection not found")?;

        // Close port
        if let Some(port_id) = ctx.port_id {
            self.state
                .port_manager
                .lock()
                .await
                .close_port(&port_id)
                .await
                .map_err(|e| e.to_string())?;
        }

        self.success_response(serde_json::json!({
            "success": true,
            "connection_id": connection_id,
        }))
    }

    /// Send data to a port
    async fn port_send(&self, params: Option<Value>) -> Result<JsonRpcResponse, String> {
        let params = params.ok_or("Missing params")?;

        let connection_id = params
            .get("connection_id")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'connection_id' parameter")?
            .to_string();

        let data_str = params
            .get("data")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'data' parameter")?;

        // Get connection context
        let port_id = {
            let connections = self.state.connections.read().await;
            let ctx = connections
                .get(&connection_id)
                .ok_or("Connection not found")?;
            ctx.port_id.clone().ok_or("No port associated")?
        };

        // Decode data (assume hex or plain string)
        let data = if data_str.starts_with("hex:") {
            hex_decode(&data_str[4..])
        } else {
            data_str.as_bytes().to_vec()
        };

        // Get port handle and send
        let port_handle = self
            .state
            .port_manager
            .lock()
            .await
            .get_port(&port_id)
            .await
            .map_err(|e| e.to_string())?;

        let mut handle = port_handle.lock().await;
        let bytes_sent = handle.write(&data).map_err(|e| e.to_string())?;

        // Update activity
        self.state.update_activity(&connection_id).await;

        self.success_response(serde_json::json!({
            "bytes_sent": bytes_sent,
        }))
    }

    /// Receive data from a port
    async fn port_recv(&self, params: Option<Value>) -> Result<JsonRpcResponse, String> {
        let params = params.ok_or("Missing params")?;

        let connection_id = params
            .get("connection_id")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'connection_id' parameter")?
            .to_string();

        let _timeout_ms = params
            .get("timeout")
            .and_then(|v| v.as_u64())
            .unwrap_or(1000) as u64;

        // Get connection context
        let port_id = {
            let connections = self.state.connections.read().await;
            let ctx = connections
                .get(&connection_id)
                .ok_or("Connection not found")?;
            ctx.port_id.clone().ok_or("No port associated")?
        };

        // Get port handle
        let port_handle = self
            .state
            .port_manager
            .lock()
            .await
            .get_port(&port_id)
            .await
            .map_err(|e| e.to_string())?;

        // Read data with timeout using spawn_blocking
        let timeout_duration = std::time::Duration::from_millis(_timeout_ms);

        let read_result = tokio::time::timeout(timeout_duration, tokio::task::spawn_blocking(move || {
            let mut buffer = vec![0u8; 4096];
            let mut handle = port_handle.blocking_lock();
            let result = handle.read(&mut buffer);
            (result, buffer)
        }))
        .await;

        let (data_hex, bytes_read, timed_out) = match read_result {
            Ok(Ok((Ok(n), buffer))) => (hex_encode(&buffer), n, false),
            Ok(Ok((Err(e), _))) => return Err(e.to_string()),
            Ok(Err(e)) => return Err(format!("Task join error: {:?}", e)),
            Err(_) => {
                // Timeout occurred
                return self.success_response(serde_json::json!({
                    "data": "",
                    "bytes_read": 0,
                    "timeout": true,
                }));
            }
        };

        // Update activity
        self.state.update_activity(&connection_id).await;

        self.success_response(serde_json::json!({
            "data": data_hex,
            "bytes_read": bytes_read,
            "timeout": timed_out,
        }))
    }

    /// List available protocols
    async fn protocol_list(&self, params: Option<Value>) -> Result<JsonRpcResponse, String> {
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

        self.success_response(serde_json::json!({ "protocols": protocol_list }))
    }

    /// Load a custom protocol
    async fn protocol_load(&self, params: Option<Value>) -> Result<JsonRpcResponse, String> {
        let params = params.ok_or("Missing params")?;

        let path = params
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'path' parameter")?;

        let name = params.get("name").and_then(|v| v.as_str());

        let path_buf = std::path::PathBuf::from(path);
        let mut manager = self.state.protocol_manager.lock().await;

        let info = manager
            .load_protocol(&path_buf)
            .await
            .map_err(|e| e.to_string())?;

        self.success_response(serde_json::json!({
            "name": info.name,
            "description": info.description,
        }))
    }

    /// Unload a custom protocol
    async fn protocol_unload(&self, params: Option<Value>) -> Result<JsonRpcResponse, String> {
        let params = params.ok_or("Missing params")?;

        let name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'name' parameter")?;

        let mut manager = self.state.protocol_manager.lock().await;
        manager.unload_protocol(name).await.map_err(|e| e.to_string())?;

        self.success_response(serde_json::json!({
            "success": true,
            "name": name,
        }))
    }

    /// List active connections
    async fn connection_list(&self, params: Option<Value>) -> Result<JsonRpcResponse, String> {
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

        self.success_response(serde_json::json!({
            "connections": connection_list,
        }))
    }

    /// Get server statistics
    async fn server_stats(&self, params: Option<Value>) -> Result<JsonRpcResponse, String> {
        let _ = params;

        let stats = self.state.connection_stats().await;

        self.success_response(serde_json::json!({
            "connections": stats,
            "max_connections": self.state.config.max_connections,
        }))
    }
}

/// Encode bytes as hex string
fn hex_encode(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join("")
}

/// Decode hex string to bytes
fn hex_decode(hex: &str) -> Vec<u8> {
    if hex.len() % 2 != 0 {
        return Vec::new();
    }

    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16).unwrap_or(0)
        })
        .collect()
}

/// Format SystemTime as ISO 8601 string
fn format_timestamp(time: SystemTime) -> String {
    use std::time::UNIX_EPOCH;
    let duration = time
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
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
        assert_eq!(hex_decode("414243"), b"ABC");
        assert_eq!(hex_decode("00ff"), b"\x00\xff");
    }

    #[test]
    fn test_json_rpc_response() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0",
            result: Some(serde_json::json!({"test": "data"})),
            error: None,
            id: Value::Number(1.into()),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains(r#""jsonrpc":"2.0""#));
        assert!(json.contains(r#""result""#));
    }

    #[tokio::test]
    async fn test_jsonrpc_parse_error() {
        let config = ServerConfig::default();
        let state = ServerState::new(config).await;
        let dispatcher = RpcDispatcher::new(state);

        // Invalid JSON should return parse error
        let response = dispatcher.handle_request("invalid json{{{").await;
        assert!(response.contains(r#""code":-32700"#));
        assert!(response.contains("Parse error"));
    }

    #[tokio::test]
    async fn test_jsonrpc_invalid_version() {
        let config = ServerConfig::default();
        let state = ServerState::new(config).await;
        let dispatcher = RpcDispatcher::new(state);

        // Invalid JSON-RPC version
        let request = r#"{"jsonrpc":"1.0","method":"port_list","id":1}"#;
        let response = dispatcher.handle_request(request).await;
        assert!(response.contains(r#""code":-32600"#));
        assert!(response.contains("Invalid Request"));
    }

    #[tokio::test]
    async fn test_jsonrpc_method_not_found() {
        let config = ServerConfig::default();
        let state = ServerState::new(config).await;
        let dispatcher = RpcDispatcher::new(state);

        // Non-existent method
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

        // Valid port_list call
        let request = r#"{"jsonrpc":"2.0","method":"port_list","params":{},"id":1}"#;
        let response = dispatcher.handle_request(request).await;

        // Debug: print response to see what we got
        eprintln!("Response: {}", response);

        // Should be valid JSON-RPC 2.0 response
        assert!(response.contains(r#""jsonrpc":"2.0""#));
        assert!(response.contains(r#""id":1"#));

        // Response should be valid JSON (can parse)
        let _: serde_json::Value = serde_json::from_str(&response)
            .expect("Response should be valid JSON");
    }

    #[tokio::test]
    async fn test_jsonrpc_response_preserves_id() {
        let config = ServerConfig::default();
        let state = ServerState::new(config).await;
        let dispatcher = RpcDispatcher::new(state);

        // Test with custom ID
        let request = r#"{"jsonrpc":"2.0","method":"port_list","params":{},"id":"test-id-123"}"#;
        let response = dispatcher.handle_request(request).await;

        // Response should preserve the ID
        assert!(response.contains(r#""id":"test-id-123""#));
    }
}
