//! End-to-end tests for Server Mode over both transports
//!
//! These tests spawn the actual server daemon directly (bypassing the session
//! file) and connect via Unix domain sockets and TCP to validate the full
//! client-server lifecycle: start → RPC calls → graceful shutdown.
//!
//! Run with: `cargo test --test e2e_server_tests -- --ignored`

#![cfg(unix)]

use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::atomic::{AtomicU16, Ordering};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::net::{TcpStream, UnixStream};

/// Unique socket path per test run to avoid conflicts
fn unique_socket_path(test_name: &str) -> PathBuf {
    let pid = std::process::id();
    PathBuf::from(format!("/tmp/serial-cli-e2e-{}-{}.sock", test_name, pid))
}

/// Unique TCP port per test run (tests run concurrently in one process)
fn next_tcp_port() -> u16 {
    static PORT_COUNTER: AtomicU16 = AtomicU16::new(0);
    24000 + PORT_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Timeout for server startup — longer in CI where compilation is slower
fn server_startup_timeout_secs() -> u64 {
    if std::env::var("CI").is_ok() {
        60
    } else {
        15
    }
}

/// Build the server binary if needed (checks mtime to avoid unnecessary rebuilds)
fn ensure_server_binary() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let binary_path = manifest_dir.join("target/debug/serial-cli");

    let needs_build = if !binary_path.exists() {
        true
    } else {
        // Check if any source file is newer than the binary
        let binary_mtime = std::fs::metadata(&binary_path)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);

        let src_dir = manifest_dir.join("src");
        let cargo_toml = manifest_dir.join("Cargo.toml");
        let cargo_lock = manifest_dir.join("Cargo.lock");

        fn is_newer(path: &PathBuf, threshold: std::time::SystemTime) -> bool {
            std::fs::metadata(path)
                .and_then(|m| m.modified())
                .map(|t| t > threshold)
                .unwrap_or(false)
        }

        is_newer(&cargo_toml, binary_mtime)
            || is_newer(&cargo_lock, binary_mtime)
            || dir_has_newer_file(&src_dir, binary_mtime)
    };

    if needs_build {
        let status = Command::new("cargo")
            .args(["build", "--bin", "serial-cli", "--quiet"])
            .status()
            .expect("cargo should be available");
        assert!(status.success(), "cargo build failed");
    }
}

