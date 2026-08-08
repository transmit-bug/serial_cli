//! Socat backend wrapper (cross-platform)
//!
//! This backend creates virtual serial port pairs using the socat utility.

use crate::error::{Result, SerialError};
use crate::serial_core::backends::{
    BackendStats, BridgeErrorRx, BridgeStats, VirtualBackend, VirtualPortEnd,
};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::{mpsc, Mutex};

/// Socat backend implementation.
///
/// Note: Socat handles bridging internally — `bytes_read`/`bytes_written`
/// in `get_stats()` are always 0. Only `uptime_seconds` is meaningful.
pub struct SocatBackend {
    /// Socat child process
    process: Option<tokio::process::Child>,
    /// Socat child process id — kept so `Drop` can kill the process
    /// synchronously (tokio's `Child::kill` can't be awaited in `drop`).
    process_id: Option<u32>,
    /// Port A path (symlink)
    port_a_path: std::path::PathBuf,
    /// Port B path (symlink)
    port_b_path: std::path::PathBuf,
    /// Statistics
    stats: Arc<Mutex<BackendStats>>,
    /// Start time for uptime calculation
    start_time: SystemTime,
    /// Whether the pair has been created
    created: bool,
}

impl SocatBackend {
    /// Create a new Socat backend
    pub fn new() -> Result<Self> {
        Ok(Self {
            process: None,
            process_id: None,
            port_a_path: std::path::PathBuf::from("/tmp/serial_cli_socat_a"),
            port_b_path: std::path::PathBuf::from("/tmp/serial_cli_socat_b"),
            stats: Arc::new(Mutex::new(BackendStats::default())),
            start_time: SystemTime::now(),
            created: false,
        })
    }

    /// Check if socat binary is available
    pub async fn check_available() -> bool {
        tokio::process::Command::new("socat")
            .arg("-V")
            .output()
            .await
            .map(|_| true)
            .unwrap_or(false)
    }
}

#[async_trait]
impl VirtualBackend for SocatBackend {
    async fn create_pair(
        &mut self,
    ) -> Result<(VirtualPortEnd, VirtualPortEnd, BridgeErrorRx, BridgeStats)> {
        // Check if socat is available
        if !Self::check_available().await {
            return Err(SerialError::MissingDependency(
                "socat".to_string(),
                "Install with: apt install socat | brew install socat".to_string(),
            ));
        }

        tracing::info!(
            "Creating Socat pair: {} and {}",
            self.port_a_path.display(),
            self.port_b_path.display()
        );

        // Spawn socat process
        let output = tokio::process::Command::new("socat")
            .args([
                "-d",
                "-d",
                &format!("pty,raw,echo=0,link={}", self.port_a_path.display()),
                &format!("pty,raw,echo=0,link={}", self.port_b_path.display()),
            ])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| SerialError::BackendInitFailed(format!("Failed to spawn socat: {e}")))?;

        self.process_id = output.id();
        self.process = Some(output);

        // Give socat time to create the PTYs and symlinks
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        // Verify the symlinks were created
        if !self.port_a_path.exists() || !self.port_b_path.exists() {
            // Clean up the process and any partially-created symlinks
            if let Some(mut process) = self.process.take() {
                process.kill().await.ok();
            }
            self.process_id = None;
            let _ = std::fs::remove_file(&self.port_a_path);
            let _ = std::fs::remove_file(&self.port_b_path);
            return Err(SerialError::BackendInitFailed(
                "Socat failed to create port symlinks".to_string(),
            ));
        }

        self.created = true;
        self.start_time = SystemTime::now();

        // Socat handles bridging internally — no error channel needed.
        // The channel is never used, but capacity must be >= 1:
        // `mpsc::channel(0)` panics ("mpsc bounded channel requires buffer > 0").
        let (error_tx, error_rx) = mpsc::channel::<String>(1);
        drop(error_tx);
        let stats = Arc::clone(&self.stats);

        Ok((
            VirtualPortEnd {
                name: "A".into(),
                path: self.port_a_path.clone(),
            },
            VirtualPortEnd {
                name: "B".into(),
                path: self.port_b_path.clone(),
            },
            error_rx,
            stats,
        ))
    }

    async fn is_healthy(&self) -> bool {
        if !self.created {
            return false;
        }

        // Check if process is still running
        if let Some(process) = &self.process {
            // Try to get the process ID
            if process.id().is_some() {
                // Also check if symlinks still exist
                return self.port_a_path.exists() && self.port_b_path.exists();
            }
        }

        false
    }

    async fn get_stats(&self) -> BackendStats {
        let mut stats = self.stats.lock().await;
        stats.uptime_seconds = self.start_time.elapsed().unwrap_or_default().as_secs();
        stats.clone()
    }

    fn backend_type(&self) -> &'static str {
        "socat"
    }

    async fn cleanup(&mut self) -> Result<()> {
        tracing::debug!("Cleaning up Socat backend");

        // Kill the socat process
        if let Some(mut process) = self.process.take() {
            tracing::debug!("Killing socat process (PID: {:?})", process.id());
            process.kill().await.ok();
        }
        self.process_id = None;

        // Remove the symlinks we created so a stopped/failed pair doesn't leak
        let _ = std::fs::remove_file(&self.port_a_path);
        let _ = std::fs::remove_file(&self.port_b_path);

        self.created = false;
        Ok(())
    }
}

