//! Virtual port pair session management
//!
//! Tracks detached (persistent) socat virtual port pairs across CLI
//! invocations using file-based state. `virtual create --backend socat`
//! spawns a session-detached socat process (setsid, stdio detached), records
//! the pair in a state file, and returns. Later `virtual list|stop|stats`
//! invocations read the state file and interact with the still-running socat
//! process — so a pair survives the creating CLI process (#70).
//!
//! Resource-release rules enforced here:
//!
//! 1. Never kill a process that is already dead: check PID liveness
//!    (`kill(pid, 0)`) before stop/prune; a dead PID just clears state.
//! 2. Symlinks are removed on every path: normal stop, prune, and crash
//!    recovery (next invocation detects a dead PID → removes stale symlinks
//!    + state entry).
//! 3. Crash recovery: stale entries are auto-cleaned by the next
//!    `list`/`create`/`stop`/`stats` invocation.
//! 4. Creation atomicity: if socat fails to come up, the spawned process and
//!    any partially-created symlinks are rolled back and no state entry is
//!    written.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::error::{Result, SerialError};

/// State file directory name (under the user's cache dir) — same convention
/// as `sniff_session.rs` and the server daemon session file.
const STATE_DIR_NAME: &str = "serial_cli";
const STATE_FILE_NAME: &str = "virtual_ports.json";

/// A detached (persistent) socat virtual port pair, persisted to the state file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocatPairEntry {
    /// Unique pair ID (UUID v4)
    pub id: String,
    /// Port A path (symlink to the PTY socat created)
    pub port_a: String,
    /// Port B path (symlink to the PTY socat created)
    pub port_b: String,
    /// PID of the detached socat process
    pub socat_pid: u32,
    /// Backend name (always "socat" for detached entries)
    pub backend: String,
    /// Creation timestamp (UNIX epoch seconds)
    pub created_at: u64,
}

impl SocatPairEntry {
    /// Path of the port-A symlink for this pair.
    pub fn symlink_a(&self) -> PathBuf {
        symlink_path(&self.id, "a")
    }

    /// Path of the port-B symlink for this pair.
    pub fn symlink_b(&self) -> PathBuf {
        symlink_path(&self.id, "b")
    }
}

/// Statistics computed for a detached pair (mirrors `VirtualStats` fields).
#[derive(Debug, Clone)]
pub struct DetachedPairStats {
    pub id: String,
    pub port_a: String,
    pub port_b: String,
    pub backend: String,
    pub running: bool,
    pub uptime_secs: u64,
    pub bytes_bridged: u64,
    pub packets_bridged: u64,
    pub bridge_errors: u64,
    pub last_error: Option<String>,
}

/// Get the directory where the virtual-port state file is stored.
fn state_dir() -> Result<PathBuf> {
    let cache = directories::BaseDirs::new()
        .ok_or_else(|| {
            SerialError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not determine user home directory",
            ))
        })?
        .cache_dir()
        .to_path_buf();
    let dir = cache.join(STATE_DIR_NAME);
    fs::create_dir_all(&dir).map_err(SerialError::Io)?;
    Ok(dir)
}

/// Get the virtual-port state file path.
fn state_file() -> Result<PathBuf> {
    Ok(state_dir()?.join(STATE_FILE_NAME))
}

/// Symlink path for one end of a pair.
fn symlink_path(id: &str, end: &str) -> PathBuf {
    std::env::temp_dir().join(format!("serial_cli_socat_{}_{}", id, end))
}

/// Current time as UNIX epoch seconds.
pub fn now_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Read all persisted pair entries (raw read — no pruning).
pub fn read_entries() -> Result<Vec<SocatPairEntry>> {
    let path = state_file()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&path).map_err(SerialError::Io)?;
    serde_json::from_str(&content).map_err(|e| {
        SerialError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to parse virtual port state file: {}", e),
        ))
    })
}

/// Persist all pair entries (atomic write: temp file + rename).
fn write_entries(entries: &[SocatPairEntry]) -> Result<()> {
    let path = state_file()?;
    let json = serde_json::to_string_pretty(entries).map_err(|e| {
        SerialError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            e.to_string(),
        ))
    })?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, json).map_err(SerialError::Io)?;
    fs::rename(&tmp, &path).map_err(SerialError::Io)?;
    Ok(())
}

/// Check if a process with the given PID is still running.
#[cfg(unix)]
pub fn is_process_running(pid: u32) -> bool {
    // SAFETY: kill syscall with sig=0 is the standard POSIX way to check process existence
    unsafe { libc::kill(pid as libc::pid_t, 0) == 0 }
}

#[cfg(windows)]
pub fn is_process_running(pid: u32) -> bool {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION};
    unsafe {
        match OpenProcess(PROCESS_QUERY_INFORMATION, false, pid) {
            Ok(handle) => {
                if handle.is_invalid() {
                    false
                } else {
                    CloseHandle(handle).ok();
                    true
                }
            }
            Err(_) => false,
        }
    }
}

