//! I/O loop for async serial operations
//!
//! This module provides the async I/O event loop for managing multiple serial ports.

use crate::error::{Result, SerialError};
use crate::serial_core::PortManager;
use bytes::BytesMut;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinHandle;
use tokio::time::{timeout, Duration};

/// IoEvent with zero-copy BytesMut for received data
#[derive(Debug, Clone)]
pub enum IoEvent {
    /// Data received from port
    DataReceived { port_id: String, data: BytesMut },
    /// Data sent to port
    DataSent { port_id: String, length: usize },
    /// Port opened
    PortOpened { port_id: String },
    /// Port closed
    PortClosed { port_id: String },
    /// Error occurred
    Error { port_id: String, error: String },
}

/// I/O loop configuration
#[derive(Debug, Clone)]
pub struct IoLoopConfig {
    /// Buffer size for each port
    pub buffer_size: usize,
    /// Read timeout in milliseconds
    pub read_timeout_ms: u64,
    /// Event channel capacity
    pub event_channel_size: usize,
}

impl Default for IoLoopConfig {
    fn default() -> Self {
        Self {
            buffer_size: 4096,
            read_timeout_ms: 100,
            event_channel_size: 100,
        }
    }
}

/// Async I/O loop
pub struct IoLoop {
    config: IoLoopConfig,
    port_manager: PortManager,
    event_tx: mpsc::Sender<IoEvent>,
    event_rx: Option<mpsc::Receiver<IoEvent>>,
    active_ports: Arc<Mutex<HashMap<String, bool>>>,
    io_task_handle: Option<JoinHandle<()>>,
    shutdown_signal: Option<mpsc::Receiver<()>>,
}

impl IoLoop {
    /// Create a new I/O loop
    pub fn new() -> Self {
        let config = IoLoopConfig::default();
        Self::with_config(config)
    }

    /// Create a new I/O loop with custom configuration
    pub fn with_config(config: IoLoopConfig) -> Self {
        let (event_tx, event_rx) = mpsc::channel(config.event_channel_size);
        let (_, shutdown_rx) = mpsc::channel(1);

        Self {
            config,
            port_manager: PortManager::new(),
            event_tx,
            event_rx: Some(event_rx),
            active_ports: Arc::new(Mutex::new(HashMap::new())),
            io_task_handle: None,
            shutdown_signal: Some(shutdown_rx),
        }
    }

    /// Get a channel sender to subscribe to events
    pub fn event_sender(&self) -> mpsc::Sender<IoEvent> {
        self.event_tx.clone()
    }

    /// Add a port to the I/O loop
    pub async fn add_port(&self, port_name: &str) -> Result<String> {
        use crate::serial_core::SerialConfig;

        let config = SerialConfig::default();
        let port_id = self.port_manager.open_port(port_name, config).await?;

        // Mark as active
        let mut ports = self.active_ports.lock().await;
        ports.insert(port_id.clone(), true);

        // Send event
        let _ = self
            .event_tx
            .send(IoEvent::PortOpened {
                port_id: port_id.clone(),
            })
            .await;

        Ok(port_id)
    }

    /// Remove a port from the I/O loop
    pub async fn remove_port(&self, port_id: &str) -> Result<()> {
        self.port_manager.close_port(port_id).await?;

        // Mark as inactive
        let mut ports = self.active_ports.lock().await;
        ports.remove(port_id);

        // Send event
        let _ = self
            .event_tx
            .send(IoEvent::PortClosed {
                port_id: port_id.to_string(),
            })
            .await;

        Ok(())
    }

    /// Run the I/O loop
    pub async fn run(&mut self) -> Result<()> {
        let mut event_rx = self.event_rx.take().ok_or_else(|| {
            SerialError::Io(std::io::Error::other("Event receiver already taken"))
        })?;

        let mut shutdown_rx = self.shutdown_signal.take().ok_or_else(|| {
            SerialError::Io(std::io::Error::other("Shutdown receiver already taken"))
        })?;

        // Spawn I/O tasks for each active port
        let active_ports = self.active_ports.clone();
        let port_manager = self.port_manager.clone();
        let event_tx = self.event_tx.clone();
        let config = self.config.clone();

        // I/O task with shutdown support
        let io_task: JoinHandle<()> = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(config.read_timeout_ms));

