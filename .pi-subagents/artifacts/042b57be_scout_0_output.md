# Frontend Investigation for AGENTS.md

## 1. Architecture & Patterns

### Routing / Navigation
**No router library.** Navigation is state-driven via `useUIStore.currentPage` (a `PageName` union: `"terminal" | "virtual" | "editor" | "server" | "settings"`). `AppShell.tsx` maps page names to components via a `PAGES` record and renders the active one. There is no URL-based routing — this is a single-page Tauri app.

### State Management (Zustand 5)
**10 stores, no barrel export.** Each store is a standalone file in `src/stores/` with a named `useXxxStore` export. There is no `stores/index.ts` — import directly from the file.

| Store | File | Purpose |
|---|---|---|
| `useConnectionStore` | `connection.ts` | Port discovery, connect/disconnect, active port, polling (2s interval for port stats) |
| `useDataStore` | `data.ts` | Packet buffer (ring buffer, max 10000), display format, search, export |
| `useUIStore` | `ui.ts` | Current page, sidebar/panel collapse state, locale (persisted to localStorage) |
| `useServerStore` | `server.ts` | TCP/Unix socket server status, has `setupServerEventListener()` for Tauri events |
| `useCommandStore` | `commands.ts` | Quick commands (localStorage-persisted), command sequences with delay/waitFor/loop |
| `useScriptStore` | `script.ts` | Registered scripts + user script files (Monaco editor state) |
| `useSerialScriptStore` | `serialScript.ts` | Per-port attached script status, UI actions, standalone script actions |
| `usePresetsStore` | `presets.ts` | Connection presets (backend-synced), cross-store: calls `useConnectionStore.getState()` |
| `useSettingsStore` | `settings.ts` | App config (load/update/reset via tauriApi) |
| `useVirtualPortStore` | `virtualPort.ts` | Virtual port CRUD, stats, throughput calculation, captured packets |
| `useLogStore` | `log.ts` | Backend log viewer (read/clear logs) |

**Key patterns:**
- Stores call `tauriApi` directly (no service layer / no React Query)
- Cross-store communication uses `useXxxStore.getState()` (Zustand external), e.g. `presets.ts` → `connection.ts`, `commands.ts` → `data.ts`
- Async actions live inside the store (not in separate thunks/services)
- Polling: `connection.ts` uses `setInterval` in a `Map<string, timer>` for port stats — not React effects
- Persistent state: `useCommandStore` and `useUIStore` use `localStorage` directly; settings/config use backend TOML

### Tauri Bridge
**Centralized API wrapper** at `src/lib/tauri-api.ts` — a single `tauriApi` object wrapping all `invoke()` calls with typed signatures. ~70 commands exposed.

**Two communication patterns:**
1. **invoke (request/response):** Components/stores call `tauriApi.xxx()` → `invoke("command_name", { args })`. All Tauri command names use `snake_case`.
2. **events (push from backend):** `useTauriEvent<T>(eventName, handler)` hook wraps `listen()` with auto-cleanup. Used in `TerminalWorkbench.tsx` for `data-received` / `data-sent` events, and in `AppShell.tsx` for `ports-changed` / `error-occurred`.

**Key Tauri events:**
- `data-received` / `data-sent` — payload: `{ port_id, data: number[], timestamp, direction }`
- `ports-changed` — payload: `{ added: string[], removed: string[] }`
- `error-occurred` — payload: `{ error: string }`
- `server-status-changed` — payload: `{ running, socket_path }`

**`useTauriCommand` hook** (`src/hooks/useTauriCommand.ts`): wraps a command with `loading`/`error` state. Used for fire-and-forget UI actions.

### Component Structure
```
src/components/
  layout/       — AppShell, Sidebar, StatusBar (app chrome)
  terminal/     — TerminalWorkbench (main page), ConnectionBar, RxViewer, TxSender, RightPanel*, SequenceEditor
  virtual/      — VirtualPortsPage, VirtualPortList, VirtualPortDetail, VirtualPacketTable, BridgeVisualization
  editor/       — EditorPage (Monaco), ProtocolList, TemplateList, StandaloneActions
  server/       — ServerPage, ServerStats, ServerStatus, ServerConfig, ServerConnections
  settings/     — SettingsPage (single file, multi-tab)
  shared/       — ErrorBoundary, ShortcutsHelp, CommandPalette, CommandSender
  ui/           — shadcn/ui components (button, dialog, select, tabs, etc.)
```

**Layout pattern:** `AppShell` = Sidebar + main content + StatusBar. Pages are full-height flex layouts. `TerminalWorkbench` uses `react-resizable-panels` for the RX/TX split and right panel.

### Styling
- **Tailwind CSS 4** with `@tailwindcss/vite` plugin (no `tailwind.config.js` — uses CSS-based config in `index.css`)
- **Catppuccin color palette** — dark (Mocha) default, light (Latte) via `[data-theme="light"]`
- **Dual CSS variable systems:**
  1. shadcn/ui variables (`--primary`, `--background`, etc.) in `:root` / `[data-theme="light"]`
  2. Custom semantic tokens (`--color-base`, `--color-surface`, `--color-text`, `--color-accent`, etc.) in `@theme` block
