# PR #3044 — feat(tags): tag system with explorer integration, and media context menu fixes

**Status:** OPEN, MERGEABLE
**Target branch:** spacedrive-data
**Branch:** `tags-and-media-fixes`
**Author:** slvnlrt
**URL:** https://github.com/spacedriveapp/spacedrive/pull/3044
**Commits:** 22
**Rebased:** onto `upstream/spacedrive-data` on 2026-04-13 (3 conflicts resolved in `@sd/ui` -> `@spacedrive/primitives` renames)

---

## Commits (22)

From `git log --oneline tags-and-media-fixes ^upstream/spacedrive-data` (2026-04-13):

```
dea06b921 fix(tags): validate entry UUIDs in create action before applying
9295f85ae fix(tags): remove redundant inline sea_orm imports
637a941e1 chore: remove dead useJobDispatch hook
4ad8df660 fix(tags): secure FTS5 escaping, batch entry lookups for performance
ed9890aed fix(tags): pre-index content rows to avoid O(n²) tag merge, require...
0b75b8272 fix(tags): add extension to root-level file paths, validate entry U...
c0efb330f fix(tags): emit file events on tag delete, refetch files.by_id for ...
93ccb62ff fix(tags): remove broken optimistic update and alert() dialog
4022524c1 fix(tags): skip rows with undecodable required fields instead of fa...
a3edd3cac fix(tags): address second round of CodeRabbit review
16daa27e7 revert(tags): restore independent tagModeActive state
c02bfa702 fix(migration): keep newest row (MAX id) when deduplicating tag app...
d59ef2a70 fix(tags): address CodeRabbit review findings on tag system
ff449d0bd fix(media): replace broken useJobDispatch with direct mutations
c61d03613 fix(core): use current device slug instead of "unknown-device" fa...
6529043b8 refactor: extract shared useRefetchTagQueries hook
b414c2025 feat(tags): add unapply/delete actions, fix tag sync and Inspector UX
a0969656f feat(tags): render tag view using standard explorer with full File ...
b4415cd7f fix(tags,ui): make tag view files navigable and wire Overview searc...
1bc19a31e fix(tags): prevent duplicate tag applications on the same file
a9dde16fb feat(tags): implement tags.by_id, tags.ancestors, tags.children, fi...
873b89d00 fix(search): apply TagFilter in search.files query
```

---

## Summary

Full tag CRUD system with explorer integration, FTS5 search, unique constraint migration, and media context menu fixes. Resolves multiple issues discovered in PR #3037 (tags filter returning no results, search bar broken in Overview, navigation from tag view).

### Features

- `tags.by_id`, `tags.ancestors`, `tags.children`, `files.by_tag` queries
- Tag view rendered using standard explorer with full `File` objects (same format as `directory_listing`)
- `find_entry_ids_for_tag()` + `resolve_tag_filter()` in `query.rs` for `search.files`
- Tag apply/unapply/delete actions with sync support
- Duplicate tag application prevention (unique constraint migration)
- FTS5 search escaping (tokens wrapped in quotes to block operator injection)
- Overview search bar wired (button navigates to `/explorer`)
- DEL key accessible from tag view context
- `SdPath::Physical` fixed to use `get_current_device_slug()` instead of `"unknown-device"` fallback

### Rebase context (2026-04-13)

Rebased onto `upstream/spacedrive-data` after Jamie merged `main` into `spacedrive-data` on 2026-04-12. Three conflicts resolved, all in `@sd/ui` -> `@spacedrive/primitives` package renames (import paths updated to match new spaceui architecture).

---

## CodeRabbit Review — state at 2026-03-27

PR #3044 rebased on upstream/main after merge of #3041 (conflict in useFileContextMenu.ts resolved). Status: CLEAN/MERGEABLE.

### Findings resolved

