# Troubleshooting & Error Reference

This guide covers common issues, debugging strategies, and error codes encountered when using Serial CLI.

## Common Issues

### Port Not Found

**Error:** `Serial port error: Port '/dev/ttyUSB0' not found`

**Causes and Solutions:**

- **Cable not connected or device not enumerated.** Verify the device appears in the system:
  - Linux: `ls /dev/ttyUSB* /dev/ttyACM*`
  - macOS: `ls /dev/cu.* /dev/tty.*`
  - Windows: `mode` or check Device Manager under "Ports (COM & LPT)"

- **Linux permission denied.** USB serial devices require membership in the `dialout` (Debian/Ubuntu) or `uucp` (Arch) group:
  ```bash
  # Debian/Ubuntu
  sudo usermod -aG dialout $USER
  # Arch Linux
  sudo usermod -aG uucp $USER
  # Apply by logging out and back in, or use:
  newgrp dialout
  ```
  Alternatively, create a udev rule for persistent access:
  ```bash
  echo 'SUBSYSTEM=="tty", ATTRS{idVendor}=="0403", ATTRS{idProduct}=="6001", MODE="0666"' \
    | sudo tee /etc/udev/rules.d/99-serial.rules
  sudo udevadm control --reload-rules
  ```

- **Wrong port name.** macOS uses `/dev/cu.usbserial-*` or `/dev/tty.usbserial-*`, not `/dev/ttyUSB*`. Windows uses `COM1`, `COM3`, etc.

### Port In Use

**Error:** `Serial port error: Port '/dev/ttyUSB0' is already in use`

**Causes and Solutions:**

- **Another process holds the port.** Identify and terminate it:
  - Linux/macOS: `lsof /dev/ttyUSB0` or `fuser /dev/ttyUSB0`
  - macOS: Check for `cu` sessions from previous terminal sessions
  - Windows: Check Device Manager or use Handle.exe from Sysinternals

- **Stale lock file.** Some systems create lock files under `/var/lock/` (e.g., `LCK..ttyUSB0`). Remove them if no process is actually using the port:
  ```bash
  sudo rm /var/lock/LCK..ttyUSB0
  ```

- **ModemManager interference (Linux).** The `ModemManager` service may auto-claim USB serial devices:
  ```bash
  sudo systemctl stop ModemManager
  sudo systemctl disable ModemManager
  ```

### Virtual Port Creation Failures

**Error:** `Virtual port error: ...`

| Issue | Cause | Solution |
|-------|-------|----------|
| `Permission denied` on `/dev/pts/*` | PTY backend requires appropriate permissions | Run with sufficient privileges; ensure user has access to `/dev/ptmx` |
| `socat: command not found` | Socat backend selected but `socat` is not installed | Install socat: `apt install socat` / `brew install socat` / `choco install socat` |
| `Unsupported backend for this platform` | NamedPipe selected on Unix or PTY on Windows | Use platform-appropriate backend: `pty` (macOS/Linux), `namedpipe` (Windows), or `socat` (cross-platform) |
| Backend creation hangs | socat process stalled or ptmx exhausted | Kill stale socat processes: `pkill -f socat`; retry with `--virtual-backend pty` |

**Backend availability by platform:**

| Backend | Linux | macOS | Windows |
|---------|-------|-------|---------|
| `pty` | Yes | Yes | No |
| `namedpipe` | No | No | Yes |
| `socat` | Yes (if installed) | Yes (if installed) | Yes (if installed) |

Use `--virtual-backend auto` to let Serial CLI select the best available backend automatically.

### Lua Script Errors

**Syntax Error:**

```
Script error: Syntax error in my_script.lua:10: unexpected symbol near '<eof>'
```

- Check the indicated line number for typos, missing `end`, unclosed strings, or mismatched parentheses.
- Validate the script independently: `luac -p my_script.lua`

**Runtime Error:**

```
Script error: Runtime error in my_script.lua: attempt to call a nil value
```

- A function call references an undefined or misspelled function name.
- Verify that required protocol API functions (`encode`, `decode`, `validate`, etc.) are properly defined.

