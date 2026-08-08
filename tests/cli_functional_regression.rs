//! Binary-level regression tests for CLI functional defects (#62, #63, #64, #65, #67)
//! and the CLI design decisions (#68, #69, #70, #71).
//!
//! These tests build (if needed) and invoke the real `serial-cli` binary,
//! mirroring the repro commands from the GitHub issues:
//!
//! - #62: `--json` stdout must contain exactly one parseable JSON document
//!   (tracing must never write to stdout).
//! - #63: `sniff start` must reach port logic instead of dying on the daemon's
//!   `--hex <bool>` argument construction.
//! - #64: `virtual create --backend socat` must not panic and must not leak a
//!   socat process or `/tmp/serial_cli_socat_{a,b}` symlinks.
//! - #65: `port_type` must be a plain string in JSON output (no double-encoding).
//! - #67: `--json` must be honored by the `virtual`/`sniff` handlers.
//! - #68: `config set` must persist immediately (write-through) using the same
//!   path resolution as `config save`.
//! - #69: `script load` must persist to `[protocols.custom]`; a new process sees
//!   it; `script unload` is session-only; `script remove` is permanent.
//! - #70: `virtual create --backend socat` spawns a detached, persistent pair
//!   tracked in a state file; list/stop/stats work across processes; dead
//!   PIDs are pruned on the next invocation; stop cleans up processes +
//!   symlinks + state; no leftovers after create/stop cycles.
//! - #71: `--version` flag, human-readable error Display at the CLI boundary,
//!   error prints on stderr, `sniff save <path>` positional, Lua chunk
//!   name stripped from validation errors, out-of-range config values
//!   rejected with a non-zero exit.

#![cfg(unix)]

use std::path::PathBuf;
use std::process::Command;
use std::sync::{Mutex, OnceLock};
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

