//! Server command handler
//!
//! Handles `serial-cli server start|stop|status|call|daemon` commands.
//!
//! The Daemon serves JSON-RPC 2.0 over two transports:
//!
//! - **Unix socket** (Unix only): local access, `0600` permissions.
//! - **TCP** (cross-platform): LAN remote access. Enabled by default with
//!   `server start`, bindable via `--bind`, disableable via `--no-tcp`.

use crate::cli::types::ServerCommand;
use crate::error::{Result, SerialError};
#[cfg(unix)]
use crate::server::listener::run_socket_server;
use crate::server::listener::{run_tcp_server, spawn_idle_cleanup_task};
use crate::server::session::{ServerSessionManager, ServerSessionMeta};
use crate::server::state::{default_log_path, default_socket_path, ServerConfig, ServerState};
use std::io;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

/// Default TCP port for LAN remote access
pub const DEFAULT_TCP_PORT: u16 = 23333;
/// Default TCP bind address
pub const DEFAULT_BIND_ADDR: &str = "0.0.0.0";

/// Dispatch a [`ServerCommand`] to the appropriate handler.
pub async fn handle_server_command(cmd: ServerCommand, json_output: bool) -> Result<()> {
    match cmd {
        ServerCommand::Start {
            socket_path,
            port,
            bind,
            no_tcp,
            log,
            max_connections,
        } => {
            start_server(
                socket_path,
                if no_tcp {
                    None
                } else {
                    Some(port.unwrap_or(DEFAULT_TCP_PORT))
                },
                bind,
                log,
                max_connections,
                json_output,
            )
            .await?;
        }
        ServerCommand::Stop => {
            stop_server(json_output).await?;
        }
        ServerCommand::Status => {
            show_server_status(json_output).await?;
        }
        ServerCommand::Call {
            method,
            args,
            stdin,
            remote,
        } => {
            call_rpc(method, args, stdin, remote).await?;
        }
        ServerCommand::Daemon {
            socket_path,
            port,
            bind,
            log,
        } => {
            // Internal foreground entry point (e2e tests, Windows detached spawn)
            let socket_path = socket_path
                .map(PathBuf::from)
                .unwrap_or_else(default_socket_path);
            let log_path = log.map(PathBuf::from).unwrap_or_else(default_log_path);
            let bind_addr = parse_bind_addr(&bind);
            run_daemon(socket_path, port, bind_addr, log_path, 10, 300).await?;
        }
        ServerCommand::Service { service_command } => {
            println!("Server Auto-Start:");
            println!();
            super::service::handle_service_command(service_command)?;
        }
    }
    Ok(())
}

/// Start the server daemon.
///
/// On Unix this forks via the `daemonize` crate; on Windows it spawns a
/// detached background process running the internal `server daemon` entry.
/// Either way the parent returns after the child reports a live session.
async fn start_server(
    socket_path: Option<String>,
    tcp_port: Option<u16>,
    bind: Option<String>,
    log: Option<String>,
    max_connections: usize,
    json_output: bool,
) -> Result<()> {
    // Check if server is already running
    if let Ok(Some(meta)) = ServerSessionManager::load_session() {
        if ServerSessionManager::is_process_running(meta.pid) {
            if json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "ok": false,
                        "error": "Server already running",
                        "pid": meta.pid,
                        "socket": meta.socket_path.display().to_string(),
                    }))
                    .unwrap()
                );
            } else {
                println!("Server is already running (PID: {})", meta.pid);
                println!("  Socket: {}", meta.socket_path.display());
                println!("  Use 'server stop' to stop the server first.");
            }
            return Err(SerialError::Io(io::Error::new(
                io::ErrorKind::AddrInUse,
                "Server already running",
            )));
        } else {
            // Stale session - clean up
            ServerSessionManager::clear_session()?;
        }
    }

    // Prepare paths
    let socket_path = socket_path
        .map(PathBuf::from)
        .unwrap_or_else(default_socket_path);
    let log_path = log.map(PathBuf::from).unwrap_or_else(default_log_path);
    let bind_addr = parse_bind_addr(&bind);

    // Launch the daemon process (platform-specific)
    #[cfg(unix)]
    launch_unix_daemon(&socket_path, tcp_port, bind_addr, &log_path)?;
    #[cfg(windows)]
    launch_windows_daemon(&socket_path, tcp_port, bind_addr, &log_path)?;

    // Wait for the child to report a live session
    wait_for_daemon_session(&socket_path, max_connections).await?;

    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "ok": true,
                "pid": ServerSessionManager::load_session()?.unwrap().pid,
                "socket": socket_path.display().to_string(),
                "tcp": tcp_desc(tcp_port, bind_addr),
                "log": log_path.display().to_string(),
                "maxConnections": max_connections,
            }))
            .unwrap()
        );
    } else {
        println!("Server started successfully");
        println!(
            "  PID: {}",
            ServerSessionManager::load_session()?.unwrap().pid
        );
        println!("  Socket: {}", socket_path.display());
        println!("  TCP: {}", tcp_desc(tcp_port, bind_addr));
        println!("  Log: {}", log_path.display());
        println!("  Max connections: {}", max_connections);
        println!();
        println!("Use 'server status' to check server status.");
        println!("Use 'server call <method> <args>' to send RPC requests.");
        println!("Use 'server call --remote <ip:port> <method> <args>' for remote devices.");
    }
    Ok(())
}

