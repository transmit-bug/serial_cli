//! NamedPipe backend for Windows
//!
//! This backend creates virtual serial port pairs using Windows Named Pipes.
//! A relay task bridges data bidirectionally between the two pipe endpoints.

use crate::error::{Result, SerialError};

#[cfg(windows)]
use crate::serial_core::backends::{
    BackendStats, BridgeErrorRx, BridgeStats, VirtualBackend, VirtualPortEnd,
};
#[cfg(windows)]
use async_trait::async_trait;
#[cfg(windows)]
use std::sync::Arc;
#[cfg(windows)]
use std::time::SystemTime;
#[cfg(windows)]
use tokio::sync::{mpsc, Mutex};

/// NamedPipe backend implementation (Windows only)
#[cfg(windows)]
pub struct NamedPipeBackend {
    /// Pipe A path/name
    pipe_a_name: String,
    /// Pipe B path/name
    pipe_b_name: String,
    /// Pipe A server handle (accepts connections)
    server_a: HANDLE,
    /// Pipe B server handle (accepts connections)
    server_b: HANDLE,
    /// Pipe A client handle (used by relay for I/O)
    client_a: HANDLE,
    /// Pipe B client handle (used by relay for I/O)
    client_b: HANDLE,
    /// Statistics
    stats: Arc<Mutex<BackendStats>>,
    /// Start time for uptime calculation
    start_time: SystemTime,
    /// Whether the pair has been created
    created: bool,
    /// Relay task handle
    relay_task: Option<tokio::task::JoinHandle<()>>,
    /// Shutdown event — signals relay threads to stop
    shutdown_event: HANDLE,
}

#[cfg(windows)]
use windows::Win32::Foundation::{CloseHandle, HANDLE};

#[cfg(windows)]
impl NamedPipeBackend {
    /// Create a new NamedPipe backend
    pub fn new() -> Result<Self> {
        let uuid_a = uuid::Uuid::new_v4();
        let uuid_b = uuid::Uuid::new_v4();

        let shutdown_event = Self::create_event().unwrap_or(HANDLE::default());

        Ok(Self {
            pipe_a_name: format!(r"\\.\pipe\serial_cli_a_{}", uuid_a),
            pipe_b_name: format!(r"\\.\pipe\serial_cli_b_{}", uuid_b),
            server_a: HANDLE::default(),
            server_b: HANDLE::default(),
            client_a: HANDLE::default(),
            client_b: HANDLE::default(),
            stats: Arc::new(Mutex::new(BackendStats::default())),
            start_time: SystemTime::now(),
            created: false,
            relay_task: None,
            shutdown_event,
        })
    }

    /// Create a manual-reset event for signaling shutdown.
    fn create_event() -> Result<HANDLE> {
        use windows::core::PCWSTR;
        use windows::Win32::System::Threading::CreateEventW;

        unsafe {
            let event = CreateEventW(None, true, false, PCWSTR::null());
            if event.is_invalid() {
                return Err(SerialError::BackendInitFailed(
                    "CreateEventW failed".to_string(),
                ));
            }
            Ok(event)
        }
    }

    /// Create a named pipe server and wait for a single client connection.
    /// Returns the client handle. The server handle is stored in self.
    fn accept_pipe_connection(&self, name: &str) -> Result<HANDLE> {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use windows::core::PCWSTR;
        use windows::Win32::Storage::FileSystem::PIPE_ACCESS_DUPLEX;
        use windows::Win32::System::Pipes::{
            ConnectNamedPipe, CreateNamedPipeW, NMPWAIT_USE_DEFAULT_WAIT, PIPE_READMODE_BYTE,
            PIPE_TYPE_BYTE, PIPE_UNLIMITED_INSTANCES, PIPE_WAIT,
        };
        use windows::Win32::System::Threading::{CreateEventW, WaitForSingleObject, INFINITE};
        use windows::Win32::System::IO::OVERLAPPED;

        let wide_name: Vec<u16> = OsStr::new(name)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let server = unsafe {
            CreateNamedPipeW(
                PCWSTR(wide_name.as_ptr()),
                PIPE_ACCESS_DUPLEX,
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
                PIPE_UNLIMITED_INSTANCES,
                8192,
                8192,
                0,
                None,
            )
        };
        if server.is_invalid() {
            return Err(SerialError::BackendInitFailed(format!(
                "CreateNamedPipeW failed for {name}: {}",
                std::io::Error::last_os_error()
            )));
        }

        // Use overlapped I/O for the connection wait.
        let event = unsafe { CreateEventW(None, true, false, PCWSTR::null()) };
        if event.is_invalid() {
            unsafe { CloseHandle(server) };
            return Err(SerialError::BackendInitFailed(
                "CreateEventW failed".to_string(),
            ));
        }

        let mut overlapped = OVERLAPPED::default();
        overlapped.hEvent = event;

        let result = unsafe { ConnectNamedPipe(server, Some(&mut overlapped)) };
        if !result.as_bool() {
            let err = std::io::Error::last_os_error();
            // 997 = ERROR_IO_PENDING — expected.
            if err.raw_os_error() != Some(997) {
                unsafe {
                    CloseHandle(event);
                    CloseHandle(server);
                }
                return Err(SerialError::BackendInitFailed(format!(
                    "ConnectNamedPipe failed for {name}: {err}"
                )));
            }
            unsafe { WaitForSingleObject(event, INFINITE) };
        }

        unsafe { CloseHandle(event) };
        Ok(server)
    }