/// #64 + #70: `virtual create --backend socat` spawns a DETACHED, persistent
/// pair by design — the socat process survives the creating CLI process (that
/// is the point of #70). What must NOT happen is a *leak*: after `virtual
/// stop` (in a new process), the socat process, its symlinks, and its state
/// entry must all be gone.
#[test]
fn virtual_socat_create_and_stop_leaves_no_stray_process_or_symlinks() {
    let _guard = vport_lock().lock().unwrap();
    let socat_available = Command::new("socat")
        .arg("-V")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !socat_available {
        eprintln!("skipping: socat not installed");
        return;
    }

    let state = vport_state_path();
    let state_backup = read_opt(&state);

    let (stdout, stderr, status) = run_cli(&["virtual", "create", "--backend", "socat"]);
    assert!(
        status.success(),
        "virtual create --backend socat should succeed; stderr: {stderr}"
    );
    assert!(
        stdout.contains("Port A:") && stdout.contains("Port B:"),
        "create should print the pair; got: {stdout}"
    );
    let info = parse_socat_create(&stdout);
    let (sym_a, sym_b) = (info.port_a.clone(), info.port_b.clone());

    // Give the OS a moment to fully set up the pair.
    std::thread::sleep(Duration::from_millis(500));

    // #70: the detached pair must SURVIVE the creating CLI process.
    assert!(
        pid_alive(info.pid),
        "detached socat (PID {}) must survive the CLI exit",
        info.pid
    );
    assert!(
        sym_a.exists() && sym_b.exists(),
        "pair symlinks should exist while running"
    );

    // Stop from a NEW process: process + symlinks + state entry all removed.
    let (stdout, stderr, status) = run_cli(&["virtual", "stop", &info.id]);
    assert!(
        status.success(),
        "stop should succeed; stderr: {stderr}\n{stdout}"
    );
    std::thread::sleep(Duration::from_millis(500));

    assert!(
        !pid_alive(info.pid),
        "no socat process may remain after stop"
    );
    assert!(
        !sym_a.exists() && !sym_b.exists(),
        "socat symlinks must be cleaned up after stop"
    );
    assert!(
        !state_has_entry(&info.id),
        "state entry must be removed after stop"
    );

    restore_file(&state, state_backup);
    // Safety net: never leave a stray socat or symlinks behind.
    if pid_alive(info.pid) {
        unsafe {
            libc::kill(info.pid as libc::pid_t, libc::SIGKILL);
        }
    }
    let _ = std::fs::remove_file(&sym_a);
    let _ = std::fs::remove_file(&sym_b);
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

// ═══════════════════════════════════════════════════════════════════════════
// #68 — `config set` write-through
// ═══════════════════════════════════════════════════════════════════════════

/// Global config path used by `config save`/`config set` (no explicit path).
fn global_config_path() -> PathBuf {
    directories::BaseDirs::new()
        .expect("home dir")
        .config_dir()
        .join("serial-cli")
        .join("config.toml")
}

/// Read a file into an Option<String> (None when absent).
fn read_opt(path: &PathBuf) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

/// Restore a file from a backup: write the backup, or remove the file if
/// there was no backup.
fn restore_file(path: &PathBuf, backup: Option<String>) {
    match backup {
        Some(content) => {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            std::fs::write(path, content).expect("restore file");
        }
        None => {
            let _ = std::fs::remove_file(path);
        }
    }
}

/// #68: `config set` must persist immediately (write-through) using the same
/// path resolution as `config save`, so the value survives the (fresh)
/// process. Backs up and restores the real global config file.
#[test]
fn config_set_persists_across_invocations() {
    let global = global_config_path();
    let backup = read_opt(&global);
    let _ = std::fs::remove_file(&global);

    let (stdout, stderr, status) = run_cli(&["config", "set", "serial.baudrate", "9600"]);
    let written = read_opt(&global);
    let printed = stdout.clone();

    // Restore before asserting so a failure cannot leave the user's config
    // dir modified.
    restore_file(&global, backup);

    assert!(
        status.success(),
        "config set should exit 0; stderr: {stderr}"
    );
    assert!(
        printed.contains("saved"),
        "output should mention the persisted save; got: {printed}"
    );
    let written = written.unwrap_or_else(|| panic!("config file should have been written"));
    assert!(
        written.contains("baudrate = 9600"),
        "config file must contain baudrate = 9600 after config set; got:\n{written}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// #69 — script load persistence / unload session-only / remove permanent
// ═══════════════════════════════════════════════════════════════════════════

/// CWD config file (the repo's `.serial-cli.toml`) — `script load` persists
/// here via `config_manager.save(None)`.
fn cwd_config_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".serial-cli.toml")
}

/// #69: `script load` persists to `[protocols.custom]`; a new process sees the
/// script; `script unload` is session-only; `script remove` is permanent.
#[test]
fn script_load_unload_remove_persistence() {
    // Isolate the config file (backup/restore).
    let config = cwd_config_path();
    let backup = read_opt(&config);

    let script_dir = std::env::temp_dir().join(format!("serial_cli_itest_{}", std::process::id()));
    std::fs::create_dir_all(&script_dir).unwrap();
    let script_path = script_dir.join("itest_persist.lua");
    std::fs::write(
        &script_path,
        "function on_send(data)\n    return data .. \"\\n\"\nend\nfunction on_recv(data)\n    return data\nend\n",
    )
    .unwrap();

    let run = |args: &[&str]| run_cli(args);
    let name = "itest_persist";

    // Load: registers + persists.
    let (stdout, stderr, status) = run(&["script", "load", script_path.to_str().unwrap()]);
    assert!(status.success(), "load failed: {stderr}\n{stdout}");

    // A NEW process sees it (auto-registered from config at ScriptManager init).
    let (stdout, stderr, status) = run(&["script", "list"]);
    assert!(status.success(), "list failed: {stderr}");
    assert!(
        stdout.lines().any(|l| l.trim() == name),
        "new process should list the persisted script; got: {stdout}"
    );

    // Unload in a NEW process: session-only, must NOT touch config.
    let (_, stderr, status) = run(&["script", "unload", name]);
    assert!(status.success(), "unload failed: {stderr}");
    let config_after_unload = read_opt(&config).unwrap_or_default();
    assert!(
        config_after_unload.contains(name),
        "unload must not remove the script from config"
    );

    // Another NEW process still sees it.
    let (stdout, _, _) = run(&["script", "list"]);
    assert!(
        stdout.lines().any(|l| l.trim() == name),
        "script should still be listed after unload in a prior process"
    );

    // Remove: permanent (config + runtime).
    let (_, stderr, status) = run(&["script", "remove", name]);
    assert!(status.success(), "remove failed: {stderr}");
    let config_after_remove = read_opt(&config).unwrap_or_default();
    assert!(
        !config_after_remove.contains(name),
        "remove must delete the script from config"
    );

    // NEW process no longer lists it.
    let (stdout, _, _) = run(&["script", "list"]);
    assert!(
        !stdout.lines().any(|l| l.trim() == name),
        "script should be gone after remove"
    );

    // Restore the original config.
    restore_file(&config, backup);
    let _ = std::fs::remove_dir_all(&script_dir);
}

// ═══════════════════════════════════════════════════════════════════════════
// #70 — detached socat pairs: state file, cross-process list/stop, pruning
// ═══════════════════════════════════════════════════════════════════════════

/// Serialize the #70 tests: they share the real virtual-port state file.
fn vport_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

/// The virtual-port state file (same path the binary uses).
fn vport_state_path() -> PathBuf {
    directories::BaseDirs::new()
        .expect("home dir")
        .cache_dir()
        .join("serial_cli")
        .join("virtual_ports.json")
}

/// Info parsed from `virtual create --backend socat` human output.
struct SocatCreateInfo {
    id: String,
    pid: u32,
    port_a: PathBuf,
    port_b: PathBuf,
}

fn parse_socat_create(stdout: &str) -> SocatCreateInfo {
    let field = |needle: &str| -> String {
        stdout
            .lines()
            .find(|l| l.contains(needle))
            .and_then(|l| l.split(needle).nth(1))
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| panic!("missing {needle:?} in create output:\n{stdout}"))
    };
    SocatCreateInfo {
        id: field("ID:"),
        pid: field("Socat PID:").parse().expect("socat pid"),
        port_a: PathBuf::from(field("Port A:")),
        port_b: PathBuf::from(field("Port B:")),
    }
}

