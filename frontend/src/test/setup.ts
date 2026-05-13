import '@testing-library/jest-dom/vitest';

// Mock @tauri-apps/api for tests running in jsdom
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(async () => undefined),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(async () => vi.fn()),
}));

vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: () => ({
    setFullscreen: vi.fn(),
    isFullscreen: vi.fn(async () => false),
  }),
}));
