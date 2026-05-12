//! Batch command handler
//!
//! Handles `serial-cli batch run` and `serial-cli batch list`.
//! Supports both single Lua scripts and multi-line batch files.

use crate::cli::batch::{BatchConfig, BatchLine, BatchRunner};
use crate::cli::types::BatchCommand;
use crate::error::{Result, SerialError};

/// Dispatch a [`BatchCommand`] to the appropriate handler.
///
/// # Errors
///
/// Propagates errors from the underlying batch execution or listing logic.
pub async fn handle_batch_command(cmd: BatchCommand, _json_output: bool) -> Result<()> {
    match cmd {
        BatchCommand::Run {
            script,
            concurrent,
            continue_on_error,
            timeout,
        } => {
            println!("Running batch script: {}", script.display());
            println!("Max concurrent tasks: {}", concurrent);
            println!();

            // Check if script exists
            if !script.exists() {
                println!("\u{2717} Batch script not found: {}", script.display());
                return Err(SerialError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Batch script not found",
                )));
            }

            // Create batch configuration
            let config = BatchConfig {
                max_concurrent: concurrent,
                timeout_secs: timeout,
                continue_on_error,
                show_progress: true,
                verbose: false,
            };

            // Create batch runner
            let mut runner = BatchRunner::new(config)?;

            // Check if it's a single script or a batch file
            if script.extension().is_some_and(|e| e == "lua") {
                // Single Lua script
                println!("Executing single script...");

                match runner.run_script(&script).await {
                    Ok(_) => {
                        println!("\u{2713} Script executed successfully");
                    }
                    Err(e) => {
                        println!("\u{2717} Script execution failed: {}", e);
                        return Err(e);
                    }
                }
            } else {
                // Assume it's a batch file
                println!("Parsing batch file...");

                let lines = runner.parse_batch_file(&script)?;

                // Report what was parsed
                let script_count = lines
                    .iter()
                    .filter(|l| matches!(l, BatchLine::Script(_)))
                    .count();
                let set_count = lines
                    .iter()
                    .filter(|l| matches!(l, BatchLine::Set { .. }))
                    .count();
                let loop_count = lines
                    .iter()
                    .filter(|l| matches!(l, BatchLine::Loop { .. }))
                    .count();
                println!(
                    "Found {} scripts, {} variable assignments, {} loops",
                    script_count, set_count, loop_count
                );
                println!();

                println!("Executing batch script file...");

                match runner.run_batch_lines(lines).await {
                    Ok(result) => {
                        println!();
                        println!("Batch execution completed:");
                        println!("  Total scripts: {}", result.results.len());

                        let successful = result.results.iter().filter(|r| r.success).count();
                        let failed = result.results.len() - successful;

                        println!("  Successful: {}", successful);
                        println!("  Failed: {}", failed);

                        if failed > 0 {
                            println!();
                            println!("Failed scripts:");
                            for r in result.results.iter().filter(|r| !r.success) {
                                if let Some(ref err) = r.error {
                                    println!("  - {} ({})", r.script, err);
                                } else {
                                    println!("  - {}", r.script);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("\u{2717} Batch execution failed: {}", e);
                        return Err(e);
                    }
                }
            }
        }
        BatchCommand::List => {
            println!("Batch scripts:");

            // Search in multiple common locations
            let mut search_dirs: Vec<std::path::PathBuf> =
                vec![std::env::current_dir().unwrap_or_default()];

            if let Some(home) = dirs_or_home() {
                search_dirs.push(home.join(".config").join("serial_cli"));
            } else {
                tracing::warn!("Could not determine home directory — skipping ~/.config/serial_cli batch search");
            }

            let batch_extensions = ["batch", "txt", "lua"];

            let mut found_any = false;
            for dir in &search_dirs {
                if !dir.exists() {
                    continue;
                }

                let entries = match std::fs::read_dir(dir) {
                    Ok(entries) => entries,
                    Err(_) => continue,
                };

                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                            if batch_extensions.contains(&ext) {
                                println!("  \u{2713} {}", path.display());
                                found_any = true;
                            }
                        }
                    }
                }
            }

            if !found_any {
                println!("  No batch scripts found");
                println!();
                println!("Create a batch script file with one Lua script per line:");
                println!("  # Comments start with #");
                println!("  set PORT /dev/ttyUSB0");
                println!("  loop 3");
                println!("    script1.lua");
                println!("    sleep 500");
                println!("  end");
            }
        }
    }
    Ok(())
}

/// Resolve the user's home directory for batch script search paths.
///
/// Uses the `directories` crate with graceful fallback if the home
/// directory cannot be determined.
fn dirs_or_home() -> Option<std::path::PathBuf> {
    directories::BaseDirs::new().map(|d| d.home_dir().to_path_buf())
}
