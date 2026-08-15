# Frontend Cleanup Inventory

**Research ticket**: #78 (Frontend: dead-code & unused-dependency inventory)
**Map**: #75 (Wayfinder map: purge redundant & legacy content)
**Blocks**: #81 (Frontend: execute cleanup per inventory)
**Date**: 2026-08-15
**Scope**: `frontend/` only (React 19 + Vite 8 + pnpm). Research only — no deletions performed.

---

## Method & Verification

Primary sources investigated: source tree (`frontend/src/`), `frontend/package.json`, `frontend/pnpm-lock.yaml`, `frontend/vite.config.ts`, `frontend/components.json`, `frontend/index.html`, `frontend/public/`, and `git log` (UI overhaul history).

Tooling run against a fresh `pnpm install --frozen-lockfile`:

| Tool | Command | Result |
|---|---|---|
| knip | `pnpm dlx knip --reporter compact` | 13 unused files, 13 unused deps, 1 unused devDep, 9 unused exports, 4 unused types |
| TypeScript | `pnpm type-check` (`tsc -b --noEmit`) | **passes** (exit 0) |
| Vite build | `pnpm build` (`vite build`) | **passes** (exit 0); warning: main chunk 672 kB / 200 kB gzip |
| vitest | `pnpm test` (`vitest run`, jsdom + setup file) | **passes** — 14 files, 156 tests |

Terminology follows map #75: **dead** = unreferenced/unreachable, verifiable by build + tooling; **stale** = still referenced but describes a superseded state; **legacy leftover** = a superseded replacement exists in-repo (e.g. ESLint→Biome).

Knip's unused-file list for `src/lib/mock/*` is a **false positive** — the mock layer is wired through Vite aliases, not imports (see §5).

---

## 1. Safe-delete: unused dependencies

All verified by two independent methods: (a) manual import map over `src/` + config files, (b) knip (`Unused dependencies` / `Unused devDependencies` report).

### 1.1 `@radix-ui/react-*` individual packages (12 packages) — duplicate of combined `radix-ui`

**Packages**: `@radix-ui/react-dialog`, `react-dropdown-menu`, `react-label`, `react-popover`, `react-scroll-area`, `react-select`, `react-separator`, `react-slot`, `react-switch`, `react-tabs`, `react-toggle`, `react-tooltip`.

**Evidence**: All Radix usage in `src/` imports the combined `radix-ui` package (10 files):
- `src/components/ui/badge.tsx:3`, `button.tsx:3`, `dialog.tsx:3`, `label.tsx:4`, `select.tsx:3`, `separator.tsx:2`, `switch.tsx:4`, `tabs.tsx:5`, `toggle.tsx:3`, `tooltip.tsx:2` — all `import { X } from "radix-ui"`.
- `grep -rn 'from "@radix-ui' src/` → **zero matches**.
- knip lists all 12 under `Unused dependencies`.
- `radix-ui@1.6.0` is a direct dep (`frontend/package.json`) and pulls the individuals in transitively (`frontend/pnpm-lock.yaml:2530`), so removing the direct entries changes nothing at build time.

**Severity**: safe-delete. **Disposition**: remove all 12 entries from `frontend/package.json`; re-run `pnpm type-check` + `pnpm build` to confirm (both pass with them removed by construction — no import sites).

### 1.2 Form stack: `react-hook-form`, `@hookform/resolvers`, `zod`

**Evidence**: zero import references in `src/` or any config file (manual map); knip lists all three under `Unused dependencies`. The frontend uses Zustand stores + controlled inputs, not react-hook-form.

**Severity**: safe-delete. **Disposition**: remove all three from `frontend/package.json`.

### 1.3 `cmdk`

**Evidence**: sole import is `src/components/shared/CommandPalette.tsx:2` (`import { Command } from "cmdk"`). CommandPalette is itself dead (§2.1). knip lists `cmdk` under `Unused dependencies`.

**Severity**: safe-delete **together with** CommandPalette (§2.1).

### 1.4 `date-fns`

**Evidence**: zero import references in `src/` or configs; knip lists it under `Unused dependencies`. (Timestamp/byte formatting is hand-rolled; no date library is imported.)

**Severity**: safe-delete.

### 1.5 `autoprefixer` (devDependency)

**Evidence**: only occurrence in the repo is `frontend/package.json:63`; no import, no PostCSS config file exists. Tailwind CSS 4 (`@tailwindcss/vite` in `vite.config.ts`) handles prefixing via lightningcss. knip lists it under `Unused devDependencies`.

**Severity**: safe-delete.

