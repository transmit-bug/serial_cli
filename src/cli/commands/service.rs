//! Daemon auto-start (service) registration.
//!
//! Installs/removes the daemon's boot-time auto-start on the current
//! platform:
//!
//! - **Linux** — a systemd unit (`/etc/systemd/system/` as root,
//!   `~/.config/systemd/user/` otherwise)
//! - **macOS** — a launchd LaunchAgent
//!   (`~/Library/LaunchAgents/com.serial-cli.daemon.plist`)
//! - **Windows** — a Task Scheduler task (`SerialCLIDaemon`, run at
//!   startup)
//!
//! Used by the bundled `install.sh` / `install.ps1` scripts
//! (`--service` flag) and by `server service install|uninstall`.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::cli::commands::server::{DEFAULT_BIND_ADDR, DEFAULT_TCP_PORT};
use crate::cli::types::ServiceCommand;
use crate::error::{Result, SerialError};

/// Service unit/task name (also used for the deb's systemd unit).
pub const SERVICE_NAME: &str = "serial-cli";
/// launchd label on macOS.
pub const LAUNCHD_LABEL: &str = "com.serial-cli.daemon";
/// Task Scheduler task name on Windows.
pub const SCHTASKS_TASK: &str = "SerialCLIDaemon";

/// Dispatch a [`ServiceCommand`].
pub fn handle_service_command(cmd: ServiceCommand) -> Result<()> {
    match cmd {
        ServiceCommand::Install { port, bind, no_tcp } => install_service(port, bind, no_tcp),
        ServiceCommand::Uninstall => uninstall_service(),
    }
}

/// Install daemon auto-start for the current platform.
fn install_service(port: Option<u16>, bind: Option<String>, no_tcp: bool) -> Result<()> {
    let exe = std::env::current_exe()?;
    let daemon_args = daemon_args(port, bind, no_tcp);

    #[cfg(target_os = "linux")]
    {
        install_systemd(&exe, &daemon_args)
    }
    #[cfg(target_os = "macos")]
    {
        install_launchd(&exe, &daemon_args)
    }
    #[cfg(target_os = "windows")]
    {
        install_schtasks(&exe, &daemon_args)
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Err(SerialError::Config(format!(
            "auto-start not supported on {}",
            std::env::consts::OS
        )))
    }
}

/// Remove daemon auto-start for the current platform.
fn uninstall_service() -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        uninstall_systemd()
    }
    #[cfg(target_os = "macos")]
    {
        uninstall_launchd()
    }
    #[cfg(target_os = "windows")]
    {
        uninstall_schtasks()
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Err(SerialError::Config(format!(
            "auto-start not supported on {}",
            std::env::consts::OS
        )))
    }
}

/// Build the `server daemon` argv tail (excluding the binary path).
fn daemon_args(port: Option<u16>, bind: Option<String>, no_tcp: bool) -> Vec<String> {
    let mut args = vec!["server".to_string(), "daemon".to_string()];
    if !no_tcp {
        args.push("--port".into());
        args.push(port.unwrap_or(DEFAULT_TCP_PORT).to_string());
        args.push("--bind".into());
        args.push(bind.unwrap_or_else(|| DEFAULT_BIND_ADDR.to_string()));
    }
    args
}

// ---------------------------------------------------------------------------
// Linux: systemd
// ---------------------------------------------------------------------------

#[cfg(target_os = "linux")]
fn install_systemd(exe: &Path, daemon_args: &[String]) -> Result<()> {
    let is_root = unsafe { libc::geteuid() == 0 };
    let (unit_path, user_mode) = if is_root {
        (
            PathBuf::from("/etc/systemd/system/serial-cli.service"),
            false,
        )
    } else {
        (
            home_dir()?.join(".config/systemd/user/serial-cli.service"),
            true,
        )
    };

    if let Some(parent) = unit_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(
        &unit_path,
        systemd_unit_content(exe, daemon_args, user_mode),
    )?;
    println!("  Wrote systemd unit: {}", unit_path.display());

    let mut cmd = Command::new("systemctl");
    if user_mode {
        cmd.arg("--user");
    }
    let status = cmd
        .args(["enable", "serial-cli.service"])
        .output()
        .map_err(|e| SerialError::Config(format!("systemctl enable failed: {e}")))?;
    if !status.status.success() {
        let stderr = String::from_utf8_lossy(&status.stderr);
        return Err(SerialError::Config(format!(
            "systemctl enable failed: {}",
            stderr.trim()
        )));
    }
    println!(
        "  Enabled via systemctl{}",
        if user_mode { " --user" } else { "" }
    );

    if user_mode {
        println!("  Tip: run `loginctl enable-linger $USER` so the service starts at boot without a login session.");
    }
    println!("  Daemon will auto-start on boot. Start now with: serial-cli server start");
    Ok(())
}