    /// Open a client connection to a named pipe.
    fn open_client(name: &str) -> Result<HANDLE> {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use windows::core::PCWSTR;
        use windows::Win32::Storage::FileSystem::{
            CreateFileW, FILE_ATTRIBUTE_NORMAL, OPEN_EXISTING,
        };

        let wide_name: Vec<u16> = OsStr::new(name)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let handle = unsafe {
            CreateFileW(
                PCWSTR(wide_name.as_ptr()),
                0xC0000000, // GENERIC_READ | GENERIC_WRITE
                0,
                None,
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                HANDLE::default(),
            )
        };
        if handle.is_invalid() {
            return Err(SerialError::BackendInitFailed(format!(
                "CreateFileW failed for {name}: {}",
                std::io::Error::last_os_error()
            )));
        }
        Ok(handle)
    }

    /// Read synchronously from a pipe handle. Returns Ok(0) on EOF/error.
    fn pipe_read(handle: HANDLE, buf: &mut [u8]) -> std::io::Result<usize> {
        use std::mem::MaybeUninit;
        use windows::Win32::Storage::FileSystem::ReadFile;

        let mut bytes_read: u32 = 0;
        let result = unsafe { ReadFile(handle, buf, Some(&mut bytes_read), None) };
        if result.as_bool() {
            return Ok(bytes_read as usize);
        }
        let err = std::io::Error::last_os_error();
        // On named pipes, we may get ERROR_BROKEN_PIPE (109) or
        // ERROR_INVALID_HANDLE (6) when the other end closes — treat as EOF.
        if err.raw_os_error() == Some(109) || err.raw_os_error() == Some(6) {
            return Ok(0);
        }
        Err(err)
    }

    /// Write synchronously to a pipe handle.
    fn pipe_write(handle: HANDLE, buf: &[u8]) -> std::io::Result<()> {
        use windows::Win32::Storage::FileSystem::WriteFile;

        let mut bytes_written: u32 = 0;
        let result = unsafe { WriteFile(handle, buf, Some(&mut bytes_written), None) };
        if result.as_bool() {
            return Ok(());
        }
        Err(std::io::Error::last_os_error())
    }
}

#[cfg(windows)]
#[async_trait]
impl VirtualBackend for NamedPipeBackend {
    async fn create_pair(
        &mut self,
    ) -> Result<(VirtualPortEnd, VirtualPortEnd, BridgeErrorRx, BridgeStats)> {
        tracing::info!(
            "Creating NamedPipe pair: {} and {}",
            self.pipe_a_name,
            self.pipe_b_name
        );

        // Create server handles and accept connections.
        let server_a = self.accept_pipe_connection(&self.pipe_a_name)?;
        let server_b = self.accept_pipe_connection(&self.pipe_b_name)?;

        // Open client handles for the relay to use.
        let client_a = Self::open_client(&self.pipe_a_name)?;
        let client_b = Self::open_client(&self.pipe_b_name)?;

        self.server_a = server_a;
        self.server_b = server_b;
        self.client_a = client_a;
        self.client_b = client_b;
        self.created = true;
        self.start_time = SystemTime::now();

        // Reset the shutdown event.
        use windows::Win32::System::Threading::ResetEvent;
        unsafe { ResetEvent(self.shutdown_event).ok() };

        // Spawn the bidirectional relay bridge.
        let stats_clone = Arc::clone(&self.stats);
        let ca_relay = self.client_a;
        let cb_relay = self.client_b;
        let shutdown = self.shutdown_event;
        self.relay_task = Some(tokio::spawn(async move {
            Self::relay_blocking(ca_relay, cb_relay, stats_clone, shutdown);
        }));

        // Create an empty error channel for API consistency.
        let (error_tx, error_rx) = mpsc::channel::<String>(0);
        drop(error_tx);
        let stats = Arc::clone(&self.stats);

        Ok((
            VirtualPortEnd {
                name: "A".into(),
                path: std::path::PathBuf::from(&self.pipe_a_name),
            },
            VirtualPortEnd {
                name: "B".into(),
                path: std::path::PathBuf::from(&self.pipe_b_name),
            },
            error_rx,
            stats,
        ))
    }

    async fn is_healthy(&self) -> bool {
        if !self.created {
            return false;
        }
        if let Some(task) = &self.relay_task {
            !task.is_finished()
        } else {
            false
        }
    }

    async fn get_stats(&self) -> BackendStats {
        let mut stats = self.stats.lock().await;
        stats.uptime_seconds = self.start_time.elapsed().unwrap_or_default().as_secs();
        stats.clone()
    }

