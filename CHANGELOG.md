# Changelog
All notable changes to this project will be documented in this file.

---

## [0.6.0] - 2026-05-14

### 🚀 Server Mode (NEW!)
- **JSON-RPC 2.0 daemon** over Unix socket for AI/automation workflows
- **Persistent connections** — 10-100x latency improvement (50-200ms → 1-5ms)
- **Protocol persistence** — Load custom protocols once, available globally
- **Multi-client support** — Up to 10 concurrent connections
- **10 RPC methods**: port_list, port_open, port_close, port_send, port_recv, protocol_list, protocol_load, protocol_unload, connection_list, server_stats
- **Session management** — PID tracking, stale session cleanup
- **Configurable** — Unix socket path, max connections, log path

---

## [0.2.1] - 2026-05-11

### 🎉 GUI Architecture Refactoring (v2.1 Design Compliance)

#### State Management Overhaul
- **Zustand Migration** - Replaced 8 React Context providers with Zustand stores
  - Created `connectionStore`, `dataStore`, `virtualPortStore`, `protocolStore`
  - Created `scriptStore`, `navigationStore`, `settingsStore`, `notificationStore`
  - Reduced provider nesting from 8 layers to 6 layers (-25%)
  - Improved performance with selective subscriptions

#### Component Library Modernization
- **shadcn/ui Integration** - Migrated from custom components to shadcn/ui + Radix UI
  - Button, Dialog, Select, Tabs, Slider, Switch components
  - Toast system migrated to Sonner (modern notification library)
  - Custom Cyber-Industrial theme variants (signal, amber, alert)
  - Consistent styling and better accessibility

#### Performance Enhancements
- **Virtual Scrolling** - Added react-virtuoso for large datasets
  - RX data viewer now supports 10000+ packets smoothly
  - Dynamic row height support for varied content
  - Memory-efficient FIFO buffer (auto cleanup at 10000 packets)

#### View Structure Restructuring
- **Merged Views** - Combined ports + data views into unified Terminal Workbench
  - Reduced from 6 views to 5 views
  - State-driven layout: DisconnectedState → ConnectedState → ErrorState
  - Improved UX with context-aware interface

#### UI Components Created
```
components/terminal/
├── TerminalWorkbench.tsx    # State-driven main container
├── DisconnectedState.tsx     # Port selection with quick connect
├── ConnectedState.tsx        # Active workbench with RX/TX/Side panels
├── ErrorState.tsx            # Error handling with recovery options
├── RxDataViewer.tsx          # Virtual scrolling data display
├── TxSender.tsx              # Multi-format data sender (HEX/ASCII)
├── SidePanel.tsx             # Port details, stats, protocol control
└── LogPanel.tsx              # Collapsible system logs
```

#### Bug Fixes (Critical)
- **Tauri 2.0 Permissions** - Fixed event.listen permission errors
  - Created `src-tauri/capabilities/default.json`
  - Added core:event, core:path, core:window permissions
- **Lua Runtime Panic** - Fixed async context runtime drop error
  - Changed `Arc<Runtime>` to `tokio::runtime::Handle`
  - Safe to use in async contexts without panics
- **React Infinite Loop** - Fixed ScriptPanel maximum update depth error
  - Wrapped callback functions with `useCallback`
  - Proper dependency array management

#### Code Quality
- **Deleted Obsolete Code** - Removed 1272 lines of legacy code
  - Deleted `components/data/` directory (merged to terminal)
  - Deleted `components/ports/` directory (merged to terminal)
  - Removed `DataContext.tsx` (migrated to dataStore)
  - Removed `ToastContext.tsx` (replaced by Sonner)

#### Navigation Updates
- Updated sidebar to 5 items (terminal, virtual, scripts, protocols, settings)
- Updated global shortcuts: ⌘1 (terminal), ⌘2 (virtual), ⌘3-5 (scripts, protocols, settings)
- Updated Command Palette with new navigation structure
- Updated Keyboard Shortcuts Help

#### Breaking Changes (Developer-facing)
```typescript
// State Management
- import { usePort } from '@/contexts/PortContext'
+ import { useConnectionStore } from '@/stores'

- import { useData } from '@/contexts/DataContext'
+ import { useDataStore } from '@/stores'

// Navigation
- navigateTo('ports')
- navigateTo('data')
+ navigateTo('terminal')

// Components
- import { PortsPanel } from '@/components/ports/PortsPanel'
- import { DataViewer } from '@/components/data/DataViewer'
+ import { TerminalWorkbench } from '@/components/terminal'

// Toast Notifications
- import { useToast } from '@/contexts/ToastContext'
+ import { toast } from 'sonner'
```