/// Send SIGTERM to a process (graceful).
#[cfg(unix)]
pub fn stop_process(pid: u32) -> Result<()> {
    // SAFETY: kill with SIGTERM is the standard way to terminate a process
    let ret = unsafe { libc::kill(pid as libc::pid_t, libc::SIGTERM) };
    if ret != 0 {
        return Err(SerialError::Io(std::io::Error::other(format!(
            "Failed to send SIGTERM to process {}",
            pid
        ))));
    }
    Ok(())
}

#[cfg(windows)]
pub fn stop_process(pid: u32) -> Result<()> {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};
    let handle = unsafe { OpenProcess(PROCESS_TERMINATE, false, pid) }.map_err(|e| {
        SerialError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to open process {}: {:?}", pid, e),
        ))
    })?;
    unsafe {
        let result = TerminateProcess(handle, 1);
        let _ = CloseHandle(handle);
        if result.is_ok() {
            Ok(())
        } else {
            Err(SerialError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to terminate process {}", pid),
            )))
        }
    }
}

/// Check if socat is installed.
fn socat_available() -> bool {
    Command::new("socat")
        .arg("-V")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// A pair is alive if its socat PID is still running AND both symlinks exist.
fn is_pair_alive(entry: &SocatPairEntry) -> bool {
    is_process_running(entry.socat_pid) && entry.symlink_a().exists() && entry.symlink_b().exists()
}

/// Best-effort removal of a pair's symlinks (idempotent).
fn remove_symlinks(entry: &SocatPairEntry) {
    let _ = fs::remove_file(entry.symlink_a());
    let _ = fs::remove_file(entry.symlink_b());
}

/// Spawn a detached socat process creating a virtual serial port pair with
/// unique per-pair symlinks. The process becomes its own session leader
/// (setsid) so it survives the CLI process exiting and terminal close. Returns
/// the socat PID.
#[cfg(unix)]
fn spawn_detached_socat(id: &str) -> Result<u32> {
    use std::os::unix::process::CommandExt;

    let a = symlink_path(id, "a");
    let b = symlink_path(id, "b");

    let mut cmd = Command::new("socat");
    cmd.arg("-d")
        .arg("-d")
        .arg(format!("pty,raw,echo=0,link={}", a.display()))
        .arg(format!("pty,raw,echo=0,link={}", b.display()));
    // New session leader: survives terminal close, reparented to init on exit.
    unsafe {
        cmd.pre_exec(|| {
            libc::setsid();
            Ok(())
        });
    }
    // Fully detach stdio so nothing holds the parent's pipes open.
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    let child = cmd
        .spawn()
        .map_err(|e| SerialError::BackendInitFailed(format!("Failed to spawn socat: {e}")))?;
    Ok(child.id())
}

/// Spawn a detached socat process (Windows: DETACHED_PROCESS + CREATE_NO_WINDOW).
#[cfg(windows)]
fn spawn_detached_socat(id: &str) -> Result<u32> {
    use std::os::windows::process::CommandExt;

    const DETACHED_PROCESS: u32 = 0x00000008;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let a = symlink_path(id, "a");
    let b = symlink_path(id, "b");

    let child = Command::new("socat")
        .args([
            "-d",
            "-d",
            &format!("pty,raw,echo=0,link={}", a.display()),
            &format!("pty,raw,echo=0,link={}", b.display()),
        ])
        .creation_flags(DETACHED_PROCESS | CREATE_NO_WINDOW)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| SerialError::BackendInitFailed(format!("Failed to spawn socat: {e}")))?;
    Ok(child.id())
}

/// Read the persisted pairs, pruning entries whose socat process is dead
/// (crash recovery). Removes stale symlinks and rewrites the state file when
/// anything is pruned. Used by `list`/`create`/`stats` on every invocation.
pub fn load_and_prune() -> Result<Vec<SocatPairEntry>> {
    let mut entries = read_entries()?;
    let before = entries.len();
    entries.retain(|e| {
        if is_pair_alive(e) {
            return true;
        }
        // Dead PID: just clear symlinks + state (no error, rule 1).
        // Alive PID but missing symlinks: kill the orphaned socat so no
        // stray process lingers after we drop its state entry.
        if is_process_running(e.socat_pid) {
            stop_process(e.socat_pid).ok();
        }
        remove_symlinks(e);
        false
    });
    if entries.len() != before {
        write_entries(&entries)?;
    }
    Ok(entries)
}

