# Serial CLI

A command-line tool for serial port communication with embedded LuaJIT scripting. Supports multiple protocols (Modbus RTU/ASCII, AT Commands, line-based, and custom Lua) with structured JSON output. Targets AI/automation workflows.

## Language

**Port**:
A serial interface on the device (e.g., `/dev/ttyUSB0`, `COM1`). Exists whether or not you're connected to it.
_Avoid_: Serial device, COM port

**Connection**:
An open handle to a port, configured with a preset (baud rate, data bits, parity, stop bits, timeout) and optionally a protocol. This is what the user interacts with after opening a port.
_Avoid_: Port handle, session

**Preset**:
A saved set of serial port settings that can be reused across connections.
_Avoid_: Profile, saved connection

**Protocol**:
A communication spec that defines how bytes are framed and interpreted (e.g., Modbus RTU, AT Commands). The tool implements protocols as handlers with `parse` (incoming) and `encode` (outgoing).
_Avoid_: Codec, formatter

**Frame**:
A complete protocol data unit — e.g., a Modbus RTU frame with slave ID, function code, data, and CRC. The protocol handler assembles frames from raw bytes and produces frames for transmission.
_Avoid_: Message, packet, PDU

**Capture**:
Recording the raw bytes flowing through a serial port for later analysis. Can run on physical ports (sniffer) or virtual ports (monitor).
_Avoid_: Sniffer, monitor, trace, log

**Virtual Port**:
An emulated serial port pair — writing to one end reads from the other. Backends include PTY (Unix), socat (cross-platform), and named pipe (Windows). Used for testing and bridging.
_Avoid_: Virtual COM, loopback, pseudo-port

**Daemon**:
A background process that serves JSON-RPC 2.0 requests from clients (AI agents, scripts, other programs). Provides the same port/protocol operations as the CLI but over a persistent socket.
_Avoid_: Server, service

**Data Listener**:
A background async task that continuously reads incoming data from an open connection and broadcasts it to subscribers. Without it, callers must poll for data.
_Avoid_: IoLoop, read loop, background reader

**Batch**:
A sequence of commands or scripts executed together with configurable concurrency, timeout, and error handling.
_Avoid_: Job, task queue