**Memory Limit Exceeded:**

```
Script error: Resource limit exceeded: memory
```

- The Lua script exceeded the configured memory limit (default: 128 MB).
- Increase the limit in `.serial-cli.toml`:
  ```toml
  [lua]
  memory_limit_mb = 256
  ```

**Timeout:**

```
Task error: Task 'script_task' timed out after 60s
```

- The script did not complete within `lua.timeout_seconds` (default: 300s) or `task.default_timeout_seconds` (default: 60s).
- Check for infinite loops or blocking I/O in the script.
- Adjust timeouts in `.serial-cli.toml`:
  ```toml
  [lua]
  timeout_seconds = 600
  [task]
  default_timeout_seconds = 120
  ```

**Sandbox Violation:**

```
Script error: Sandbox violation: os.execute
```

- The Lua sandbox blocks dangerous operations (`os.execute`, `io.popen`, etc.).
- Disable sandboxing if you trust the script (not recommended for untrusted scripts):
  ```toml
  [lua]
  enable_sandbox = false
  ```

### Protocol Loading Failures

**Error:** `Protocol error: Protocol 'my_protocol' not found`

- The protocol name must match a built-in protocol (`modbus_rtu`, `modbus_ascii`, `at_commands`, `line_based`) or a registered custom protocol.
- List available protocols with the `protocol list` command.

**Error:** `Protocol error: Invalid frame format: ...`

- The data being encoded/decoded does not match the protocol's expected frame structure.
- For Modbus: ensure the frame includes a valid slave address, function code, and data payload.
- For custom Lua protocols: verify the `encode` and `decode` functions handle the input format correctly.

**Error:** `Protocol error: Checksum mismatch (expected: 0x1234, got: 0x5678)`

- The computed checksum of the frame does not match the expected value.
- Check that the protocol's checksum algorithm (CRC-16, LRC, etc.) matches the device's expectations.
- Verify byte order (big-endian vs. little-endian).

**Error:** `Protocol error: Invalid protocol state: closed`

- The serial port was closed before the protocol operation completed.
- Check for connection drops, timeout settings, or device resets.

### Sniff Daemon Issues

**Sniff process not responding:**

- The sniff daemon runs as a long-lived async task. If it becomes unresponsive:
  - Check for zombie processes: `ps aux | grep serial` (Unix) or Task Manager (Windows)
  - Kill the parent Serial CLI process and restart

**No captured packets appearing:**

- Verify the sniffer is attached to the correct port name.
- Check that data is actually flowing on the port (use a hardware loopback or another tool to confirm).
- Ensure `capture_packets` is enabled in the sniffer configuration.

**High CPU usage during sniffing:**

- Sniffing uses an async polling loop. If CPU usage is excessive:
  - Reduce the bridge poll interval in `.serial-cli.toml`:
    ```toml
    [virtual_ports]
    bridge_poll_interval_ms = 50  # increase from default 10ms
    ```

## Debugging Tips

### Use `--verbose` for Detailed Logging

The `--verbose` flag (or `-v`) increases the log level to `debug`, providing detailed information about:

- Port opening and configuration
- Protocol frame encoding/decoding
- Lua script loading and execution
- Virtual port backend initialization
- Task scheduling and execution

```bash
serial-cli --verbose send --port /dev/ttyUSB0 --data "AT\r\n"
```

For maximum verbosity, set the logging level to `trace` in `.serial-cli.toml`:

```toml
[logging]
level = "trace"
```

### Use `--json` for Structured Output

The `--json` flag produces machine-readable JSON output, useful for scripting and automated analysis:

```bash
serial-cli --json send --port /dev/ttyUSB0 --data "AT\r\n"
```

Configure pretty-printed JSON with timestamps in `.serial-cli.toml`:

```toml
[output]
json_pretty = true
show_timestamp = true
```

### Check Log Files

If a log file path is configured, all output is written there in addition to (or instead of) stdout:

```toml
[logging]
level = "debug"
format = "text"
file = "/tmp/serial-cli.log"
```