/// Recursively check if any file in directory is newer than threshold
fn dir_has_newer_file(dir: &PathBuf, threshold: std::time::SystemTime) -> bool {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if dir_has_newer_file(&path, threshold) {
                    return true;
                }
            } else if let Ok(metadata) = std::fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    if modified > threshold {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Start the server daemon directly (bypassing session file) as a background
/// process. The returned Child is the actual daemon process, so SIGKILL stops
/// it cleanly.
fn start_server(socket_path: &PathBuf, tcp_port: Option<u16>) -> Child {
    // Clean up existing socket
    if socket_path.exists() {
        let _ = std::fs::remove_file(socket_path);
    }

    ensure_server_binary();

    let binary_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/debug/serial-cli");

    let mut cmd = Command::new(&binary_path);
    cmd.args([
        "server",
        "daemon",
        "--socket-path",
        socket_path.to_str().unwrap(),
    ]);
    if let Some(port) = tcp_port {
        cmd.arg("--port").arg(port.to_string());
    }
    cmd.stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("daemon should start")
}

/// Stop the server daemon and clean up the socket file.
fn stop_server(mut child: Child, socket_path: &PathBuf) {
    #[cfg(unix)]
    {
        let pid = child.id() as libc::pid_t;
        unsafe {
            let _ = libc::kill(pid, libc::SIGKILL);
        }
    }
    let _ = child.wait();

    // Clean up socket file
    if socket_path.exists() {
        let _ = std::fs::remove_file(socket_path);
    }
}

/// Wait for the server's Unix socket to accept connections (up to timeout)
async fn wait_for_server(socket_path: &PathBuf, timeout_secs: u64) -> Result<(), String> {
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(timeout_secs);

    loop {
        if socket_path.exists() {
            if UnixStream::connect(socket_path).await.is_ok() {
                return Ok(());
            }
        }

        if start.elapsed() > timeout {
            return Err(format!(
                "Server did not start within {}s (socket: {})",
                timeout_secs,
                socket_path.display()
            ));
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

/// Wait for the server's TCP listener to accept connections (up to timeout)
async fn wait_for_tcp(port: u16, timeout_secs: u64) -> Result<(), String> {
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(timeout_secs);

    loop {
        if TcpStream::connect(("127.0.0.1", port)).await.is_ok() {
            return Ok(());
        }

        if start.elapsed() > timeout {
            return Err(format!(
                "Server did not start within {}s (tcp port {})",
                timeout_secs, port
            ));
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

/// Read a single line (terminated by `\n`) from the stream.
/// The server appends `\n` to every response for line-based framing.
async fn read_response_line<S>(stream: &mut S) -> Result<String, Box<dyn std::error::Error>>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    let n = reader.read_line(&mut line).await?;
    if n == 0 {
        return Err("Unexpected EOF while reading response".into());
    }
    // Strip the trailing newline
    line.pop(); // remove \n
    Ok(line)
}

/// Simple JSON-RPC 2.0 client for E2E testing, transport-agnostic.
struct E2EClient<S> {
    stream: S,
    request_id: u64,
}

impl E2EClient<UnixStream> {
    async fn connect_unix(socket_path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let stream = UnixStream::connect(socket_path).await?;
        Ok(Self {
            stream,
            request_id: 0,
        })
    }
}

impl E2EClient<TcpStream> {
    async fn connect_tcp(port: u16) -> Result<Self, Box<dyn std::error::Error>> {
        let stream = TcpStream::connect(("127.0.0.1", port)).await?;
        Ok(Self {
            stream,
            request_id: 0,
        })
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin> E2EClient<S> {
    /// Send a JSON-RPC request and parse the response.
    /// Reads a single `\n`-terminated line since the server uses line-based framing.
    async fn call(
        &mut self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        self.request_id += 1;
        let id = self.request_id;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": id
        });

        let request_str = serde_json::to_string(&request)?;
        // Line-framed protocol: requests must be newline-terminated so the
        // server's LinesCodec can emit them without waiting for EOF.
        self.stream.write_all(request_str.as_bytes()).await?;
        self.stream.write_all(b"\n").await?;
        self.stream.flush().await?;

        // Read a single \n-terminated line
        let response_str = read_response_line(&mut self.stream).await?;

        let response: serde_json::Value = serde_json::from_str(&response_str)?;
        Ok(response)
    }

    /// Call and assert no error in response
    async fn call_ok(
        &mut self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let response = self.call(method, params).await?;
        assert!(
            response.get("error").is_none() || response["error"].is_null(),
            "Expected success but got error: {}",
            response["error"]
        );
        assert!(response.get("result").is_some(), "Missing result field");
        Ok(response["result"].clone())
    }
}

// ============================================================================
// Unix socket transport tests
// ============================================================================

/// Test 1: Server starts and accepts connections
#[tokio::test]
#[ignore] // Run with: cargo test --test e2e_server_tests -- --ignored
async fn e2e_server_starts_and_accepts_connections() {
    let socket_path = unique_socket_path("starts");
    let server = start_server(&socket_path, None);

    let result = wait_for_server(&socket_path, server_startup_timeout_secs()).await;

    stop_server(server, &socket_path);

    assert!(result.is_ok(), "Server should start: {:?}", result);
}

/// Test 2: Server responds to valid JSON-RPC requests
#[tokio::test]
#[ignore]
async fn e2e_server_responds_to_port_list() {
    let socket_path = unique_socket_path("port_list");
    let server = start_server(&socket_path, None);

    if wait_for_server(&socket_path, server_startup_timeout_secs())
        .await
        .is_err()
    {
        stop_server(server, &socket_path);
        panic!("Server did not start in time");
    }

    let mut client = E2EClient::connect_unix(&socket_path)
        .await
        .expect("should connect");

    let result = client.call_ok("port_list", serde_json::json!({})).await;

    stop_server(server, &socket_path);

    let result = result.expect("port_list should succeed");
    assert!(
        result.get("ports").is_some(),
        "Response should have 'ports'"
    );
}

/// Test 3: Server responds to server_stats
#[tokio::test]
#[ignore]
async fn e2e_server_stats_returns_connection_info() {
    let socket_path = unique_socket_path("stats");
    let server = start_server(&socket_path, None);

    if wait_for_server(&socket_path, server_startup_timeout_secs())
        .await
        .is_err()
    {
        stop_server(server, &socket_path);
        panic!("Server did not start in time");
    }

    let mut client = E2EClient::connect_unix(&socket_path)
        .await
        .expect("should connect");

    let result = client.call_ok("server_stats", serde_json::json!({})).await;

    stop_server(server, &socket_path);

    let result = result.expect("server_stats should succeed");
    assert!(result.get("connections").is_some());
    assert!(result.get("max_connections").is_some());
}

/// Test 4: Server responds to script_list
#[tokio::test]
#[ignore]
async fn e2e_script_list_returns_scripts() {
    let socket_path = unique_socket_path("script_list");
    let server = start_server(&socket_path, None);

    if wait_for_server(&socket_path, server_startup_timeout_secs())
        .await
        .is_err()
    {
        stop_server(server, &socket_path);
        panic!("Server did not start in time");
    }

    let mut client = E2EClient::connect_unix(&socket_path)
        .await
        .expect("should connect");

    let result = client.call_ok("script_list", serde_json::json!({})).await;

    stop_server(server, &socket_path);

    let result = result.expect("script_list should succeed");
    assert!(result.get("scripts").is_some());
}

/// Test 5: Server returns error for invalid JSON
#[tokio::test]
#[ignore]
async fn e2e_server_returns_error_for_invalid_json() {
    let socket_path = unique_socket_path("invalid_json");
    let server = start_server(&socket_path, None);

    if wait_for_server(&socket_path, server_startup_timeout_secs())
        .await
        .is_err()
    {
        stop_server(server, &socket_path);
        panic!("Server did not start in time");
    }

    // Send raw malformed JSON directly via socket
    let mut stream = UnixStream::connect(&socket_path)
        .await
        .expect("should connect");

    stream.write_all(b"{{{invalid json}}\n").await.unwrap();
    stream.flush().await.unwrap();

    let response_str = read_response_line(&mut stream).await.unwrap();

    stop_server(server, &socket_path);

    let response: serde_json::Value =
        serde_json::from_str(&response_str).expect("valid JSON response");
    assert!(
        response.get("error").is_some() && !response["error"].is_null(),
        "Should return error for invalid JSON"
    );
    assert_eq!(
        response["error"]["code"].as_i64(),
        Some(-32700),
        "Should be parse error code"
    );
}

/// Test 6: Multiple sequential RPC calls work correctly
#[tokio::test]
#[ignore]
async fn e2e_multiple_sequential_calls() {
    let socket_path = unique_socket_path("multi_call");
    let server = start_server(&socket_path, None);

    if wait_for_server(&socket_path, server_startup_timeout_secs())
        .await
        .is_err()
    {
        stop_server(server, &socket_path);
        panic!("Server did not start in time");
    }

    let mut client = E2EClient::connect_unix(&socket_path)
        .await
        .expect("should connect");

    // Call 1: port_list
    let r1 = client.call_ok("port_list", serde_json::json!({})).await;
    assert!(r1.is_ok());

    // Call 2: server_stats
    let r2 = client.call_ok("server_stats", serde_json::json!({})).await;
    assert!(r2.is_ok());

    // Call 3: script_list
    let r3 = client.call_ok("script_list", serde_json::json!({})).await;
    assert!(r3.is_ok());

    // Call 4: connection_list
    let r4 = client
        .call_ok("connection_list", serde_json::json!({}))
        .await;
    assert!(r4.is_ok());

    stop_server(server, &socket_path);
}

/// Test 7: Server handles connection_list with no active connections
#[tokio::test]
#[ignore]
async fn e2e_connection_list_empty() {
    let socket_path = unique_socket_path("conn_list");
    let server = start_server(&socket_path, None);

    if wait_for_server(&socket_path, server_startup_timeout_secs())
        .await
        .is_err()
    {
        stop_server(server, &socket_path);
        panic!("Server did not start in time");
    }

    let mut client = E2EClient::connect_unix(&socket_path)
        .await
        .expect("should connect");

    let result = client
        .call_ok("connection_list", serde_json::json!({}))
        .await;

    stop_server(server, &socket_path);

    let result = result.expect("connection_list should succeed");
    let connections = result["connections"].as_array().unwrap();
    assert!(
        connections.is_empty(),
        "Connection list should be empty initially"
    );
}

/// Test 8: Server handles method not found error
#[tokio::test]
#[ignore]
async fn e2e_method_not_found_error() {
    let socket_path = unique_socket_path("method_not_found");
    let server = start_server(&socket_path, None);

    if wait_for_server(&socket_path, server_startup_timeout_secs())
        .await
        .is_err()
    {
        stop_server(server, &socket_path);
        panic!("Server did not start in time");
    }

    let mut client = E2EClient::connect_unix(&socket_path)
        .await
        .expect("should connect");

    let response = client
        .call("nonexistent_method", serde_json::json!({}))
        .await;

    stop_server(server, &socket_path);

    let response = response.expect("should get response");
    let error = response.get("error");
    assert!(
        error.is_some() && !error.unwrap().is_null(),
        "Should have error"
    );
    assert_eq!(
        response["error"]["code"].as_i64(),
        Some(-32601),
        "Should be method not found error code"
    );
}

/// Test 9: Server handles JSON-RPC version validation
#[tokio::test]
#[ignore]
async fn e2e_invalid_jsonrpc_version() {
    let socket_path = unique_socket_path("bad_version");
    let server = start_server(&socket_path, None);

    if wait_for_server(&socket_path, server_startup_timeout_secs())
        .await
        .is_err()
    {
        stop_server(server, &socket_path);
        panic!("Server did not start in time");
    }

    // Send raw request with wrong version
    let mut stream = UnixStream::connect(&socket_path)
        .await
        .expect("should connect");

    let bad_request = r#"{"jsonrpc":"1.0","method":"port_list","id":99}"#;
    stream.write_all(bad_request.as_bytes()).await.unwrap();
    stream.write_all(b"\n").await.unwrap();
    stream.flush().await.unwrap();

    let response_str = read_response_line(&mut stream).await.unwrap();

    stop_server(server, &socket_path);

    let response: serde_json::Value = serde_json::from_str(&response_str).expect("valid JSON");
    assert_eq!(
        response["error"]["code"].as_i64(),
        Some(-32600),
        "Should be invalid request error code"
    );
}

// ============================================================================
// TCP transport tests (LAN remote access)
// ============================================================================

/// Test T1: TCP daemon starts and accepts connections
#[tokio::test]
#[ignore]
async fn e2e_tcp_server_starts_and_accepts_connections() {
    let socket_path = unique_socket_path("tcp_starts");
    let port = next_tcp_port();
    let server = start_server(&socket_path, Some(port));

    let result = wait_for_tcp(port, server_startup_timeout_secs()).await;

    stop_server(server, &socket_path);

    assert!(result.is_ok(), "TCP server should start: {:?}", result);
}

/// Test T2: TCP server responds to port_list
#[tokio::test]
#[ignore]
async fn e2e_tcp_responds_to_port_list() {
    let socket_path = unique_socket_path("tcp_port_list");
    let port = next_tcp_port();
    let server = start_server(&socket_path, Some(port));

    if wait_for_tcp(port, server_startup_timeout_secs())
        .await
        .is_err()
    {
        stop_server(server, &socket_path);
        panic!("Server did not start in time");
    }

    let mut client = E2EClient::connect_tcp(port).await.expect("should connect");

    let result = client.call_ok("port_list", serde_json::json!({})).await;

    stop_server(server, &socket_path);

    let result = result.expect("port_list should succeed");
    assert!(
        result.get("ports").is_some(),
        "Response should have 'ports'"
    );
}

/// Test T3: TCP server returns error for invalid JSON
#[tokio::test]
#[ignore]
async fn e2e_tcp_returns_error_for_invalid_json() {
    let socket_path = unique_socket_path("tcp_invalid_json");
    let port = next_tcp_port();
    let server = start_server(&socket_path, Some(port));

    if wait_for_tcp(port, server_startup_timeout_secs())
        .await
        .is_err()
    {
        stop_server(server, &socket_path);
        panic!("Server did not start in time");
    }

    let mut stream = TcpStream::connect(("127.0.0.1", port))
        .await
        .expect("should connect");

    stream.write_all(b"{{{invalid json}}\n").await.unwrap();
    stream.flush().await.unwrap();

    let response_str = read_response_line(&mut stream).await.unwrap();

    stop_server(server, &socket_path);

    let response: serde_json::Value =
        serde_json::from_str(&response_str).expect("valid JSON response");
    assert_eq!(
        response["error"]["code"].as_i64(),
        Some(-32700),
        "Should be parse error code"
    );
}

/// Test T4: TCP server handles multiple sequential calls
#[tokio::test]
#[ignore]
async fn e2e_tcp_multiple_sequential_calls() {
    let socket_path = unique_socket_path("tcp_multi");
    let port = next_tcp_port();
    let server = start_server(&socket_path, Some(port));

    if wait_for_tcp(port, server_startup_timeout_secs())
        .await
        .is_err()
    {
        stop_server(server, &socket_path);
        panic!("Server did not start in time");
    }

    let mut client = E2EClient::connect_tcp(port).await.expect("should connect");

    assert!(client
        .call_ok("port_list", serde_json::json!({}))
        .await
        .is_ok());
    assert!(client
        .call_ok("server_stats", serde_json::json!({}))
        .await
        .is_ok());
    assert!(client
        .call_ok("connection_list", serde_json::json!({}))
        .await
        .is_ok());

    stop_server(server, &socket_path);
}

/// Test T5: Two concurrent TCP clients both receive valid responses
#[tokio::test]
#[ignore]
async fn e2e_tcp_two_concurrent_clients() {
    let socket_path = unique_socket_path("tcp_two");
    let port = next_tcp_port();
    let server = start_server(&socket_path, Some(port));

    if wait_for_tcp(port, server_startup_timeout_secs())
        .await
        .is_err()
    {
        stop_server(server, &socket_path);
        panic!("Server did not start in time");
    }

    let mut client1 = E2EClient::connect_tcp(port)
        .await
        .expect("client1 connects");
    let mut client2 = E2EClient::connect_tcp(port)
        .await
        .expect("client2 connects");

    let (r1, r2) = tokio::join!(
        client1.call_ok("port_list", serde_json::json!({})),
        client2.call_ok("port_list", serde_json::json!({}))
    );

    stop_server(server, &socket_path);

    assert!(r1.expect("client1 port_list").get("ports").is_some());
    assert!(r2.expect("client2 port_list").get("ports").is_some());
}

/// Test T6: Large frame (>8KB, beyond LinesCodec's default) round-trips over TCP
#[tokio::test]
#[ignore]
async fn e2e_tcp_large_frame_round_trip() {
    let socket_path = unique_socket_path("tcp_big");
    let port = next_tcp_port();
    let server = start_server(&socket_path, Some(port));

    if wait_for_tcp(port, server_startup_timeout_secs())
        .await
        .is_err()
    {
        stop_server(server, &socket_path);
        panic!("Server did not start in time");
    }

    let mut client = E2EClient::connect_tcp(port).await.expect("should connect");

    // ~100KB hex payload in a port_send request (the default 8KB LinesCodec
    // limit would kill this connection before a response; the 1MB limit
    // lets it through and the server answers with "Connection not found").
    let big_data = format!("hex:{}", "ab".repeat(50_000));
    let response = client
        .call(
            "port_send",
            serde_json::json!({
                "connection_id": "does-not-exist",
                "data": big_data,
            }),
        )
        .await
        .expect("server should respond to a ~100KB frame");

    stop_server(server, &socket_path);

    assert_eq!(
        response["error"]["code"].as_i64(),
        Some(-32603),
        "Should get a connection-not-found error proving the large frame was parsed"
    );
}

/// Test T8: Library `RemoteRpcClient` (used by the GUI) talks to a TCP daemon
#[tokio::test]
#[ignore]
async fn e2e_tcp_library_client() {
    use serial_cli::server::client::RemoteRpcClient;

    let socket_path = unique_socket_path("tcp_client");
    let port = next_tcp_port();
    let server = start_server(&socket_path, Some(port));

    if wait_for_tcp(port, server_startup_timeout_secs())
        .await
        .is_err()
    {
        stop_server(server, &socket_path);
        panic!("Server did not start in time");
    }

    let client = RemoteRpcClient::new("127.0.0.1", port).expect("client resolves");

    let stats = client.server_stats().await.expect("server_stats");
    assert!(stats.max_connections >= 1);
    assert!(stats.total_requests >= 1);

    let ports = client.port_list().await.expect("port_list");
    assert!(ports.len() >= 0);

    let conns = client.connection_list().await.expect("connection_list");
    assert!(conns.is_empty(), "no connections yet");

    // Error path: unknown method surfaces as an error with the server message
    let err = client
        .call::<serde_json::Value>("nonexistent_method", serde_json::json!({}))
        .await
        .err()
        .expect("unknown method should error");

    stop_server(server, &socket_path);

    assert!(err.to_string().contains("Method not found"), "got: {}", err);
}

/// Test T7: `server call --remote <ip:port>` CLI path reaches a remote daemon
#[tokio::test]
#[ignore]
async fn e2e_tcp_cli_remote_call() {
    let socket_path = unique_socket_path("tcp_cli");
    let port = next_tcp_port();
    let server = start_server(&socket_path, Some(port));

    if wait_for_tcp(port, server_startup_timeout_secs())
        .await
        .is_err()
    {
        stop_server(server, &socket_path);
        panic!("Server did not start in time");
    }

    let binary_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/debug/serial-cli");
    let output = Command::new(&binary_path)
        .args([
            "server",
            "call",
            "--remote",
            &format!("127.0.0.1:{}", port),
            "port_list",
            "{}",
        ])
        .output()
        .expect("server call should run");

    stop_server(server, &socket_path);

    assert!(
        output.status.success(),
        "server call --remote should exit 0: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("\"jsonrpc\":\"2.0\"") && stdout.contains("\"ports\""),
        "Response should contain the JSON-RPC port_list result, got: {}",
        stdout
    );
}
