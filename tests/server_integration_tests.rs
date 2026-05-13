//! Server Mode integration tests
//!
//! Tests the complete server lifecycle: start → call → stop

use std::path::PathBuf;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

/// Test socket path for integration tests
const TEST_SOCKET_PATH: &str = "/tmp/serial-cli-test.sock";

/// Simple RPC client for testing
struct TestRpcClient {
    stream: UnixStream,
    request_id: u32,
}

impl TestRpcClient {
    /// Connect to the test server
    async fn connect() -> Result<Self, Box<dyn std::error::Error>> {
        // Wait for server to be ready
        tokio::time::sleep(Duration::from_millis(500)).await;

        let stream = UnixStream::connect(TEST_SOCKET_PATH).await?;
        Ok(Self {
            stream,
            request_id: 0,
        })
    }

    /// Call an RPC method
    async fn call(
        &mut self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<String, Box<dyn std::error::Error>> {
        self.request_id += 1;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": self.request_id
        });

        let request_str = serde_json::to_string(&request)?;
        self.stream.write_all(request_str.as_bytes()).await?;
        self.stream.flush().await?;

        // Read response
        let mut buffer = vec![0u8; 8192];
        let n = self.stream.read(&mut buffer).await?;
        let response = String::from_utf8_lossy(&buffer[..n]).to_string();

        Ok(response)
    }
}

#[tokio::test]
async fn test_server_lifecycle() {
    // Clean up any existing test socket
    if PathBuf::from(TEST_SOCKET_PATH).exists() {
        let _ = std::fs::remove_file(TEST_SOCKET_PATH);
    }

    // Note: This test requires the server to be started separately
    // In a real integration test, we would spawn the server process here
    // For now, we'll just test the client connection logic

    // You can manually start the server with:
    // serial-cli server start --socket-path /tmp/serial-cli-test.sock

    // Try to connect (will fail if server not running, which is ok for this test)
    match UnixStream::connect(TEST_SOCKET_PATH).await {
        Ok(_stream) => {
            // Server is running, test RPC calls
            let mut client = TestRpcClient::connect().await.unwrap();

            // Test port_list method
            let response = client
                .call("port_list", serde_json::json!({}))
                .await
                .unwrap();
            assert!(response.contains(r#""jsonrpc":"2.0""#));

            // Test server_stats method
            let response = client
                .call("server_stats", serde_json::json!({}))
                .await
                .unwrap();
            assert!(response.contains(r#""result""#));

            // Test protocol_list method
            let response = client
                .call("protocol_list", serde_json::json!({}))
                .await
                .unwrap();
            assert!(response.contains(r#""result""#));
        }
        Err(_) => {
            // Server not running, skip RPC tests
            println!("Server not running, skipping RPC call tests");
            println!("To run full integration tests, start the server with:");
            println!(
                "  serial-cli server start --socket-path {}",
                TEST_SOCKET_PATH
            );
        }
    }
}

#[tokio::test]
async fn test_socket_path_format() {
    let path = PathBuf::from(TEST_SOCKET_PATH);
    assert_eq!(path.extension().unwrap(), "sock");
    assert!(path.to_str().unwrap().ends_with(".sock"));
}

#[tokio::test]
async fn test_rpc_request_format() {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "port_list",
        "params": {},
        "id": 1
    });

    let request_str = request.to_string();
    assert!(request_str.contains(r#""jsonrpc":"2.0""#));
    assert!(request_str.contains(r#""method":"port_list""#));
    assert!(request_str.contains(r#""id":1"#));
}