- Components use **custom semantic tokens** (`bg-base`, `text-text`, `text-text-muted`, `bg-surface`, `border-border`, `text-accent`, `bg-success`, `text-danger`, etc.) — NOT the shadcn variables directly
- `cn()` utility = `clsx` + `tailwind-merge` (standard shadcn pattern)
- Theme applied via `data-theme` attribute on `<html>`, set in `useTheme` hook and `applyStoredTheme()` in `main.tsx`

### shadcn/ui
- Config: `components.json` — style `new-york`, base color `zinc`, CSS variables enabled, icons via `lucide-react`
- Components in `src/components/ui/` — standard shadcn pattern with `cva` variants
- Use `npx shadcn@latest add <component>` to add new ones

### i18n
- `i18next` + `react-i18next` + `i18next-browser-languagedetector`
- Two locales: `en` and `zh` (JSON files in `src/i18n/locales/`)
- Fallback: `zh`
- Detection order: localStorage (`serial-cli-locale`) → navigator
- Usage: `const { t } = useTranslation()` then `t("nav.terminal")`

## 2. Key Conventions

### Naming
- **Components:** PascalCase, one per file (e.g. `ConnectionBar.tsx`)
- **Stores:** `useXxxStore` named export, camelCase file name (e.g. `useConnectionStore` from `connection.ts`)
- **Hooks:** `useXxx` prefix, camelCase file (e.g. `useTauriEvent.ts`)
- **Types:** PascalCase interfaces, all in `src/types/index.ts` (single file)
- **Tauri commands:** snake_case strings (e.g. `"open_port"`, `"list_scripts"`)
- **Tauri event names:** kebab-case (e.g. `"data-received"`, `"ports-changed"`)

### Data Flow
1. Backend emits events → `useTauriEvent` in components → store actions update state
2. User action → store action → `tauriApi.xxx()` → `invoke()` → backend
3. Store-to-store: `useXxxStore.getState().action()` (external, no React re-render needed)
4. Config persistence: settings → backend TOML; quick commands/sequences → localStorage; locale → localStorage

### Test Patterns
- **vitest** with `jsdom` environment, globals enabled
- **Test setup** (`src/test/setup.ts`): mocks `@tauri-apps/api/core`, `@tauri-apps/api/event`, `@tauri-apps/plugin-dialog`, `react-i18next`, `sonner`, and `localStorage`
- Store tests mock `@/lib/tauri-api` directly with `vi.mock()`
- Each store has a `resetStore()` helper that calls `useXxxStore.setState({...})` to reset between tests
- Test files co-located: `stores/connection.test.ts`, `stores/data.test.ts`, etc.
- Run: `pnpm test` (single run), `pnpm test:watch` (watch mode)

### Build & Dev
- **Dev server:** Vite on port 1420 (strict port), `envPrefix: ["VITE_", "TAURI_"]`
- **Path alias:** `@/*` → `./src/*` (configured in both `tsconfig.app.json` and `vite.config.ts`)
- **Biome** (not ESLint) for formatting/linting — but ESLint config also exists (legacy?)
  - Biome: 2-space indent, double quotes, semicolons always
  - Some rules disabled: `noArrayIndexKey`, `noNonNullAssertion`, `useButtonType`
- **TypeScript 6.0** with `verbatimModuleSyntax`, `erasableSyntaxOnly`
- **Build target:** `esnext`

## 3. Non-Obvious Rules & Gotchas

### DO NOT:
- **Don't add a router.** Navigation is store-driven. Adding react-router would conflict with the existing `PageName` pattern.
- **Don't call `invoke()` directly from components.** Always go through `tauriApi` in `lib/tauri-api.ts`.
- **Don't use shadcn CSS variables in components.** Use the custom semantic tokens (`bg-base`, `text-text`, `bg-surface`, `text-accent`, etc.). The shadcn variables (`--primary`, `--background`) are only used by shadcn/ui components internally.
- **Don't create a `stores/index.ts` barrel.** The project deliberately doesn't have one.
- **Don't use `tailwind.config.js`.** Tailwind 4 uses CSS-based config in `index.css` via `@theme`.
- **Don't add new root-level docs** (per AGENTS.md constraints).

### Gotchas:
- **`data-theme` attribute** controls theming, not a class. Light theme = `[data-theme="light"]`.
- **`applyStoredTheme()`** must run before React renders (called in `main.tsx`) to avoid flash.
- **Connection polling** is managed outside React (module-level `Map<string, timer>` in `connection.ts`), not in useEffect. This is intentional — it survives re-renders.
- **`useSerialScriptStore` and `useStandaloneScriptStore`** are both exported from the same file (`serialScript.ts`).
- **Command sequences** use `AbortController` for cancellation (module-level variable in `commands.ts`).
- **Ring buffer** in `useDataStore` — when `maxPackets` is hit, oldest packet is dropped (`slice(1)`).
- **`@tanstack/react-virtual`** is used in `RxViewer` for virtualized packet list rendering.
- **`react-resizable-panels`** (`Group`, `Panel`, `Separator`) used for the terminal split layout — note the API uses `Group` not `PanelGroup`.
- **Monaco editor** (`@monaco-editor/react`) used for Lua script editing in `EditorPage`.
- **`sonner`** for toast notifications — configured in `App.tsx` with theme-aware styling.

