//! Serial read pump — the blocking read loop that pulls bytes off a
//! [`SerialPortHandle`] and forwards them over an mpsc channel.
//!
//! Extracted from the Tauri sniffer (map #85) so the serial pipeline lives in
//! the library. The async consumer (stats, events, GUI) is owned by the caller.
//! This supersedes the old `io_loop.rs` module (deleted in #85): the port's
//! read loop is now one implementation shared by all consumers.

use crate::serial_core::port::SerialPortHandle;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;

/// Spawn the blocking read task for `port_handle`.
///
/// The task loops: check the stop flag, lock the handle for one read, forward
/// non-empty data over `tx`. An empty `Vec` is sent as a **disconnect
/// sentinel** before the loop exits (on broken-pipe / disconnect errors).
/// Closing `tx` also stops the loop (async side shut down).
pub fn spawn_read_pump(
    port_handle: Arc<tokio::sync::Mutex<SerialPortHandle>>,
    tx: Sender<Vec<u8>>,
    stop: Arc<AtomicBool>,
) -> JoinHandle<()> {
    tokio::task::spawn_blocking(move || {
        let mut buffer = vec![0u8; 4096];

        loop {
            // Check stop signal between reads
            if stop.load(Ordering::Relaxed) {
                break;
            }

            // Lock the port handle, do one read, release lock
            // Lock is released after each read so writes can interleave
            let mut handle = port_handle.blocking_lock();
            match handle.read(&mut buffer) {
                Ok(n) if n > 0 => {
                    if tx.blocking_send(buffer[..n].to_vec()).is_err() {
                        // Channel closed — async side shut down
                        break;
                    }
                }
                Ok(_) => {} // 0 bytes (serialport timeout)
                Err(e) => {
                    let msg = e.to_string();
                    if msg.contains("timed out") || msg.contains("timeout") {
                        continue;
                    }
                    if msg.contains("Broken pipe") || msg.contains("disconnected") {
                        tracing::debug!("Port disconnected in read pump");
                        let _ = tx.blocking_send(vec![]); // disconnect sentinel
                        break;
                    }
                    tracing::debug!("Read error in pump: {msg}");
                    // Brief pause before retrying to avoid tight error loop
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
        }
    })
}