/// Spawn the daemon as a detached background process (Unix).
///
/// Starts a fresh process via the internal `server daemon` entry point with
/// `setsid()`, so the child becomes its own session leader. This avoids the
/// tokio-after-fork corruption that breaks the old `daemonize` fork approach
/// (the runtime now initializes cleanly inside the child).
#[cfg(unix)]
fn launch_unix_daemon(
    socket_path: &PathBuf,
    tcp_port: Option<u16>,
    bind_addr: IpAddr,
    log_path: &PathBuf,
) -> Result<()> {
    use std::os::unix::process::CommandExt;
    use std::process::{Command, Stdio};

    let stdout_file = log_path.with_extension("out");
    let stderr_file = log_path.with_extension("err");

    let mut cmd = Command::new(std::env::current_exe()?);
    cmd.arg("server")
        .arg("daemon")
        .arg("--socket-path")
        .arg(socket_path)
        .arg("--log")
        .arg(log_path);
    if let Some(port) = tcp_port {
        cmd.arg("--port").arg(port.to_string());
        cmd.arg("--bind").arg(bind_addr.to_string());
    }
    // New session leader: survives terminal close, reparented to init on exit.
    unsafe {
        cmd.pre_exec(|| {
            libc::setsid();
            Ok(())
        });
    }
    cmd.stdin(Stdio::null())
        .stdout(Stdio::from(
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&stdout_file)?,
        ))
        .stderr(Stdio::from(
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&stderr_file)?,
        ))
        .spawn()?;
    Ok(())
}

/// Detached-process daemonization (Windows).
///
/// Spawns the same binary's internal `server daemon` entry as a background
/// process detached from the console, redirecting output to log files.
#[cfg(windows)]
fn launch_windows_daemon(
    socket_path: &PathBuf,
    tcp_port: Option<u16>,
    bind_addr: IpAddr,
    log_path: &PathBuf,
) -> Result<()> {
    use std::os::windows::process::CommandExt;
    use std::process::{Command, Stdio};

    const DETACHED_PROCESS: u32 = 0x00000008;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let stdout_file = log_path.with_extension("out");
    let stderr_file = log_path.with_extension("err");

    let mut cmd = Command::new(std::env::current_exe()?);
    cmd.arg("server")
        .arg("daemon")
        .arg("--socket-path")
        .arg(socket_path)
        .arg("--log")
        .arg(log_path);
    if let Some(port) = tcp_port {
        cmd.arg("--port").arg(port.to_string());
        cmd.arg("--bind").arg(bind_addr.to_string());
    }
    cmd.creation_flags(DETACHED_PROCESS | CREATE_NO_WINDOW)
        .stdin(Stdio::null())
        .stdout(Stdio::from(
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&stdout_file)?,
        ))
        .stderr(Stdio::from(
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&stderr_file)?,
        ))
        .spawn()?;
    Ok(())
}

