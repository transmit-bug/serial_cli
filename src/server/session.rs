//! Server session management
//!
//! Tracks active server sessions across CLI invocations using file-based state.
//! Similar to sniff_session.rs but for the server daemon.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{Result, SerialError};

/// Session file directory name (under the user's cache dir)
const SESSION_DIR_NAME: &str = "serial_cli";
/// Server session file name
const SESSION_FILE_NAME: &str = "server_session.json";

/// Active server session metadata persisted to disk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSessionMeta {
    /// Process ID of the background server process
    pub pid: u32,

    /// Unix socket path (or TCP address)
    pub socket_path: PathBuf,

    /// TCP port (if using TCP instead of Unix socket)
    pub tcp_port: Option<u16>,

    /// Start timestamp (UNIX epoch seconds)
    pub started_at: u64,

    /// Log file path
    pub log_path: PathBuf,

    /// Max connections
    pub max_connections: usize,
}

/// Server session manager
pub struct ServerSessionManager;

impl ServerSessionManager {
    /// Get the directory where session files are stored
    fn session_dir() -> Result<PathBuf> {
        let cache = directories::BaseDirs::new()
            .ok_or_else(|| {
                SerialError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Could not determine user home directory",
                ))
            })?
            .cache_dir()
            .to_path_buf();
        let dir = cache.join(SESSION_DIR_NAME);
        fs::create_dir_all(&dir).map_err(SerialError::Io)?;
        Ok(dir)
    }

    /// Get the session file path
    fn session_file() -> Result<PathBuf> {
        Ok(Self::session_dir()?.join(SESSION_FILE_NAME))
    }

    /// Save session metadata to disk
    pub fn save_session(meta: &ServerSessionMeta) -> Result<()> {
        let path = Self::session_file()?;
        let json = serde_json::to_string_pretty(meta).map_err(|e| {
            SerialError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e.to_string(),
            ))
        })?;
        fs::write(&path, json).map_err(SerialError::Io)?;
        Ok(())
    }

    /// Load session metadata from disk
    pub fn load_session() -> Result<Option<ServerSessionMeta>> {
        let path = Self::session_file()?;
        if !path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(&path).map_err(SerialError::Io)?;
        let meta: ServerSessionMeta = serde_json::from_str(&content).map_err(|e| {
            SerialError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e.to_string(),
            ))
        })?;
        Ok(Some(meta))
    }

    /// Remove the session file (clears active session)
    pub fn clear_session() -> Result<()> {
        let path = Self::session_file()?;
        if path.exists() {
            fs::remove_file(&path).map_err(SerialError::Io)?;
        }
        Ok(())
    }

    /// Check if a process with the given PID is still running
    #[cfg(unix)]
    pub fn is_process_running(pid: u32) -> bool {
        // SAFETY: kill syscall with sig=0 is the standard POSIX way to check process existence
        unsafe { libc::kill(pid as libc::pid_t, 0) == 0 }
    }

    #[cfg(windows)]
    pub fn is_process_running(pid: u32) -> bool {
        use windows::Win32::Foundation::CloseHandle;
        use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION};

        let rights = PROCESS_QUERY_INFORMATION;
        unsafe {
            match OpenProcess(rights, false, pid) {
                Ok(handle) => {
                    if handle.is_invalid() {
                        false
                    } else {
                        let _ = CloseHandle(&handle);
                        true
                    }
                }
                Err(_) => false,
            }
        }
    }

    /// Send SIGTERM to a process
    #[cfg(unix)]
    pub fn stop_process(pid: u32) -> Result<()> {
        // SAFETY: kill with SIGTERM is the standard way to terminate a process
        let ret = unsafe { libc::kill(pid as libc::pid_t, libc::SIGTERM) };
        if ret != 0 {
            return Err(SerialError::Io(std::io::Error::other(format!(
                "Failed to send SIGTERM to process {}",
                pid
            ))));
        }
        Ok(())
    }

    #[cfg(windows)]
    pub fn stop_process(pid: u32) -> Result<()> {
        use windows::Win32::Foundation::CloseHandle;
        use windows::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};

        let rights = PROCESS_TERMINATE;
        let handle = unsafe { OpenProcess(rights, false, pid) }.map_err(|e| {
            SerialError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to open process {}: {:?}", pid, e),
            ))
        })?;
        unsafe {
            let result = TerminateProcess(&handle, 1);
            let _ = CloseHandle(&handle);
            if result.is_ok() {
                Ok(())
            } else {
                Err(SerialError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to terminate process {}", pid),
                )))
            }
        }
    }

    /// Get current timestamp as UNIX epoch seconds
    pub fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_serialization() {
        let meta = ServerSessionMeta {
            pid: 12345,
            socket_path: PathBuf::from("/tmp/test.sock"),
            tcp_port: None,
            started_at: 1234567890,
            log_path: PathBuf::from("/tmp/test.log"),
            max_connections: 10,
        };

        let json = serde_json::to_string_pretty(&meta).unwrap();
        assert!(json.contains("12345"));
        assert!(json.contains("/tmp/test.sock"));

        let deserialized: ServerSessionMeta = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.pid, 12345);
        assert_eq!(deserialized.socket_path, PathBuf::from("/tmp/test.sock"));
    }
}