            // Pre-allocate a reusable buffer per port (pool pattern)
            let mut port_buffers: HashMap<String, BytesMut> = HashMap::new();

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        // Get list of active ports
                        let ports = {
                            let ports_guard = active_ports.lock().await;
                            ports_guard.keys().cloned().collect::<Vec<_>>()
                        };

                        // Try to read from each port
                        for port_id in ports {
                            let port_handle = match port_manager.get_port(&port_id).await {
                                Ok(handle) => handle,
                                Err(_) => continue,
                            };

                            // Get or create a reusable buffer for this port
                            let buffer = port_buffers
                                .entry(port_id.clone())
                                .or_insert_with(|| BytesMut::with_capacity(config.buffer_size));

                            // Ensure buffer has enough capacity
                            buffer.resize(config.buffer_size, 0);

                            let mut handle = port_handle.lock().await;

                            // Non-blocking read with timeout
                            match timeout(Duration::from_millis(10), async {
                                handle.read(buffer.as_mut())
                            })
                            .await
                            {
                                Ok(Ok(n)) if n > 0 => {
                                    buffer.truncate(n);
                                    // Split to get owned BytesMut without copying
                                    let data = buffer.split();

                                    let _ = event_tx
                                        .send(IoEvent::DataReceived {
                                            port_id: port_id.clone(),
                                            data,
                                        })
                                        .await;
                                }
                                Ok(Ok(_)) => {
                                    // No data available
                                }
                                Ok(Err(e)) => {
                                    let _ = event_tx
                                        .send(IoEvent::Error {
                                            port_id: port_id.clone(),
                                            error: format!("{:?}", e),
                                        })
                                        .await;
                                }
                                Err(_) => {
                                    // Timeout - no data available
                                }
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        // Shutdown signal received
                        tracing::info!("IoLoop shutdown signal received");
                        break;
                    }
                }
            }
        });

        self.io_task_handle = Some(io_task);

        // Event processing loop
        while let Some(event) = event_rx.recv().await {
            match event {
                IoEvent::DataReceived { port_id, data } => {
                    tracing::debug!("Received {} bytes from {}", data.len(), port_id);
                }
                IoEvent::DataSent { port_id, length } => {
                    tracing::debug!("Sent {} bytes to {}", length, port_id);
                }
                IoEvent::PortOpened { port_id } => {
                    tracing::info!("Port opened: {}", port_id);
                }
                IoEvent::PortClosed { port_id } => {
                    tracing::info!("Port closed: {}", port_id);
                }
                IoEvent::Error { port_id, error } => {
                    tracing::error!("Error on port {}: {}", port_id, error);
                }
            }
        }

        // Clean up I/O task
        if let Some(handle) = self.io_task_handle.take() {
            handle.abort();
        }

        Ok(())
    }

    /// Shutdown the I/O loop gracefully
    pub async fn shutdown(&mut self) -> Result<()> {
        // Abort I/O task if still running
        if let Some(handle) = self.io_task_handle.take() {
            handle.abort();
            let _ = timeout(Duration::from_secs(5), handle).await;
        }

        tracing::info!("IoLoop shutdown complete");
        Ok(())
    }

    /// Check if the I/O loop is running
    pub fn is_running(&self) -> bool {
        self.io_task_handle.is_some()
    }

    /// Write data to a port
    pub async fn write(&self, port_id: &str, data: &[u8]) -> Result<()> {
        let port_handle = self.port_manager.get_port(port_id).await?;
        let mut handle = port_handle.lock().await;

        let bytes_written = handle.write(data)?;

        // Send event
        let _ = self
            .event_tx
            .send(IoEvent::DataSent {
                port_id: port_id.to_string(),
                length: bytes_written,
            })
            .await;

        Ok(())
    }

    /// Read data from a port (blocking)
    pub async fn read(&self, port_id: &str, buf: &mut [u8]) -> Result<usize> {
        let port_handle = self.port_manager.get_port(port_id).await?;
        let mut handle = port_handle.lock().await;

        let bytes_read = handle.read(buf)?;

        Ok(bytes_read)
    }
}