/// Poll for the daemon's session file to appear and its process to be alive.
async fn wait_for_daemon_session(socket_path: &PathBuf, _max_connections: usize) -> Result<()> {
    let deadline = std::time::Instant::now() + Duration::from_secs(5);

    while std::time::Instant::now() < deadline {
        if let Ok(Some(meta)) = ServerSessionManager::load_session() {
            if meta.socket_path == *socket_path
                && ServerSessionManager::is_process_running(meta.pid)
            {
                return Ok(());
            }
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    eprintln!("Server process exited immediately");
    eprintln!(
        "  Check the log file for details: {}",
        default_log_path().display()
    );
    Err(SerialError::Io(io::Error::other("Server process exited")))
}

/// Stop the server daemon
async fn stop_server(json_output: bool) -> Result<()> {
    // Load session
    let meta = ServerSessionManager::load_session()?.ok_or_else(|| {
        SerialError::Io(io::Error::new(
            io::ErrorKind::NotFound,
            "Server is not running",
        ))
    })?;

    // Check if process is running
    if !ServerSessionManager::is_process_running(meta.pid) {
        if json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "ok": true,
                    "stale": true
                }))
                .unwrap()
            );
        } else {
            println!("✗ Server is not running (stale session)");
        }
        ServerSessionManager::clear_session()?;
        return Ok(());
    }

    // Stop the process
    ServerSessionManager::stop_process(meta.pid)?;

    // Wait a bit for graceful shutdown
    std::thread::sleep(Duration::from_millis(500));

    // Check if it stopped
    if !ServerSessionManager::is_process_running(meta.pid) {
        if json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "ok": true,
                    "pid": meta.pid
                }))
                .unwrap()
            );
        } else {
            println!("✓ Server stopped successfully");
        }
        ServerSessionManager::clear_session()?;
    } else {
        if json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "ok": false,
                    "error": format!("Server did not stop gracefully (PID: {})", meta.pid)
                }))
                .unwrap()
            );
        } else {
            println!("⚠ Server did not stop gracefully (PID: {})", meta.pid);
            println!("  You may need to manually kill the process:");
            println!("  kill {}", meta.pid);
        }
    }

    Ok(())
}

/// Show server status
async fn show_server_status(json_output: bool) -> Result<()> {
    match ServerSessionManager::load_session()? {
        Some(meta) => {
            let running = ServerSessionManager::is_process_running(meta.pid);

            if json_output {
                let mut status = serde_json::json!({
                    "running": running,
                    "pid": meta.pid,
                    "socket": meta.socket_path.display().to_string(),
                    "tcpPort": meta.tcp_port,
                    "log": meta.log_path.display().to_string(),
                    "maxConnections": meta.max_connections,
                });
                if let Ok(started) = std::time::UNIX_EPOCH.elapsed() {
                    status["uptimeSecs"] = serde_json::json!(started.as_secs() - meta.started_at);
                }
                println!("{}", serde_json::to_string_pretty(&status).unwrap());
                return Ok(());
            }

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
            println!(
                "  TCP Port: {}",
                meta.tcp_port
                    .map(|p| p.to_string())
                    .unwrap_or_else(|| "disabled".into())
            );
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
            if json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "running": false
                    }))
                    .unwrap()
                );
            } else {
                println!("Server Status: Not running");
                println!();
                println!("Use 'server start' to start the server daemon.");
            }
        }
    }

    Ok(())
}