    fn backend_type(&self) -> &'static str {
        "namedpipe"
    }

    async fn cleanup(&mut self) -> Result<()> {
        if !self.created {
            return Ok(());
        }
        tracing::debug!("Cleaning up NamedPipe backend");

        // Signal shutdown to relay threads.
        use windows::Win32::System::Threading::SetEvent;
        unsafe { SetEvent(self.shutdown_event).ok() };

        // Close all pipe handles — this breaks the relay's blocking I/O.
        unsafe {
            if !self.client_a.is_invalid() {
                CloseHandle(self.client_a);
                self.client_a = HANDLE::default();
            }
            if !self.client_b.is_invalid() {
                CloseHandle(self.client_b);
                self.client_b = HANDLE::default();
            }
            if !self.server_a.is_invalid() {
                CloseHandle(self.server_a);
                self.server_a = HANDLE::default();
            }
            if !self.server_b.is_invalid() {
                CloseHandle(self.server_b);
                self.server_b = HANDLE::default();
            }
        }

        // Wait for relay to exit.
        if let Some(task) = self.relay_task.take() {
            match tokio::time::timeout(std::time::Duration::from_millis(500), task).await {
                Ok(_) => tracing::debug!("NamedPipe relay exited gracefully"),
                Err(_) => {
                    tracing::debug!("NamedPipe relay did not exit within timeout");
                }
            }
        }

        self.created = false;
        Ok(())
    }
}

/// Bidirectional relay between two pipe handles using blocking I/O.
/// Runs inside `tokio::task::spawn_blocking`.
#[cfg(windows)]
impl NamedPipeBackend {
    fn relay_blocking(
        client_a: HANDLE,
        client_b: HANDLE,
        stats: Arc<Mutex<BackendStats>>,
        shutdown_event: HANDLE,
    ) {
        use windows::Win32::Foundation::WAIT_OBJECT_0;
        use windows::Win32::System::Threading::WaitForMultipleObjects;

        let mut buf = vec![0u8; 8192];

        // We run two relay loops in parallel via thread parking.
        // For simplicity and correctness, use a single-threaded select:
        // poll both pipes alternately, checking shutdown between reads.

        let stats_a = Arc::clone(&stats);
        let stats_b = Arc::clone(&stats);

        // Thread for A → B
        let t1 = std::thread::spawn(move || {
            let mut buf = buf.clone();
            loop {
                // Check shutdown event (non-blocking).
                let wait = unsafe {
                    WaitForMultipleObjects(
                        &[shutdown_event],
                        false,
                        0, // timeout=0 → non-blocking
                    )
                };
                if wait == WAIT_OBJECT_0 {
                    break;
                }

                match Self::pipe_read(client_a, &mut buf) {
                    Ok(0) => break, // EOF or closed
                    Ok(n) => {
                        if Self::pipe_write(client_b, &buf[..n]).is_err() {
                            tracing::warn!("relay a→b write failed");
                            break;
                        }
                        let mut s = stats_a.blocking_lock();
                        s.bytes_read += n as u64;
                        s.bytes_written += n as u64;
                    }
                    Err(e) => {
                        tracing::debug!("relay a→b read failed: {e}");
                        break;
                    }
                }
            }
        });

        // Thread for B → A
        let t2 = std::thread::spawn(move || {
            let mut buf = buf;
            loop {
                let wait = unsafe { WaitForMultipleObjects(&[shutdown_event], false, 0) };
                if wait == WAIT_OBJECT_0 {
                    break;
                }

                match Self::pipe_read(client_b, &mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        if Self::pipe_write(client_a, &buf[..n]).is_err() {
                            tracing::warn!("relay b→a write failed");
                            break;
                        }
                        let mut s = stats_b.blocking_lock();
                        s.bytes_read += n as u64;
                        s.bytes_written += n as u64;
                    }
                    Err(e) => {
                        tracing::debug!("relay b→a read failed: {e}");
                        break;
                    }
                }
            }
        });

        let _ = t1.join();
        let _ = t2.join();
        tracing::debug!("NamedPipe relay task stopped");
    }
}

/// NamedPipe backend is not available on non-Windows platforms
#[cfg(not(windows))]
pub struct NamedPipeBackend;

#[cfg(not(windows))]
impl NamedPipeBackend {
    pub fn new() -> Result<Self> {
        Err(SerialError::UnsupportedBackend(
            "NamedPipe backend is only available on Windows".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn test_named_pipe_backend_creation() {
        let backend = NamedPipeBackend::new();
        assert!(backend.is_ok());
        let backend = backend.unwrap();
        assert!(!backend.created);
        assert!(!backend.pipe_a_name.is_empty());
        assert!(!backend.pipe_b_name.is_empty());
    }

    #[cfg(not(windows))]
    #[test]
    fn test_named_pipe_backend_not_available() {
        let backend = NamedPipeBackend::new();
        assert!(backend.is_err());
    }
}
