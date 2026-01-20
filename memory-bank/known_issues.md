# Known Issues - Windows Spacedrive

## Issue 1: Slow Indexation Discovery Phase

**Symptom**: Discovery phase takes a long time on large NTFS volumes (238k+ files), percentage stays at 0%.

**Technical Analysis**:

- Discovery uses standard `tokio::fs::read_dir` recursion
- `IndexMode` affects _processing depth_, not discovery speed:
  - `Shallow` = metadata only
  - `Content` = + content hashing for dedup
  - `Deep` = + text extraction, thumbnails
- All modes still traverse the entire filesystem during Discovery

**Potential Solutions**:

1. **MFT parsing** (Windows NTFS only) - Read the Master File Table directly for O(1) file enumeration. Would require admin privileges and `ntfs` crate.
2. **Windows Search API** - Leverage existing Windows Search index if available
3. **Parallel discovery** - Multiple concurrent `read_dir` operations
4. **Progressive discovery** - Show results as they're found (already partially implemented)

**Complexity**: 🔴 High - Would require significant refactoring of discovery phase

---

## Issue 2: Location Removal Log Spam

**Symptom**: After removing a location, logs show continuous errors:

```
Failed to apply batch for location <uuid>: Location not found
```

**Cause**: The filesystem watcher and indexer jobs continue running after the location is deleted from the database.

**Fix Required**:

- `LocationManager::remove_location()` should:
  1. Stop the filesystem watcher for that location
  2. Cancel any in-progress indexing jobs for that location
  3. Clean up the `PendingChanges` queue

**Complexity**: 🟡 Medium

---

## Issue 3: Cancel/Pause Job Not Working

**Symptom**:

- Clicking **Pause**: Backend logs "Job status changed to: Paused", but UI takes minutes to update.
- Clicking **Cancel**: Logs "Failed to cancel job ... Job not found".

**Cause**:

- **Latency**: Event broadcasting or frontend state updates are heavily delayed or blocked (possibly by the main thread or expensive re-renders).
- **Cancel Failure**: Attempts to look up the ephemeral job in the database instead of memory, or the job handle is lost/invalidated incorrectly.

**Fix Required**:

- Optimization of frontend event handling to reduce latency.
- Ensure `CancelJob` action checks in-memory ephemeral jobs when DB lookup fails.
- Verify ephemeral job persistence in `JobManager` memory map.

**Complexity**: 🟡 Medium

---

## Issue 4: PowerShell Spawn for ReFS Block Cloning Check

**Symptom**: PowerShell transcript logs appear periodically, checking ReFS volume capabilities.

**Source**: `core/src/volume/fs/refs.rs:89-134` - `supports_block_cloning()` function.

**Usage**: Called during file copy to decide if `FastCopyStrategy` (CoW) can be used.

**Fix Options**:

1. **Native Windows API** - Use `DeviceIoControl` with `FSCTL_GET_REFS_VOLUME_DATA`
2. **Assume support** - ReFS v2+ (Win Server 2016/Win 10 1703+) always supports block cloning. Check OS version instead.
3. **Cache result** - Call once per volume, cache the result

**Complexity**: 🟢 Easy-Medium

---

## Issue 5: Window Decoration Flicker

**Symptom**: Custom Spacedrive window style reverts to default Windows decorations on click/move.

**Configuration** (tauri.conf.json):

```json
"decorations": true,
"transparent": true,
"hiddenTitle": true
```

**Potential Causes**:

- Conflict between `transparent: true` and `decorations: true`
- `windowEffects.sidebar` may not be compatible on Windows
- Tauri v2 window handling behavior on Windows

**Fix Exploration**:

- Try `decorations: false` with custom titlebar
- Check Tauri v2 Windows-specific documentation
- Test without `transparent: true`

**Complexity**: 🔴 High - May require custom titlebar implementation

---

## Issue 6: "Location root path is not local" Error

**Symptom**: Ephemeral indexing jobs fail immediately with error:

```
Error in job execution: Location root path is not local
```

**Context**: Observed when volumes like Google Drive (mounted as FAT32 at `C:\Users\...\Accès Google Drive\`) are detected by sysinfo.

**Potential Cause**: The indexer checks if the path is "local" before proceeding. Cloud-synced folders (Google Drive, OneDrive) may be detected as volumes but fail this check.

**Fix Exploration**:

- Check the "is_local" detection logic in indexer
- Possibly filter out virtual/cloud filesystems from volume detection
- Or allow ephemeral indexing on non-local paths with appropriate warnings

**Complexity**: 🟡 Medium

---

## Issue 7: False Positive "High Risk Path" on OneDrive Folders

**Symptom**: Standard user folders (e.g., `OneDrive/Images/Screenshots`) are flagged as "System Directory" during onboarding:

```
High Risk Path Detected
This is a system directory that contains OS files
```

**Cause**: The existing system directory check in `core/src/ops/locations/validate/query.rs` likely uses overly broad matching or misidentifies certain paths (possibly due to `desktop.ini` or hidden system attributes common in OneDrive).

**Fix Required**:

- Refine `is_system_directory` logic to stricter matching.
- Explicitly whitelist known user data paths (Documents, Pictures, OneDrive roots).

**Complexity**: 🟢 Easy

---

## Issue 8: "Database Locked" Error on Location Removal

**Symptom**: Removing a location fails intermittently with SQLite `SQLITE_BUSY` (code 517):

```
Error: Action failed: ... (code: 517) database is locked
```

**Trace Analysis**:

- First attempt fails with lock error.
- Second attempt fails with "Location not found".
- **Observation**: Restarting the app shows the location _was_ successfully deleted.

**Cause**:

- The DB transaction likely committed successfully despite the error bubbling up (or a race condition occurred where deletion happened but response failed).
- `SQLITE_BUSY` might be triggering a rollback in the application logic, but the underlying WAL write might have persisted, or the error handling logic inaccurately reports failure for a simpler timeout.
- The UI does not invalidate its cache or optimistically remove the location when this specific error occurs, leading to a "ghost" location.

**Fix Required**:

- Wrap removal in a retry mechanism for `SQLITE_BUSY`.
- **Crucial**: Ensure watcher/indexer is STOPPED before attempting removal (same fix as Issue #2).
- Improve frontend error handling to check if the entity still exists if a "Location not found" error returns, or force a state refresh.

**Complexity**: 🟡 Medium

---

## Priority Recommendation

| Priority | Issue            | Effort | Impact         |
| -------- | ---------------- | ------ | -------------- |
| 1        | #8 DB Lock       | Medium | Stability      |
| 2        | #7 OneDrive      | Easy   | Onboarding UX  |
| 3        | #4 PowerShell    | Easy   | Low noise      |
| 4        | #2 Location spam | Medium | Better UX      |
| 5        | #3 Job cancel    | Medium | Functional bug |
| 6        | #1 Indexation    | High   | Performance    |
| 7        | #5 Window        | High   | Cosmetic       |