#### Documentation
- Created `MIGRATION.md` - Comprehensive migration guide for developers
- Updated architecture documentation
- Commit messages follow conventional commits format

#### Testing
- TypeScript compilation: 0 errors
- Rust compilation: 0 errors, 31 warnings (unused code only)
- All functionality tested and working

#### Dependencies Added
```json
{
  "@radix-ui/react-*": "latest",
  "tailwindcss-animate": "^1.0.7",
  "zustand": "^4.5.0",
  "react-virtuoso": "^4.7.0",
  "sonner": "^1.4.0",
  "react-hook-form": "^7.50.0",
  "zod": "^3.22.0",
  "@hookform/resolvers": "^3.3.0"
}
```

#### Performance Metrics
| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Provider Layers | 8 | 6 | -25% |
| Context Files | 8 | 6 | -25% |
| Custom Components | 100% | ~40% | -60% |
| Virtual Scrolling | ❌ | ✅ | New |
| Lines of Code | ~3500 | ~2228 | -36% |
| TypeScript Errors | 0 | 0 | ✅ |

---

---

## Release Process

### Prerequisites

- Rust toolchain installed
- git-cliff: `cargo install git-cliff`
- Write access to GitHub Repository

### 1. Prepare Release

```bash
# Check readiness
./scripts/release.sh v1.2.3 --check-only

# Prepare new version (runs checks then updates versions)
./scripts/release.sh v1.2.3

# Or skip checks if you already verified
./scripts/release.sh v1.2.3 --no-checks

# Review changes
git diff
git status

# Commit version changes
git commit -am "chore: prepare release v1.2.3"
```

### 2. Create Release

```bash
# Create and push tag
git tag -a v1.2.3 -m "Release v1.2.3"
git push origin v1.2.3
```

After pushing the tag, GitHub Actions will:
1. Build binaries for all platforms
2. Create GitHub Release
3. Update Homebrew, Scoop, and AUR
4. Publish to crates.io

### 3. Verify Release

- [ ] GitHub Release created
- [ ] All platform builds successful
- [ ] crates.io publish successful
- [ ] CHANGELOG.md updated

### Rollback

If release fails or issues are found:

```bash
# Delete GitHub Release and tag
gh release delete v1.2.3 --cleanup-tag

# Delete local tag
git tag -d v1.2.3

# Fix issues and re-release
```

### Conventional Commits

Commit message format: `<type>(<scope>): <subject>`

**Types:** `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`, `ci`

**Examples:**
```bash
git commit -m "feat(cli): add protocol list command"
git commit -m "fix(protocol): handle empty response correctly"
git commit -m "docs(readme): update installation instructions"
```

---

## [0.4.0] - 2026-04-24 ✅ RELEASED

### 🎉 Virtual Port Backend Architecture

- **Pluggable backend system** - `VirtualBackend` trait enables extensible backends
- **PTY Backend** (Unix/macOS) - Refactored from existing code, improved performance
- **NamedPipe Backend** (Windows) - Native Windows named pipes implementation
- **Socat Backend** (Cross-platform) - Socat-based virtual ports with auto-detection
- **Platform auto-detection** - Automatically selects best backend (PTY on Unix, NamedPipe on Windows)
- **BackendFactory** - Priority chain: CLI flag → config file → auto-detection
- **Config integration** - `virtual.backend` setting for default backend
- **CLI enhancement** - `--backend` flag on `virtual create` command
- **Error handling** - New error types: `UnsupportedBackend`, `MissingDependency`, `BackendInitFailed`
- **Helpful error messages** - Installation hints for missing dependencies (socat)

### Architecture

- Created `serial_core/backends/` module with trait-based design
- `VirtualBackend` trait: `create_pair()`, `is_healthy()`, `get_stats()`, `cleanup()`
- Backend implementations: `PtyBackend`, `NamedPipeBackend`, `SocatBackend`
- Runtime polymorphism via `Box<dyn VirtualBackend>`
- Backward compatibility maintained via type aliases

### Usage

```bash
# Auto-detect (recommended)
serial-cli virtual create

# Explicit backend selection
serial-cli virtual create --backend pty
serial-cli virtual create --backend socat
serial-cli virtual create --backend namedpipe

# Set default in config
serial-cli config set virtual.backend socat
```