/// Create a detached socat virtual port pair and persist it to the state
/// file. On any failure the spawned process and partially-created symlinks
/// are rolled back, and no state entry is written (rule 4).
pub fn create_socat_pair() -> Result<SocatPairEntry> {
    // Crash recovery: clean up stale entries first so the state file only
    // tracks live pairs.
    load_and_prune()?;

    if !socat_available() {
        return Err(SerialError::MissingDependency(
            "socat".to_string(),
            "Install with: apt install socat | brew install socat".to_string(),
        ));
    }

    let id = uuid::Uuid::new_v4().to_string();

    // Spawn the detached socat process.
    let socat_pid = spawn_detached_socat(&id)?;

    let sym_a = symlink_path(&id, "a");
    let sym_b = symlink_path(&id, "b");

    // Wait for socat to create the PTY symlinks (up to ~1s).
    let mut alive = false;
    for _ in 0..20 {
        if is_process_running(socat_pid) && sym_a.exists() && sym_b.exists() {
            alive = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    if !alive {
        // Roll back: kill socat (best effort) and remove any symlinks that
        // were created. No state entry is written.
        stop_process(socat_pid).ok();
        let _ = fs::remove_file(&sym_a);
        let _ = fs::remove_file(&sym_b);
        return Err(SerialError::BackendInitFailed(
            "Socat failed to create the port pair".to_string(),
        ));
    }

    let entry = SocatPairEntry {
        id: id.clone(),
        port_a: sym_a.to_string_lossy().to_string(),
        port_b: sym_b.to_string_lossy().to_string(),
        socat_pid,
        backend: "socat".to_string(),
        created_at: now_epoch_secs(),
    };

    // Persist only after the pair is verified live.
    let mut entries = read_entries()?;
    entries.push(entry.clone());
    write_entries(&entries)?;

    Ok(entry)
}

/// Stop a detached pair by ID: kill the socat process, remove the symlinks,
/// and drop the state entry. Works across processes.
///
/// Returns `Ok(Some(entry))` when the entry was found (and cleaned up — the
/// entry is dead-but-cleaned iff `entry.is_dead`), `Ok(None)` when no entry
/// with that ID exists in the state file. A stale (already-dead) entry is
/// cleaned up without error.
pub fn stop_socat_pair(id: &str) -> Result<Option<(SocatPairEntry, bool)>> {
    let mut entries = read_entries()?;
    let idx = match entries.iter().position(|e| e.id == id) {
        Some(i) => i,
        None => return Ok(None),
    };
    let entry = entries.remove(idx);

    let was_running = is_process_running(entry.socat_pid);
    if was_running {
        stop_process(entry.socat_pid)?;
        // Give socat a moment to exit; escalate to SIGKILL if needed.
        std::thread::sleep(Duration::from_millis(200));
        if is_process_running(entry.socat_pid) {
            #[cfg(unix)]
            {
                // SAFETY: SIGKILL to force-terminate a non-responsive process
                unsafe { libc::kill(entry.socat_pid as libc::pid_t, libc::SIGKILL) };
            }
            std::thread::sleep(Duration::from_millis(200));
        }
    }

    // Symlinks are removed on every stop path (rule 2), including stale ones.
    remove_symlinks(&entry);
    write_entries(&entries)?;

    Ok(Some((entry, was_running)))
}

/// Compute stats for a detached pair entry.
pub fn entry_stats(entry: &SocatPairEntry) -> DetachedPairStats {
    let alive = is_pair_alive(entry);
    DetachedPairStats {
        id: entry.id.clone(),
        port_a: entry.port_a.clone(),
        port_b: entry.port_b.clone(),
        backend: entry.backend.clone(),
        running: alive,
        uptime_secs: now_epoch_secs().saturating_sub(entry.created_at),
        bytes_bridged: 0,   // socat bridges internally; not tracked
        packets_bridged: 0, // socat bridges internally; not tracked
        bridge_errors: 0,
        last_error: None,
    }
}

/// Remove the state file entirely (used by tests/cleanup).
pub fn clear_state() -> Result<()> {
    let path = state_file()?;
    if path.exists() {
        fs::remove_file(&path).map_err(SerialError::Io)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_serialization_roundtrip() {
        let entry = SocatPairEntry {
            id: "test-id-1".to_string(),
            port_a: "/tmp/serial_cli_socat_test-id-1_a".to_string(),
            port_b: "/tmp/serial_cli_socat_test-id-1_b".to_string(),
            socat_pid: 4242,
            backend: "socat".to_string(),
            created_at: 1_700_000_000,
        };
        let json = serde_json::to_string_pretty(&entry).unwrap();
        let parsed: SocatPairEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, entry.id);
        assert_eq!(parsed.port_a, entry.port_a);
        assert_eq!(parsed.port_b, entry.port_b);
        assert_eq!(parsed.socat_pid, 4242);
        assert_eq!(parsed.backend, "socat");
        assert_eq!(parsed.created_at, 1_700_000_000);
    }

    #[test]
    fn test_symlink_paths_are_unique_per_pair_and_end() {
        let a = symlink_path("pair1", "a");
        let b = symlink_path("pair1", "b");
        let other = symlink_path("pair2", "a");
        assert_ne!(a, b);
        assert_ne!(a, other);
        assert!(a.to_string_lossy().contains("pair1"));
    }

    #[test]
    fn test_entry_stats_reports_running_state() {
        // A PID that cannot exist -> pair is reported not running without error.
        let entry = SocatPairEntry {
            id: "test-id-2".to_string(),
            port_a: symlink_path("test-id-2", "a").to_string_lossy().to_string(),
            port_b: symlink_path("test-id-2", "b").to_string_lossy().to_string(),
            socat_pid: u32::MAX - 1,
            backend: "socat".to_string(),
            created_at: now_epoch_secs(),
        };
        let stats = entry_stats(&entry);
        assert_eq!(stats.id, entry.id);
        assert!(!stats.running);
        assert_eq!(stats.backend, "socat");
    }
}