/// PID liveness check (kill(pid, 0)).
fn pid_alive(pid: u32) -> bool {
    // SAFETY: kill with signal 0 is the standard POSIX existence check
    unsafe { libc::kill(pid as libc::pid_t, 0) == 0 }
}

/// State file contains no entry for the given id.
fn state_has_entry(id: &str) -> bool {
    read_opt(&vport_state_path())
        .map(|c| c.contains(id))
        .unwrap_or(false)
}

fn socat_available() -> bool {
    Command::new("socat")
        .arg("-V")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// #70 rule 5a: a detached socat pair survives the creating CLI process;
/// `virtual stop` (in a NEW process) kills the socat, removes the symlinks
/// and the state entry; no process/symlink leftovers remain.
#[test]
fn virtual_socat_detached_pair_survives_cli_and_stop_cleans_up() {
    let _guard = vport_lock().lock().unwrap();
    if !socat_available() {
        eprintln!("skipping: socat not installed");
        return;
    }

    let state = vport_state_path();
    let state_backup = read_opt(&state);

    let (stdout, stderr, status) = run_cli(&["virtual", "create", "--backend", "socat"]);
    assert!(status.success(), "create failed: {stderr}");
    let info = parse_socat_create(&stdout);
    let (sym_a, sym_b) = (info.port_a.clone(), info.port_b.clone());

    // Give the process a moment to be fully set up.
    std::thread::sleep(Duration::from_millis(300));

    // Pair must be ALIVE after the creating CLI exited.
    assert!(
        pid_alive(info.pid),
        "socat (PID {}) must survive the creating CLI process",
        info.pid
    );
    assert!(sym_a.exists() && sym_b.exists(), "symlinks should exist");

    // Stop from a NEW process.
    let (stdout, stderr, status) = run_cli(&["virtual", "stop", &info.id]);
    assert!(status.success(), "stop failed: {stderr}\n{stdout}");
    std::thread::sleep(Duration::from_millis(300));

    assert!(!pid_alive(info.pid), "socat should be dead after stop");
    assert!(
        !sym_a.exists() && !sym_b.exists(),
        "symlinks must be removed on stop"
    );
    assert!(
        !state_has_entry(&info.id),
        "state entry must be removed on stop"
    );

    restore_file(&state, state_backup);
    // Safety net: never leave a stray socat or symlinks behind.
    if pid_alive(info.pid) {
        unsafe {
            libc::kill(info.pid as libc::pid_t, libc::SIGKILL);
        }
    }
    let _ = std::fs::remove_file(&sym_a);
    let _ = std::fs::remove_file(&sym_b);
}

/// #70 rule 5b: killing the socat process leaves a stale entry; the next
/// `virtual list` auto-prunes it (exit 0, no error) and removes the symlinks.
#[test]
fn virtual_socat_kill_triggers_auto_prune_on_next_list() {
    let _guard = vport_lock().lock().unwrap();
    if !socat_available() {
        eprintln!("skipping: socat not installed");
        return;
    }

    let state = vport_state_path();
    let state_backup = read_opt(&state);

    let (stdout, stderr, status) = run_cli(&["virtual", "create", "--backend", "socat"]);
    assert!(status.success(), "create failed: {stderr}");
    let info = parse_socat_create(&stdout);
    let (sym_a, sym_b) = (info.port_a.clone(), info.port_b.clone());
    assert!(state_has_entry(&info.id), "create must write a state entry");

    // Simulate a crash: kill the socat, leaving the stale entry + symlinks.
    // SAFETY: SIGKILL in the test process
    unsafe {
        libc::kill(info.pid as libc::pid_t, libc::SIGKILL);
    }
    std::thread::sleep(Duration::from_millis(300));

    // Next `virtual list` must prune it without error.
    let (stdout, stderr, status) = run_cli(&["virtual", "list"]);
    assert!(
        status.success(),
        "list must not error on stale entries: {stderr}"
    );
    assert!(
        !stdout.contains(&info.id),
        "pruned pair must not be listed; got: {stdout}"
    );
    assert!(!state_has_entry(&info.id), "state entry must be pruned");
    assert!(
        !sym_a.exists() && !sym_b.exists(),
        "symlinks must be removed during prune"
    );

    restore_file(&state, state_backup);
    let _ = std::fs::remove_file(&sym_a);
    let _ = std::fs::remove_file(&sym_b);
}

/// #70 rule 5c: `virtual stop` of a stale (dead-PID) id reports cleanly
/// (exit 0) and cleans up the entry + symlinks.
#[test]
fn virtual_socat_stop_stale_id_reports_cleanly() {
    let _guard = vport_lock().lock().unwrap();
    if !socat_available() {
        eprintln!("skipping: socat not installed");
        return;
    }

    let state = vport_state_path();
    let state_backup = read_opt(&state);

    let (stdout, stderr, status) = run_cli(&["virtual", "create", "--backend", "socat"]);
    assert!(status.success(), "create failed: {stderr}");
    let info = parse_socat_create(&stdout);
    let (sym_a, sym_b) = (info.port_a.clone(), info.port_b.clone());
    assert!(state_has_entry(&info.id), "create must write a state entry");

    // Kill the socat but leave the state entry in place.
    // SAFETY: SIGKILL in the test process
    unsafe {
        libc::kill(info.pid as libc::pid_t, libc::SIGKILL);
    }
    std::thread::sleep(Duration::from_millis(300));
    assert!(
        state_has_entry(&info.id),
        "state entry should still exist (stale)"
    );

    // Stale stop must exit 0 and clean everything up.
    let (stdout, stderr, status) = run_cli(&["virtual", "stop", &info.id]);
    assert!(
        status.success(),
        "stale stop must exit 0: {stderr}\n{stdout}"
    );
    assert!(
        stdout.contains("stale"),
        "output should mention the stale pair; got: {stdout}"
    );
    assert!(
        !state_has_entry(&info.id),
        "stale stop must drop the state entry"
    );
    assert!(
        !sym_a.exists() && !sym_b.exists(),
        "stale stop must remove the symlinks"
    );

    restore_file(&state, state_backup);
    let _ = std::fs::remove_file(&sym_a);
    let _ = std::fs::remove_file(&sym_b);
}

// ═══════════════════════════════════════════════════════════════════════════
// #71 — CLI polish
// ═══════════════════════════════════════════════════════════════════════════

/// #71.1: `--version` / `-V` prints the version and exits 0.
#[test]
fn version_flag_prints_version_and_exits_zero() {
    let (stdout, stderr, status) = run_cli(&["--version"]);
    assert!(status.success(), "stderr: {stderr}");
    assert!(
        stdout.trim_start().starts_with("serial-cli "),
        "version output should start with the binary name; got: {stdout}"
    );
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")), "got: {stdout}");

    let (stdout, _, status) = run_cli(&["-V"]);
    assert!(status.success());
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")));
}

/// #71.2 + #71.3 + #71.7: errors print as human-readable `Display` to stderr
/// (not `Debug`, not stdout), and out-of-range config values exit non-zero.
#[test]
fn config_errors_are_human_readable_on_stderr_and_fail_fast() {
    // Invalid value: parse error.
    let (stdout, stderr, status) = run_cli(&["config", "set", "serial.baudrate", "notanumber"]);
    assert!(!status.success(), "invalid baudrate must fail");
    assert!(
        stderr.contains("Error: Configuration error: Invalid baudrate"),
        "stderr must carry a human-readable error; got: {stderr}"
    );
    assert!(
        !stderr.contains("Config("),
        "must not print the Debug form; got: {stderr}"
    );
    assert!(
        !stdout.contains("Failed to set configuration"),
        "handler error prints must go to stderr, not stdout"
    );

    // Out-of-range value (#71.7): exit non-zero with a clear error.
    let (stdout, stderr, status) = run_cli(&["config", "set", "serial.databits", "99"]);
    assert!(
        !status.success(),
        "databits 99 must fail with non-zero exit"
    );
    assert!(
        stderr.contains("Databits") || stderr.contains("databits"),
        "stderr should explain the range error; got: {stderr}"
    );
    assert!(
        !stdout.contains("Configuration updated"),
        "failed set must not claim success on stdout"
    );
}

/// #71.5: `sniff save <path>` accepts a positional path (no clap error).
#[test]
fn sniff_save_accepts_positional_path() {
    let _ = serial_cli::cli::sniff_session::clear_session();

    let (stdout, stderr, status) = run_cli(&["sniff", "save", "/tmp/serial_cli_itest.pcap"]);
    let combined = format!("{stdout}\n{stderr}");
    assert!(
        !combined.contains("unexpected argument"),
        "positional path must be accepted by clap; got: {combined}"
    );
    assert!(
        combined.contains("No active sniff session"),
        "should reach the session logic; got: {combined}"
    );
    assert!(!status.success(), "no session -> save must fail");
}

/// #71.6: Lua validation errors must not leak the internal chunk name.
#[test]
fn script_validate_strips_lua_chunk_name() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/protocols/test_syntax_error.lua");
    let (stdout, stderr, status) = run_cli(&["script", "validate", fixture.to_str().unwrap()]);
    assert!(
        !status.success(),
        "syntax-error fixture must fail validation"
    );
    let combined = format!("{stdout}\n{stderr}");
    assert!(
        !combined.contains("[string \""),
        "chunk-name prefix must be stripped; got: {combined}"
    );
    assert!(
        !combined.contains("script.rs"),
        "internal source location must not leak; got: {combined}"
    );
    assert!(
        combined.contains("5: ')' expected"),
        "the actual Lua error (message + line) must be visible; got: {combined}"
    );
}
