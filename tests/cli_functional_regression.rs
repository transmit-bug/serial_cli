//! Binary-level regression tests for CLI functional defects (#62, #63, #64, #65, #67).
//!
//! These tests build (if needed) and invoke the real `serial-cli` binary,
//! mirroring the repro commands from the GitHub issues:
//!
//! - #62: `--json` stdout must contain exactly one parseable JSON document
//!        (tracing must never write to stdout).
//! - #63: `sniff start` must reach port logic instead of dying on the daemon's
//!        `--hex <bool>` argument construction.
//! - #64: `virtual create --backend socat` must not panic and must not leak a
//!        socat process or `/tmp/serial_cli_socat_{a,b}` symlinks.
//! - #65: `port_type` must be a plain string in JSON output (no double-encoding).
//! - #67: `--json` must be honored by the `virtual`/`sniff` handlers.

#![cfg(unix)]

use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

/// Path to the debug binary (same convention as `tests/e2e_server_tests.rs`).
fn binary_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/debug/serial-cli")
}

/// Build the CLI binary if any source file is newer than the existing binary.
fn ensure_binary() {
    let binary = binary_path();

    let needs_build = if !binary.exists() {
        true
    } else {
        let binary_mtime = std::fs::metadata(&binary)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);

        let src_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
        let cargo_toml = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        let cargo_lock = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.lock");

        fn is_newer(path: &PathBuf, threshold: std::time::SystemTime) -> bool {
            std::fs::metadata(path)
                .and_then(|m| m.modified())
                .map(|t| t > threshold)
                .unwrap_or(false)
        }
        fn dir_has_newer_file(dir: &PathBuf, threshold: std::time::SystemTime) -> bool {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        if dir_has_newer_file(&path, threshold) {
                            return true;
                        }
                    } else if let Ok(metadata) = std::fs::metadata(&path) {
                        if let Ok(modified) = metadata.modified() {
                            if modified > threshold {
                                return true;
                            }
                        }
                    }
                }
            }
            false
        }

        is_newer(&cargo_toml, binary_mtime)
            || is_newer(&cargo_lock, binary_mtime)
            || dir_has_newer_file(&src_dir, binary_mtime)
    };

    if needs_build {
        let status = Command::new("cargo")
            .args(["build", "--bin", "serial-cli", "--quiet"])
            .status()
            .expect("cargo should be available");
        assert!(status.success(), "cargo build failed");
    }
}

/// Run the CLI binary and capture (stdout, stderr, status).
fn run_cli(args: &[&str]) -> (String, String, std::process::ExitStatus) {
    ensure_binary();
    let output = Command::new(binary_path())
        .args(args)
        .output()
        .expect("serial-cli should run");
    (
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
        output.status,
    )
}

/// #62 + #65: `--json port list` stdout is exactly one JSON document, with no
/// tracing log pollution, and `port_type` is a plain string.
#[test]
fn json_port_list_stdout_is_one_json_document() {
    let (stdout, stderr, status) = run_cli(&["--json", "port", "list"]);

    assert!(
        status.success(),
        "port list should exit 0; stderr: {}",
        stderr
    );

    let doc: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!(
            "stdout must be a single JSON document (got {} bytes): {e}",
            stdout.len()
        )
    });

    let ports = doc.as_array().expect("port list JSON should be an array");
    // #65: any port entry must carry a plain-string port_type, never a
    // double-encoded value like "\"PciPort\"".
    for port in ports {
        let port_type = port
            .get("port_type")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("port_type must be a plain string: {port}"));
        assert!(
            !port_type.starts_with('"') && !port_type.ends_with('"'),
            "port_type must not be double-encoded, got: {port_type:?}"
        );
        assert!(
            port.get("port_name").is_some(),
            "entry should carry port_name: {port}"
        );
    }
}

