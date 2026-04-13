> STATUS: DONE -- Implemented in PR #3041 (merged)

# Implementation Plan: Content Path Resolution & Delete Fix

**Status**: Draft — for separate PR targeting `upstream/v2`
**Dependency**: None (independent of windows-fixes-v2 PR)
**Scope**: Fix Content path resolution in backend, fix delete for multi-file, fix deprecated device_id() calls

---

## Problem Statement

When files are indexed in a Spacedrive library, their `sd_path` becomes `SdPath::Content { content_id }` (content-addressed). The actual filesystem paths are available in `alternate_paths: Vec<SdPath::Physical>`.

Three backend operations **cannot handle Content paths**:

1. **`resolve_in_job()`** (`domain/addressing.rs:675-693`) — returns `Err(NoOnlineInstancesFound)` for Content. This is a TODO stub.
2. **`PathResolver::resolve()`** (`ops/addressing.rs:40`) — calls `unimplemented!()` for Content. **Will panic.**
3. **`DeleteJob`** (`ops/files/delete/job.rs`) — never calls `resolve_in_job()` at all (unlike `CopyJob` which does).

Additionally, Jamie's `device_id → device_slug` refactor (commit `53a722a93`) left 3 call sites using the deprecated `device_id()` which always returns `None`, breaking remote delete routing.

---

## Architecture Context

### How Copy handles this (the model to follow)

`CopyJob::run()` in `ops/files/copy/job.rs:341-355`:

```rust
// Resolve destination
let resolved_destination = self.destination.resolve_in_job(&ctx).await?;
self.destination = resolved_destination;

// For each source:
let resolved_source = source.resolve_in_job(&ctx).await?;
```

Copy resolves Content → Physical **before** passing to strategies. Delete should do the same.

### How File objects carry both paths

`File::from_entry_uuids()` in `domain/file.rs:877-945`:

- If entry has `content_id` → `sd_path = SdPath::Content { content_id }`
- Physical path is added to `alternate_paths` via `PathResolver::get_full_path(db, entry.id)`
- Other entries with same `content_id` are also added to `alternate_paths` (duplicates)

### What `resolve_in_job()` needs to do

For `SdPath::Content { content_id }`:

1. Query DB: find all entries with matching `content_identity.uuid == content_id`
2. For each entry, resolve its physical path via `PathResolver::get_full_path(db, entry.id)`
3. Check which ones are local (current device)
4. Return the local Physical path if available
5. If no local instance, return the first available remote Physical path (for remote operations)

---

## Implementation Steps

### Step 1: Implement `resolve_in_job()` for Content paths

**File**: `core/src/domain/addressing.rs` (lines 674-693)

Replace the TODO stub with actual DB resolution:

```rust
Self::Content { content_id } => {
    // Query library DB for entries with this content_id
    let db = job_ctx.library_db()
        .ok_or(PathResolutionError::NoActiveLibrary)?;

    use crate::infra::db::entities::{content_identity, entry};
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

    // Find ContentIdentity by UUID
    let ci = content_identity::Entity::find()
        .filter(content_identity::Column::Uuid.eq(Some(*content_id)))
        .one(db)
        .await
        .map_err(|e| PathResolutionError::DatabaseError(e.to_string()))?
        .ok_or(PathResolutionError::NoOnlineInstancesFound(*content_id))?;

    // Find all entries with this content
    let entries = entry::Entity::find()
        .filter(entry::Column::ContentId.eq(ci.id))
        .all(db)
        .await
        .map_err(|e| PathResolutionError::DatabaseError(e.to_string()))?;

    // Try to find a local physical path
    for entry in &entries {
        if let Ok(physical_path) =
            crate::ops::indexing::PathResolver::get_full_path(db, entry.id).await
        {
            let device_slug = crate::device::get_current_device_slug();
            let candidate = SdPath::Physical { device_slug, path: physical_path };
            if candidate.is_local() {
                return Ok(candidate);
            }
        }
    }

    // No local instance found
    Err(PathResolutionError::NoOnlineInstancesFound(*content_id))
}
```

**Prerequisite**: Verify that `JobContext` exposes `library_db()` or equivalent. If not, we need to add it.

### Step 2: Fix `PathResolver::resolve()` for Content paths

**File**: `core/src/ops/addressing.rs` (line 40)

Replace `unimplemented!()` with the same logic (or delegate to `resolve_in_job` if a JobContext is available). At minimum, change `unimplemented!()` to a proper error:

```rust
SdPath::Content { content_id } => {
    // Resolve via CoreContext's library
    // Similar logic to resolve_in_job but using CoreContext
    Err(PathResolutionError::NoOnlineInstancesFound(*content_id))
}
```

