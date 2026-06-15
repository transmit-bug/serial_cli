//! Serial core module
//!
//! This module provides serial port management and I/O operations.

pub mod backends;
pub mod factory;
pub mod io_loop;
pub mod port;
pub mod serial_script;
pub mod signals;
pub mod sniffer;
pub mod virtual_port;

#[cfg(windows)]
pub mod windows_signals;

pub use backends::BackendType;
pub use factory::BackendFactory;
pub use io_loop::IoLoop;
pub use port::{FlowControl, Parity, PortManager, SerialConfig, SerialPortHandle, SerialPortInfo};
pub use serial_script::SerialScriptEngine;
pub use signals::{create_signal_controller, PlatformSignals, SignalState};
pub use sniffer::{CapturedPacket, PacketDirection, SerialSniffer, SnifferConfig, SnifferSession};
pub use virtual_port::{
    CapturedPacket as VirtualCapturedPacket, PacketCapture,
    PacketDirection as VirtualPacketDirection, VirtualConfig, VirtualSerialPair, VirtualStats,
};

#[cfg(windows)]
pub use windows_signals::WindowsSignalControl;