### Documentation

- Updated README.md with virtual port examples
- Added backend installation guide (socat dependencies)
- Updated feature list and troubleshooting section
- Added design spec: `docs/superpowers/specs/2026-04-24-virtual-port-backends-design.md`

### Testing

- All 214 tests passing
- Added unit tests for backend implementations
- Added BackendType parsing and detection tests
- Property-based tests for backend selection

---

## [0.3.0] - 2026-04-24

### Sniffing — Session Management

- **`sniff start`** now spawns a background daemon process, freeing the parent shell for further commands
- **`sniff stop`** — gracefully stops an active sniff session (SIGTERM → SIGKILL fallback)
- **`sniff stats`** — shows port, PID, uptime, and config for the active session
- **`sniff save`** — saves captured packets from the session's output file to a specified path
- Session registry uses file-based state (PID + config in cache dir) with stale session auto-cleanup
- Cross-platform process management (Unix via libc, Windows via Win32 API)

### Batch Processing — Enhanced

- **Variable substitution** — `${VAR}` and `$VAR` syntax in script paths and `set` values, with environment variable fallback
- **`set KEY value`** directive — define variables within batch files
- **Loop blocks** — `loop N` ... `end` with validated parsing (detects unclosed loops, unexpected `end`, nested loops)
- **`sleep MS`** directive — add delays between script executions
- **`batch list`** — now searches current directory + `~/.config/serial_cli/` for `.batch`, `.txt`, `.lua` files
- **Error reporting** — per-script error messages displayed in batch summary output

### Fixes

- Fixed daemon pipe leak: explicitly close stdin/stdout/stderr handles before `std::mem::forget(child)`, preventing blocking writes under load
- Removed unused `_display` parameter from sniff daemon (reduces CLI surface)

---

## [0.2.0] - 2025-04-09

### 🎉 Major Features - GUI Application Complete

#### Frontend (React + Tauri)
- ✅ **Complete UI Overhaul** - Cyber-industrial aesthetic design
- ✅ **Serial Port Management** - Full port configuration, open/close, status monitoring
- ✅ **Real-time Data Monitoring** - Live data display with RX/TX distinction
- ✅ **Lua Script Editor** - Monaco Editor integration with syntax highlighting
- ✅ **Protocol Management** - Built-in and custom protocol loading with validation
- ✅ **Settings System** - Comprehensive configuration with persistence
- ✅ **Data Export** - TXT/CSV/JSON formats with filtering options
- ✅ **System Notifications** - Cross-platform desktop notifications
- ✅ **Keyboard Shortcuts** - Command palette and global shortcuts
- ✅ **Data Persistence** - Auto-save for settings, scripts, protocols, and recent ports

#### Backend (Tauri Commands)
- ✅ **Serial Port Commands** - list_ports, open_port, close_port, get_port_status
- ✅ **Data Transfer** - send_data, read_data with event emission
- ✅ **Script Execution** - execute_script with real LuaJIT runtime
- ✅ **Protocol Management** - load_protocol, validate_protocol, list_protocols
- ✅ **Configuration** - get_config, update_config
- ✅ **Window Control** - show_window, hide_window, toggle_window

#### Design System
- ✅ **Icon System** - lucide-react SVG icons (replaced emoji)
- ✅ **Color Scheme** - signal (green), alert (red), amber (yellow), info (blue)
- ✅ **Typography** - Instrument Sans, JetBrains Mono, Instrument Serif
- ✅ **Animations** - fade-in, slide-up, pulse-slow transitions
- ✅ **Components** - Panel, Toast, CommandPalette with consistent styling

### Technical Achievements
- ✅ **Type Safety** - 100% TypeScript strict mode compliance
- ✅ **State Management** - Context-based with proper separation of concerns
- ✅ **Event System** - Real-time data flow with Tauri events
- ✅ **Persistence** - localStorage integration for all user data
- ✅ **Error Handling** - Comprehensive error catching and user feedback
- ✅ **Performance** - Optimized rendering and data handling

### Breaking Changes
- None (backward compatible)

### Known Issues
- None (production ready)

---

## [0.1.0] - 2025-04-01

### Features
- Initial release of Serial CLI
- Core serial port management
- Lua scripting support
- Modbus RTU/ASCII protocols
- AT Command protocol
- Interactive CLI mode
- Batch execution mode
- JSON output format

### Bug Fixes
- Initial implementation

### Documentation
- Initial documentation setup
