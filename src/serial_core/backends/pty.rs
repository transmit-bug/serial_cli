//! PTY (pseudo-terminal) backend for Unix/macOS
//!
//! This backend creates virtual serial port pairs using POSIX PTY.

use crate::error::{Result, SerialError};
use crate::serial_core::backends::{
    BackendStats, BridgeErrorRx, BridgeStats, VirtualBackend, VirtualPortEnd,
};
use async_trait::async_trait;
use std::os::unix::io::{AsRawFd, FromRawFd, OwnedFd};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::io::unix::AsyncFd;
use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinHandle;
use tokio::time::Duration;

/// PTY backend implementation
pub struct PtyBackend {
    /// First PTY master (for port A)
    master_a: Option<Arc<AsyncFd<OwnedFd>>>,
    /// Second PTY master (for port B)
    master_b: Option<Arc<AsyncFd<OwnedFd>>>,
    /// Slave path for port A
    slave_a_path: Option<String>,
    /// Slave path for port B
    slave_b_path: Option<String>,
    /// Running state
    running: Arc<AtomicBool>,
    /// Statistics
    stats: Arc<Mutex<BackendStats>>,
    /// Bridge task handle
    bridge_task: Option<JoinHandle<()>>,
    /// Start time for uptime calculation
    start_time: SystemTime,
}

impl PtyBackend {
    /// Create a new PTY backend
    pub fn new() -> Result<Self> {
        Ok(Self {
            master_a: None,
            master_b: None,
            slave_a_path: None,
            slave_b_path: None,
            running: Arc::new(AtomicBool::new(false)),
            stats: Arc::new(Mutex::new(BackendStats::default())),
            bridge_task: None,
            start_time: SystemTime::now(),
        })
    }