Tail the log in real time while running commands:

```bash
tail -f /tmp/serial-cli.log
```

### Reset Configuration to Recover from Bad State

If Serial CLI is misbehaving due to a corrupted or invalid configuration file, reset to defaults:

```bash
serial-cli config reset
```

This restores all settings to their default values in memory. To persist the reset, save the configuration afterward:

```bash
serial-cli config save
```

Alternatively, manually delete the config file:

```bash
# Project-level config
rm .serial-cli.toml

# Global config (Linux/macOS)
rm ~/.config/serial-cli/config.toml

# Global config (Windows)
del %APPDATA%\serial-cli\config.toml
```

### Validate Configuration

Check the current configuration for validity:

```bash
serial-cli config validate
```

This checks for common issues such as:

- Zero baudrate
- Databits outside 5-8 range
- Stopbits not 1 or 2
- Invalid parity (must be `none`, `odd`, or `even`)
- Invalid logging level
- Zero max concurrent tasks

## Error Codes Reference

Serial CLI uses the following error types, defined in `src/error.rs`. Each error includes a human-readable message.

### SerialError (Top-Level)

| Variant | Display Format | Description |
|---------|---------------|-------------|
| `Serial(SerialPortError)` | `Serial port error: {detail}` | Wraps a serial port operation error |
| `Protocol(ProtocolError)` | `Protocol error: {detail}` | Wraps a protocol processing error |
| `Script(ScriptError)` | `Script error: {detail}` | Wraps a Lua script execution error |
| `Task(TaskError)` | `Task error: {detail}` | Wraps a task scheduler error |
| `Config(String)` | `Configuration error: {detail}` | Configuration loading or validation error |
| `Io(Error)` | `I/O error: {detail}` | Standard I/O error (file, pipe, etc.) |
| `Parse(String)` | `Parse error: {detail}` | Data parsing or format error |
| `InvalidInput(String)` | `Invalid input: {detail}` | Invalid command-line argument or parameter |
| `Lua(Error)` | `Lua error: {detail}` | LuaJIT runtime error |
| `VirtualPort(String)` | `Virtual port error: {detail}` | Virtual serial port creation or operation error |
| `UnsupportedBackend(String)` | `Unsupported backend for this platform: {detail}` | Selected backend not available on current OS |
| `MissingDependency(String, String)` | `Missing required dependency: {name}\nHint: {hint}` | External tool (e.g., socat) not found |
| `BackendInitFailed(String)` | `Backend initialization failed: {detail}` | Virtual backend failed to initialize |

### SerialPortError

| Variant | Display Format | Description |
|---------|---------------|-------------|
| `PortNotFound(String)` | `Port '{port}' not found` | Specified port does not exist |
| `PermissionDeniedWithHelp(String)` | `Permission denied for port '{port}': {help}` | Insufficient OS permissions |
| `Timeout(String)` | `Operation timeout on port '{port}'` | I/O operation exceeded timeout |
| `PortBusyWithHelp(String)` | `Port '{port}' is already in use: {help}` | Port held by another process |
| `InvalidConfig(String)` | `Invalid port configuration: {detail}` | Invalid serial port settings |
| `IoError(String)` | `Serial I/O error: {detail}` | Low-level serial I/O failure |

### ProtocolError

| Variant | Display Format | Description |
|---------|---------------|-------------|
| `NotFound(String)` | `Protocol '{name}' not found` | Requested protocol is not registered |
| `InvalidFrame(String)` | `Invalid frame format: {detail}` | Data does not match protocol structure |
| `ChecksumFailed` | `Checksum mismatch (expected: {exp}, got: {got})` | Frame checksum validation failed |
| `UnexpectedResponse(String)` | `Unexpected response: {detail}` | Device response did not match expected format |
| `Timeout(String)` | `Protocol timeout: {detail}` | No response received within timeout |
| `InvalidState(String)` | `Invalid protocol state: {detail}` | Protocol in an invalid state for the operation |

### ScriptError