## 4. Project Structure (High-Level)

```
frontend/
  src/
    main.tsx              — Entry point, applies theme before render
    App.tsx               — Root component (ErrorBoundary + AppShell + Toaster + ShortcutsHelp)
    index.css             — Tailwind 4 config + Catppuccin theme variables
    components/
      layout/             — App shell (sidebar + page + statusbar)
      terminal/           — Main terminal workbench (17 files)
      virtual/            — Virtual port management
      editor/             — Monaco-based Lua script editor
      server/             — TCP/Unix socket server UI
      settings/           — App settings (single page, multi-tab)
      shared/             — Cross-cutting components (ErrorBoundary, CommandPalette)
      ui/                 — shadcn/ui primitives
    stores/               — 10 Zustand stores (no barrel export)
    hooks/                — useTauriCommand, useTauriEvent, useTheme, useKeyboardShortcuts
    lib/
      tauri-api.ts        — Centralized Tauri invoke wrapper (~70 commands)
      utils.ts            — cn(), formatBytes, hexToBytes, etc.
      highlight.ts        — Search highlight splitting for RxViewer
    types/
      index.ts            — All TypeScript interfaces/types (single file)
    i18n/
      index.ts            — i18next init
      locales/en.json     — English translations
      locales/zh.json     — Chinese translations
    test/
      setup.ts            — Vitest global mocks (Tauri, i18next, sonner, localStorage)
  components.json         — shadcn/ui config
  biome.json              — Biome linter/formatter config
  vite.config.ts          — Vite + Vitest + Tailwind + path alias
  tsconfig.app.json       — TypeScript config with @/* alias
```

## 5. Files Retrieved

1. `frontend/package.json` — dependencies, scripts
2. `frontend/vite.config.ts` — build config, test config, path alias
3. `frontend/biome.json` — linting/formatting rules
4. `frontend/components.json` — shadcn/ui config
5. `frontend/tsconfig.app.json` — TypeScript config
6. `frontend/eslint.config.js` — ESLint (appears secondary to Biome)
7. `frontend/src/main.tsx` — entry point
8. `frontend/src/App.tsx` — root component
9. `frontend/src/index.css` — theme system (Catppuccin, dual CSS vars)
10. `frontend/src/types/index.ts` — all shared types
11. `frontend/src/lib/tauri-api.ts` — Tauri invoke bridge (~70 commands)
12. `frontend/src/lib/utils.ts` — utility functions
13. `frontend/src/lib/highlight.ts` — search highlight logic
14. `frontend/src/hooks/useTauriCommand.ts` — invoke wrapper hook
15. `frontend/src/hooks/useTauriEvent.ts` — event listener hook
16. `frontend/src/hooks/useTheme.ts` — theme management
17. `frontend/src/hooks/useKeyboardShortcuts.ts` — global shortcuts
18. `frontend/src/stores/connection.ts` — connection store with polling
19. `frontend/src/stores/data.ts` — packet buffer store
20. `frontend/src/stores/ui.ts` — UI state store
21. `frontend/src/stores/server.ts` — server store with event listener
22. `frontend/src/stores/commands.ts` — quick commands + sequences
23. `frontend/src/stores/script.ts` — script management store
24. `frontend/src/stores/serialScript.ts` — per-port script store
25. `frontend/src/stores/presets.ts` — connection presets store
26. `frontend/src/stores/settings.ts` — settings store
27. `frontend/src/stores/virtualPort.ts` — virtual port store
28. `frontend/src/stores/log.ts` — log viewer store
29. `frontend/src/components/layout/AppShell.tsx` — main layout + event wiring
30. `frontend/src/components/layout/Sidebar.tsx` — navigation sidebar
31. `frontend/src/components/layout/StatusBar.tsx` — status bar
32. `frontend/src/components/terminal/TerminalWorkbench.tsx` — terminal page
33. `frontend/src/components/terminal/ConnectionBar.tsx` — connection controls
34. `frontend/src/components/terminal/RxViewer.tsx` — virtualized packet viewer
35. `frontend/src/components/ui/button.tsx` — shadcn button pattern
36. `frontend/src/components/shared/ErrorBoundary.tsx` — error boundary
37. `frontend/src/i18n/index.ts` — i18n setup
38. `frontend/src/test/setup.ts` — vitest mocks
39. `frontend/src/stores/connection.test.ts` — store test pattern
40. `frontend/src/stores/data.test.ts` — store test pattern
41. `frontend/TODO.md` — feature status tracking