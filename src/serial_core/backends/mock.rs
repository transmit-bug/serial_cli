//! Mock serial port for testing
//!
//! Provides a configurable mock implementation of [`serialport::SerialPort`]
//! that can be used to test [`SerialPortHandle`](super::super::port::SerialPortHandle)
//! and other I/O-dependent code without a physical serial port.

use serialport::{ClearBuffer, DataBits, FlowControl, Parity, SerialPort, StopBits};
use std::io::{self, Read, Write};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Configurable mock serial port for testing.
///
/// Uses internal buffers to simulate serial I/O:
/// - `read_buffer`: data returned by `read()` calls
/// - `write_capture`: data captured from `write()` / `write_all()` calls
///
/// # Example
///
/// ```
/// use serial_cli::serial_core::backends::mock::MockSerialPort;
/// use std::io::Read;
///
/// let mock = MockSerialPort::builder()
///     .with_read_data(b"Hello, World!")
///     .build();
///
/// let mut buf = [0u8; 32];
/// let n = (&mock).read(&mut buf).unwrap();
/// assert_eq!(&buf[..n], b"Hello, World!");
/// ```
pub struct MockSerialPort {
    read_buffer: Arc<Mutex<Vec<u8>>>,
    write_capture: Arc<Mutex<Vec<u8>>>,
    timeout: Duration,
    /// If set, read() will return WouldBlock/Timeout after this many bytes
    read_limit: Option<usize>,
    bytes_read: Arc<Mutex<usize>>,
}

/// Builder for [`MockSerialPort`] with fluent API.
pub struct MockSerialPortBuilder {
    read_data: Vec<u8>,
    timeout: Duration,
    read_limit: Option<usize>,
}

impl MockSerialPortBuilder {
    /// Set the data that `read()` will return.
    pub fn with_read_data(mut self, data: &[u8]) -> Self {
        self.read_data = data.to_vec();
        self
    }

    /// Build the mock port.
    pub fn build(self) -> MockSerialPort {
        MockSerialPort {
            read_buffer: Arc::new(Mutex::new(self.read_data)),
            write_capture: Arc::new(Mutex::new(Vec::new())),
            timeout: self.timeout,
            read_limit: self.read_limit,
            bytes_read: Arc::new(Mutex::new(0)),
        }
    }
}

impl MockSerialPort {
    /// Create a builder for configuring the mock port.
    pub fn builder() -> MockSerialPortBuilder {
        MockSerialPortBuilder {
            read_data: Vec::new(),
            timeout: Duration::from_millis(1000),
            read_limit: None,
        }
    }

    /// Create a mock port with pre-loaded read data.
    pub fn with_read_data(data: &[u8]) -> Self {
        Self::builder().with_read_data(data).build()
    }

    /// Create an empty mock port (no data to read).
    pub fn empty() -> Self {
        Self::builder().build()
    }

    /// Get all data written to this port via `write()` / `write_all()`.
    pub fn written_data(&self) -> Vec<u8> {
        self.write_capture.lock().unwrap().clone()
    }

    /// Get the number of bytes written to this port.
    pub fn written_len(&self) -> usize {
        self.write_capture.lock().unwrap().len()
    }

    /// Get a reference to the write capture for shared verification.
    pub fn write_capture_ref(&self) -> Arc<Mutex<Vec<u8>>> {
        self.write_capture.clone()
    }

    /// Get a reference to the read buffer for shared access.
    pub fn read_buffer_ref(&self) -> Arc<Mutex<Vec<u8>>> {
        self.read_buffer.clone()
    }
}

impl Read for MockSerialPort {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut read_buf = self.read_buffer.lock().unwrap();
        let mut bytes_read = self.bytes_read.lock().unwrap();

        if read_buf.is_empty() {
            if self.timeout.is_zero() {
                return Err(io::Error::new(io::ErrorKind::WouldBlock, "no data"));
            }
            std::thread::sleep(Duration::from_millis(1));
            return Err(io::Error::new(io::ErrorKind::TimedOut, "read timeout"));
        }

