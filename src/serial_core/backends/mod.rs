//! Virtual backend implementations
//!
//! This module contains all virtual backend implementations including:
//! - PTY backend (Unix/macOS)
//! - NamedPipe backend (Windows)
//! - Socat backend (cross-platform)

mod named_pipe;
#[cfg(unix)]
mod pty;
mod socat;
mod r#trait;

pub use named_pipe::NamedPipeBackend;
#[cfg(unix)]
pub use pty::PtyBackend;
pub use r#trait::{BackendStats, BridgeErrorRx, BridgeStats, VirtualBackend, VirtualPortEnd};
pub use socat::SocatBackend;

/// Virtual backend type selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendType {
    /// Auto-detect based on platform
    Auto,
    /// PTY (pseudo-terminal) - Unix/macOS only
    Pty,
    /// NamedPipe - Windows only
    NamedPipe,
    /// Socat - Cross-platform (requires socat binary)
    Socat,
}

impl std::fmt::Display for BackendType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackendType::Auto => write!(f, "auto"),
            BackendType::Pty => write!(f, "pty"),
            BackendType::NamedPipe => write!(f, "namedpipe"),
            BackendType::Socat => write!(f, "socat"),
        }
    }
}

impl std::str::FromStr for BackendType {
    type Err = crate::error::SerialError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "auto" => Ok(BackendType::Auto),
            "pty" => Ok(BackendType::Pty),
            "namedpipe" | "named-pipe" | "pipe" => Ok(BackendType::NamedPipe),
            "socat" => Ok(BackendType::Socat),
            _ => Err(crate::error::SerialError::VirtualPort(format!(
                "Unknown backend type: {s}. Valid options: auto, pty, namedpipe, socat"
            ))),
        }
    }
}

impl BackendType {
    /// Detect the best backend for the current platform
    pub fn detect() -> Self {
        #[cfg(windows)]
        {
            BackendType::NamedPipe
        }

        #[cfg(not(windows))]
        {
            BackendType::Pty
        }
    }

    /// Check if this backend is available on the current platform
    pub fn is_available(&self) -> bool {
        match self {
            BackendType::Auto => true,
            BackendType::Pty => cfg!(unix),
            BackendType::NamedPipe => cfg!(windows),
            BackendType::Socat => true, // Available everywhere if installed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_type_parsing() {
        assert_eq!("auto".parse::<BackendType>().unwrap(), BackendType::Auto);
        assert_eq!("pty".parse::<BackendType>().unwrap(), BackendType::Pty);
        assert_eq!(
            "namedpipe".parse::<BackendType>().unwrap(),
            BackendType::NamedPipe
        );
        assert_eq!("socat".parse::<BackendType>().unwrap(), BackendType::Socat);

        // Case insensitive
        assert_eq!("PTY".parse::<BackendType>().unwrap(), BackendType::Pty);
        assert_eq!(
            "NamedPipe".parse::<BackendType>().unwrap(),
            BackendType::NamedPipe
        );

        // Invalid
        assert!("invalid".parse::<BackendType>().is_err());
    }

    #[test]
    fn test_backend_type_display() {
        assert_eq!(BackendType::Auto.to_string(), "auto");
        assert_eq!(BackendType::Pty.to_string(), "pty");
        assert_eq!(BackendType::NamedPipe.to_string(), "namedpipe");
        assert_eq!(BackendType::Socat.to_string(), "socat");
    }

    #[test]
    fn test_backend_detection() {
        let detected = BackendType::detect();
        #[cfg(windows)]
        assert_eq!(detected, BackendType::NamedPipe);
        #[cfg(unix)]
        assert_eq!(detected, BackendType::Pty);
    }

    #[test]
    fn test_backend_availability() {
        assert!(BackendType::Auto.is_available());

        let pty = BackendType::Pty;
        assert_eq!(pty.is_available(), cfg!(unix));

        let named_pipe = BackendType::NamedPipe;
        assert_eq!(named_pipe.is_available(), cfg!(windows));

        assert!(BackendType::Socat.is_available());
    }
}