/// Run the daemon process (foreground entry point; the real daemon body).
async fn run_daemon(
    socket_path: PathBuf,
    tcp_port: Option<u16>,
    bind_addr: IpAddr,
    log_path: PathBuf,
    max_connections: usize,
    idle_timeout_secs: u64,
) -> Result<()> {
    // Create server state
    let config = ServerConfig {
        socket_path: Some(socket_path.clone()),
        tcp_port,
        tcp_bind: bind_addr,
        max_connections,
        log_path: log_path.clone(),
        idle_timeout_secs,
    };

    let state = ServerState::new(config).await;

    // Save session
    let current_pid = std::process::id();
    let meta = ServerSessionMeta {
        pid: current_pid,
        socket_path: socket_path.clone(),
        tcp_port,
        started_at: ServerSessionManager::current_timestamp(),
        log_path,
        max_connections,
    };
    ServerSessionManager::save_session(&meta)?;

    // CancellationToken for graceful shutdown
    let token = CancellationToken::new();

    // Spawn SIGTERM handler (Unix only)
    #[cfg(unix)]
    {
        let token_clone = token.clone();
        tokio::spawn(async move {
            if let Ok(mut sigterm) =
                tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            {
                sigterm.recv().await;
                tracing::info!("Received SIGTERM, initiating graceful shutdown...");
                token_clone.cancel();
            }
        });
    }

    // Spawn idle connection cleanup task
    spawn_idle_cleanup_task(state.clone(), token.clone());

    // Run listener(s). Any listener returning (shutdown or fatal bind error)
    // ends the daemon.
    #[cfg(unix)]
    {
        let socket_fut = run_socket_server(state.clone(), socket_path.clone(), token.clone());
        match tcp_port {
            Some(port) => {
                let tcp_fut = run_tcp_server(
                    state.clone(),
                    SocketAddr::new(bind_addr, port),
                    token.clone(),
                );
                tokio::select! {
                    r = socket_fut => r?,
                    r = tcp_fut => r?,
                }
            }
            None => {
                socket_fut.await?;
            }
        }
    }
    #[cfg(not(unix))]
    {
        match tcp_port {
            Some(port) => {
                run_tcp_server(
                    state.clone(),
                    SocketAddr::new(bind_addr, port),
                    token.clone(),
                )
                .await?;
            }
            None => {
                // No listeners configured — wait for shutdown
                token.cancelled().await;
            }
        }
    }

    // Cleanup after shutdown
    tracing::info!("Shutting down: clearing session...");
    ServerSessionManager::clear_session()?;

    Ok(())
}

