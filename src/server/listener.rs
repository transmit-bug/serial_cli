//! Unix socket listener for server mode
//!
//! Accepts client connections and dispatches requests to the RPC handler.
//! Uses `LinesCodec` for proper message framing and `CancellationToken`
//! for graceful shutdown.

use crate::error::Result;
use crate::server::rpc::RpcDispatcher;
use crate::server::state::ServerState;
use futures::SinkExt;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixListener;
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, LinesCodec};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

/// Run the Unix socket server with graceful shutdown support.
///
/// Uses `LinesCodec` for message framing: each JSON-RPC request/response
/// is terminated by `\n`. This handles partial reads and message batching.
///
/// The server stops when the provided `CancellationToken` is cancelled.
pub async fn run_socket_server(
    state: ServerState,
    socket_path: PathBuf,
    token: CancellationToken,
) -> Result<()> {
    // Remove existing socket file
    if socket_path.exists() {
        std::fs::remove_file(&socket_path)?;
    }

    let listener = UnixListener::bind(&socket_path)?;

    // Set socket permissions to owner-only (0o600) for security
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&socket_path)?.permissions();
        perms.set_mode(0o600);
        std::fs::set_permissions(&socket_path, perms)?;
    }

    let rpc = Arc::new(RpcDispatcher::new(state.clone()));

    info!("Server listening on: {}", socket_path.display());

    loop {
        tokio::select! {
            _ = token.cancelled() => {
                info!("Shutdown signal received, stopping accept loop");
                break;
            }
            result = listener.accept() => {
                match result {
                    Ok((stream, _addr)) => {
                        let rpc = Arc::clone(&rpc);
                        let state = state.clone();
                        let token = token.clone();

                        tokio::spawn(async move {
                            handle_connection(stream, rpc, state, token).await;
                        });
                    }
                    Err(e) => {
                        error!("Failed to accept connection: {}", e);
                    }
                }
            }
        }
    }

    // Cleanup after shutdown
    if socket_path.exists() {
        let _ = std::fs::remove_file(&socket_path);
    }
    info!("Server stopped, socket file removed");
    Ok(())
}

/// Handle a single client connection.
///
/// Uses `LinesCodec` to frame messages: each line (terminated by `\n`)
/// is one JSON-RPC request/response. Simultaneously listens for
/// broadcast data pushes from the port's IoLoop.
async fn handle_connection(
    stream: impl AsyncReadExt + AsyncWriteExt + Unpin,
    rpc: Arc<RpcDispatcher>,
    _state: ServerState,
    _conn_token: CancellationToken,
) {
    let mut framed = Framed::new(stream, LinesCodec::new());

    loop {
        tokio::select! {
            // Client request
            result = framed.next() => {
                match result {
                    Some(Ok(line)) => {
                        let response = rpc.handle_request(&line).await;
                        if let Err(e) = framed.send(response).await {
                            error!("Failed to send response: {}", e);
                            break;
                        }
                    }
                    Some(Err(e)) => {
                        error!("Failed to read from client: {}", e);
                        break;
                    }
                    None => {
                        info!("Client disconnected");
                        break;
                    }
                }
            }
            // Shutdown signal
            _ = _conn_token.cancelled() => {
                info!("Server shutting down, closing connection");
                break;
            }
        }
    }
}

/// Spawn a background cleanup task for idle connections.
/// Stops when the provided `CancellationToken` is cancelled.
pub fn spawn_idle_cleanup_task(state: ServerState, token: CancellationToken) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let removed = state.cleanup_idle_connections().await;
                    if !removed.is_empty() {
                        info!("Cleaned up {} idle connections", removed.len());
                    }
                }
                _ = token.cancelled() => {
                    info!("Idle cleanup task stopped");
                    break;
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socket_path_has_correct_extension() {
        let path = PathBuf::from("/tmp/test-serial-cli.sock");
        assert_eq!(path.extension().unwrap(), "sock");
    }
}
