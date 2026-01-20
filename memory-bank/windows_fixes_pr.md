# Windows Compatibility & Stability Fixes

This documentation aggregates the targeted fixes applied to improve Spacedrive's stability and functionality on Windows.

---

## Commits Overview

### 1. `fix(core): resolve Windows build errors after upstream merge`

**Fix compilation errors after merging upstream changes:**

- Added explicit type annotation in `location/manager.rs` closure (E0282)
- Replaced unstable `windows_by_handle` with stable alternative in `indexing/phases/processing.rs`
- Fixed lifetime issue (E0716) in `volume/platform/windows.rs`
- Enhanced `is_library_directory` to verify paths are actually directories

### 2. `Fix build errors and library initialization crash on Windows`

**Additional build fixes and runtime stability:**

- Updated DaemonRequest format in `main.rs` for RPC compatibility
- Fixed unreachable code warnings in drag/drop and keybind handlers
- Resolved `LocationCacheDemo` missing export in frontend

### 3. `feat(ui): add specialized context menu options for Locations`

**Enhanced sidebar context menus:**

- Added "Quick Rescan" and "Full Rescan" options for locations
- Added "Remove Location" option (distinct from "Remove from Space")
- Added direct link to Library Settings

### 4. `fix(volume): improve detection and untrack stability on Windows`

**Critical volume detection and identity stability fixes:**

- **Path prefix stripping**: Windows canonicalized paths include `\\?\` prefix (e.g., `\\?\E:\Folder`) that was causing "No volume found" errors. Fixed in NTFS, ReFS, and generic filesystem handlers.
- **Deterministic UUIDs**: Changed from random `Uuid::new_v4()` to deterministic `Uuid::new_v5()` based on volume fingerprint. This ensures the same physical volume always gets the same UUID across daemon restarts, preventing UI duplication.
- **Untrack cache update**: `VolumeManager::untrack_volume()` now properly updates the in-memory cache and emits `ResourceChanged` instead of `ResourceDeleted` (since the volume still exists physically).
- **Logic consolidation**: `VolumeUntrackAction` now delegates to `VolumeManager` to avoid duplicate code paths.

### 5. `fix(jobs): ensure ephemeral jobs emit completion events`

**Fix stuck progress bars for indexing jobs:**

- Ephemeral jobs (like indexing) were not emitting completion events because the logic checked `should_persist` instead of `should_emit_events`
- `JobExecutor` now skips database updates for ephemeral jobs, avoiding "Job not found" errors
- UI now correctly receives `JobCompleted` events for all job types

### 6. `fix(files): use native trash and fix UI latency on Windows`

**Reliable file deletion and responsive UI:**

- Replaced custom trash implementation with the `trash` crate for reliable cross-platform recycle bin support
- Fixed path separator mismatch in `useNormalizedQuery.ts` that caused multi-second delays in UI updates on Windows (backslashes vs forward slashes)

### 7. `fix(tauri): use MSI-compatible version and add menu permission`

**Build compatibility:**

- Changed version from `2.0.0-pre.1` to `2.0.0` - MSI installers require numeric-only versions
- Added `core:menu:default` permission (Note: This may not be strictly required as the app uses custom React menus, but it's harmless to include and could be useful for future native menu integration)

### 8. `perf(refs): replace PowerShell with native Windows API for ReFS detection`

**Eliminate PowerShell spawning for ReFS operations:**

- Replaced PowerShell-based block cloning check with native `DeviceIoControl` + `FSCTL_GET_REFS_VOLUME_DATA`
- Added static cache to avoid repeated IOCTL calls (check once per volume path)
- Replaced PowerShell volume enumeration with `sysinfo` (consistent with `ntfs.rs`)
- Added `Win32_System_Ioctl` and `Win32_System_IO` features to `windows-sys`
- ReFS v2+ detection (Windows 10 1703+) now instant instead of spawning PowerShell process

### 9. `fix(tests): update test suite for VolumeFingerprint API and domain model changes`

**Fix blocking test errors and warnings:**

- **`volume_fingerprint_stability_test.rs`**: Migrated tests from deprecated `VolumeFingerprint::new()` to new constructors (`from_primary_volume()`, `from_external_volume()`)
- **`library_demo.rs`**: Updated domain model fields (removed `device_id`, added `volume_id` to Entry and Location)
- **`manager.rs` test**: `test_is_library_directory` now creates actual temp directories since `is_library_directory()` checks `path.is_dir()`
- **`progress.rs` tests**: Fixed float precision assertions and completion expectations to match implementation (files+dirs count)
- **Cleanup**: Removed unused `verify_children_count` in ephemeral_watcher_test, prefixed unused variables, restored `read_only` method with `#[allow(dead_code)]`

---

## Testing Performed

- ✅ Clean build on Windows 11
- ✅ Volume detection works for NTFS, ReFS (DevDrive), and external drives
- ✅ Adding and removing locations functions correctly
- ✅ File operations (copy, delete to trash) complete with proper UI feedback
- ✅ Indexing jobs complete and progress bars clear properly
- ✅ Context menus display correct options
- ✅ No more PowerShell windows spawning during ReFS operations
- ✅ `cargo test -p sd-core --lib` passes (321 tests)

## Known Remaining Issues

- **Location watcher cleanup**: When removing a location, the filesystem watcher may continue running briefly, causing log spam until the next refresh cycle. This is a minor issue that doesn't affect functionality.
- **Historical volume duplicates**: Users who encountered volume duplication before this fix may have orphaned entries in their database. A fresh library or database reset will resolve this.
- **Cloud drive indexing**: Volumes detected as cloud-synced folders (Google Drive, OneDrive) may fail ephemeral indexing with "Location root path is not local" error.
