# Backlog — Known Issues, Future Ideas, Discoveries

Updated: 2026-04-13

Items discovered but not actively being worked on. Ranges from real bugs to minor nitpicks to future plans.

---

## Known Bugs (no PR, no immediate plan)

### window.confirm() broken on WebView2/Windows

**Status:** Needs re-implementation for new architecture
**Original commit:** `5d98d2e12` on `windows-local-fixes` (archived)
**Problem:** WebView2 (Tauri/Windows) silently suppresses `window.alert()` and `window.confirm()`, making all confirmation dialogs non-functional.
**Original fix:** `ConfirmDialog` component in `packages/ui/` (now deleted upstream). Used `dialogManager` + `Dialog` existants, returned `Promise<boolean>`. All production `confirm()` → `confirmDialog()`, all `alert()` → `toast.error/success/info`. `platform.confirm()` (Tauri) updated to use `confirmDialog()`.
**Blocker:** Fix targeted `packages/ui/` which no longer exists. Must be re-implemented targeting `@spaceui/primitives` (spaceui repo at `E:\spaceui`).
**Note:** TagsGroup.tsx and useFileContextMenu.ts were excluded from original fix (depend on PR #3044).

### Inode `None` in persistent indexer

**File:** `persistent.rs:651-661`
**Impact:** File renames not detected on Windows. Windows file indices are unstable across reboots.
(Discovered 2026-03-28)

### Tests with hardcoded Unix paths

**Files:** `snapshot.rs:287`, `sidecar/path.rs:120,145`
**Impact:** Tests fail on Windows.
(Discovered 2026-03-28)

### Copy strategy `ends_with('/')`

**File:** `copy/strategy.rs:582`
**Impact:** Minor fallback issue.
(Discovered 2026-03-28)

### SdPath `C://` parsed as scheme

**File:** `addressing.rs:413`
**Impact:** Theoretical only (`C:\` not affected in practice).
(Discovered 2026-03-28)

---

## Minor UI/Code Quality (from CodeRabbit review, 2026-03-27)

| Item                                               | File                                                   | Notes                                        |
| -------------------------------------------------- | ------------------------------------------------------ | -------------------------------------------- |
| Filter by media type in context menu               | `useFileContextMenu.ts:355-388`                        | Backend ignores non-supported types silently |
| `type="button"` missing on Tag remove              | `Tag.tsx:42`                                           | No parent form in practice, cosmetic         |
| DB errors silent in MIME resolution                | `thumbnail/action.rs:127-149`                          | Errors swallowed during thumbnail generation |
| Import ordering                                    | `thumbnail/action.rs:3-15`                             | Style nitpick                                |
| Split tag queries into standard modules            | `ancestors.rs`, `children.rs`                          | Organizational improvement                   |
| Rustdoc missing                                    | `unapply/output.rs:7-10`                               | Documentation gap                            |
| `any` casts                                        | `TagSelector.tsx:44-50`, `useExplorerFiles.ts:146-150` | TypeScript type safety                       |
| Indexer rows by content_identity_uuid before merge | `files_by_tag.rs:321-342`                              | Performance/correctness improvement          |

---

## Performance

### Indexation slow on large volumes (#1, P4)

Discovery stays at 0% for a long time on 200k+ files. Uses `tokio::fs::read_dir` recursively. Possible approaches: MFT parsing, Windows Search API, parallel discovery.
(Discovered pre-2026-03-28)

---

## Architecture / Future Plans

### Device-level auth plan

Not implemented. Plan documented in `memory-bank/plans/device_auth.md`. Full security audit in `memory-bank/archive/security_audit_full.md`. Tracked as P1 in open-issues.md but implementation is not scheduled.

### useRefetchTagQueries workaround

Same pattern as `LocationInspector` (manual refetch). Backend emits correct events. Accepted as temporary project pattern.
(Noted during arch audit, 2026-03-25)

### Validation in `from_input()` vs `validate()` trait method

Works but doesn't support `RequiresConfirmation`. Acceptable for current tag operations.
(Noted during arch audit, 2026-03-25)

### DWM dark titlebar approach

Win32 direct `DwmSetWindowAttribute` — no Tauri wrapper available. Only option on Windows.
(Noted during arch audit, 2026-03-25)

---

## spacedrive-data Exploration Findings (2026-03-29)

### SpaceUI package dependency

Frontend on `spacedrive-data` requires three packages from the separate SpaceUI repo (`E:\spaceui`):

```
@spaceui/ai, @spaceui/primitives, @spaceui/tokens
```

These replaced the old `@sd/ui` workspace package. `packages/ui/` directory only contains a stale `tsbuildinfo` file.

**Resolution (2026-04-13):** Clone spaceui to `E:\spaceui`, `bun install && bun run build && bun link` in each package. Spacedrive's `vite.config.ts` has source aliases pointing to `../../../spaceui/packages/...`. Frontend now works on `spacedrive-data`.

**Note:** Package names were renamed between exploration dates. Original exploration (2026-03-29) references `@spaceui/*` but the actual packages may use `@spacedrive/*` naming — verify in `package.json`.

### Jamie's upstream work on spacedrive-data (fetched 2026-03-29)

- `9f0685abc` — Updated React types (19.1.10 → 19.2.14), removed `npm:types-react@rc` alias, Spacebot UI changes (ChatComposer, ConversationScreen, TasksRoute)
- `907ab4e26` — Added archive sources UI with full Rust integration: `core/src/data/manager.rs` (219 lines), `core/src/ops/sources/` CRUD, `core/src/ops/adapters/`, frontend source components, +2737/-218 lines across 43 files

### Media architecture (clarified 2026-03-24)

All media processors follow intentional 3-level design:

1. **Watcher** (automatic): file modified → reactive processor (e.g. `ThumbnailProcessor`)
2. **Single-file action** (UI): `media.thumbnail.regenerate` etc. → direct processor, immediate return
3. **Bulk job**: `ThumbnailJob` → batch with phases, persistence, progress

The context menu uses level (2). The broken `useJobDispatch` was calling a non-existent generic `jobs.dispatch` endpoint — it was the bug, not an incomplete design.

---

## Spacebot Windows Port (2026-04-11)

Server port complete (7 files modified, builds and runs on Windows). 3 desktop bugs found but not fixed:

- Sidecar scope issue
- Alt+Space shortcut conflict
- Invisible window

All desktop changes reverted to Jamie's originals. Full details: `memory-bank/spacebot-windows-port.md`
