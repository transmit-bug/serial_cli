//! Unix socket listener for server mode
//!
//! Accepts client connections and dispatches requests to the RPC handler.

use crate::error::Result;
use crate::server::rpc::RpcDispatcher;
use crate::server::state::ServerState;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixListener;
use tracing::{error, info};

/// Run the Unix socket server
pub async fn run_socket_server(state: ServerState, socket_path: PathBuf) -> Result<()> {
    // Remove existing socket file
    if socket_path.exists() {
        std::fs::remove_file(&socket_path)?;
    }

    let listener = UnixListener::bind(&socket_path)?;
    // Wrap RpcDispatcher in Arc so it can be cloned
    let rpc = Arc::new(RpcDispatcher::new(state));

    info!("✓ Server listening on: {}", socket_path.display());

    loop {
        match listener.accept().await {
            Ok((mut stream, addr)) => {
                info!("New connection from: {:?}", addr);

                let rpc = Arc::clone(&rpc);

                tokio::spawn(async move {
                    let mut buf = vec![0u8; 8192];

                    loop {
                        match stream.read(&mut buf).await {
                            Ok(0) => {
                                info!("Client disconnected");
                                break;
                            }
                            Ok(n) => {
                                let request = String::from_utf8_lossy(&buf[..n]);
                                let response: String = rpc.handle_request(&request).await;

                                if let Err(e) = stream.write_all(response.as_bytes()).await {
                                    error!("Failed to send response: {}", e);
                                    break;
                                }
                            }
                            Err(e) => {
                                error!("Failed to read from socket: {}", e);
                                break;
                            }
                        }
                    }
                });
            }
            Err(e) => {
                error!("Failed to accept connection: {}", e);
            }
        }
    }
}

/// Spawn a background cleanup task for idle connections
pub fn spawn_idle_cleanup_task(state: ServerState) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));

        loop {
            interval.tick().await;

            let removed = state.cleanup_idle_connections().await;
            if !removed.is_empty() {
                info!("Cleaned up {} idle connections", removed.len());
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socket_path_creation() {
        let path = PathBuf::from("/tmp/test-serial-cli.sock");
        assert_eq!(path.extension(), None);
        assert!(path.to_str().unwrap().ends_with(".sock"));
    }
}