impl Drop for SocatBackend {
    fn drop(&mut self) {
        // Synchronous best-effort cleanup: kill the socat child (by PID, since
        // tokio's `Child::kill` can't be awaited in `drop`) and remove the
        // symlinks. Runs on any drop path — failed create, `virtual stop`,
        // or the backend going out of scope — so a dead CLI process never
        // leaks a stray socat process or /tmp/serial_cli_socat_{a,b} symlinks.
        #[cfg(unix)]
        if let Some(pid) = self.process_id {
            // SAFETY: kill with SIGTERM is the standard way to terminate a process
            unsafe {
                libc::kill(pid as libc::pid_t, libc::SIGTERM);
            }
        }
        #[cfg(windows)]
        if let Some(pid) = self.process_id {
            use windows::Win32::Foundation::CloseHandle;
            use windows::Win32::System::Threading::{
                OpenProcess, TerminateProcess, PROCESS_TERMINATE,
            };
            unsafe {
                if let Ok(handle) = OpenProcess(PROCESS_TERMINATE, false, pid) {
                    let _ = TerminateProcess(handle, 1);
                    let _ = CloseHandle(handle);
                }
            }
        }
        let _ = std::fs::remove_file(&self.port_a_path);
        let _ = std::fs::remove_file(&self.port_b_path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::OnceLock;

    /// Serializes socat-backed tests: they all share the fixed
    /// `/tmp/serial_cli_socat_{a,b}` symlink paths and would race when run
    /// in parallel (a second socat can't recreate an existing symlink).
    static SOCAT_TEST_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

    fn socat_test_lock() -> &'static tokio::sync::Mutex<()> {
        SOCAT_TEST_LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
    }

    /// Best-effort pre-clean of stale symlinks before creating a pair.
    fn pre_clean_symlinks(backend: &SocatBackend) {
        let _ = std::fs::remove_file(&backend.port_a_path);
        let _ = std::fs::remove_file(&backend.port_b_path);
    }

    #[tokio::test]
    async fn test_socat_availability() {
        let available = SocatBackend::check_available().await;
        tracing::info!("Socat available: {}", available);
        // We don't assert on this since socat might not be installed
    }

    #[test]
    fn test_socat_backend_creation() {
        let backend = SocatBackend::new();
        assert!(backend.is_ok());
        let backend = backend.unwrap();
        assert!(!backend.created);
        assert!(backend.process.is_none());
    }

    #[tokio::test]
    async fn test_socat_create_pair_when_available() {
        let _guard = socat_test_lock().lock().await;
        if !SocatBackend::check_available().await {
            tracing::info!("Skipping test: socat not available");
            return;
        }

        let mut backend = SocatBackend::new().unwrap();
        pre_clean_symlinks(&backend);
        let result = backend.create_pair().await;

        if let Err(e) = &result {
            tracing::warn!("Failed to create socat pair: {}", e);
            // This might fail in some environments
            return;
        }

        let (port_a, port_b, _error_rx, _stats) = result.unwrap();
        assert_eq!(port_a.name, "A");
        assert_eq!(port_b.name, "B");
        assert!(backend.created);

        // Cleanup
        backend.cleanup().await.unwrap();
    }

    #[tokio::test]
    async fn test_socat_create_pair_no_panic_and_cleanup_removes_symlinks() {
        let _guard = socat_test_lock().lock().await;
        if !SocatBackend::check_available().await {
            tracing::info!("Skipping test: socat not available");
            return;
        }

        // Regression for #64: `create_pair` must not panic (the old code used
        // `mpsc::channel(0)`, which panics with "mpsc bounded channel requires
        // buffer > 0" and leaked the spawned socat process + symlinks).
        let mut backend = SocatBackend::new().unwrap();
        pre_clean_symlinks(&backend);
        let (port_a, port_b, _error_rx, _stats) = backend
            .create_pair()
            .await
            .expect("socat create_pair should succeed");

        assert!(backend.port_a_path.exists(), "port A symlink should exist");
        assert!(backend.port_b_path.exists(), "port B symlink should exist");
        assert!(
            backend.process_id.is_some(),
            "socat process id should be recorded"
        );

        // Cleanup must kill the process and remove both symlinks.
        backend.cleanup().await.expect("cleanup should succeed");

        assert!(
            !backend.port_a_path.exists(),
            "port A symlink should be removed"
        );
        assert!(
            !backend.port_b_path.exists(),
            "port B symlink should be removed"
        );
        assert!(backend.process.is_none(), "socat child should be taken");
        assert!(
            backend.process_id.is_none(),
            "socat process id should be cleared"
        );

        let _ = (port_a, port_b);
    }

    #[tokio::test]
    async fn test_socat_drop_removes_symlinks() {
        let _guard = socat_test_lock().lock().await;
        if !SocatBackend::check_available().await {
            tracing::info!("Skipping test: socat not available");
            return;
        }

        // Regression for #64: dropping the backend (e.g. failed create, or the
        // CLI exiting) must kill the socat child and remove the symlinks so
        // nothing leaks after the process exits.
        let mut backend = SocatBackend::new().unwrap();
        pre_clean_symlinks(&backend);
        let _ = backend
            .create_pair()
            .await
            .expect("socat create_pair should succeed");
        let path_a = backend.port_a_path.clone();
        let path_b = backend.port_b_path.clone();

        drop(backend);

        assert!(!path_a.exists(), "port A symlink should be removed on drop");
        assert!(!path_b.exists(), "port B symlink should be removed on drop");
    }
}