| Finding                                    | Commit      | Fix                                        |
| ------------------------------------------ | ----------- | ------------------------------------------ |
| TOCTOU race metadata/manager.rs            | `04a181535` | ON CONFLICT atomic upsert                  |
| MIN(id) → MAX(id) migration                | `8e4a7d6bd` | Keeps the most recent row                  |
| children vs descendants                    | `04a181535` | `get_direct_children()` (depth=1)          |
| Delete cascade non atomique                | `04a181535` | Transaction                                |
| include_children/min_confidence ignored    | `04a181535` | Implemented                                |
| Champs File fabricated                     | `5bf5584f8` | Skip row + warn if required fields missing |
| Tags entry+content non merged              | `04a181535` | extend + dedup                             |
| Unapply notification content-scoped        | `04a181535` | Collection via content_id                  |
| Version not incremented on upsert          | `3d078f410` | `version + 1` in ON CONFLICT               |
| Unapply over-reports entries_affected      | `3d078f410` | Skip if 0 rows deleted                     |
| Optimistic update Inspector broken         | `d4ae1cc78` | Removed, refetch suffices                  |
| alert() in TagsGroup                       | `d4ae1cc78` | Removed                                    |
| Tag badges remain after delete             | `5c4608654` | Emit "file" events before delete           |
| Inspector stale after tag mutation         | `5c4608654` | Refetch files.by_id                        |
| tagModeActive vs mode (false positive)     | `99c9d272a` | Revert — independent concepts              |
| Inline imports redundant (apply/action.rs) | `45c1141e1` | Removed sea_orm inline, top-level suffices |
| FTS5 injection risk (manager.rs)           | `72ce78c0e` | Tokens wrapped in double quotes            |
| Per-entry UUID lookups (apply/action.rs)   | `72ce78c0e` | Batch query via WHERE IN                   |
| Dead useJobDispatch hook                   | `6880ccc23` | File deleted                               |
| UNC strip too aggressive (volume GUIDs)    | `48af496c0` | Conditioned on drive letter                |

### Findings open — sync (separate PR)

The sync insert works, but neither `tags.delete` nor `tags.unapply` propagate `ChangeType::Delete`. Documented via `TODO(sync)` in `delete/action.rs` and `unapply/action.rs`.

Related: expose insert vs update in the return of `apply_semantic_tags()` so callers emit the correct ChangeType.

### Findings open — real bugs not fixed

