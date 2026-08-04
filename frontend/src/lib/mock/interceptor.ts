import type { MockState } from "./state";

export type Handler = (
  args: Record<string, unknown>,
  state: MockState,
) => unknown;

const handlers = new Map<string, Handler>();

export function registerHandlers(map: Record<string, Handler>): void {
  for (const [name, fn] of Object.entries(map)) {
    handlers.set(name, fn);
  }
}

export function dispatch(
  command: string,
  args: Record<string, unknown>,
  state: MockState,
): Promise<unknown> {
  const handler = handlers.get(command);
  if (!handler) {
    console.warn(`[mock] No handler for "${command}", returning undefined`);
    return Promise.resolve(undefined);
  }
  try {
    const result = handler(args, state);
    return Promise.resolve(result);
  } catch (err) {
    console.error(`[mock] Handler "${command}" threw:`, err);
    return Promise.reject(err);
  }
}