/// Call RPC method over the local Unix socket or a remote TCP address.
async fn call_rpc(
    method: String,
    args: String,
    use_stdin: bool,
    remote: Option<String>,
) -> Result<()> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    /// A client connection that is either a local Unix socket or a TCP
    /// stream. Wrapping in an enum lets us box a single `AsyncRead +
    /// AsyncWrite` object (dyn objects can only carry one non-auto trait).
    enum Conn {
        #[cfg(unix)]
        Unix(tokio::net::UnixStream),
        Tcp(tokio::net::TcpStream),
    }

    impl tokio::io::AsyncRead for Conn {
        fn poll_read(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            buf: &mut tokio::io::ReadBuf<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            match self.get_mut() {
                #[cfg(unix)]
                Conn::Unix(s) => std::pin::Pin::new(s).poll_read(cx, buf),
                Conn::Tcp(s) => std::pin::Pin::new(s).poll_read(cx, buf),
            }
        }
    }

    impl tokio::io::AsyncWrite for Conn {
        fn poll_write(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            buf: &[u8],
        ) -> std::task::Poll<std::io::Result<usize>> {
            match self.get_mut() {
                #[cfg(unix)]
                Conn::Unix(s) => std::pin::Pin::new(s).poll_write(cx, buf),
                Conn::Tcp(s) => std::pin::Pin::new(s).poll_write(cx, buf),
            }
        }

        fn poll_flush(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            match self.get_mut() {
                #[cfg(unix)]
                Conn::Unix(s) => std::pin::Pin::new(s).poll_flush(cx),
                Conn::Tcp(s) => std::pin::Pin::new(s).poll_flush(cx),
            }
        }

        fn poll_shutdown(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            match self.get_mut() {
                #[cfg(unix)]
                Conn::Unix(s) => std::pin::Pin::new(s).poll_shutdown(cx),
                Conn::Tcp(s) => std::pin::Pin::new(s).poll_shutdown(cx),
            }
        }
    }

    enum Target {
        #[cfg(unix)]
        Unix(PathBuf),
        Tcp(SocketAddr),
    }

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
    // Line-framed protocol: requests are newline-terminated. This lets the
    // server's LinesCodec emit the request immediately, without relying on
    // the half-close to trigger EOF decoding.
    let request_str = format!("{}\n", request_str);

    // Resolve connection target: explicit --remote wins; otherwise use the
    // local daemon session (Unix socket first, TCP localhost fallback).
    let mut target: Option<Target> = None;

    if let Some(remote_str) = remote {
        let addr = remote_str.parse::<SocketAddr>().map_err(|e| {
            SerialError::Io(io::Error::other(format!(
                "Invalid --remote address '{}': {}",
                remote_str, e
            )))
        })?;
        target = Some(Target::Tcp(addr));
    } else {
        let meta = ServerSessionManager::load_session()?.ok_or_else(|| {
            SerialError::Io(io::Error::new(
                io::ErrorKind::NotConnected,
                "Server is not running. Use 'server start' first, or pass --remote <ip:port>.",
            ))
        })?;

        #[cfg(unix)]
        {
            if meta.socket_path.exists() {
                target = Some(Target::Unix(meta.socket_path.clone()));
            }
        }

        if target.is_none() {
            match meta.tcp_port {
                Some(port) => {
                    target = Some(Target::Tcp(SocketAddr::from(([127, 0, 0, 1], port))));
                }
                None => {
                    return Err(SerialError::Io(io::Error::new(
                        io::ErrorKind::NotConnected,
                        "Server has no reachable listener (no Unix socket, TCP disabled)",
                    )));
                }
            }
        }
    }

    // Connect
    let mut stream: Conn = match target.unwrap() {
        #[cfg(unix)]
        Target::Unix(path) => {
            Conn::Unix(tokio::net::UnixStream::connect(&path).await.map_err(|e| {
                SerialError::Io(io::Error::new(
                    io::ErrorKind::NotConnected,
                    format!("Failed to connect to socket {}: {}", path.display(), e),
                ))
            })?)
        }
        Target::Tcp(addr) => {
            Conn::Tcp(tokio::net::TcpStream::connect(addr).await.map_err(|e| {
                SerialError::Io(io::Error::new(
                    io::ErrorKind::NotConnected,
                    format!("Failed to connect to {}: {}", addr, e),
                ))
            })?)
        }
    };

    // Send request
    stream
        .write_all(request_str.as_bytes())
        .await
        .map_err(|e| SerialError::Io(io::Error::other(format!("Failed to send request: {}", e))))?;

    // Shutdown write side (half-close) so the server sees end of request
    stream.shutdown().await.map_err(|e| {
        SerialError::Io(io::Error::other(format!(
            "Failed to shutdown stream: {}",
            e
        )))
    })?;

    // Read response until newline or EOF (line-based framing)
    let mut response_buf: Vec<u8> = Vec::new();
    let mut tmp = [0u8; 4096];
    loop {
        let n = stream.read(&mut tmp).await.map_err(|e| {
            SerialError::Io(io::Error::other(format!("Failed to read response: {}", e)))
        })?;
        if n == 0 {
            break;
        }
        response_buf.extend_from_slice(&tmp[..n]);
        if response_buf.contains(&b'\n') || response_buf.len() >= 1024 * 1024 {
            break;
        }
    }

    println!("{}", String::from_utf8_lossy(&response_buf));
    Ok(())
}

/// Parse a `--bind` argument into an `IpAddr`.
fn parse_bind_addr(bind: &Option<String>) -> IpAddr {
    bind.as_deref()
        .unwrap_or(DEFAULT_BIND_ADDR)
        .parse()
        .unwrap_or_else(|_| IpAddr::from([0, 0, 0, 0]))
}

/// Human-readable TCP endpoint description for output.
fn tcp_desc(tcp_port: Option<u16>, bind_addr: IpAddr) -> String {
    match tcp_port {
        Some(port) => format!("{}:{}", bind_addr, port),
        None => "disabled".to_string(),
    }
}