**Extension lost on root-level files (#16)** — RESOLVED: commit `f9d254994`

**Validate EntryUuid before tagging (#4)** — RESOLVED: commit `f9d254994`

**Performance O(n^2) tag merge (#NEW)** — RESOLVED: commit `7bf06c579`, pre-indexation HashMap

**Tag filter backend not wired in UI**: `search.files` accepts a `TagFilter` on the backend (`search/query.rs`) but no frontend component uses it. The tag filter in the search bar is not accessible to the user. To be wired in the UI search when advanced filters are implemented.

**Tag filter + pagination FTS (#13)**: `search/query.rs` applies `LIMIT/OFFSET` before tag filtering via `retain()`. Pre-existing bug in search logic, not in the tags code.

**`window.confirm()` suppressed by WebView2** — RESOLVED: branch `windows-local-fixes` (commit `5d98d2e12`)

- WebView2 silently suppresses `alert()`/`confirm()`/`prompt()` under Tauri/Windows
- Added `ConfirmDialog` in `@sd/ui`: uses `dialogManager` + `Dialog` existing, returns `Promise<boolean>`
- All production `confirm()` -> `confirmDialog()`, all `alert()` -> `toast.error/success/info`
- `platform.confirm()` (Tauri) updated to use `confirmDialog()`
- Note: standalone local commit, not yet in a PR. TagsGroup.tsx and useFileContextMenu.ts excluded (depend on PR #3044).
- Note (2026-04-13): This fix targets `packages/ui/` which no longer exists (replaced by `@spacedrive/primitives`). Problem still valid, needs re-implementation.

### Findings open — minor

| Finding                               | File                            | Impact                            |
| ------------------------------------- | ------------------------------- | --------------------------------- |
| Filter by media type in context menu  | `useFileContextMenu.ts:355-388` | Backend ignores unsupported types |
| `type="button"` missing on Tag remove | `Tag.tsx:42`                    | No parent form in practice        |

### Nitpicks (style/doc)

| Finding                                            | File                                                   |
| -------------------------------------------------- | ------------------------------------------------------ |
| DB errors silent in MIME resolution                | `thumbnail/action.rs:127-149`                          |
| Import ordering                                    | `thumbnail/action.rs:3-15`                             |
| Split tag queries into standard modules            | `ancestors.rs`, `children.rs`                          |
| Rustdoc missing                                    | `unapply/output.rs:7-10`                               |
| Cast `any`                                         | `TagSelector.tsx:44-50`, `useExplorerFiles.ts:146-150` |
| Indexer rows by content_identity_uuid before merge | `files_by_tag.rs:321-342`                              |

---

## Architectural Audit (2026-03-25)

Full audit of all commits post-PR #3037, verifying conformance with the project's architectural patterns.

### Result: no "ephemeral" workarounds

Our only confirmed workaround (wait_for_indexing synchronous in directory_listing.rs, PR #3037) was reverted by the maintainer (commit `b7bbf29db` on `spacedrive-data`). Cherry-picked onto `windows-local-fixes`. The streaming design is restored with 3 race conditions fixed at the source.

### Corrections following the audit

| Fix                    | Commit      | PR    | Description                                                       |
| ---------------------- | ----------- | ----- | ----------------------------------------------------------------- |
| UNC helper DRY         | `dbbe52bb4` | #3043 | 3 duplications → `common::utils::strip_windows_extended_prefix()` |
| Volume GUID protection | `48af496c0` | #3043 | Only strips `\\?\` on drive letters                               |
| Unwatch on removal     | `dbbe52bb4` | #3043 | `unwatch_location()` symmetric to `watch_location()`              |
| FTS5 escaping          | `72ce78c0e` | #3044 | Tokens wrapped in quotes, blocks operator injection               |
| Batch entry lookups    | `72ce78c0e` | #3044 | 2 branches of apply/action.rs: N queries → 1 query                |
| Dead code removal      | `6880ccc23` | #3044 | `useJobDispatch.ts` deleted (0 imports)                           |
| Redundant imports      | `45c1141e1` | #3044 | Inline `use sea_orm::` removed (already at top-level)             |

### Points identified but not corrected (acceptable)

- **useRefetchTagQueries**: same workaround pattern as `LocationInspector` (manual refetch). The backend already emits the correct events. Accepted as a temporary project pattern.
- **Validation in `from_input()` vs `validate()` trait method**: works but doesn't support `RequiresConfirmation`. Acceptable for current tag operations.
- **DWM dark titlebar**: direct Win32 approach, no Tauri wrapper. This is the only option on Windows.

### Upstream analysis: commit `b7bbf29db` (James Pine)

The maintainer reverted our synchronous approach for ephemeral and fixed the real bugs:

1. `subscriptionManager.ts`: listener pre-registered before `transport.subscribe()` (buffer replay events lost)
2. `useNormalizedQuery.ts`: TanStack cache seeded instead of dropping events if `oldData` undefined
3. `directory_listing.rs`: subdirectories without indexed children fall through to a new job

Our workaround (polling 25ms, timeout 10s) masked these 3 races. The maintainer is building Spacebot (AI assistant with EventSource streaming) and a column view — he needs the reliable event pipeline.

---

## Design Decisions

### Tag view architecture

Tags are rendered using the standard explorer with full `File` objects (same format as `directory_listing`). This means tag views get sorting, filtering, context menus, and all explorer features for free. The alternative (custom tag-only view) was rejected because it would require duplicating explorer functionality.

### FTS5 search escaping

User input tokens are wrapped in double quotes before passing to FTS5. This prevents FTS5 operator injection (e.g., `NOT`, `OR`, `NEAR`) while still allowing natural search terms. The alternative (stripping special characters) would break legitimate queries containing quotes.

### Duplicate prevention

A unique constraint migration deduplicates existing tag applications (keeps MAX id, i.e., the newest row) and adds a UNIQUE index to prevent future duplicates. ON CONFLICT atomic upsert handles the race condition.

### Device slug fallback

`SdPath::Physical` previously used `"unknown-device"` as a fallback device slug, which broke navigation from tag view to file location. Fixed to use `get_current_device_slug()` which returns the actual device identifier.
