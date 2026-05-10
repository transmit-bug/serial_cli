//! Virtual Serial Port Demo
//!
//! Demonstrates creating a pair of virtual serial ports using PTY backend.
//! The created slave devices (e.g., /dev/ttys015, /dev/ttys017) are registered
//! as system character devices and can be opened by any application.
//!
//! Run with:
//!   cargo run --example virtual_port_demo
//!
//! Unix/macOS only (requires PTY support).

use serial_cli::serial_core::{VirtualConfig, VirtualSerialPair};
use std::fs;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for debug output
    tracing_subscriber::fmt()
        .with_env_filter("serial_cli::serial_core::virtual_port=debug")
        .with_target(false)
        .init();

    println!("=== Virtual Serial Port Demo ===\n");

    // ──────────────────────────────────────────────
    // Step 1: Create a virtual serial port pair
    // ──────────────────────────────────────────────
    println!("Step 1: Creating virtual serial port pair...");

    let config = VirtualConfig::default();
    let pair = VirtualSerialPair::create(config).await?;

    println!("  Virtual pair created!");
    println!("  ID:      {}", pair.id);
    println!(
        "  Port A:  {} (system-registered character device)",
        pair.port_a
    );
    println!(
        "  Port B:  {} (system-registered character device)",
        pair.port_b
    );
    println!("  Backend: PTY (POSIX pseudo-terminal)");
    println!("  Running: {}\n", pair.is_running());

    // ──────────────────────────────────────────────
    // Step 2: Verify the devices exist on the system
    // ──────────────────────────────────────────────
    println!("Step 2: Verifying system device registration...");

    let metadata_a = fs::metadata(&pair.port_a);
    let metadata_b = fs::metadata(&pair.port_b);

    match (&metadata_a, &metadata_b) {
        (Ok(_), Ok(_)) => {
            println!("  Port A: {} exists", pair.port_a);
            println!("  Port B: {} exists", pair.port_b);
            println!("  Both devices are registered on the system.\n");
        }
        _ => {
            println!("  Warning: one or both devices not found via fs::metadata\n");
        }
    }

    // ──────────────────────────────────────────────
    // Step 3: Demonstrate external accessibility
    // ──────────────────────────────────────────────
    println!("Step 3: Using these virtual ports externally");
    println!();
    println!("  These PTY slave devices can be opened by any application:");
    println!();
    println!("  # Terminal 1 - listen on Port A:");
    println!("  $ cat {}", pair.port_a);
    println!();
    println!("  # Terminal 2 - send data to Port B:");
    println!("  $ echo 'hello' > {}", pair.port_b);
    println!();
    println!("  Data written to Port A will be forwarded to Port B by the bridge,");
    println!("  and vice versa, enabling bidirectional communication testing.");
    println!();
    println!("  # Or use with socat for a manual bridge test:");
    println!("  $ socat - {}", pair.port_a);

    // Small delay to let bridge task accumulate stats
    tokio::time::sleep(Duration::from_millis(50)).await;

    // ──────────────────────────────────────────────
    // Step 4: Show statistics
    // ──────────────────────────────────────────────
    println!("\nStep 4: Virtual port statistics...");

    let stats = pair.stats().await;
    println!("  Uptime:          {} seconds", stats.uptime_secs);
    println!("  Bytes bridged:   {}", stats.bytes_bridged);
    println!("  Packets bridged: {}", stats.packets_bridged);
    println!("  Bridge errors:   {}", stats.bridge_errors);

    // ──────────────────────────────────────────────
    // Step 5: Clean shutdown
    // ──────────────────────────────────────────────
    println!("\nStep 5: Stopping virtual port pair...");

    pair.stop().await?;
    println!("  Virtual pair stopped and resources released.\n");

    println!("=== Demo completed successfully ===");
    Ok(())
}