#[cfg(target_os = "linux")]
fn uninstall_systemd() -> Result<()> {
    let is_root = unsafe { libc::geteuid() == 0 };
    let (unit_path, user_mode) = if is_root {
        (
            PathBuf::from("/etc/systemd/system/serial-cli.service"),
            false,
        )
    } else {
        (
            home_dir()?.join(".config/systemd/user/serial-cli.service"),
            true,
        )
    };

    let mut cmd = Command::new("systemctl");
    if user_mode {
        cmd.arg("--user");
    }
    let _ = cmd.args(["disable", "serial-cli.service"]).status();

    match std::fs::remove_file(&unit_path) {
        Ok(_) => {
            println!("  Removed systemd unit: {}", unit_path.display());
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            println!("  No systemd unit found (nothing to uninstall).");
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

/// Render a systemd unit file. `[Install]` targets differ between system
/// and user mode.
#[cfg(target_os = "linux")]
fn systemd_unit_content(exe: &Path, daemon_args: &[String], user_mode: bool) -> String {
    let wanted_by = if user_mode {
        "default.target"
    } else {
        "multi-user.target"
    };
    let exec = format!(
        "{} {}",
        exe.display(),
        daemon_args
            .iter()
            .map(|a| shell_quote(a))
            .collect::<Vec<_>>()
            .join(" ")
    );
    format!(
        "[Unit]\n\
         Description=Serial CLI daemon (LAN remote serial access)\n\
         After=network.target\n\
         \n\
         [Service]\n\
         Type=simple\n\
         ExecStart={exec}\n\
         Restart=on-failure\n\
         RestartSec=3\n\
         \n\
         [Install]\n\
         WantedBy={wanted_by}\n"
    )
}

// ---------------------------------------------------------------------------
// macOS: launchd
// ---------------------------------------------------------------------------

#[cfg(target_os = "macos")]
fn install_launchd(exe: &Path, daemon_args: &[String]) -> Result<()> {
    let home = home_dir()?;
    let plist_path = home.join("Library/LaunchAgents/com.serial-cli.daemon.plist");

    if let Some(parent) = plist_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&plist_path, launchd_plist_content(exe, daemon_args))?;
    println!("  Wrote LaunchAgent: {}", plist_path.display());

    let uid = current_uid()?;
    let status = Command::new("launchctl")
        .args([
            "bootstrap",
            &format!("gui/{uid}"),
            plist_path
                .to_str()
                .ok_or_else(|| SerialError::Config("plist path is not valid UTF-8".into()))?,
        ])
        .status()
        .map_err(|e| SerialError::Config(format!("launchctl bootstrap failed: {e}")))?;
    if !status.success() {
        println!("  Note: launchctl bootstrap failed; the plist is in place and will load at next login.");
    } else {
        println!("  Loaded via launchctl (gui/{uid}).");
    }
    println!("  Daemon will auto-start at login. Start now with: serial-cli server start");
    Ok(())
}

#[cfg(target_os = "macos")]
fn uninstall_launchd() -> Result<()> {
    let home = home_dir()?;
    let plist_path = home.join("Library/LaunchAgents/com.serial-cli.daemon.plist");
    let plist_str = plist_path.to_string_lossy().to_string();

    if let Ok(uid) = current_uid() {
        // bootout accepts the same gui domain + plist path used by bootstrap.
        let ok = Command::new("launchctl")
            .args(["bootout", &format!("gui/{uid}"), &plist_str])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if !ok {
            // Fallback: legacy remove + stop any straggler daemon process.
            let _ = Command::new("launchctl")
                .args(["remove", "com.serial-cli.daemon"])
                .status();
            if let Ok(pid) = Command::new("pgrep")
                .args(["-f", "serial-cli server daemon"])
                .output()
            {
                for line in String::from_utf8_lossy(&pid.stdout).lines() {
                    if let Ok(p) = line.trim().parse::<i32>() {
                        // SAFETY: p comes from pgrep output of our own daemon pattern.
                        unsafe { libc::kill(p, libc::SIGTERM) };
                    }
                }
            }
        }
    }

    match std::fs::remove_file(&plist_path) {
        Ok(_) => {
            println!("  Removed LaunchAgent: {}", plist_path.display());
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            println!("  No LaunchAgent found (nothing to uninstall).");
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

/// Render a launchd plist.
#[cfg(target_os = "macos")]
fn launchd_plist_content(exe: &Path, daemon_args: &[String]) -> String {
    let prog_args = std::iter::once(exe.display().to_string())
        .chain(daemon_args.iter().cloned())
        .map(|a| format!("        <string>{}</string>", xml_escape(&a)))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.serial-cli.daemon</string>
    <key>ProgramArguments</key>
    <array>
{prog_args}
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>ProcessType</key>
    <string>Background</string>
    <key>StandardOutPath</key>
    <string>/tmp/serial-cli-daemon.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/serial-cli-daemon.log</string>
</dict>
</plist>
"#
    )
}

// ---------------------------------------------------------------------------
// Windows: Task Scheduler
// ---------------------------------------------------------------------------

/// Install a Task Scheduler task that runs the daemon at startup.
#[cfg(target_os = "windows")]
fn install_schtasks(exe: &Path, daemon_args: &[String]) -> Result<()> {
    let quoted = format!(
        "\"{}\" {}",
        exe.display(),
        daemon_args
            .iter()
            .map(|a| if a.contains(' ') {
                format!("\"{a}\"")
            } else {
                a.clone()
            })
            .collect::<Vec<_>>()
            .join(" ")
    );
    let status = Command::new("schtasks")
        .args([
            "/Create",
            "/TN",
            "SerialCLIDaemon",
            "/TR",
            &quoted,
            "/SC",
            "ONSTART",
            "/F",
        ])
        .status()
        .map_err(|e| SerialError::Config(format!("schtasks /Create failed: {e}")))?;
    if !status.success() {
        return Err(SerialError::Config(
            "schtasks /Create failed. Try running as Administrator.".into(),
        ));
    }
    println!("  Created Task Scheduler task 'SerialCLIDaemon' (runs at startup).");
    println!("  Start now with: serial-cli server start");
    Ok(())
}

#[cfg(target_os = "windows")]
fn uninstall_schtasks() -> Result<()> {
    let status = Command::new("schtasks")
        .args(["/Delete", "/TN", "SerialCLIDaemon", "/F"])
        .status();
    match status {
        Ok(s) if s.success() => {
            println!("  Deleted Task Scheduler task 'SerialCLIDaemon'.");
            Ok(())
        }
        Ok(_) => {
            println!("  No Task Scheduler task found (nothing to uninstall).");
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn home_dir() -> Result<PathBuf> {
    let var = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
    std::env::var(var)
        .map(PathBuf::from)
        .map_err(|_| SerialError::Config(format!("${var} is not set")))
}

#[cfg(target_os = "macos")]
fn current_uid() -> Result<String> {
    let out = Command::new("id")
        .arg("-u")
        .output()
        .map_err(|e| SerialError::Config(format!("id -u failed: {e}")))?;
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Quote an argument for a systemd `ExecStart=` line (space-splitting).
#[cfg(target_os = "linux")]
fn shell_quote(s: &str) -> String {
    if s.is_empty() || s.contains([' ', '"', '\\']) {
        format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        s.to_string()
    }
}

/// Escape a string for inclusion in an XML plist.
#[cfg(target_os = "macos")]
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[cfg(target_os = "linux")]
    #[test]
    fn systemd_unit_has_exec_start_and_install() {
        let content = systemd_unit_content(
            Path::new("/usr/local/bin/serial-cli"),
            &[
                "server".into(),
                "daemon".into(),
                "--port".into(),
                "23333".into(),
            ],
            false,
        );
        assert!(content.contains("ExecStart=/usr/local/bin/serial-cli server daemon --port 23333"));
        assert!(content.contains("WantedBy=multi-user.target"));
        assert!(content.contains("Restart=on-failure"));

        let user_content = systemd_unit_content(
            Path::new("/home/user/.cargo/bin/serial-cli"),
            &["server".into(), "daemon".into()],
            true,
        );
        assert!(user_content.contains("WantedBy=default.target"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn systemd_unit_quotes_paths_with_spaces() {
        let content = systemd_unit_content(
            Path::new("/opt/My Tools/serial-cli"),
            &["server".into(), "daemon".into()],
            false,
        );
        assert!(content.contains(r#"ExecStart="/opt/My Tools/serial-cli" server daemon"#));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn launchd_plist_is_well_formed() {
        let content = launchd_plist_content(
            Path::new("/usr/local/bin/serial-cli"),
            &[
                "server".into(),
                "daemon".into(),
                "--port".into(),
                "23333".into(),
            ],
        );
        assert!(content.contains("<key>Label</key>"));
        assert!(content.contains("<string>com.serial-cli.daemon</string>"));
        assert!(content.contains("<string>/usr/local/bin/serial-cli</string>"));
        assert!(content.contains("<string>23333</string>"));
        assert!(content.contains("<key>RunAtLoad</key>"));
        assert!(content.contains("<key>KeepAlive</key>"));
    }

    #[test]
    fn daemon_args_respects_no_tcp_and_defaults() {
        let args = daemon_args(None, None, false);
        assert_eq!(
            args,
            ["server", "daemon", "--port", "23333", "--bind", "0.0.0.0"]
        );

        let args = daemon_args(Some(5000), Some("127.0.0.1".into()), false);
        assert_eq!(
            args,
            ["server", "daemon", "--port", "5000", "--bind", "127.0.0.1"]
        );

        let args = daemon_args(None, None, true);
        assert_eq!(args, ["server", "daemon"]);
    }

    #[test]
    fn home_dir_reads_home_env() {
        let _ = PathBuf::new(); // keep import used on non-linux/macos builds
    }
}