    /// Create the actual PTY pair
    #[allow(clippy::type_complexity)]
    fn create_pty_pair(
        &mut self,
        buffer_size: usize,
    ) -> Result<(
        String,
        String,
        Arc<AsyncFd<OwnedFd>>,
        Arc<AsyncFd<OwnedFd>>,
        JoinHandle<()>,
        mpsc::Receiver<String>,
    )> {
        use libc::{grantpt, posix_openpt, ptsname, unlockpt, O_NOCTTY, O_RDWR};
        use std::ffi::CStr;

        // Create first PTY master
        let master1_fd = unsafe { posix_openpt(O_RDWR | O_NOCTTY) };
        if master1_fd == -1 {
            return Err(SerialError::VirtualPort(
                "Failed to open first PTY master".to_string(),
            ));
        }
        if unsafe { grantpt(master1_fd) } == -1 {
            unsafe { libc::close(master1_fd) };
            return Err(SerialError::VirtualPort(
                "Failed to grant first PTY".to_string(),
            ));
        }
        if unsafe { unlockpt(master1_fd) } == -1 {
            unsafe { libc::close(master1_fd) };
            return Err(SerialError::VirtualPort(
                "Failed to unlock first PTY".to_string(),
            ));
        }
        let slave1_path = unsafe {
            let ptr = ptsname(master1_fd);
            if ptr.is_null() {
                libc::close(master1_fd);
                return Err(SerialError::VirtualPort(
                    "Failed to get first PTY slave name".to_string(),
                ));
            }
            CStr::from_ptr(ptr)
        }
        .to_string_lossy()
        .to_string();

        // Create second PTY master
        let master2_fd = unsafe { posix_openpt(O_RDWR | O_NOCTTY) };
        if master2_fd == -1 {
            unsafe { libc::close(master1_fd) };
            return Err(SerialError::VirtualPort(
                "Failed to open second PTY master".to_string(),
            ));
        }
        if unsafe { grantpt(master2_fd) } == -1 {
            unsafe { libc::close(master1_fd) };
            unsafe { libc::close(master2_fd) };
            return Err(SerialError::VirtualPort(
                "Failed to grant second PTY".to_string(),
            ));
        }
        if unsafe { unlockpt(master2_fd) } == -1 {
            unsafe { libc::close(master1_fd) };
            unsafe { libc::close(master2_fd) };
            return Err(SerialError::VirtualPort(
                "Failed to unlock second PTY".to_string(),
            ));
        }
        let slave2_path = unsafe {
            let ptr = ptsname(master2_fd);
            if ptr.is_null() {
                libc::close(master1_fd);
                libc::close(master2_fd);
                return Err(SerialError::VirtualPort(
                    "Failed to get second PTY slave name".to_string(),
                ));
            }
            CStr::from_ptr(ptr)
        }
        .to_string_lossy()
        .to_string();

        tracing::debug!(
            "PTY pair created: slave1={}, slave2={}",
            slave1_path,
            slave2_path
        );

        // SAFETY: we own these fds and will close them exactly once via OwnedFd
        let master1_owned = unsafe { OwnedFd::from_raw_fd(master1_fd) };
        let master2_owned = unsafe { OwnedFd::from_raw_fd(master2_fd) };

        let master1_async = AsyncFd::new(master1_owned).map_err(|e| {
            SerialError::VirtualPort(format!("Failed to register first PTY with epoll: {e}"))
        })?;
        let master2_async = AsyncFd::new(master2_owned).map_err(|e| {
            SerialError::VirtualPort(format!("Failed to register second PTY with epoll: {e}"))
        })?;

        // Create channels for bridge communication
        let (error_tx, error_rx) = mpsc::channel(10);

        // Wrap in Arc for shared ownership across select branches
        let m1 = Arc::new(master1_async);
        let m2 = Arc::new(master2_async);

        // Start running state
        self.running.store(true, Ordering::SeqCst);

        // Spawn event-driven bridge task
        let stats_clone = Arc::clone(&self.stats);
        let running_clone = Arc::clone(&self.running);
        let error_tx_clone = error_tx.clone();
        let m1_bridge = Arc::clone(&m1);
        let m2_bridge = Arc::clone(&m2);

        let bridge_task = tokio::spawn(async move {
            tracing::debug!("PTY bridge task started (event-driven)");

            let mut buffer = vec![0u8; buffer_size];

            loop {
                let m1_readable = Arc::clone(&m1_bridge);
                let m2_readable = Arc::clone(&m2_bridge);
                let m1_read = Arc::clone(&m1_bridge);
                let m2_write = Arc::clone(&m2_bridge);
                let m2_read = Arc::clone(&m2_bridge);
                let m1_write = Arc::clone(&m1_bridge);

                tokio::select! {
                    // Wait for master1 to become readable
                    result = m1_readable.readable() => {
                        if !running_clone.load(Ordering::SeqCst) { break; }
                        let mut guard = match result {
                            Ok(g) => g,
                            Err(e) => {
                                let error = format!("master1 readable() failed: {e}");
                                tracing::error!("{}", error);
                                let _ = error_tx_clone.send(error).await;
                                break;
                            }
                        };
                        let fd = m1_read.as_raw_fd();
                        let n1 = guard.try_io(|_| Ok(unsafe {
                            libc::read(
                                fd,
                                buffer.as_mut_ptr() as *mut libc::c_void,
                                buffer.len(),
                            )
                        }));

                        match n1 {
                            Ok(Ok(n)) if n > 0 => {
                                let n1_usize = n as usize;
                                let fd2 = m2_write.as_raw_fd();
                                let mut written: isize = 0;
                                while written < n1_usize as isize {
                                    let n = unsafe {
                                        libc::write(
                                            fd2,
                                            buffer.as_ptr().add(written as usize) as *const libc::c_void,
                                            (n1_usize as isize - written) as usize,
                                        )
                                    };
                                    if n > 0 {
                                        written += n;
                                    } else {
                                        let error = format!(
                                            "Partial write failed: {} bytes remaining",
                                            n1_usize as isize - written,
                                        );
                                        tracing::error!("{}", error);
                                        let _ = error_tx_clone.send(error).await;
                                        break;
                                    }
                                }
                                let mut s = stats_clone.lock().await;
                                s.bytes_read += n1_usize as u64;
                                s.bytes_written += n1_usize as u64;
                            }
                            Ok(Ok(_)) => {} // EOF or spurious
                            Ok(Err(e)) => {
                                let error = format!("master1 read failed: {e}");
                                tracing::error!("{}", error);
                                let _ = error_tx_clone.send(error).await;
                            }
                            Err(_) => break,
                        }
                    }

                    // Wait for master2 to become readable
                    result = m2_readable.readable() => {
                        if !running_clone.load(Ordering::SeqCst) { break; }
                        let mut guard = match result {
                            Ok(g) => g,
                            Err(e) => {
                                let error = format!("master2 readable() failed: {e}");
                                tracing::error!("{}", error);
                                let _ = error_tx_clone.send(error).await;
                                break;
                            }
                        };
                        let fd = m2_read.as_raw_fd();
                        let n2 = guard.try_io(|_| Ok(unsafe {
                            libc::read(
                                fd,
                                buffer.as_mut_ptr() as *mut libc::c_void,
                                buffer.len(),
                            )
                        }));

                        match n2 {
                            Ok(Ok(n)) if n > 0 => {
                                let n2_usize = n as usize;
                                let fd1 = m1_write.as_raw_fd();
                                let mut written: isize = 0;
                                while written < n2_usize as isize {
                                    let n = unsafe {
                                        libc::write(
                                            fd1,
                                            buffer.as_ptr().add(written as usize) as *const libc::c_void,
                                            (n2_usize as isize - written) as usize,
                                        )
                                    };
                                    if n > 0 {
                                        written += n;
                                    } else {
                                        let error = format!(
                                            "Partial write failed: {} bytes remaining",
                                            n2_usize as isize - written,
                                        );
                                        tracing::error!("{}", error);
                                        let _ = error_tx_clone.send(error).await;
                                        break;
                                    }
                                }
                                let mut s = stats_clone.lock().await;
                                s.bytes_read += n2_usize as u64;
                                s.bytes_written += n2_usize as u64;
                            }
                            Ok(Ok(_)) => {}
                            Ok(Err(e)) => {
                                let error = format!("master2 read failed: {e}");
                                tracing::error!("{}", error);
                                let _ = error_tx_clone.send(error).await;
                            }
                            Err(_) => break,
                        }
                    }

                    // Exit signal
                    _ = tokio::time::sleep(Duration::from_millis(100)) => {
                        if !running_clone.load(Ordering::SeqCst) { break; }
                    }
                }
            }

            tracing::debug!("PTY bridge task stopped");
        });

        Ok((slave1_path, slave2_path, m1, m2, bridge_task, error_rx))
    }
}

