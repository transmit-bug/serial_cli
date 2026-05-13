# Event System Documentation

## Overview

The Serial CLI event system provides real-time updates for various application events using Tauri's event emitter. This allows components to react to changes without constant polling.

## Available Events

### Serial Data Events

#### `data-received`
Emitted when data is received from a serial port.

**Payload:**
```typescript
{
  port_id: string
  data: number[]      // Raw bytes
  timestamp: number   // Unix timestamp in milliseconds
  direction: 'rx'
}
```

#### `data-sent`
Emitted when data is sent to a serial port.

**Payload:**
```typescript
{
  port_id: string
  data: number[]      // Raw bytes
  timestamp: number   // Unix timestamp in milliseconds
  direction: 'tx'
}
```

### Port Status Events

#### `port-status-changed`
Emitted when a port's status changes (opened, closed, error, etc.).

**Payload:**
```typescript
{
  port_id: string
  status: PortStatus  // Port status object
  timestamp: number
}
```

### Virtual Port Events

#### `virtual-port-created`
Emitted when a virtual port is created.

**Payload:**
```typescript
{
  port_id: string
  port_info: VirtualPortInfo
  timestamp: number
}
```

#### `virtual-port-stopped`
Emitted when a virtual port is stopped.

**Payload:**
```typescript
{
  port_id: string
  timestamp: number
}
```

#### `virtual-port-stats-updated`
Emitted when virtual port statistics are updated.

**Payload:**
```typescript
{
  port_id: string
  stats: PortStats
  timestamp: number
}
```

### Error Events

#### `error-occurred`
Emitted when an application error occurs.

**Payload:**
```typescript
{
  error: string      // Error message
  timestamp: number
}
```

## Frontend Usage

### Basic Event Listening

Using the `useEvents` hook:

```typescript
import { useEvents } from '@/hooks/useEvents'

function MyComponent() {
  const { onDataReceived, onError } = useEvents()

  useEffect(() => {
    const cleanup = onDataReceived((event) => {
      console.log('Data received:', event.data)
      console.log('From port:', event.port_id)
    })

    return cleanup
  }, [onDataReceived])

  return <div>...</div>
}
```

### Event Filtering

Filter events by specific criteria:

```typescript
useEffect(() => {
  const cleanup = onDataReceived(
    (event) => {
      // Only process events from specific port
      console.log('Data from COM1:', event.data)
    },
    (event) => event.port_id === 'COM1' // Filter function
  )

  return cleanup
}, [onDataReceived])
```

### Port-Specific Events

Using the specialized `useSerialDataEvents` hook:

```typescript
import { useSerialDataEvents } from '@/hooks/useEvents'
import { useConnectionStore } from '@/stores'

function DataViewer() {
  const { portId } = useConnectionStore()
  const { onDataReceived } = useSerialDataEvents(portId)

  useEffect(() => {
    const cleanup = onDataReceived((event) => {
      // Only receives data for the current port
      console.log('Port data:', event.data)
    })

    return cleanup
  }, [onDataReceived])

  return <div>...</div>
}
```

### Custom Events

Listen to custom events not predefined in the system:

```typescript
const { onCustomEvent } = useEvents()

useEffect(() => {
  const cleanup = onCustomEvent('my-custom-event', (data) => {
    console.log('Custom event data:', data)
  })

  return cleanup
}, [onCustomEvent])
```

### Error Tracking

Track errors throughout the application:

```typescript
import { ErrorEventToast, useErrorCount } from '@/components/error/ErrorEventToast'

function App() {
  return (
    <>
      <ErrorEventToast /> {/* Global error handler */}
      {/* Your app content */}
    </>
  )
}

function ErrorStats() {
  const { errorCount, recentErrors, resetErrorCount } = useErrorCount()

  return (
    <div>
      <p>Total errors: {errorCount}</p>
      <button onClick={resetErrorCount}>Reset</button>
      <ul>
        {recentErrors.map((error, i) => (
          <li key={i}>{error.error}</li>
        ))}
      </ul>
    </div>
  )
}
```

## Backend Usage

### Emitting Events

From Rust backend code:

```rust
use crate::events::emitter;

// Emit data received event
emitter::emit_data_received(
    app_handle,
    "port_id".to_string(),
    vec![0x01, 0x02, 0x03],
).await?;

// Emit port status change
emitter::emit_port_status_changed(
    app_handle,
    "port_id".to_string(),
    serde_json::json!({"status": "open"}),
).await?;

// Emit error
emitter::emit_error(
    app_handle,
    "Something went wrong".to_string(),
).await?;
```

## Event Filtering Best Practices

1. **Filter early**: Use the filter parameter in event hooks to avoid unnecessary re-renders
2. **Port-specific filtering**: Use `useSerialDataEvents` for port-specific data
3. **Cleanup**: Always return the cleanup function from useEffect
4. **Performance**: Avoid expensive operations in event callbacks

## Example: Real-time Port Monitor

```typescript
import { useEvents } from '@/hooks/useEvents'
import { useState } from 'react'

function PortMonitor({ portId }: { portId: string }) {
  const { onDataReceived, onDataSent } = useEvents()
  const [rxBytes, setRxBytes] = useState(0)
  const [txBytes, setTxBytes] = useState(0)

  useEffect(() => {
    const cleanupRx = onDataReceived(
      (event) => {
        setRxBytes((prev) => prev + event.data.length)
      },
      (event) => event.port_id === portId // Filter for specific port
    )

    const cleanupTx = onDataSent(
      (event) => {
        setTxBytes((prev) => prev + event.data.length)
      },
      (event) => event.port_id === portId
    )

    return () => {
      cleanupRx()
      cleanupTx()
    }
  }, [onDataReceived, onDataSent, portId])

  return (
    <div>
      <h3>Port: {portId}</h3>
      <p>RX: {rxBytes} bytes</p>
      <p>TX: {txBytes} bytes</p>
    </div>
  )
}
```

## Available Components

### `PortStatusIndicator`
Real-time port status indicator with activity pulses.

### `VirtualPortEventLog`
Live event log for virtual port activities.

### `ErrorEventToast`
Global error handler that displays toasts for error events.

## Type Definitions

```typescript
interface SerialEventData {
  port_id: string
  data: number[]
  timestamp: number
  direction: 'rx' | 'tx'
}

interface PortStatusEventData {
  port_id: string
  status: any
  timestamp: number
}

interface VirtualPortEventData {
  port_id: string
  port_info?: any
  stats?: any
  timestamp: number
}

interface ErrorEventData {
  error: string
  timestamp: number
}
```
