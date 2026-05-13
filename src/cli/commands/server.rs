//! Server command handler
//!
//! Handles `serial-cli server start|stop|status|call` commands.

#[cfg(unix)]
use crate::cli::types::ServerCommand;
use crate::error::{Result, SerialError};
#[cfg(unix)]
use crate::server::listener::{run_socket_server, spawn_idle_cleanup_task};
#[cfg(unix)]
use crate::server::session::{ServerSessionManager, ServerSessionMeta};
#[cfg(unix)]
use crate::server::state::{default_log_path, default_socket_path, ServerConfig, ServerState};
use std::io;
use std::path::PathBuf;
use std::process::Command;

/// Dispatch a [`ServerCommand`] to the appropriate handler.
#[cfg(unix)]
pub async fn handle_server_command(cmd: ServerCommand, _json_output: bool) -> Result<()> {
    match cmd {
        ServerCommand::Start {
            socket_path,
            port,
            log,
            max_connections,
        } => {
            start_server(socket_path, port, log, max_connections).await?;
        }
        ServerCommand::Stop => {
            stop_server().await?;
        }
        ServerCommand::Status => {
            show_server_status().await?;
        }
        ServerCommand::Daemon { socket_path, port } => {
            // This is the entry point for the daemon process
            run_daemon(socket_path, port).await?;
        }
        ServerCommand::Call {
            method,
            args,
            stdin,
        } => {
            call_rpc(method, args, stdin).await?;
        }
    }
    Ok(())
}

/// Server mode is not supported on Windows
#[cfg(windows)]
pub async fn handle_server_command(
    _cmd: crate::cli::types::ServerCommand,
    _json_output: bool,
) -> Result<()> {
    Err(SerialError::Io(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Server mode is not supported on Windows. Unix sockets are required for server mode.",
    )))
}