#[async_trait]
impl VirtualBackend for PtyBackend {
    async fn create_pair(
        &mut self,
    ) -> Result<(VirtualPortEnd, VirtualPortEnd, BridgeErrorRx, BridgeStats)> {
        let buffer_size = 8192;
        let (slave_a, slave_b, master_a, master_b, bridge_task, error_rx) =
            self.create_pty_pair(buffer_size)?;

        self.slave_a_path = Some(slave_a.clone());
        self.slave_b_path = Some(slave_b.clone());
        self.master_a = Some(master_a);
        self.master_b = Some(master_b);
        self.bridge_task = Some(bridge_task);
        self.start_time = SystemTime::now();

        let stats = Arc::clone(&self.stats);

        Ok((
            VirtualPortEnd {
                name: "A".into(),
                path: std::path::PathBuf::from(&slave_a),
            },
            VirtualPortEnd {
                name: "B".into(),
                path: std::path::PathBuf::from(&slave_b),
            },
            error_rx,
            stats,
        ))
    }

    async fn is_healthy(&self) -> bool {
        if !self.running.load(Ordering::SeqCst) {
            return false;
        }

        // Check if bridge task is still running
        if let Some(task) = &self.bridge_task {
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
        "pty"
    }

    async fn cleanup(&mut self) -> Result<()> {
        tracing::debug!("Cleaning up PTY backend");

        // Signal shutdown
        self.running.store(false, Ordering::SeqCst);

        // Wait for bridge task to finish
        if let Some(bridge_task) = self.bridge_task.take() {
            match tokio::time::timeout(Duration::from_millis(100), bridge_task).await {
                Ok(_) => tracing::debug!("PTY bridge task exited gracefully"),
                Err(_) => {
                    tracing::debug!("PTY bridge task did not exit within timeout");
                }
            }
        }

        // Drop master file descriptors (AsyncFd drops its inner OwnedFd automatically)
        self.master_a.take();
        self.master_b.take();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pty_backend_creation() {
        let backend = PtyBackend::new();
        assert!(backend.is_ok());
        let backend = backend.unwrap();
        assert!(!backend.running.load(std::sync::atomic::Ordering::SeqCst));
        assert!(backend.master_a.is_none());
        assert!(backend.master_b.is_none());
    }

    #[test]
    fn test_pty_backend_type() {
        let backend = PtyBackend::new().unwrap();
        assert_eq!(backend.backend_type(), "pty");
    }

    #[tokio::test]
    async fn test_pty_create_pair_and_data_flow() {
        let mut backend = PtyBackend::new().unwrap();

        let (port_a, port_b, _error_rx, _stats) = backend.create_pair().await.unwrap();

        // Verify port paths exist and are different
        assert!(!port_a.path.to_string_lossy().is_empty());
        assert!(!port_b.path.to_string_lossy().is_empty());
        assert_ne!(port_a.path, port_b.path);
        assert_eq!(port_a.name, "A");
        assert_eq!(port_b.name, "B");

        // Verify backend is healthy after creation
        assert!(backend.is_healthy().await);

        // Verify stats are accessible
        let stats = backend.get_stats().await;
        assert_eq!(stats.uptime_seconds, 0); // just created

        // Cleanup
        backend.cleanup().await.unwrap();
        assert!(!backend.is_healthy().await);
    }

    #[tokio::test]
    async fn test_pty_cleanup() {
        let mut backend = PtyBackend::new().unwrap();
        let (_port_a, _port_b, _, _) = backend.create_pair().await.unwrap();

        assert!(backend.is_healthy().await);
        backend.cleanup().await.unwrap();
        assert!(!backend.is_healthy().await);
        assert!(backend.bridge_task.is_none());
        assert!(backend.master_a.is_none());
        assert!(backend.master_b.is_none());
    }

    #[tokio::test]
    async fn test_pty_not_healthy_before_creation() {
        let backend = PtyBackend::new().unwrap();
        assert!(!backend.is_healthy().await);
    }
}
