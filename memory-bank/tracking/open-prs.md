# Open Pull Requests

Updated: 2026-04-13

All open PRs target `spacedrive-data`.

## PR #3044 — feat(tags): Tag system + media context menu fixes

- **Branch:** `tags-and-media-fixes`
- **Base:** `spacedrive-data`
- **Status:** OPEN, MERGEABLE
- **Commits:** 22
- **Description:** Complete tag system + explorer integration + media context menu fixes + FTS5 escaping + batch entry lookups + dead code removal

### Key commits

| Commit      | Description                                                                                                                                                                                                                 |
| ----------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `04a181535` | TOCTOU race fix (ON CONFLICT atomic upsert), get_direct_children (depth=1), delete cascade in transaction, include_children/min_confidence implemented, tags entry+content merge dedup, unapply notification via content_id |
| `8e4a7d6bd` | MIN(id) → MAX(id) migration (keep newest row)                                                                                                                                                                               |
| `5bf5584f8` | Skip row + warn if required fields missing                                                                                                                                                                                  |
| `3d078f410` | Version increment on upsert, skip unapply if 0 rows deleted                                                                                                                                                                 |
| `d4ae1cc78` | Remove broken optimistic update Inspector, remove alert() in TagsGroup                                                                                                                                                      |
| `5c4608654` | Emit "file" events before delete (fix stale tag badges), refetch files.by_id                                                                                                                                                |
| `99c9d272a` | Revert tagModeActive vs mode (independent concepts)                                                                                                                                                                         |
| `45c1141e1` | Remove redundant inline `use sea_orm::` imports                                                                                                                                                                             |
| `72ce78c0e` | FTS5 injection fix (tokens in double quotes), batch entry lookups (WHERE IN)                                                                                                                                                |
| `6880ccc23` | Delete dead `useJobDispatch.ts`                                                                                                                                                                                             |
| `48af496c0` | UNC strip conditioned on drive letter only (preserve Volume GUIDs)                                                                                                                                                          |
| `f9d254994` | Fix extension lost on root files, validate EntryUuid before tagging                                                                                                                                                         |
| `7bf06c579` | O(n²) tag merge → HashMap pre-indexation                                                                                                                                                                                    |

### Resolved CodeRabbit findings (2026-03-27)

All 22 findings from CodeRabbit review resolved. See `memory-bank/archive/open_issues_2026-03-29.md` for full table.

### Still-open findings from CodeRabbit

- **Sync propagation:** Neither `tags.delete` nor `tags.unapply` propagate `ChangeType::Delete`. Documented via `TODO(sync)`.
- **Tag filter not wired in UI:** `search.files` accepts `TagFilter` backend-side but no frontend component exposes it. For future advanced filters.
- **Tag filter + pagination FTS (#13):** `search/query.rs` applies `LIMIT/OFFSET` before `retain()` tag filtering. Pre-existing bug in search logic, not tag code.
- **Minor:** Filter by media type in context menu (`useFileContextMenu.ts:355-388`), `type="button"` missing on Tag remove (`Tag.tsx:42`)
- **Nitpicks:** DB errors silent in MIME resolution (`thumbnail/action.rs:127-149`), import ordering (`thumbnail/action.rs:3-15`), split tag queries into standard modules, rustdoc missing (`unapply/output.rs:7-10`), `any` casts (`TagSelector.tsx:44-50`, `useExplorerFiles.ts:146-150`), indexer rows by content_identity_uuid before merge (`files_by_tag.rs:321-342`)

---

## PR #3048 — fix(ts-client): Prevent Overview crash from single-resource cache seeding

- **Branch:** `fix/overview-crash-single-resource-seeding`
- **Base:** `spacedrive-data`
- **Status:** OPEN
- **Commits:** 1
- **Description:** `updateSingleResource` in `useNormalizedQuery.ts` returns `undefined` (TanStack no-op) when `oldData` is null, instead of seeding with a list-shaped object that breaks non-list queries.

### Root cause

Race condition: buffer replay delivers `ResourceChanged` for `library` (from `recalculate_statistics()`) before `queryFn` resolves. The fallback `oldData === null` created `{ files: [Library], total_count: 1, has_more: false }`, wrong shape for single-resource queries. Overview read `.statistics` on the list-shaped object and crashed with `TypeError: Cannot read properties of undefined (reading 'total_capacity')`.

### Impact

The old fallback `{ files: [...] }` was only correct for file listing queries. All other queries (Library, Device[], Location, tags) received incorrect shape. `updateBatchResources` keeps its seeding since batch events only concern file listings.

### Note

Bug introduced by commit `243ef8264` (James Pine). Not yet on upstream/main — will appear when spacedrive-data merges to main.

**File:** `packages/ts-client/src/hooks/useNormalizedQuery.ts:530-533`

---

## PR #3052 — fix(windows): icon.ico placeholder + SVG gradient decimal separators

- **Branch:** `fix/windows-ico-and-svg-gradients`
- **Base:** `spacedrive-data`
- **Status:** OPEN
- **Commits:** 1
- **Description:** Fixes broken icon.ico (103 bytes placeholder on upstream) and SVG gradient decimal separator issues (Obsidian etc.)

---

## Merged PRs (reference)

| PR    | Base | Description                                                                       |
| ----- | ---- | --------------------------------------------------------------------------------- |
| #3043 | main | Watcher registration + canonicalize + UNC helper + unwatch on removal (4 commits) |
| #3041 | main | Content path resolution, DEL keybinds, progress, useDeleteFiles hook (5 commits)  |
| #3040 | main | Dark titlebar DWM + sidebar hover (2 commits)                                     |
| #3037 | main | 29 commits, Windows fixes v2 (volumes, paths, ephemeral, trash, etc.)             |
| #2944 | main | Path traversal security fix                                                       |
