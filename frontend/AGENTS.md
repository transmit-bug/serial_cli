# AGENTS.md — Frontend

Tauri 2.0 GUI app. React 19 + TypeScript, Zustand 5 for state, Tailwind CSS 4 + shadcn/ui for styling.

## Architecture

### Navigation — No Router

Navigation is **store-driven** via `useUIStore.currentPage` (a `PageName` union). `AppShell` maps page names to components via a `PAGES` record. There is no URL-based routing. **Do not add react-router** — it would conflict with the existing pattern.

### State Management

11 Zustand stores in `src/stores/`, **no barrel export** — import directly from the file (e.g. `import { useConnectionStore } from "@/stores/connection"`). The remote device store (`stores/remote.ts`) manages the device registry plus the workbench and per-connection streaming state (`streaming`, `rxBuffers`).

**Patterns:**
- Stores call `tauriApi` directly (no service layer, no React Query)
- Async actions live inside the store (not in separate thunks)
- Cross-store communication: `useXxxStore.getState().action()` (Zustand external, no React re-render)
- Persistent state: settings → backend TOML; quick commands & locale → `localStorage`

### Tauri Bridge

All Tauri communication goes through **`src/lib/tauri-api.ts`** — a single `tauriApi` object wrapping `invoke()` calls. ~70 commands, all `snake_case` strings.

**Two communication modes:**
1. **invoke (request/response):** `tauriApi.xxx()` → `invoke("command_name", { args })`
2. **events (push from backend):** `useTauriEvent<T>(eventName, handler)` hook with auto-cleanup

**Key events:** `data-received`, `data-sent`, `ports-changed`, `error-occurred`, `server-status-changed`, plus remote-stream events `remote-data-received` / `remote-stream-error` / `remote-stream-closed` (listened via `setupRemoteDataListener()` in `stores/remote.ts`).

### Component Structure

```
components/
  layout/       — AppShell, Sidebar, StatusBar (app chrome)
  terminal/     — TerminalWorkbench (main page), ConnectionBar, RxViewer, TxSender
  virtual/      — Virtual port management
  remote/       — RemotePage: LAN device registry + workbench (open port, send/recv, live stream)
  editor/       — Monaco-based Lua script editor
  server/       — Embedded JSON-RPC server UI
  settings/     — Single page, multi-tab
  shared/       — ErrorBoundary, CommandPalette, ShortcutsHelp
  ui/           — shadcn/ui primitives
```

Layout: `AppShell` = Sidebar + main content + StatusBar. Pages are full-height flex layouts. `TerminalWorkbench` uses `react-resizable-panels` (API uses `Group`, not `PanelGroup`).

## Styling

**Tailwind CSS 4** with CSS-based config in `index.css` (no `tailwind.config.js`).

**Catppuccin color palette** — dark (Mocha) default, light (Latte) via `[data-theme="light"]`.

**Dual CSS variable system:**
1. shadcn/ui variables (`--primary`, `--background`) — used **only** by shadcn/ui components internally
2. Custom semantic tokens (`--color-base`, `--color-surface`, `--color-text`, `--color-accent`) — used in **all custom components**

**In components, use:** `bg-base`, `text-text`, `text-text-muted`, `bg-surface`, `border-border`, `text-accent`, `bg-success`, `text-danger`. **Do not** use `--primary`, `--background` etc. directly.

Theme applied via `data-theme` attribute on `<html>`, not a class.

## Conventions

- **Components:** PascalCase, one per file
- **Stores:** `useXxxStore` named export, camelCase file name
- **Hooks:** `useXxx` prefix, camelCase file
- **Types:** All in `src/types/index.ts` (single file), PascalCase interfaces
- **Path alias:** `@/*` → `./src/*`
- **Biome** for linting/formatting (not ESLint)
- **UI components:** Use shadcn/ui (`npx shadcn@latest add <component>`), don't hand-write basic components
- **i18n:** `useTranslation()` → `t("nav.terminal")`, locales in `src/i18n/locales/{en,zh}.json`

## Testing

**vitest** with `jsdom`, globals enabled. Test setup in `src/test/setup.ts` mocks Tauri APIs, i18next, sonner, localStorage.

Store tests mock `@/lib/tauri-api` with `vi.mock()`. Each store has a `resetStore()` helper. Test files co-located: `stores/connection.test.ts`.

## Mock Layer (pnpm dev without Rust)

`pnpm dev` runs independently of `cargo tauri dev`. Vite aliases redirect Tauri imports to `src/lib/mock/`:

- `@/lib/tauri-api` → `src/lib/mock/index.ts`
- `@tauri-apps/api/event` → `src/lib/mock/events.ts`
- `@tauri-apps/plugin-dialog` → `src/lib/mock/dialog.ts`

Switch: `process.env.TAURI_PLATFORM` (set by `cargo tauri dev`, absent in `pnpm dev`).

**Debug events from browser console:**
```js
__MOCK_EMIT__("data-received", { port_id: "/dev/ttyUSB0", data: [72,69,76,76,79], timestamp: Date.now(), direction: "rx" })
```

Mock code is fully isolated — `cargo tauri dev` / `cargo tauri build` never touches it.

## Gotchas

- **Connection polling** is managed outside React (module-level `Map<string, timer>` in `connection.ts`), not in useEffect — intentional, survives re-renders
- **`useSerialScriptStore` and `useStandaloneScriptStore`** are both exported from `serialScript.ts`
- **Command sequences** use `AbortController` for cancellation (module-level variable in `commands.ts`)
- **Ring buffer** in `useDataStore` — when `maxPackets` hit, oldest packet dropped
- **`@tanstack/react-virtual`** used in `RxViewer` for virtualized packet list
- **Monaco editor** (`@monaco-editor/react`) for Lua script editing
- **`sonner`** for toast notifications, configured in `App.tsx`
