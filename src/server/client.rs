//! Remote JSON-RPC client for LAN access to a Daemon
//!
//! A minimal, dependency-free client that connects to a [`crate::server`]
//! daemon over TCP and speaks the same newline-framed JSON-RPC 2.0 protocol
//! as the Unix socket listener. Each call opens a fresh connection (LAN
//! latency makes this ~1ms) and closes it after the response, so there is no
//! stale-connection state to manage when the remote device restarts.
//!
//! Used by the Tauri GUI's remote device feature; the same surface is the
//! foundation for future official SDKs.

use crate::error::{Result, SerialError};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::io;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

/// Connect timeout for remote daemons (LAN)
const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);
/// Maximum response frame size (matches the server's 1 MiB line limit)
const MAX_FRAME_BYTES: usize = 1024 * 1024;

/// Remote JSON-RPC client bound to one daemon address.
#[derive(Debug, Clone)]
pub struct RemoteRpcClient {
    addr: SocketAddr,
}

/// Serial port listing entry
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RemotePortInfo {
    pub port_name: String,
    pub port_type: String,
}

/// Result of a successful `port_open`
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RemoteOpenResult {
    pub connection_id: String,
    pub port: String,
    #[serde(default)]
    pub protocol: Option<String>,
}

/// Result of a `port_recv` call (data is hex-encoded)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RemoteRecvResult {
    /// Hex-encoded bytes received
    pub data: String,
    pub bytes_read: usize,
    #[serde(default)]
    pub timeout: bool,
}

/// Active connection summary
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RemoteConnectionInfo {
    pub connection_id: String,
    #[serde(default)]
    pub port_id: Option<String>,
    #[serde(default)]
    pub protocol: Option<String>,
}

/// One `port_data` push notification received on a stream
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RemoteStreamData {
    pub connection_id: String,
    /// Hex-encoded bytes
    pub data_hex: String,
    pub bytes_read: usize,
    pub timestamp: u64,
}

/// Event yielded by [`RemoteDataStream`]
#[derive(Debug, Clone)]
pub enum RemoteStreamEvent {
    /// A data push notification
    Data(RemoteStreamData),
    /// The subscribe request failed (e.g. unknown connection id)
    Error(String),
}

/// Persistent streaming client for a remote connection's data pushes.
///
/// Opens one TCP connection, sends `port_subscribe`, then yields every
/// `port_data` notification as it arrives. The connection stays open until
/// [`RemoteDataStream::close`], EOF, or an error.
#[derive(Debug)]
pub struct RemoteDataStream {
    addr: SocketAddr,
    connection_id: String,
    rx: mpsc::Receiver<RemoteStreamEvent>,
    shutdown: Option<CancellationToken>,
    task: JoinHandle<()>,
}

impl RemoteDataStream {
    /// Connect, subscribe, and start streaming. The subscribe response is
    /// processed in the background task; failures surface as
    /// [`RemoteStreamEvent::Error`] on [`recv`](Self::recv).
    pub async fn connect(addr: SocketAddr, connection_id: &str) -> Result<Self> {
        let mut stream = tokio::time::timeout(CONNECT_TIMEOUT, TcpStream::connect(addr))
            .await
            .map_err(|_| {
                SerialError::Io(io::Error::new(
                    io::ErrorKind::TimedOut,
                    format!("Connection to {} timed out", addr),
                ))
            })?
            .map_err(|e| {
                SerialError::Io(io::Error::new(
                    io::ErrorKind::NotConnected,
                    format!("Failed to connect to {}: {}", addr, e),
                ))
            })?;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "port_subscribe",
            "params": { "connection_id": connection_id },
            "id": 1,
        });
        let line = format!(
            "{}\n",
            serde_json::to_string(&request).map_err(|e| {
                SerialError::Io(io::Error::other(format!("Serialize subscribe: {}", e)))
            })?
        );
        stream
            .write_all(line.as_bytes())
            .await
            .map_err(|e| SerialError::Io(io::Error::other(format!("Send subscribe: {}", e))))?;

        let (tx, rx) = mpsc::channel::<RemoteStreamEvent>(256);
        let token = CancellationToken::new();
        let task = tokio::spawn(run_stream(
            stream,
            connection_id.to_string(),
            tx,
            token.clone(),
        ));

        Ok(Self {
            addr,
            connection_id: connection_id.to_string(),
            rx,
            shutdown: Some(token),
            task,
        })
    }

    /// Target daemon address
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Connection id this stream is subscribed to
    pub fn connection_id(&self) -> &str {
        &self.connection_id
    }

    /// Wait for the next stream event. Returns `None` when the stream ends.
    pub async fn recv(&mut self) -> Option<RemoteStreamEvent> {
        self.rx.recv().await
    }

    /// Unsubscribe and close the stream.
    pub async fn close(mut self) {
        if let Some(token) = self.shutdown.take() {
            token.cancel();
        }
        // Give the task a moment to send unsubscribe and flush
        let _ = tokio::time::timeout(Duration::from_millis(300), &mut self.task).await;
    }
}

