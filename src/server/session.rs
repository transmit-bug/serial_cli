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
    pub fn session_dir() -> Result<PathBuf> {
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
    pub fn is_process_running(pid: u32) -> bool {
        let sys = sysinfo::System::new_with_specifics(
            sysinfo::RefreshKind::new()
                .with_processes(sysinfo::ProcessRefreshKind::new()),
        );
        sys.process(sysinfo::Pid::from_u32(pid)).is_some()
    }

    /// Terminate a process by PID
    pub fn stop_process(pid: u32) -> Result<()> {
        let sys = sysinfo::System::new_with_specifics(
            sysinfo::RefreshKind::new()
                .with_processes(sysinfo::ProcessRefreshKind::new()),
        );
        sys.process(sysinfo::Pid::from_u32(pid))
            .ok_or_else(|| {
                SerialError::Io(std::io::Error::other(format!(
                    "Process {} not found",
                    pid
                )))
            })?
            .kill();
        Ok(())
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