impl Default for IoLoop {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serial_core::backends::mock::MockSerialPort;
    use crate::serial_core::{SerialConfig, SerialPortHandle};

    #[test]
    fn test_io_loop_creation() {
        let io_loop = IoLoop::new();
        assert!(io_loop.event_rx.is_some());
        assert!(!io_loop.is_running());
    }

    #[test]
    fn test_io_loop_config_default() {
        let config = IoLoopConfig::default();
        assert_eq!(config.buffer_size, 4096);
        assert_eq!(config.read_timeout_ms, 100);
        assert_eq!(config.event_channel_size, 100);
    }

    #[tokio::test]
    async fn test_event_channel() {
        let mut io_loop = IoLoop::new();
        let mut rx = io_loop.event_rx.take().unwrap();

        let _ = io_loop
            .event_tx
            .send(IoEvent::PortOpened {
                port_id: "test".to_string(),
            })
            .await;

        let event = rx.recv().await.unwrap();
        match event {
            IoEvent::PortOpened { port_id } => {
                assert_eq!(port_id, "test");
            }
            _ => panic!("Unexpected event"),
        }
    }

    #[tokio::test]
    async fn test_ioloop_lifecycle() {
        let mut io_loop = IoLoop::new();
        assert!(!io_loop.is_running());
        assert!(io_loop.event_rx.is_some());

        let tx = io_loop.event_sender();
        let result = tx
            .send(IoEvent::PortOpened {
                port_id: "lifecycle_test".to_string(),
            })
            .await;
        assert!(result.is_ok());

        let shutdown_result = io_loop.shutdown().await;
        assert!(shutdown_result.is_ok());
        assert!(!io_loop.is_running());
    }

    // ── Mock-based data flow tests ──────────────────────────────────────

    #[tokio::test]
    async fn test_write_to_mock_port() {
        let mock = MockSerialPort::empty();
        let write_capture = mock.write_capture_ref();
        let handle = SerialPortHandle::new_with_port(
            "mock-tty".to_string(),
            Box::new(mock),
            SerialConfig::default(),
        );

        let io_loop = IoLoop::new();
        let port_id = io_loop.port_manager.insert_handle(handle).await;

        // Write through IoLoop
        io_loop.write(&port_id, b"ATZ\r\n").await.unwrap();

        // Verify mock captured the write
        let written = write_capture.lock().unwrap();
        assert_eq!(*written, b"ATZ\r\n");
    }

    #[tokio::test]
    async fn test_read_from_mock_port() {
        let mock = MockSerialPort::with_read_data(b"Hello from device");
        let handle = SerialPortHandle::new_with_port(
            "mock-tty".to_string(),
            Box::new(mock),
            SerialConfig::default(),
        );

        let io_loop = IoLoop::new();
        let port_id = io_loop.port_manager.insert_handle(handle).await;

        // Read through IoLoop
        let mut buf = [0u8; 64];
        let n = io_loop.read(&port_id, &mut buf).await.unwrap();
        assert_eq!(&buf[..n], b"Hello from device");
    }

    #[tokio::test]
    async fn test_write_then_read_roundtrip() {
        // Simulate: write a command, then read the response
        let mock = MockSerialPort::with_read_data(b"OK\r\n");
        let write_capture = mock.write_capture_ref();
        let handle = SerialPortHandle::new_with_port(
            "mock-tty".to_string(),
            Box::new(mock),
            SerialConfig::default(),
        );

        let io_loop = IoLoop::new();
        let port_id = io_loop.port_manager.insert_handle(handle).await;

        // Write command
        io_loop.write(&port_id, b"AT\r\n").await.unwrap();
        assert_eq!(*write_capture.lock().unwrap(), b"AT\r\n");

        // Read response
        let mut buf = [0u8; 64];
        let n = io_loop.read(&port_id, &mut buf).await.unwrap();
        assert_eq!(&buf[..n], b"OK\r\n");
    }