/// Background reader: forwards `port_data` notifications and surfaces errors.
async fn run_stream(
    mut stream: TcpStream,
    connection_id: String,
    tx: mpsc::Sender<RemoteStreamEvent>,
    token: CancellationToken,
) {
    let mut buf: Vec<u8> = Vec::new();
    let mut tmp = [0u8; 4096];

    loop {
        tokio::select! {
            _ = token.cancelled() => {
                // Best-effort unsubscribe before closing
                let req = serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "port_unsubscribe",
                    "params": { "connection_id": connection_id },
                    "id": 2,
                });
                let _ = stream
                    .write_all(format!("{}\n", req).as_bytes())
                    .await;
                break;
            }
            r = stream.read(&mut tmp) => {
                match r {
                    Ok(0) => break, // EOF — remote closed the connection
                    Ok(n) => {
                        buf.extend_from_slice(&tmp[..n]);
                        while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
                            let mut line: Vec<u8> = buf.drain(..=pos).collect();
                            line.pop(); // strip \n
                            let line = String::from_utf8_lossy(&line);
                            if let Ok(v) = serde_json::from_str::<Value>(&line) {
                                match v.get("method").and_then(|m| m.as_str()) {
                                    Some("port_data") => {
                                        if let Some(params) = v.get("params").cloned() {
                                            if let Ok(data) =
                                                serde_json::from_value::<RemoteStreamData>(params)
                                            {
                                                if tx
                                                    .send(RemoteStreamEvent::Data(data))
                                                    .await
                                                    .is_err()
                                                {
                                                    return;
                                                }
                                            }
                                        }
                                    }
                                    _ => {
                                        // Response to subscribe/unsubscribe
                                        if let Some(err) = v.get("error") {
                                            if !err.is_null() {
                                                let msg = err
                                                    .get("message")
                                                    .and_then(|m| m.as_str())
                                                    .unwrap_or("RPC error")
                                                    .to_string();
                                                let _ = tx
                                                    .send(RemoteStreamEvent::Error(msg))
                                                    .await;
                                                return;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx
                            .send(RemoteStreamEvent::Error(e.to_string()))
                            .await;
                        break;
                    }
                }
            }
        }
    }
}

/// Daemon statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RemoteServerStats {
    pub max_connections: usize,
    pub total_requests: u64,
    pub total_errors: u64,
    pub started_at: u64,
}

impl RemoteRpcClient {
    /// Create a client for `host:port`. `host` may be an IP literal or a
    /// hostname (resolved synchronously).
    pub fn new(host: &str, port: u16) -> Result<Self> {
        let addr = if let Ok(ip) = host.parse::<std::net::IpAddr>() {
            SocketAddr::new(ip, port)
        } else {
            std::net::ToSocketAddrs::to_socket_addrs(&(host, port))
                .map_err(|e| {
                    SerialError::Io(io::Error::other(format!(
                        "Cannot resolve '{}': {}",
                        host, e
                    )))
                })?
                .next()
                .ok_or_else(|| {
                    SerialError::Io(io::Error::other(format!("No addresses for '{}'", host)))
                })?
        };
        Ok(Self { addr })
    }

    /// Address this client targets
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Send a JSON-RPC request and return the `result` value.
    /// Any non-null `error` in the response becomes an error.
    async fn call_raw(&self, method: &str, params: Value) -> Result<Value> {
        let mut stream = tokio::time::timeout(CONNECT_TIMEOUT, TcpStream::connect(self.addr))
            .await
            .map_err(|_| {
                SerialError::Io(io::Error::new(
                    io::ErrorKind::TimedOut,
                    format!("Connection to {} timed out", self.addr),
                ))
            })?
            .map_err(|e| {
                SerialError::Io(io::Error::new(
                    io::ErrorKind::NotConnected,
                    format!("Failed to connect to {}: {}", self.addr, e),
                ))
            })?;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1,
        });
        let line = format!(
            "{}\n",
            serde_json::to_string(&request).map_err(|e| {
                SerialError::Io(io::Error::other(format!("Serialize request: {}", e)))
            })?
        );
        stream
            .write_all(line.as_bytes())
            .await
            .map_err(|e| SerialError::Io(io::Error::other(format!("Send request: {}", e))))?;

        // Read the response line (newline-framed)
        let mut buf: Vec<u8> = Vec::new();
        let mut tmp = [0u8; 4096];
        loop {
            let n = stream
                .read(&mut tmp)
                .await
                .map_err(|e| SerialError::Io(io::Error::other(format!("Read response: {}", e))))?;
            if n == 0 {
                break;
            }
            buf.extend_from_slice(&tmp[..n]);
            if buf.contains(&b'\n') || buf.len() >= MAX_FRAME_BYTES {
                break;
            }
        }
        if buf.is_empty() {
            return Err(SerialError::Io(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                format!("{} closed the connection without responding", self.addr),
            )));
        }

        let response: Value = serde_json::from_slice(&buf).map_err(|e| {
            SerialError::Io(io::Error::other(format!(
                "Invalid JSON-RPC response: {}",
                e
            )))
        })?;

        if let Some(error) = response.get("error") {
            if !error.is_null() {
                let message = error
                    .get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("RPC error")
                    .to_string();
                return Err(SerialError::Io(io::Error::other(message)));
            }
        }

        response
            .get("result")
            .cloned()
            .ok_or_else(|| SerialError::Io(io::Error::other("Response missing result")))
    }

    /// Call a method and deserialize the result into a typed value.
    pub async fn call<T: DeserializeOwned>(&self, method: &str, params: Value) -> Result<T> {
        let value = self.call_raw(method, params).await?;
        serde_json::from_value(value)
            .map_err(|e| SerialError::Io(io::Error::other(format!("Bad response shape: {}", e))))
    }

    // ── Convenience methods ──────────────────────────────────────────────

    /// List available serial ports on the remote device.
    pub async fn port_list(&self) -> Result<Vec<RemotePortInfo>> {
        let value: Value = self.call("port_list", serde_json::json!({})).await?;
        let ports = value.get("ports").cloned().unwrap_or(Value::Array(vec![]));
        serde_json::from_value(ports)
            .map_err(|e| SerialError::Io(io::Error::other(format!("Bad port_list shape: {}", e))))
    }

    /// Open a serial port on the remote device.
    pub async fn port_open(
        &self,
        port: &str,
        baudrate: u32,
        protocol: Option<&str>,
    ) -> Result<RemoteOpenResult> {
        let mut params = serde_json::json!({ "port": port, "baudrate": baudrate });
        if let Some(p) = protocol {
            params["protocol"] = Value::String(p.to_string());
        }
        self.call("port_open", params).await
    }

    /// Close a remote connection.
    pub async fn port_close(&self, connection_id: &str) -> Result<Value> {
        self.call(
            "port_close",
            serde_json::json!({ "connection_id": connection_id }),
        )
        .await
    }

    /// Send raw bytes to a remote connection.
    pub async fn port_send(&self, connection_id: &str, data: &[u8]) -> Result<usize> {
        let hex = crate::utils::hex::hex_encode_simple(data);
        let value: Value = self
            .call(
                "port_send",
                serde_json::json!({
                    "connection_id": connection_id,
                    "data": format!("hex:{}", hex),
                }),
            )
            .await?;
        value
            .get("bytes_sent")
            .and_then(|v| v.as_u64())
            .map(|n| n as usize)
            .ok_or_else(|| SerialError::Io(io::Error::other("Missing bytes_sent in response")))
    }

    /// Receive data from a remote connection with a timeout (ms).
    pub async fn port_recv(
        &self,
        connection_id: &str,
        timeout_ms: u64,
    ) -> Result<RemoteRecvResult> {
        self.call(
            "port_recv",
            serde_json::json!({
                "connection_id": connection_id,
                "timeout": timeout_ms,
            }),
        )
        .await
    }

    /// List active connections on the remote daemon.
    pub async fn connection_list(&self) -> Result<Vec<RemoteConnectionInfo>> {
        let value: Value = self.call("connection_list", serde_json::json!({})).await?;
        let conns = value
            .get("connections")
            .cloned()
            .unwrap_or(Value::Array(vec![]));
        serde_json::from_value(conns).map_err(|e| {
            SerialError::Io(io::Error::other(format!(
                "Bad connection_list shape: {}",
                e
            )))
        })
    }

    /// Get remote daemon statistics.
    pub async fn server_stats(&self) -> Result<RemoteServerStats> {
        self.call("server_stats", serde_json::json!({})).await
    }
}