/// #62: even without `--json`, logs must go to stderr, never stdout — and the
/// ScriptManager protocol-discovery events were downgraded to DEBUG so a
/// normal run doesn't spam stderr either.
#[test]
fn logs_never_write_to_stdout() {
    let (stdout, stderr, status) = run_cli(&["port", "list"]);
    assert!(status.success());
    assert!(
        !stdout.contains("INFO") && !stdout.contains("Loaded external protocol"),
        "stdout must not contain tracing output; got: {}",
        stdout.lines().take(3).collect::<Vec<_>>().join("\n")
    );
    assert!(
        !stderr.contains("Loaded external protocol")
            && !stderr.contains("Discovered external protocol"),
        "protocol discovery should not be logged at the default INFO level"
    );
}

/// #63: `sniff start` on a nonexistent port must fail with a *port* error, not
/// the clap "unexpected argument 'false' found" daemon-arg error. Exercises both
/// the default (raw) and `--format hex` display paths.
#[test]
fn sniff_start_nonexistent_port_reports_port_error_not_clap_error() {
    // Clear any stale session so the test starts clean.
    let _ = serial_cli::cli::sniff_session::clear_session();

    for format_args in [vec![], vec!["--format", "hex"]] {
        let mut args = vec!["sniff", "start", "--port", "/dev/tty.NONEXISTENT"];
        args.extend(format_args.iter().copied());
        let (stdout, stderr, status) = run_cli(&args);

        assert!(
            !status.success(),
            "sniff start on a nonexistent port should fail"
        );
        let combined = format!("{stdout}\n{stderr}");
        assert!(
            !combined.contains("unexpected argument"),
            "daemon must not die on a clap arg error; got: {combined}"
        );
        assert!(
            combined.contains("not found") || combined.contains("Failed to open port"),
            "failure should be a port-open error; got: {combined}"
        );

        // No session file should be left behind by a failed start.
        assert!(
            serial_cli::cli::sniff_session::load_session()
                .ok()
                .flatten()
                .is_none(),
            "failed sniff start must not leave a session"
        );
    }
}

/// #64: `virtual create --backend socat` must succeed without panicking and
/// must not leak a socat process or /tmp/serial_cli_socat_{a,b} symlinks after
/// the CLI process exits.
#[test]
fn virtual_socat_create_leaves_no_stray_process_or_symlinks() {
    let socat_available = Command::new("socat")
        .arg("-V")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !socat_available {
        eprintln!("skipping: socat not installed");
        return;
    }

    let symlink_a = PathBuf::from("/tmp/serial_cli_socat_a");
    let symlink_b = PathBuf::from("/tmp/serial_cli_socat_b");
    // Best-effort pre-clean of any stale symlinks from earlier runs.
    let _ = std::fs::remove_file(&symlink_a);
    let _ = std::fs::remove_file(&symlink_b);

    let socat_count = || -> usize {
        Command::new("pgrep")
            .arg("-c")
            .arg("socat")
            .output()
            .map(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .trim()
                    .parse::<usize>()
                    .unwrap_or(0)
            })
            .unwrap_or(0)
    };

    let before = socat_count();

    let (stdout, stderr, status) = run_cli(&["virtual", "create", "--backend", "socat"]);
    assert!(
        status.success(),
        "virtual create --backend socat should succeed; stderr: {stderr}"
    );
    assert!(
        stdout.contains("Port A:") && stdout.contains("Port B:"),
        "create should print the pair; got: {stdout}"
    );

    // Give the OS a moment to reap the killed socat child.
    std::thread::sleep(Duration::from_millis(500));

    assert_eq!(
        socat_count(),
        before,
        "no extra socat processes may remain after the CLI exits"
    );
    assert!(
        !symlink_a.exists() && !symlink_b.exists(),
        "socat symlinks must be cleaned up after the CLI exits"
    );
}

/// #67: `--json virtual list` must emit a single parseable JSON document
/// (an empty pair list in a fresh process) instead of human text.
#[test]
fn json_virtual_list_emits_json() {
    let (stdout, stderr, status) = run_cli(&["--json", "virtual", "list"]);
    assert!(status.success(), "stderr: {stderr}");

    let doc: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("stdout must be one JSON document: {e}"));
    let pairs = doc
        .get("pairs")
        .and_then(|v| v.as_array())
        .expect("virtual list JSON should carry a 'pairs' array");
    assert_eq!(
        doc.get("count").and_then(|v| v.as_u64()),
        Some(pairs.len() as u64),
        "count field must match pairs length"
    );
}
