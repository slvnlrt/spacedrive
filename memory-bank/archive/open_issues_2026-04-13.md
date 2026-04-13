# Open Issues Tracker

## Active Issues

### 1. Overview Crash - single-resource cache seeding

**Status:** Fixed, PR #3048 open (rebased on spacedrive-data)
**Branch:** `fix/overview-crash-single-resource-seeding`

**Symptom:** `TypeError: Cannot read properties of undefined (reading 'total_capacity')` at Overview page.

**Root cause:** `updateSingleResource` in `useNormalizedQuery.ts` returns `{ files: [resource], total_count, has_more }` when `oldData` is null. This list-shaped object breaks non-list queries (like `libraries.info`) which expect the resource directly.

**Fix:** Return `undefined` (TanStack no-op) when `oldData` is null. 3 lines.

**Note:** Bug is NOT on upstream/main yet. It will appear when `spacedrive-data` merges to main.

### 2. Tag View "Tag Not Found"

**Status:** Expected on branches without tags PR
**Cause:** Tag view queries (`tags.by_id`, `tags.ancestors`, etc.) only exist in PR #3044.

### 3. spacedrive-data Frontend Blocked

**Status:** RESOLVED
**Cause:** `@spacedrive/ai`, `@spacedrive/primitives`, `@spacedrive/tokens` referenced as `link:` dependencies. These come from the separate SpaceUI repo (`https://github.com/spacedriveapp/spaceui`).
**Resolution:** Clone spaceui to `E:\spaceui`, `bun install && bun run build && bun link` in each package. Spacedrive's `vite.config.ts` has source aliases pointing to `../../../spaceui/packages/...`. Frontend now works on `spacedrive-data`.

### 4. Spacebot Windows Port (2026-04-11)

**Status:** Server port complete, desktop bugs found but not fixed
**Details:** See `memory-bank/spacebot-windows-port.md`
**Summary:** 7 server files modified (builds and runs on Windows). 3 desktop bugs found (sidecar scope, Alt+Space conflict, invisible window) but all desktop changes reverted to Jamie's originals.

## Repo State Summary (2026-04-13)

### Upstream context

Jamie merged `main` into `upstream/spacedrive-data` on 2026-04-12. `spacedrive-data` = `main` + 37 exclusive commits (sources UI, sync jobs, server work, CI fixes). All PRs now target `spacedrive-data`.

### Local branches

| Branch                                       | Ahead of upstream     | Covered by PR? | Notes                                                                          |
| -------------------------------------------- | --------------------- | -------------- | ------------------------------------------------------------------------------ |
| `tags-and-media-fixes`                       | 22 commits            | PR #3044       | Rebased on spacedrive-data                                                     |
| `fix/overview-crash-single-resource-seeding` | 1 commit              | PR #3048       | Rebased on spacedrive-data                                                     |
| `fix/windows-ico-and-svg-gradients`          | 1 commit              | PR #3052       | Cherry-picked from spacedrive-data local                                       |
| `spacedrive-data`                            | 4 commits (21 behind) | 3/4 covered    | Only icon.ico commit was orphan, now PR #3052. Branch needs reset to upstream. |
| `windows-local-fixes`                        | 2 commits             | No             | ARCHIVED — both commits obsolete (see below)                                   |
| `security-fix`                               | 8 commits             | No             | NET-01 vuln fix + security audit docs + memory-bank docs                       |
| `main`                                       | 0                     | N/A            | In sync with upstream                                                          |

### Other repos

| Repo          | Branch    | State                                                                               |
| ------------- | --------- | ----------------------------------------------------------------------------------- |
| `E:\spacebot` | `spaceui` | 7 modified + 1 new (server Windows port only). Desktop reverted. Not committed yet. |
| `E:\spaceui`  | `main`    | Clean. Cloned, built, linked. No local changes.                                     |

### Untracked

- `memory-bank/` — local docs (not committed)
- `nul` — Windows artifact, can be deleted

### Stashes

6 stashes with content (rtk `--stat` misreported them as empty due to PowerShell escaping).

- @{0}: `main.rs` Alt+Space shortcut graceful fail (spacebot-related)
- @{1}-@{3}: migration mod.rs + overview formatting + memory-bank README (duplicates, content in PR #3044)
- @{4}-@{5}: old memory-bank/activeContext.md updates

## PRs

| PR    | Base            | Status          | Description                                         |
| ----- | --------------- | --------------- | --------------------------------------------------- |
| #3052 | spacedrive-data | OPEN            | icon.ico placeholder fix + Obsidian SVG gradients   |
| #3048 | spacedrive-data | OPEN            | Overview crash fix                                  |
| #3044 | spacedrive-data | OPEN, MERGEABLE | Tags system + media context menu fixes (22 commits) |
| #3043 | main            | MERGED          | —                                                   |
| #3041 | main            | MERGED          | —                                                   |
| #3040 | main            | MERGED          | —                                                   |
| #3037 | main            | MERGED          | —                                                   |
| #2944 | main            | MERGED          | —                                                   |

## Not yet in any PR

1. **`security-fix`** — 8 commits:
   - `b7f75ba91` fix(security): NET-01 - Prevent arbitrary file write in P2P transfers
   - Security audit reports and docs
   - Memory bank documentation
   - Delete .gitmodules

## Known issues (no PR yet)

### window.confirm() broken on WebView2/Windows

**Status:** Needs re-implementation for new architecture
**Original commit:** `5d98d2e12` on `windows-local-fixes` (archived)
**Problem:** WebView2 (Tauri/Windows) silently suppresses `window.alert()` and `window.confirm()`, making all confirmation dialogs non-functional.
**Original fix:** `ConfirmDialog` component in `packages/ui/` (now deleted upstream). The fix needs to be re-done targeting `@spacedrive/primitives` (spaceui repo) instead.

## Archived branches

### `windows-local-fixes` — archived 2026-04-13

Both exclusive commits are obsolete:

- `243ef8264` fix(ephemeral) — Jamie integrated the same fixes upstream
- `5d98d2e12` fix(ui) dialog system — targets `packages/ui/` which no longer exists (replaced by `@spacedrive/primitives`). Problem still valid, needs re-implementation. See "Known issues" above.