        // Check read limit first
        if let Some(limit) = self.read_limit {
            if *bytes_read >= limit {
                return Err(io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "read limit reached",
                ));
            }
            let remaining = limit - *bytes_read;
            let to_read = remaining.min(read_buf.len()).min(buf.len());
            buf[..to_read].copy_from_slice(&read_buf[..to_read]);
            read_buf.drain(..to_read);
            *bytes_read += to_read;
            return Ok(to_read);
        }

        let to_read = read_buf.len().min(buf.len());
        buf[..to_read].copy_from_slice(&read_buf[..to_read]);
        read_buf.drain(..to_read);
        *bytes_read += to_read;

        Ok(to_read)
    }
}

impl Read for &MockSerialPort {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut read_buf = self.read_buffer.lock().unwrap();
        let mut bytes_read = self.bytes_read.lock().unwrap();

        if read_buf.is_empty() {
            if self.timeout.is_zero() {
                return Err(io::Error::new(io::ErrorKind::WouldBlock, "no data"));
            }
            std::thread::sleep(Duration::from_millis(1));
            return Err(io::Error::new(io::ErrorKind::TimedOut, "read timeout"));
        }

        if let Some(limit) = self.read_limit {
            if *bytes_read >= limit {
                return Err(io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "read limit reached",
                ));
            }
            let remaining = limit - *bytes_read;
            let to_read = remaining.min(read_buf.len()).min(buf.len());
            buf[..to_read].copy_from_slice(&read_buf[..to_read]);
            read_buf.drain(..to_read);
            *bytes_read += to_read;
            return Ok(to_read);
        }

        let to_read = read_buf.len().min(buf.len());
        buf[..to_read].copy_from_slice(&read_buf[..to_read]);
        read_buf.drain(..to_read);
        *bytes_read += to_read;

        Ok(to_read)
    }
}

impl Write for MockSerialPort {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write_capture.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Write for &MockSerialPort {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write_capture.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl SerialPort for MockSerialPort {
    fn name(&self) -> Option<String> {
        Some("mock-port".to_string())
    }

    fn baud_rate(&self) -> serialport::Result<u32> {
        Ok(115200)
    }

    fn data_bits(&self) -> serialport::Result<DataBits> {
        Ok(DataBits::Eight)
    }

    fn flow_control(&self) -> serialport::Result<FlowControl> {
        Ok(FlowControl::None)
    }

    fn parity(&self) -> serialport::Result<Parity> {
        Ok(Parity::None)
    }

    fn stop_bits(&self) -> serialport::Result<StopBits> {
        Ok(StopBits::One)
    }

    fn timeout(&self) -> Duration {
        self.timeout
    }

    fn set_baud_rate(&mut self, _baud_rate: u32) -> serialport::Result<()> {
        Ok(())
    }

    fn set_data_bits(&mut self, _data_bits: DataBits) -> serialport::Result<()> {
        Ok(())
    }

    fn set_flow_control(&mut self, _flow_control: FlowControl) -> serialport::Result<()> {
        Ok(())
    }

    fn set_parity(&mut self, _parity: Parity) -> serialport::Result<()> {
        Ok(())
    }

    fn set_stop_bits(&mut self, _stop_bits: StopBits) -> serialport::Result<()> {
        Ok(())
    }

    fn set_timeout(&mut self, timeout: Duration) -> serialport::Result<()> {
        self.timeout = timeout;
        Ok(())
    }

    fn write_request_to_send(&mut self, _level: bool) -> serialport::Result<()> {
        Ok(())
    }

    fn write_data_terminal_ready(&mut self, _level: bool) -> serialport::Result<()> {
        Ok(())
    }

    fn read_clear_to_send(&mut self) -> serialport::Result<bool> {
        Ok(false)
    }

    fn read_data_set_ready(&mut self) -> serialport::Result<bool> {
        Ok(false)
    }

    fn read_ring_indicator(&mut self) -> serialport::Result<bool> {
        Ok(false)
    }

    fn read_carrier_detect(&mut self) -> serialport::Result<bool> {
        Ok(false)
    }

