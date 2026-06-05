# Event System Documentation

## Overview

The Serial CLI event system provides real-time updates using Tauri's event emitter. The Rust backend emits events; the React frontend listens via the `useTauriEvent` hook.

## Available Events

| Event | Direction | Payload |
|-------|-----------|---------|
| `data-received` | Backend → Frontend | `{ port_id, data: number[], timestamp, direction: "rx" }` |
| `data-sent` | Backend → Frontend | `{ port_id, data: number[], timestamp, direction: "tx" }` |
| `port-status-changed` | Backend → Frontend | `{ port_id, status: object, timestamp }` |
| `ports-changed` | Backend → Frontend | `{ added?: PortInfo[], removed?: string[] }` |
| `error-occurred` | Backend → Frontend | `{ error: string, timestamp: number }` |
| `virtual-port-created` | Backend → Frontend | `{ port_id, port_info: object, timestamp }` |
| `virtual-port-stopped` | Backend → Frontend | `{ port_id, timestamp }` |
| `virtual-port-stats-updated` | Backend → Frontend | `{ port_id, stats: object, timestamp }` |

### Payload Types

```typescript
interface DataEventPayload {
  port_id: string
  data: number[]      // Raw bytes
  timestamp: number   // Unix timestamp in milliseconds
  direction: 'rx' | 'tx'
}

interface PortsChangedPayload {
  added?: PortInfo[]
  removed?: string[]  // port IDs
}

interface ErrorPayload {
  error: string
  timestamp: number
}
```

## Frontend Usage

### useTauriEvent Hook

All event listening uses the generic `useTauriEvent<T>` hook with automatic cleanup:

```typescript
import { useTauriEvent } from '@/hooks/useTauriEvent'

function DataViewer() {
  const addPacket = useDataStore((s) => s.addPacket)

  useTauriEvent<DataEventPayload>('data-received', (payload) => {
    addPacket(payload)
  })

  useTauriEvent<DataEventPayload>('data-sent', (payload) => {
    addPacket(payload)
  })
}
```

### Error Handling

Global error events are handled in `AppShell`:

```typescript
import { listen } from '@tauri-apps/api/event'
import { toast } from 'sonner'

// In AppShell useEffect:
listen<ErrorPayload>('error-occurred', (e) => {
  toast.error(e.payload.error)
})
```

## Backend Usage

Emit events from Rust backend code:

```rust
use crate::events::emitter;

// Emit data received event
emitter::emit_data_received(
    app_handle,
    "port_id".to_string(),
    vec![0x01, 0x02, 0x03],
).await?;

// Emit error
emitter::emit_error(
    app_handle,
    "Something went wrong".to_string(),
).await?;
```

## Event Flow

```
Rust Backend                    React Frontend
────────────                    ──────────────
serial_core (read task)
  → emitter::emit_data_received
  → Tauri event bus ──────────→ useTauriEvent('data-received')
                                  → dataStore.addPacket()
                                  → Virtual list re-render

commands (error paths)
  → emitter::emit_error
  → Tauri event bus ──────────→ AppShell listen('error-occurred')
                                  → toast.error()
```

## Best Practices

1. **Use `useTauriEvent` hook** — handles listener setup and cleanup automatically
2. **Type payloads** — always pass a generic type parameter (`useTauriEvent<T>`)
3. **Keep handlers lightweight** — delegate to store actions, avoid expensive work in callbacks
4. **No polling** — use events instead of polling for real-time data; use polling only for periodic stats