/// Start the server daemon
#[cfg(unix)]
async fn start_server(
    socket_path: Option<String>,
    port: Option<u16>,
    log: Option<String>,
    max_connections: usize,
) -> Result<()> {
    // Check if server is already running
    if let Ok(Some(meta)) = ServerSessionManager::load_session() {
        if ServerSessionManager::is_process_running(meta.pid) {
            println!("✗ Server is already running (PID: {})", meta.pid);
            println!("  Socket: {}", meta.socket_path.display());
            println!("  Use 'server stop' to stop the server first.");
            return Err(SerialError::Io(io::Error::new(
                io::ErrorKind::AddrInUse,
                "Server already running",
            )));
        } else {
            // Stale session - clean up
            ServerSessionManager::clear_session()?;
        }
    }

    // Prepare configuration
    let socket_path = socket_path
        .map(PathBuf::from)
        .unwrap_or_else(default_socket_path);

    let log_path = log.map(PathBuf::from).unwrap_or_else(default_log_path);

    // Spawn daemon process
    let current_exe = std::env::current_exe()?;
    let mut args = vec![
        "server".to_string(),
        "daemon".to_string(),
        "--socket-path".to_string(),
        socket_path.to_str().unwrap().to_string(),
    ];

    if let Some(p) = port {
        args.push("--port".to_string());
        args.push(p.to_string());
    }

    let mut child = Command::new(&current_exe)
        .args(&args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    // Give the daemon a moment to start
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Check if it's still running
    match child.try_wait() {
        Ok(Some(status)) => {
            println!("✗ Server failed to start");
            println!("  Exit status: {}", status);
            return Err(SerialError::Io(io::Error::other("Server process exited")));
        }
        Ok(None) => {
            // Still running - success
            let pid = child.id();
            println!("✓ Server started successfully");
            println!("  PID: {}", pid);
            println!("  Socket: {}", socket_path.display());
            println!("  Log: {}", log_path.display());
            println!("  Max connections: {}", max_connections);
            println!();
            println!("Use 'server status' to check server status.");
            println!("Use 'server call <method> <args>' to send RPC requests.");
        }
        Err(e) => {
            println!("✗ Failed to check server status: {}", e);
            return Err(SerialError::Io(e));
        }
    }

    Ok(())
}

/// Stop the server daemon
#[cfg(unix)]
async fn stop_server() -> Result<()> {
    // Load session
    let meta = ServerSessionManager::load_session()?.ok_or_else(|| {
        SerialError::Io(io::Error::new(
            io::ErrorKind::NotFound,
            "Server is not running",
        ))
    })?;

    // Check if process is running
    if !ServerSessionManager::is_process_running(meta.pid) {
        println!("✗ Server is not running (stale session)");
        ServerSessionManager::clear_session()?;
        return Ok(());
    }

    // Stop the process
    ServerSessionManager::stop_process(meta.pid)?;

    // Wait a bit for graceful shutdown
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Check if it stopped
    if !ServerSessionManager::is_process_running(meta.pid) {
        println!("✓ Server stopped successfully");
        ServerSessionManager::clear_session()?;
    } else {
        println!("⚠ Server did not stop gracefully (PID: {})", meta.pid);
        println!("  You may need to manually kill the process:");
        println!("  kill {}", meta.pid);
    }

    Ok(())
}

/// Show server status
#[cfg(unix)]
async fn show_server_status() -> Result<()> {
    match ServerSessionManager::load_session()? {
        Some(meta) => {
            let running = ServerSessionManager::is_process_running(meta.pid);

            println!("Server Status:");
            println!();
            println!("  PID: {}", meta.pid);
            println!(
                "  Status: {}",
                if running {
                    "Running ✓"
                } else {
                    "Stopped ✗"
                }
            );
            println!("  Socket: {}", meta.socket_path.display());
            println!("  TCP Port: {:?}", meta.tcp_port);
            println!("  Log: {}", meta.log_path.display());
            println!("  Max Connections: {}", meta.max_connections);

            if let Ok(started) = std::time::UNIX_EPOCH.elapsed() {
                let uptime = started.as_secs() - meta.started_at;
                let mins = uptime / 60;
                let secs = uptime % 60;
                println!("  Uptime: {}m {}s", mins, secs);
            }

            if !running {
                println!();
                println!("⚠ Server process is not running (stale session)");
                println!("  Use 'server stop' to clean up");
            }
        }
        None => {
            println!("Server Status: Not running");
            println!();
            println!("Use 'server start' to start the server daemon.");
        }
    }

    Ok(())
}

/// Run the daemon process (entry point for daemon)
#[cfg(unix)]
async fn run_daemon(socket_path: Option<String>, port: Option<u16>) -> Result<()> {
    let socket_path = socket_path
        .map(PathBuf::from)
        .unwrap_or_else(default_socket_path);

    // Create server state
    let config = ServerConfig {
        socket_path: Some(socket_path.clone()),
        tcp_port: port,
        max_connections: 10,
        log_path: default_log_path(),
        idle_timeout_secs: 300,
    };

    let state = ServerState::new(config).await;

    // Save session
    let current_pid = std::process::id();
    let meta = ServerSessionMeta {
        pid: current_pid,
        socket_path: socket_path.clone(),
        tcp_port: port,
        started_at: ServerSessionManager::current_timestamp(),
        log_path: PathBuf::from("/tmp/serial-cli-server.log"),
        max_connections: 10,
    };
    ServerSessionManager::save_session(&meta)?;

    // Spawn idle connection cleanup task
    spawn_idle_cleanup_task(state.clone());

    // Run server (blocking)
    run_socket_server(state, socket_path).await?;

    Ok(())
}

/// Call RPC method over Unix socket
#[cfg(unix)]
async fn call_rpc(method: String, args: String, use_stdin: bool) -> Result<()> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixStream;

    // Get socket path
    let socket_path = ServerSessionManager::load_session()?
        .ok_or_else(|| {
            SerialError::Io(io::Error::new(
                io::ErrorKind::NotConnected,
                "Server is not running. Use 'server start' first.",
            ))
        })?
        .socket_path;

    // Read args from stdin if requested
    let args_str = if use_stdin {
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer)?;
        buffer.trim().to_string()
    } else {
        args
    };

    // Build JSON-RPC request
    let params: serde_json::Value =
        serde_json::from_str(&args_str).unwrap_or(serde_json::Value::Null);

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    });

    let request_str = serde_json::to_string(&request).map_err(|e| {
        SerialError::Io(io::Error::other(format!(
            "Failed to serialize request: {}",
            e
        )))
    })?;

    // Connect to socket
    let mut stream = UnixStream::connect(&socket_path).await.map_err(|e| {
        SerialError::Io(io::Error::new(
            io::ErrorKind::NotConnected,
            format!("Failed to connect to server: {}", e),
        ))
    })?;

    // Send request
    stream
        .write_all(request_str.as_bytes())
        .await
        .map_err(|e| SerialError::Io(io::Error::other(format!("Failed to send request: {}", e))))?;

    // Shutdown write side
    stream.shutdown().await.map_err(|e| {
        SerialError::Io(io::Error::other(format!(
            "Failed to shutdown stream: {}",
            e
        )))
    })?;

    // Read response
    let mut response_buffer = vec![0u8; 8192];
    let n = stream.read(&mut response_buffer).await.map_err(|e| {
        SerialError::Io(io::Error::other(format!("Failed to read response: {}", e)))
    })?;
    response_buffer.truncate(n);

    let response_str = String::from_utf8_lossy(&response_buffer);
    println!("{}", response_str);

    Ok(())
}