    fn bytes_to_read(&self) -> serialport::Result<u32> {
        Ok(self.read_buffer.lock().unwrap().len() as u32)
    }

    fn bytes_to_write(&self) -> serialport::Result<u32> {
        Ok(0)
    }

    fn clear(&self, buffer_to_clear: ClearBuffer) -> serialport::Result<()> {
        match buffer_to_clear {
            ClearBuffer::Input | ClearBuffer::All => {
                self.read_buffer.lock().unwrap().clear();
            }
            ClearBuffer::Output => {}
        }
        Ok(())
    }

    fn try_clone(&self) -> serialport::Result<Box<dyn SerialPort>> {
        Ok(Box::new(MockSerialPort {
            read_buffer: self.read_buffer.clone(),
            write_capture: self.write_capture.clone(),
            timeout: self.timeout,
            read_limit: self.read_limit,
            bytes_read: self.bytes_read.clone(),
        }))
    }

    fn set_break(&self) -> serialport::Result<()> {
        Ok(())
    }

    fn clear_break(&self) -> serialport::Result<()> {
        Ok(())
    }
}

// Safety: MockSerialPort uses Arc<Mutex> for all shared state.
unsafe impl Send for MockSerialPort {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_read() {
        let mock = MockSerialPort::with_read_data(b"Hello");
        let mut buf = [0u8; 32];

        // Read through &mock (ref impl)
        let n = (&mock).read(&mut buf).unwrap();
        assert_eq!(&buf[..n], b"Hello");
    }

    #[test]
    fn test_mock_read_empty_returns_timeout() {
        let mock = MockSerialPort::empty();
        let mut buf = [0u8; 32];

        let result = (&mock).read(&mut buf);
        assert!(result.is_err());
    }

    #[test]
    fn test_mock_write_capture() {
        let mut mock = MockSerialPort::empty();
        mock.write_all(b"ATZ\r\n").unwrap();
        assert_eq!(mock.written_data(), b"ATZ\r\n");
    }

    #[test]
    fn test_mock_write_capture_multiple() {
        let mut mock = MockSerialPort::empty();
        mock.write_all(b"AT").unwrap();
        mock.write_all(b"Z\r\n").unwrap();
        assert_eq!(mock.written_data(), b"ATZ\r\n");
        assert_eq!(mock.written_len(), 5);
    }


    #[test]
    fn test_mock_clear_buffer() {
        let mock = MockSerialPort::with_read_data(b"data");
        mock.clear(ClearBuffer::Input).unwrap();

        let mut buf = [0u8; 32];
        let result = (&mock).read(&mut buf);
        assert!(result.is_err()); // Should timeout since buffer was cleared
    }

    #[test]
    fn test_mock_serial_port_trait_methods() {
        let mock = MockSerialPort::empty();
        assert_eq!(mock.name(), Some("mock-port".to_string()));
        assert_eq!(mock.baud_rate().unwrap(), 115200);
        assert_eq!(mock.data_bits().unwrap(), DataBits::Eight);
        assert_eq!(mock.timeout(), Duration::from_millis(1000));
    }

    #[test]
    fn test_mock_set_timeout() {
        let mut mock = MockSerialPort::empty();
        mock.set_timeout(Duration::from_millis(500)).unwrap();
        assert_eq!(mock.timeout(), Duration::from_millis(500));
    }

    #[test]
    fn test_mock_builder() {
        let mock = MockSerialPort::builder()
            .with_read_data(b"test")
            .build();


        let mut buf = [0u8; 32];
        let n = (&mock).read(&mut buf).unwrap();
        assert_eq!(&buf[..n], b"test");
    }

    #[test]
    fn test_mock_try_clone() {
        let mock = MockSerialPort::with_read_data(b"shared");
        let cloned = mock.try_clone().unwrap();

        // Cloned port shares the same buffers
        let mut buf = [0u8; 32];
        let n = cloned.take(32).read(&mut buf).unwrap();
        assert_eq!(&buf[..n], b"shared");
    }


}
