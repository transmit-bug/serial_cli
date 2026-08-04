type EventHandler<T = unknown> = (event: { payload: T }) => void;
type UnlistenFn = () => void;

const listeners = new Map<string, Set<EventHandler>>();

/**
 * Mock implementation of @tauri-apps/api/event listen().
 * Same signature as the real function.
 */
export function listen<T>(
  event: string,
  handler: EventHandler<T>,
): Promise<UnlistenFn> {
  if (!listeners.has(event)) {
    listeners.set(event, new Set());
  }
  listeners.get(event)!.add(handler as EventHandler);

  const unlisten = () => {
    listeners.get(event)?.delete(handler as EventHandler);
  };

  return Promise.resolve(unlisten);
}

/**
 * Mock implementation of @tauri-apps/api/event emit().
 */
export function emit(event: string, payload?: unknown): Promise<void> {
  mockEmit(event, payload);
  return Promise.resolve();
}

/**
 * Developer tool: manually trigger events from browser console.
 * Usage: __MOCK_EMIT__("data-received", { port_id: "/dev/ttyUSB0", data: [72,69,76,76,79] })
 */
export function mockEmit(event: string, payload?: unknown): void {
  const handlers = listeners.get(event);
  if (handlers) {
    handlers.forEach((fn) => fn({ payload }));
  }
}

// Expose to window for console access
if (typeof window !== "undefined") {
  // biome-ignore lint/suspicious/noExplicitAny: developer debug tool
  (window as any).__MOCK_EMIT__ = mockEmit;
}
