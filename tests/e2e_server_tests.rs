//! End-to-end tests for Server Mode
//!
//! These tests spawn the actual server daemon directly (bypassing the session
//! file) and connect via Unix domain sockets to validate the full client-server
//! lifecycle: start → RPC calls → graceful shutdown.
//!
//! Run with: `cargo test --test e2e_server_tests -- --ignored`

#![cfg(unix)]

use std::path::PathBuf;
use std::process::{Child, Command};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

/// Unique socket path per test run to avoid conflicts
fn unique_socket_path(test_name: &str) -> PathBuf {
    let pid = std::process::id();
    PathBuf::from(format!("/tmp/serial-cli-e2e-{}-{}.sock", test_name, pid))
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

/// Start the server daemon directly (bypassing session file) as a background process.
///
/// We use `server daemon` (internal entry point) instead of `server start`
/// (CLI wrapper) to avoid session file competition between parallel tests.
/// The returned Child is the actual daemon process, so SIGTERM stops it cleanly.
fn start_server(socket_path: &PathBuf) -> Child {
    // Clean up existing socket
    if socket_path.exists() {
        let _ = std::fs::remove_file(socket_path);
    }

    ensure_server_binary();

    let binary_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/debug/serial-cli");

    Command::new(&binary_path)
        .args([
            "server",
            "daemon",
            "--socket-path",
            socket_path.to_str().unwrap(),
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("daemon should start")
}

/// Stop the server daemon and clean up the socket file.
///
/// Uses SIGKILL because the daemon's Tokio event loop doesn't handle SIGTERM
/// by default (no signal handler installed).
fn stop_server(mut child: Child, socket_path: &PathBuf) {
    // Force kill — daemon ignores SIGTERM (Tokio event loop)
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

/// Wait for the server socket to appear (up to timeout seconds)
async fn wait_for_server(socket_path: &PathBuf, timeout_secs: u64) -> Result<(), String> {
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(timeout_secs);

    loop {
        if socket_path.exists() {
            // Socket exists, try to connect once to confirm it's accepting
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

/// Read a single line (terminated by `\n`) from the stream.
/// The server now appends `\n` to every response for line-based framing.
async fn read_response_line(stream: &mut UnixStream) -> Result<String, Box<dyn std::error::Error>> {
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

/// Simple JSON-RPC 2.0 client for E2E testing
struct E2EClient {
    stream: UnixStream,
    request_id: u64,
}

impl E2EClient {
    async fn connect(socket_path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let stream = UnixStream::connect(socket_path).await?;
        Ok(Self {
            stream,
            request_id: 0,
        })
    }

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
        self.stream.write_all(request_str.as_bytes()).await?;
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
// E2E Test Cases
// ============================================================================

/// Test 1: Server starts and accepts connections
#[tokio::test]
#[ignore] // Run with: cargo test --test e2e_server_tests -- --ignored
async fn e2e_server_starts_and_accepts_connections() {
    let socket_path = unique_socket_path("starts");
    let server = start_server(&socket_path);

    let result = wait_for_server(&socket_path, server_startup_timeout_secs()).await;

    stop_server(server, &socket_path);

    assert!(result.is_ok(), "Server should start: {:?}", result);
}

/// Test 2: Server responds to valid JSON-RPC requests
#[tokio::test]
#[ignore]
async fn e2e_server_responds_to_port_list() {
    let socket_path = unique_socket_path("port_list");
    let server = start_server(&socket_path);

    if wait_for_server(&socket_path, server_startup_timeout_secs())
        .await
        .is_err()
    {
        stop_server(server, &socket_path);
        panic!("Server did not start in time");
    }

    let mut client = E2EClient::connect(&socket_path)
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
    let server = start_server(&socket_path);

    if wait_for_server(&socket_path, server_startup_timeout_secs())
        .await
        .is_err()
    {
        stop_server(server, &socket_path);
        panic!("Server did not start in time");
    }

    let mut client = E2EClient::connect(&socket_path)
        .await
        .expect("should connect");

    let result = client.call_ok("server_stats", serde_json::json!({})).await;

    stop_server(server, &socket_path);

    let result = result.expect("server_stats should succeed");
    assert!(result.get("connections").is_some());
    assert!(result.get("max_connections").is_some());
}

/// Test 4: Server responds to protocol_list
#[tokio::test]
#[ignore]
async fn e2e_protocol_list_returns_empty_list() {
    let socket_path = unique_socket_path("proto_list");
    let server = start_server(&socket_path);

    if wait_for_server(&socket_path, server_startup_timeout_secs())
        .await
        .is_err()
    {
        stop_server(server, &socket_path);
        panic!("Server did not start in time");
    }

    let mut client = E2EClient::connect(&socket_path)
        .await
        .expect("should connect");

    let result = client.call_ok("protocol_list", serde_json::json!({})).await;

    stop_server(server, &socket_path);

    let result = result.expect("protocol_list should succeed");
    assert!(result.get("protocols").is_some());
}

/// Test 5: Server returns error for invalid JSON
#[tokio::test]
#[ignore]
async fn e2e_server_returns_error_for_invalid_json() {
    let socket_path = unique_socket_path("invalid_json");
    let server = start_server(&socket_path);

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

    stream.write_all(b"{{{invalid json}}").await.unwrap();
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
    let server = start_server(&socket_path);

    if wait_for_server(&socket_path, server_startup_timeout_secs())
        .await
        .is_err()
    {
        stop_server(server, &socket_path);
        panic!("Server did not start in time");
    }

    let mut client = E2EClient::connect(&socket_path)
        .await
        .expect("should connect");

    // Call 1: port_list
    let r1 = client.call_ok("port_list", serde_json::json!({})).await;
    assert!(r1.is_ok());

    // Call 2: server_stats
    let r2 = client.call_ok("server_stats", serde_json::json!({})).await;
    assert!(r2.is_ok());

    // Call 3: protocol_list
    let r3 = client.call_ok("protocol_list", serde_json::json!({})).await;
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
    let server = start_server(&socket_path);

    if wait_for_server(&socket_path, server_startup_timeout_secs())
        .await
        .is_err()
    {
        stop_server(server, &socket_path);
        panic!("Server did not start in time");
    }

    let mut client = E2EClient::connect(&socket_path)
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
    let server = start_server(&socket_path);

    if wait_for_server(&socket_path, server_startup_timeout_secs())
        .await
        .is_err()
    {
        stop_server(server, &socket_path);
        panic!("Server did not start in time");
    }

    let mut client = E2EClient::connect(&socket_path)
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
    let server = start_server(&socket_path);

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
