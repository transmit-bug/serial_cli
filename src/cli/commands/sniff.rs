//! Sniff command handler
//!
//! Handles `serial-cli sniff start|stop|stats|save`.
//! Spawns a background daemon process for continuous port monitoring.

use std::time::Duration;

use crate::cli::sniff_session::{
    get_session_stats, read_captured_packets, spawn_sniff_daemon, stop_active_session,
};
use crate::cli::types::SniffCommand;
use crate::error::{Result, SerialError};

/// Dispatch a [`SniffCommand`] to start, stop, show stats, or save captures.
///
/// The `Start` variant spawns a background daemon process that monitors the
/// serial port. A session metadata file tracks the daemon PID for later
/// `stop` and `stats` commands.
///
/// # Errors
///
/// Returns [`SerialError::Io`] if:
/// - An active session is already running
/// - Session metadata cannot be read or written
/// - Captured data files cannot be accessed
pub async fn handle_sniff_command(cmd: SniffCommand, json_output: bool) -> Result<()> {
    match cmd {
        SniffCommand::Start {
            port,
            output,
            max_packets,
            display,
            format: display_format,
        } => {
            // Check if there's already an active session
            if let Ok(Some(meta)) = crate::cli::sniff_session::load_session() {
                if crate::cli::sniff_session::is_process_running(meta.pid) {
                    return Err(SerialError::Io(std::io::Error::new(
                        std::io::ErrorKind::AlreadyExists,
                        format!(
                            "An active sniff session is already running on port '{}' (PID {}). Use 'sniff stop' first.",
                            meta.port, meta.pid
                        ),
                    )));
                } else {
                    // Stale session — clean up
                    crate::cli::sniff_session::clear_session()?;
                }
            }

            // Spawn background daemon process
            let meta = spawn_sniff_daemon(
                &port,
                output.as_deref(),
                max_packets,
                display_format == "hex",
            )?;

            println!(
                "✓ Sniffing started on port: {} (PID: {})",
                meta.port, meta.pid
            );
            if display {
                println!("  Real-time display enabled");
            }
            if let Some(ref p) = meta.output_path {
                println!("  Output file: {}", p.display());
            }
            let max_packets_str = if meta.max_packets == 0 {
                "unlimited".to_string()
            } else {
                meta.max_packets.to_string()
            };
            println!("  Max packets:  {}", max_packets_str);
            println!("  Use 'sniff stats' to view statistics");
            println!("  Use 'sniff stop' to stop sniffing");
        }
        SniffCommand::Stop => {
            stop_active_session()?;
        }
        SniffCommand::Stats => {
            let meta = get_session_stats()?;
            let elapsed = Duration::from_secs(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    - meta.started_at,
            );

            let max_packets_display = if meta.max_packets == 0 {
                "unlimited".to_string()
            } else {
                meta.max_packets.to_string()
            };
            println!("Sniff session statistics:");
            println!("  Port:         {}", meta.port);
            println!("  PID:          {}", meta.pid);
            println!("  Started:      {} ago", format_duration(elapsed));
            println!("  Max packets:  {}", max_packets_display);
            println!("  Hex display:  {}", meta.hex_display);

            if let Some(ref output) = meta.output_path {
                if output.exists() {
                    let content = read_captured_packets(output)?;
                    let line_count = content.lines().count();
                    println!(
                        "  Output file:  {} ({} lines)",
                        output.display(),
                        line_count
                    );
                } else {
                    println!("  Output file:  {} (not yet written)", output.display());
                }
            }
        }
        SniffCommand::Save { path } => {
            let meta = get_session_stats()?;

            // Try to read from the session's output file if one exists
            if let Some(ref session_output) = meta.output_path {
                if session_output.exists() {
                    let content = read_captured_packets(session_output)?;
                    std::fs::write(&path, content).map_err(SerialError::Io)?;
                    println!("✓ Captured packets saved to: {}", path.display());
                } else {
                    return Err(SerialError::Io(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "No captured data available yet",
                    )));
                }
            } else {
                return Err(SerialError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Session has no output file configured. Restart with --output to enable saving.",
                )));
            }
        }
    }
    Ok(())
}

/// Format a [`Duration`] into a human-readable string (e.g., `30s`, `5m 12s`, `1h 30m`).
fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    }
}