    #[tokio::test]
    async fn test_read_empty_port_returns_timeout() {
        let mock = MockSerialPort::empty();
        let handle = SerialPortHandle::new_with_port(
            "mock-tty".to_string(),
            Box::new(mock),
            SerialConfig::default(),
        );

        let io_loop = IoLoop::new();
        let port_id = io_loop.port_manager.insert_handle(handle).await;

        // Read from empty port should timeout
        let mut buf = [0u8; 64];
        let result = io_loop.read(&port_id, &mut buf).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_write_to_nonexistent_port() {
        let io_loop = IoLoop::new();
        let result = io_loop.write("nonexistent", b"data").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_read_from_nonexistent_port() {
        let io_loop = IoLoop::new();
        let mut buf = [0u8; 64];
        let result = io_loop.read("nonexistent", &mut buf).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_multiple_mock_ports() {
        let mock1 = MockSerialPort::with_read_data(b"port1 data");
        let mock2 = MockSerialPort::with_read_data(b"port2 data");

        let handle1 = SerialPortHandle::new_with_port(
            "mock-1".to_string(),
            Box::new(mock1),
            SerialConfig::default(),
        );
        let handle2 = SerialPortHandle::new_with_port(
            "mock-2".to_string(),
            Box::new(mock2),
            SerialConfig::default(),
        );

        let io_loop = IoLoop::new();
        let id1 = io_loop.port_manager.insert_handle(handle1).await;
        let id2 = io_loop.port_manager.insert_handle(handle2).await;

        // Read from both ports
        let mut buf = [0u8; 64];
        let n1 = io_loop.read(&id1, &mut buf).await.unwrap();
        assert_eq!(&buf[..n1], b"port1 data");

        let n2 = io_loop.read(&id2, &mut buf).await.unwrap();
        assert_eq!(&buf[..n2], b"port2 data");
    }

    #[tokio::test]
    async fn test_write_multiple_commands() {
        let mock = MockSerialPort::empty();
        let write_capture = mock.write_capture_ref();
        let handle = SerialPortHandle::new_with_port(
            "mock-tty".to_string(),
            Box::new(mock),
            SerialConfig::default(),
        );

        let io_loop = IoLoop::new();
        let port_id = io_loop.port_manager.insert_handle(handle).await;

        // Write multiple commands
        io_loop.write(&port_id, b"AT\r\n").await.unwrap();
        io_loop.write(&port_id, b"ATI\r\n").await.unwrap();
        io_loop.write(&port_id, b"ATZ\r\n").await.unwrap();

        let written = write_capture.lock().unwrap();
        assert_eq!(*written, b"AT\r\nATI\r\nATZ\r\n");
    }

    #[tokio::test]
    async fn test_push_data_after_creation() {
        // Simulate data arriving after the port was opened
        let mock = MockSerialPort::empty();
        let mock_ref = mock.read_buffer_ref();
        let handle = SerialPortHandle::new_with_port(
            "mock-tty".to_string(),
            Box::new(mock),
            SerialConfig::default(),
        );

        let io_loop = IoLoop::new();
        let port_id = io_loop.port_manager.insert_handle(handle).await;

        // Push data after port creation
        mock_ref.lock().unwrap().extend_from_slice(b"delayed data");

        let mut buf = [0u8; 64];
        let n = io_loop.read(&port_id, &mut buf).await.unwrap();
        assert_eq!(&buf[..n], b"delayed data");
    }

    #[test]
    fn test_io_event_variants() {
        // Verify all IoEvent variants can be constructed
        let events = vec![
            IoEvent::DataReceived {
                port_id: "p1".to_string(),
                data: BytesMut::from(b"hello".as_slice()),
            },
            IoEvent::DataSent {
                port_id: "p1".to_string(),
                length: 5,
            },
            IoEvent::PortOpened {
                port_id: "p1".to_string(),
            },
            IoEvent::PortClosed {
                port_id: "p1".to_string(),
            },
            IoEvent::Error {
                port_id: "p1".to_string(),
                error: "test error".to_string(),
            },
        ];
        assert_eq!(events.len(), 5);
    }

    #[test]
    fn test_io_loop_config_custom() {
        let config = IoLoopConfig {
            buffer_size: 8192,
            read_timeout_ms: 200,
            event_channel_size: 50,
        };
        assert_eq!(config.buffer_size, 8192);
        assert_eq!(config.read_timeout_ms, 200);
        assert_eq!(config.event_channel_size, 50);
    }
}
