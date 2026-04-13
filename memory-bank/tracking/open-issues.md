# Open Issues — Active / Needs Fixing

Updated: 2026-04-13

Issues that are STILL OPEN and need action. Resolved items are in archived docs.

---

## P1 — Critical

### Device-level auth missing (arch #24)

PR #2944 fixed path validation (path traversal). The **identity** validation is still missing:

- `file_delete.rs:54` : `_from_device` ignored
- `file_transfer.rs` : fallback `Uuid::new_v4()` (fail-open)
- `sync/handler.rs` : state changes without path validation
- EphemeralShare : auto-accept

**Plan:** `memory-bank/plans/device_auth.md`
**Full audit:** `memory-bank/archive/security_audit_full.md`
**Related:** `RemoteDeleteStrategy` uses deprecated `device_id()` — needs device auth PR to fix properly.

### security-fix branch — NET-01 not upstreamed

8 commits on `security-fix` branch (including `b7f75ba91` fix for arbitrary file write in P2P transfers). Not in any PR. Needs to be submitted upstream.

---

## P2 — Important

### `libraries.list` handler missing (#15)

`validate_and_reset_library_if_needed()` → `{"Error":{"OperationFailed":"Unknown method: query:libraries.list"}}`. Handler absent on daemon side. Non-blocking (app works), but deleted libraries are not cleaned up at restart.

### Sync propagation for tag delete/unapply

Neither `tags.delete` nor `tags.unapply` propagate `ChangeType::Delete`. Documented via `TODO(sync)` in `delete/action.rs` and `unapply/action.rs`. Related: expose insert vs update return from `apply_semantic_tags()` for correct ChangeType emission.

(From CodeRabbit review of PR #3044, 2026-03-27)

### Tag filter + pagination FTS bug (#13)

`search/query.rs` applies `LIMIT/OFFSET` before tag filtering via `retain()`. Pre-existing bug in the search logic — results can be under-counted when tags filter removes items after pagination.

### Missing unique constraint migration

Upstream `spacedrive-data` is missing the unique constraint migration for tag applications. Our PR #3044 includes the fix (`8e4a7d6bd` — MIN(id) → MAX(id), keep newest row).

---

## P3 — Moderate

### Job cancel/pause non-functional (#3)

Pause: backend OK but UI takes minutes. Cancel: "Job not found" (searches DB instead of in-memory state).

### Location removal log spam (#2)

Watcher + indexer continue after DB deletion → `"Failed to apply batch: Location not found"` in a loop.

### Thumbnails not auto-generated on new files (#18) — partially fixed

Watcher indexes new files but thumbnail generation not triggered. Requires manual "Regenerate thumbnails".

**Fixed (2026-03-21):**

- Context menu: replaced `useJobDispatch` (broken, `jobs.dispatch` doesn't exist) with direct mutations (`media.thumbnail.regenerate`, `media.ocr.extract`, etc.)
- MIME type: fallback by extension when content_identity doesn't provide MIME
- `useJobDispatch.ts` deleted (commit `6880ccc23`, 2026-03-25)

**Still open:**

- Thumbnails on ephemeral files (non-indexed) impossible: sidecar system requires `content_identity` (FK constraint). Would need sidecar pipeline refactor or on-the-fly content_identity creation.

### Tag filter not wired in frontend UI

`search.files` accepts `TagFilter` backend-side (`search/query.rs`) but no frontend component exposes it. The tag filter in the search bar is not accessible to the user. To be wired when advanced filters are implemented.

(From CodeRabbit review of PR #3044, 2026-03-27)

### Broken icon.ico on upstream (103 bytes)

Upstream `spacedrive-data` has a 103-byte placeholder icon.ico. Our PR #3052 has the fix.

---

## P2-P3 — Waiting on merge

These are fixed in open PRs but not yet merged:

| Issue                                                           | Fixed in               | PR Status       |
| --------------------------------------------------------------- | ---------------------- | --------------- |
| Overview crash (total_capacity undefined)                       | PR #3048               | OPEN            |
| Tag view "Tag Not Found" on branches without tags PR            | PR #3044               | OPEN, MERGEABLE |
| icon.ico broken + SVG gradient decimals                         | PR #3052               | OPEN            |
| Tag duplicate prevention, DEL key for tags, search bar Overview | PR #3044               | OPEN, MERGEABLE |
| Extension lost on root files, EntryUuid validation              | PR #3044 (`f9d254994`) | OPEN, MERGEABLE |
| O(n²) tag merge performance                                     | PR #3044 (`7bf06c579`) | OPEN, MERGEABLE |
| FTS5 injection risk                                             | PR #3044 (`72ce78c0e`) | OPEN, MERGEABLE |
| Batch entry lookups (N queries → 1)                             | PR #3044 (`72ce78c0e`) | OPEN, MERGEABLE |