---

## 2. Safe-delete: dead components / hooks / assets

### 2.1 `src/components/shared/CommandPalette.tsx`

**Evidence**: zero importers in `src/` (reverse import graph; grep for `CommandPalette` outside its own file returns only its own declarations). App.tsx renders `ShortcutsHelp`, not the palette. knip lists it under `Unused files`. Its `commandPalette.*` i18n keys are already absent from both locale files (`src/i18n/locales/en.json` / `zh.json`) — the component renders raw key strings (e.g. `CommandPalette.tsx:92` `t("commandPalette.navigation")`), i.e. it was orphaned when the UI overhaul removed its invocation and keys.

**Severity**: safe-delete. **Disposition**: delete file; delete `cmdk` dep (§1.3). The `commandPalette.*` keys are already absent from both locale files, so no locale cleanup is needed.

### 2.2 `src/hooks/useTauriCommand.ts`

**Evidence**: zero importers (reverse import graph; `grep -rn useTauriCommand src/` matches only the file itself). Exported `useTauriCommand<TResult>` never called. knip lists under `Unused files`.

**Severity**: safe-delete.

### 2.3 `src/components/ui/tooltip.tsx` / `switch.tsx` / `separator.tsx`

**Evidence**: zero importers (reverse import graph; no references in `src/index.css` or anywhere else). knip lists all three under `Unused files`. These are shadcn/ui scaffolds — `components.json` (shadcn config) is intact, so each can be regenerated in seconds with `npx shadcn add tooltip|switch|separator`.

**Severity**: safe-delete (regenerable via shadcn CLI). **Disposition**: delete; if a future feature needs them, re-add via `npx shadcn@latest add`.

### 2.4 `public/icons.svg` — orphaned asset

**Evidence**: `frontend/public/` contains `favicon.svg` + `icons.svg`. `index.html:5` references `/favicon.svg` only. grep for `icons.svg` across `src/`, `index.html`, `index.css` → zero matches. (Vite copies `public/` verbatim into `dist/`; the built output would contain an unreferenced file.)

**Severity**: safe-delete. `favicon.svg` stays (referenced at `index.html:5`).

### 2.5 Dead `tauriApi` methods: `checkPortHealth`, `getScriptInfo`, `validateScript`

**Evidence**: `src/lib/tauri-api.ts` defines 69 methods; a call-graph of `tauriApi.<method>(` across non-mock, non-test `src/` shows 66 called. Never called:
- `checkPortHealth` — `tauri-api.ts:36` (+ mock twin `src/lib/mock/index.ts:78`)
- `getScriptInfo` — `tauri-api.ts:58` (`invoke<Script>("get_script_info", ...)`)
- `validateScript` — `tauri-api.ts:61`; EditorPage only calls `validateScriptFile` (`EditorPage.tsx:218,256`) and `validateScriptDetailed` (`EditorPage.tsx:242`)

Note: `get_script_info` was exposed to the GUI in issue #28 but the frontend never consumed it; the same commands stay live on the Rust side (`src-tauri/`), so this is frontend-only dead surface.