| Variant | Display Format | Description |
|---------|---------------|-------------|
| `Syntax` | `Syntax error in {script}:{line}: {message}` | Lua parse error |
| `Runtime` | `Runtime error in {script}: {message}` | Lua runtime error |
| `ApiError(String)` | `Script API error: {detail}` | Error in script-to-host API call |
| `NotFound(PathBuf)` | `Script not found: {path}` | Script file does not exist |
| `SandboxViolation(String)` | `Sandbox violation: {detail}` | Script attempted a blocked operation |
| `ResourceLimitExceeded(String)` | `Resource limit exceeded: {detail}` | Script exceeded memory or time limits |

### TaskError

| Variant | Display Format | Description |
|---------|---------------|-------------|
| `Timeout(String, u64)` | `Task '{name}' timed out after {secs}s` | Task exceeded its timeout |
| `DependencyFailed(String)` | `Task dependency '{name}' failed` | A dependent task failed |
| `ResourceExhausted(String)` | `Resource exhausted: {detail}` | System resources depleted |
| `Cancelled(String)` | `Task '{name}' was cancelled` | Task was cancelled |
| `Deadlock(String)` | `Deadlock detected in task '{name}'` | Circular task dependency detected |
| `InvalidState(String)` | `Invalid task state: {detail}` | Task in an invalid state |

## Platform-Specific Issues

### Linux

- **Permission denied on `/dev/ttyUSB*` or `/dev/ttyACM*`:** Add user to `dialout` or `uucp` group (see "Port Not Found" section above).
- **ModemManager auto-claiming devices:** Disable or uninstall `ModemManager` if it interferes.
- **PTY limits:** The kernel limits PTY allocation. Check with `cat /proc/sys/kernel/pty/max` (default: 4096). If exhausted, increase: `sysctl kernel.pty.max=8192`.
- **Socat not installed:** Install via package manager (`apt`, `pacman`, `dnf`).

### macOS

- **Port names differ from Linux:** Use `/dev/cu.usbserial-*` or `/dev/tty.usbserial-*`, not `/dev/ttyUSB*`.
- **Third-party USB-serial drivers:** FTDI, Prolific, and CH340 chips may require separate driver installation. Check `System Information > USB` to verify the device is detected.
- **SIP restrictions:** macOS System Integrity Protection may block access to certain device paths. This rarely affects standard serial ports but can impact custom drivers.
- **socat installation:** `brew install socat`

### Windows

- **COM port naming:** Use `COM1`, `COM3`, etc. Do not use device paths like `\\.\COM3` unless the application specifically requires it.
- **USB-serial drivers:** Most USB-to-serial adapters require driver installation (FTDI, Prolific, Silicon Labs). Verify in Device Manager.
- **Named Pipe backend:** The `namedpipe` backend is Windows-native and does not require external dependencies.
- **Antivirus interference:** Some antivirus software may block access to COM ports. Add an exclusion if needed.
- **Elevated privileges:** Some COM ports (especially `COM1` and `COM2`) may require running Serial CLI as Administrator.

## When to File a Bug Report

File an issue on the [GitHub repository](https://github.com/pony/serial-cli/issues) when:

- **A clear error message is not provided.** If the CLI panics with a Rust backtrace instead of a structured error, this is a bug.
- **The error message is misleading.** If "Port not found" appears when the port clearly exists and is accessible.
- **A feature documented in this guide does not work as described.**
- **Platform-specific behavior differs significantly from expectations.** Include OS version, architecture, and Serial CLI version.
- **Performance regressions.** If a command that previously ran quickly now takes significantly longer.

When filing a bug report, include:

1. **Serial CLI version:** `serial-cli --version`
2. **Operating system and version:** `uname -a` (Unix) or `winver` (Windows)
3. **The exact command run** and its full output
4. **Log output** with `--verbose` enabled (or `logging.level = "trace"` in config)
5. **Steps to reproduce** the issue consistently
6. **Expected vs. actual behavior**
7. **Hardware details** (USB-to-serial adapter model, target device) if relevant