### Step 3: Add `resolve_in_job()` call to DeleteJob

**File**: `core/src/ops/files/delete/job.rs` (in `run()` method, before strategy selection)

Add path resolution like CopyJob does:

```rust
// Resolve Content paths to Physical paths before strategy selection
let mut resolved_paths = Vec::new();
for path in &self.targets.paths {
    let resolved = path.resolve_in_job(&ctx).await.map_err(|e| {
        JobError::execution(format!("Failed to resolve path: {}", e))
    })?;
    resolved_paths.push(resolved);
}
self.targets = SdPathBatch::new(resolved_paths);
```

Insert this **after** `validate_targets()` and **before** `DeleteStrategyRouter::select_strategy()`.

### Step 4: Fix deprecated `device_id()` calls

#### 4a. `delete/strategy.rs:343` — RemoteDeleteStrategy

**Current** (broken):

```rust
if let Some(device_id) = path.device_id() {
    by_device.entry(device_id).or_default().push(path.clone());
}
```

**Fix**: Use `device_slug()` and group by slug. The networking layer needs slug → device resolution.

```rust
if let Some(slug) = path.device_slug() {
    by_device.entry(slug.to_string()).or_default().push(path.clone());
}
```

Change `by_device` type from `HashMap<Uuid, Vec<SdPath>>` to `HashMap<String, Vec<SdPath>>`.

Then in `delete_on_device()`, resolve slug to UUID via library's device manager:

```rust
let device_id = ctx.resolve_device_slug(&device_slug).await
    .ok_or_else(|| anyhow::anyhow!("Unknown device: {}", device_slug))?;
```

#### 4b. `delete/routing.rs:36` — describe_strategy

Same pattern: `device_id()` → `device_slug()`, adjust HashSet type.

#### 4c. `search/query.rs:836` — search scope filtering

Same migration pattern.

### Step 5: Add tests

**File**: New test module in `core/src/ops/files/delete/` or extend existing tests

1. **Test resolve_in_job with Content path**: Create entry with content_identity, verify resolution to Physical
2. **Test DeleteJob with Content paths**: Verify deletion works end-to-end with Content sd_paths
3. **Test mixed Content + Physical batch**: Verify SdPathBatch with mixed paths resolves correctly
4. **Test Content path with no local instance**: Verify proper error (not panic)

---

## Verification

1. `cargo check -p sd-core` — compilation
2. `cargo test -p sd-core --lib` — unit tests pass
3. Manual test: select multiple files in indexed location → Delete → files actually deleted
4. Manual test: DEL key → files actually deleted (once frontend keybind is wired)

---

## Files Modified

| File                                    | Change                                                      |
| --------------------------------------- | ----------------------------------------------------------- |
| `core/src/domain/addressing.rs`         | Implement `resolve_in_job()` for Content                    |
| `core/src/ops/addressing.rs`            | Fix `unimplemented!()` → proper error or resolution         |
| `core/src/ops/files/delete/job.rs`      | Add resolve loop before strategy selection                  |
| `core/src/ops/files/delete/strategy.rs` | Fix `device_id()` → `device_slug()` in RemoteDeleteStrategy |
| `core/src/ops/files/delete/routing.rs`  | Fix `device_id()` → `device_slug()` in describe_strategy    |
| `core/src/ops/search/query.rs`          | Fix `device_id()` → `device_slug()`                         |

---

## Risks & Mitigations

- **Risk**: `JobContext` might not expose DB access for `resolve_in_job()`
  - **Mitigation**: Check `JobContext` API; may need to add `library_db()` method

- **Risk**: `PathResolver::get_full_path()` is async — verify it works within job context
  - **Mitigation**: CopyJob already calls it via `resolve_in_job`, so the pattern exists

- **Risk**: Content path with NO local instance — should delete fail or delegate to remote?
  - **Mitigation**: Follow CopyJob pattern: return error if no local instance. Remote deletion is a separate concern (RemoteDeleteStrategy handles it once device_slug is fixed).

---

## Out of Scope (but noted)

These are **separate concerns** that should be tracked independently:

### Security: Remote action authorization gaps

See `memory-bank/security_audit_remote_actions.md` for full details.

### Frontend: DEL key binding

The DEL key keybind (`explorer.delete`) needs to be wired in `useExplorerKeyboard.ts`. This should send the file's `sd_path` as-is (Content or Physical) and let the backend resolve. Once this plan is implemented, the frontend can simply send `sd_path` without workaround.

### Frontend: Confirm dialog for permanent delete

Currently removed (Tauri blocks `window.confirm()`). Need a proper Tauri-native dialog component.