**Severity**: needs-decision (see §4) — removal is trivially safe for the frontend, but confirm with the Rust inventory (#79) that the backend commands are likewise unused before pruning there.

---

## 3. Stale content (needs-decision)

### 3.1 i18n locale keys — ~59 unreferenced keys per locale

**Evidence**: static scan of every string literal in `src/` (all `t("...")`, `labelKey: "..."`, and dotted-key literals) against flattened `src/i18n/locales/en.json` (366 keys) and `zh.json` (364 keys). 61 keys have no matching literal; removing 2 plural-form false positives (`server.activeConnections_one/_other` — resolved via i18next plurals when the base `server.activeConnections` is used with `{count}`) leaves **59 stale candidates in each locale**, e.g.:
- `terminal.*` (20): `rxViewer`, `txSender`, `stats`, `quickCommands`, `addQuickCommand`, `editQuickCommand`, `deleteQuickCommand`, `commandLabel`, `commandData`, `commandFormat`, `duration`, `exportData`, `exportTxt`, `exportCsv`, `exportJson`, `bytesSent`, `bytesReceived`, `loopInterval`, `historyUp`, `historyDown`
- `protocols.*` (9): `builtIn`, `custom`, `loadProtocol`, `protocolEditor`, `unload`, `newProtocol`, `noProtocols`, `validationFail`, `validationErrors`
- `common.*` (9): `connecting`, `connected`, `disconnected`, `error`, `confirm`, `create`, `back`, `flowControl`, `protocol`
- `scripts.*` (6): `newScript`, `output`, `validateFail`, `emptyState`, `templateTip`, `validateScriptInfo`
- `virtual.*` (5): `capture`, `noPorts`, `startCapture`, `stopCapture`, `maxPackets`
- `settings.*` (4): `title`, `serial`, `customDir`, `defaultFormat`
- plus `shortcuts.navServer`, `server.portOccupied`, `history.export`, `commands.newCommand`, `protocolTester.selectProtocol`, `presets.manageTitle`

**Caveat**: the scan cannot see fully dynamic keys (`t(\`settings.${tab}\`)` in `SettingsPage.tsx`). Spot-checks of the flagged keys (`common.error`, `terminal.stats`, `settings.title`, `presets.manageTitle`, `virtual.capture`, `scripts.emptyState`, `protocols.builtIn`, `common.flowControl`) confirm zero static references. These describe UI that the overhaul replaced (old terminal panels, the old Protocols page, quick-command CRUD).

**Severity**: needs-decision (low risk). **Disposition**: run one dynamic-key verification pass, then delete the confirmed-dead keys from **both** `en.json` and `zh.json`. If key-count parity matters to the cleanup, keep both files in lockstep.

### 3.2 `frontend/README.md` — boilerplate template README

**Evidence**: the file is the unmodified Vite React-TS template README — it documents "two official plugins", `@vitejs/plugin-react-swc`, React Compiler, and "Expanding the ESLint configuration" (`frontend/README.md`). None of that matches the actual frontend (Tailwind 4, Biome, Zustand, Tauri); accurate guidance lives in `frontend/AGENTS.md`.

**Severity**: needs-decision. **Disposition**: replace with a real frontend README (dev commands, mock mode, structure) or delete — coordinate with the docs scope (#80).

### 3.3 `frontend/TODO.md` — legacy scratch doc

**Evidence**: header says "v0.9.0, updated 2026-05-24". References nonexistent files (`tailwind.config.ts`, a `components/settings/LogViewer` component) and duplicated trackers that per AGENTS.md now live in GitHub Issues. It is a legacy leftover of the pre-issue-tracker workflow.

**Severity**: needs-decision. **Disposition**: delete or migrate any still-live items to GitHub Issues; coordinate with repo-hygiene (#76) / docs (#80).

---

## 4. Needs-decision (keep or trim with care)

### 4.1 Unused shadcn exports (low churn value)

**Evidence** (knip `Unused exports`): `badgeVariants` (`badge.tsx`), `buttonVariants` (`button.tsx`), `tabsListVariants` (`tabs.tsx`), `toggleVariants` (`toggle.tsx`), `CardFooter/CardAction/CardDescription` (`card.tsx`), `DialogClose/DialogDescription/DialogOverlay/DialogPortal/DialogTrigger` (`dialog.tsx`), `SelectGroup/SelectLabel/SelectScrollDownButton/SelectScrollUpButton/SelectSeparator` (`select.tsx`).

**Disposition**: these are the standard shadcn component API surface (variants + composable parts). Trimming them saves nothing meaningful and makes future `shadcn add`/updates noisier. **Recommendation: leave as-is** unless the cleanup ticket wants a strict no-dead-export pass.

### 4.2 Unused exported types

**Evidence** (knip `Unused exported types`): `ShortcutDef` (`src/hooks/useKeyboardShortcuts.ts:6`), `ConnectionEntry` (`src/stores/connection.ts:19`), `ExportOptions` (`src/stores/data.ts:23` — used internally, export unneeded), and in `src/types/index.ts`: `ScriptMeta`, `SerialConfigData`, `LoggingConfigData`, `LuaConfigData`, `OutputConfigData`, `ProtocolsConfigData`, `VirtualPortsConfigData`, `DisplayConfigData`, `OutputLine`.

**Disposition**: the `*ConfigData` slice types are leftovers of an older config shape (current state uses `ConfigData` from `stores/settings.ts`); deleting them is safe and reduces the shared types surface. `ShortcutDef`/`ConnectionEntry`/`ExportOptions` are used within their own files — either keep the exports or drop the `export` keyword. Low risk; bundle into the cleanup ticket as a single type-trim commit.

### 4.3 ESLint dev-stack — legacy leftover (coordinated with #77)

**Evidence**: `frontend/package.json` has **no `eslint` script** — `check`/`lint`/`fmt` all run Biome (`frontend/package.json` scripts; `frontend/biome.json`). `eslint.config.js` exists and imports `eslint`, `@eslint/js`, `eslint-plugin-react-hooks`, `eslint-plugin-react-refresh`, `globals`, `typescript-eslint`, but nothing in the build/test/CI cycle invokes it (knip does not flag these because it counts `eslint.config.js` as usage — manual verification required). This is the canonical "legacy leftover" from map #75 (ESLint→Biome) and is already ticket #77 ("Frontend: remove legacy ESLint config").

**Disposition**: remove `eslint.config.js` **and** the 6 ESLint devDependencies in the same commit as #77. Do not delete the deps before the config, or the config file breaks.

### 4.4 Mock layer — NOT dead (documented false positive)

**Evidence**: knip reports `src/lib/mock/*` (index.ts, dialog.ts, handlers/*) as unused files, but they are deliberately wired through Vite aliases in `vite.config.ts:10-15` (mock mode for `pnpm dev` without Rust; `TAURI_PLATFORM` switch), documented in `frontend/AGENTS.md` ("Mock Layer" section). `src/lib/mock/index.ts` is the alias target for `@/lib/tauri-api`; `mock/dialog.ts` for `@tauri-apps/plugin-dialog`; `mock/events.ts` for `@tauri-apps/api/event`. The mock mirrors all 69 `tauriApi` methods (72 entries incl. 3 mock-only helpers) — it must stay in sync with `tauri-api.ts`.

**Disposition**: **do not delete**. If knip is adopted permanently, add the alias targets to its `entry` config so the false positive disappears.

---

## 5. Adjacent observations (out of scope for #78, note for other tickets)

- **Bundle size**: `pnpm build` warns main chunk is 672 kB (200 kB gzip) — Monaco editor (`@monaco-editor/react` in `EditorPage.tsx:1`, on the `editor` page only) is the dominant weight. Code-splitting the editor page would help, but that is an optimization, not dead-code removal (out of map #75 scope).
- **Root `package.json`** (repo root) declares `@testing-library/user-event` as a devDependency with no consumer at root; the frontend already has its own copy (`frontend/package.json`). Repo-hygiene item for #76.
- **`src/i18n/index.ts` default export** is unused (knip) — the module is consumed as a side effect (`src/main.tsx:3` `import "@/i18n"`); `export default i18n` can drop the `default`.
- **`src/lib/mock/state.ts` `defaultSignals`** is flagged by knip but used internally (`state.ts:363,370`); drop the `export` keyword only if desired.

---

## Summary table

| Item | Category | Severity | Disposition |
|---|---|---|---|
| 12× `@radix-ui/react-*` | dead dep (duplicate of `radix-ui`) | safe-delete | remove from package.json |
| `react-hook-form` + `@hookform/resolvers` + `zod` | dead dep (form stack) | safe-delete | remove from package.json |
| `cmdk` | dead dep (only used by dead component) | safe-delete | remove together with CommandPalette |
| `date-fns` | dead dep | safe-delete | remove from package.json |
| `autoprefixer` | dead devDep (Tailwind 4) | safe-delete | remove from package.json |
| `CommandPalette.tsx` | dead component | safe-delete | delete file |
| `useTauriCommand.ts` | dead hook | safe-delete | delete file |
| `ui/tooltip.tsx`, `ui/switch.tsx`, `ui/separator.tsx` | dead shadcn scaffolds (regenerable) | safe-delete | delete; re-add via `shadcn add` if needed |
| `public/icons.svg` | orphaned asset | safe-delete | delete file |
| `tauriApi.checkPortHealth/getScriptInfo/validateScript` | dead API surface | needs-decision | confirm with Rust inventory (#79), then trim from `tauri-api.ts` + mock |
| ~59 i18n keys ×2 locales | stale content | needs-decision | dynamic-key pass, then delete from both locales |
| `frontend/README.md` | stale boilerplate | needs-decision | rewrite or delete (docs scope #80) |
| `frontend/TODO.md` | legacy leftover | needs-decision | delete/migrate (#76/#80) |
| ESLint stack (config + 6 devDeps) | legacy leftover | needs-decision | remove with #77 |
| shadcn variant/part exports | dead exports, low value | needs-decision | leave (recommended) |
| unused types (`*ConfigData` etc.) | dead exports | needs-decision | trim in one type-cleanup commit |
| Mock layer (`src/lib/mock/*`) | NOT dead (vite-alias false positive) | keep | add knip entry config if adopting knip |

**Net effect of safe-deletes**: 17 unused dependency entries, 6 dead files, 1 orphaned asset — all verifiable by `pnpm type-check` + `pnpm build` after removal.
