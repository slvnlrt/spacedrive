# PR #3037 — fix(windows): Windows v2 Desktop Fixes

**Status:** MERGED into `main`
**Date:** ~2026-03-27
**Author:** slvnlrt
**Branch:** `windows-fixes-v2`
**URL:** https://github.com/spacedriveapp/spacedrive/pull/3037
**Stats:** 41 files changed, +1037 -1395 lines (net negative — more code removed than added)

---

## Windows compatibility fixes and cross-platform improvements

### Summary

This PR fixes 29+ issues encountered while building and running Spacedrive v2 on Windows 11. The changes fall into two categories:

- **Windows-specific (17 commits):** path separator handling, Win32 API replacements for deprecated PowerShell/wmic, hidden file detection, MSI build target
- **Cross-platform (12 commits):** daemon RPC format, SQLite contention, trash deletion, ephemeral volume browsing, event filtering, job progress UI
- **Review feedback (5 commits):** CodeRabbit/Tembo review fixes — SQLite per-connection PRAGMAs, ReFS IOCTL improvements, volume UUID stability, dev script portability

Net result: 41 files changed, +1037 −1395 lines (more code removed than added — primarily PowerShell/wmic removal).

No new dependencies except `trash` v3.3 (replaces custom platform-specific trash implementations). `sysinfo` and `windows-sys` were already in Cargo.toml.

### Context and scope

The goal of this work was to stabilize the core user experience on Windows: volumes appearing correctly in the sidebar, ephemeral browsing actually showing files, file deletion going to the real Recycle Bin, hidden files being detected properly, and the app not crashing due to path separator issues or PowerShell failures.

This covers the "local filesystem" layer of Spacedrive — the part that is most platform-dependent: volume detection, path handling, filesystem feature queries, device identification. All PowerShell and wmic usage has been eliminated in favor of native Win32 APIs or cross-platform crates (`sysinfo`, `trash`).

The advanced VDFS features (peer-to-peer sync, remote file operations, content-addressed deduplication, cloud storage backends) have not been tested or worked on in this PR. That said, the networking and sync layer (`core/src/service/network/`) is almost entirely platform-agnostic — it uses iroh (QUIC/UDP transport), msgpack serialization, and has no `#[cfg(windows)]` outside of test code. If peering and sync work on macOS, they should work on Windows with minimal issues. The main risk area would be path separators in sync messages, which this PR already addresses for file transfers (commit `fix(networking): normalize path separators`).

The pre-existing issues documented below (Content path resolution in delete, device-level authorization in remote actions) are architectural gaps that affect all platforms equally and should be planned for separate PRs.

---

### Detailed Changes

#### 1. Volume Detection — Replace PowerShell/wmic with native APIs

**Problem:** `Get-Volume | ConvertTo-Json` fails on many Windows configurations — BOM artifacts, encoding issues, localized error messages — while still exiting with code 0. This silently aborts volume detection entirely, leaving the app with no volumes. The fallback `wmic logicaldisk` had similar issues with output format variations across Windows versions and locales.

**Solution:** Use `sysinfo::Disks` (already a dependency) for volume enumeration. Zero shell spawning, zero JSON parsing. `sysinfo` uses native Win32 APIs internally (`GetLogicalDriveStringsW`, `GetDiskFreeSpaceExW`, `GetVolumeInformationW`). Bonus: `disk.kind()` maps directly to `DiskType::SSD/HDD` (previously always `Unknown` because the PowerShell parsing didn't extract media type).

| Commit                                                  | Scope                             | Details                                                                                                                                                                                                                                                                                                                                                           |
| ------------------------------------------------------- | --------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `perf(volume): replace PowerShell with sysinfo`         | `volume/platform/windows.rs`      | Full rewrite of `detect_volumes()`                                                                                                                                                                                                                                                                                                                                |
| `fix(volume): normalize mount point path`               | `volume/fingerprint.rs`           | Lowercase + trim trailing slashes to prevent duplicate fingerprints (`C:\` vs `c:\`). Windows filesystems are case-insensitive, so the same volume can produce different fingerprints depending on how the OS reports the mount point                                                                                                                             |
| `fix(volume): restore UUID from database at startup`    | `volume/manager.rs`               | Volume UUIDs were regenerated (`Uuid::new_v4()`) on every restart because `tracked_volumes_map` didn't store the DB UUID. React uses `volume.id` as component key → new UUID = full remount of volume components on every app restart. Now restores from DB during `reconcile_volumes()`                                                                          |
| `fix(volume): use Win32 APIs for same_physical_storage` | `volume/fs/generic.rs`, `ntfs.rs` | Was hardcoded `false` on Windows (`#[cfg(windows)] { return false }`). This forced all same-volume operations through the streaming copy path instead of fast `fs::rename()`. Now uses `GetVolumeInformationW` (serial number comparison) for FAT32/exFAT in `generic.rs`, and `GetVolumeNameForVolumeMountPointW` (stable GUID comparison) for NTFS in `ntfs.rs` |

#### 2. NTFS/ReFS — Remove all PowerShell, clean up dead code

The NTFS handler had accumulated several layers of feature-detection code that spawned PowerShell to query filesystem capabilities. In practice:

- `get_ntfs_features()` queried compression/encryption support via PowerShell, but every NTFS volume since Windows Vista supports both. The result was always the same — replaced with hardcoded values.
- `resolve_ntfs_path()` called PowerShell to resolve 8.3 short names, but `std::fs::canonicalize()` does the same thing natively via `GetFinalPathNameByHandleW`.
- `supports_hardlinks()` and `supports_junctions()` always returned `true` unconditionally — NTFS has supported both since Windows 2000. Dead code.
- `enhance_volume()` was a no-op after the features above were removed.
- `NtfsFeatures` struct had no remaining consumers.

For ReFS, block-cloning detection (`BLOCK_CLONING` feature) was also done via PowerShell. Replaced with a direct `DeviceIoControl` call using `FSCTL_GET_REFS_VOLUME_DATA`, which is the documented Win32 approach.

| Commit                                                       | Scope               | Details                                                                                                                                                                                                                            |
| ------------------------------------------------------------ | ------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `perf(refs): replace PowerShell with native Win32 IOCTL`     | `volume/fs/refs.rs` | ReFS block-cloning detection via `FSCTL_GET_REFS_VOLUME_DATA` `DeviceIoControl`. Results cached per volume in a static `HashMap`                                                                                                   |
| `perf(ntfs): replace PowerShell with native implementations` | `volume/fs/ntfs.rs` | `get_ntfs_features()` → hardcoded (compression/encryption always supported since Vista). `resolve_ntfs_path()` → `std::fs::canonicalize()`                                                                                         |
| `perf(ntfs): remove remaining dead code`                     | `volume/fs/ntfs.rs` | Removed `NtfsFeatures` struct, `resolve_ntfs_path()`, `enhance_volume()` — none had any callers after the PowerShell replacement above                                                                                             |
| `fix(ntfs): remove dead code supports_hardlinks/junctions`   | `volume/fs/ntfs.rs` | Separate commit for `supports_hardlinks()` and `supports_junctions()` removal — both returned `true` unconditionally (NTFS has supported both since Windows 2000), but these were public API so split out for easier review/revert |

#### 3. Device Detection — Replace wmic with registry APIs

`wmic` is deprecated since Windows 10 21H1 and removed in Windows 11 25H2. On systems where wmic is absent, all device detection functions (`detect_hardware_model`, `detect_manufacturer`, `detect_form_factor`, `detect_gpu_models`, `detect_boot_disk_type`) silently returned empty strings or defaults, resulting in incomplete device profiles.

All wmic/PowerShell calls replaced with direct registry reads (`HKLM\HARDWARE\DESCRIPTION\System\BIOS\*`) and Win32 API calls. These are the same data sources that wmic itself reads internally, without the overhead of COM initialization and WMI query parsing:

| Commit                                                         | Scope               | Details                                                                                                                                                                                                                                                                               |
| -------------------------------------------------------------- | ------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `perf(device): replace wmic/powershell with native Win32 APIs` | `domain/device.rs`  | `detect_hardware_model` → registry `SystemProductName`, `detect_manufacturer` → `SystemManufacturer`, `detect_form_factor` → `GetSystemPowerStatus` (battery flag), `detect_gpu_models` → registry display adapter enum, `detect_boot_disk_type/capacity` → `sysinfo::Disks` on `C:\` |
| `perf(device): replace wmic in device/manager.rs`              | `device/manager.rs` | Last remaining wmic call → registry API                                                                                                                                                                                                                                               |

#### 4. Path Separator Issues (Windows `\` vs Unix `/`)

| Commit                                                                   | Scope                                 | Details                                                                                                                                     |
| ------------------------------------------------------------------------ | ------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| `fix(ui): normalize Windows path separators in useNormalizedQuery`       | `useNormalizedQuery.ts`               | `pathStr.lastIndexOf("/")` returned -1 on Windows paths → all files filtered out of batch cache updates                                     |
| `fix(ephemeral): handle Windows path separators in tree operations`      | `ephemeral/index.rs`                  | Three bugs: `count_entries_under_path()` only checked `b'/'`, `remove_directory_tree()` only matched "/" suffix, root node hardcoded to "/" |
| `fix(networking): normalize path separators for cross-platform transfer` | `file_transfer.rs`, `file_sharing.rs` | Windows→macOS transfer sent backslash paths. Sender normalizes to `/`, receiver converts to `MAIN_SEPARATOR`                                |

#### 5. Daemon RPC Format (all platforms)

**Problem:** Two code paths in `main.rs` sent JSON-RPC 2.0 format but the daemon expects serde-tagged `DaemonRequest` enums.

| Commit                                                      | Scope     | Details                                                                                                                                                                                                |
| ----------------------------------------------------------- | --------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `fix(tauri): use DaemonRequest format for daemon RPC calls` | `main.rs` | `{"jsonrpc":"2.0","method":"query:..."}` → `{"Query":{"method":"..."}}`. Fixed both `validate_and_reset_library_if_needed()` and file-association handler. Response parsing: `"JsonOk"` not `"result"` |
| `fix(tauri): use correct registry method names`             | `main.rs` | Handler registry uses full prefixed names (`query:libraries.list`). Bare names caused "Unknown method" at startup                                                                                      |

#### 6. SQLite Contention (all platforms)

**Problem:** `remove_location()` called `delete_subtree()` with a separate connection while holding a transaction → `SQLITE_BUSY`. The location's database entries were partially deleted, leaving ghost locations in the UI.

| Commit                                      | Scope             | Details                                                                                                                                |
| ------------------------------------------- | ----------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| `fix(db): add PRAGMA busy_timeout=5000`     | `database.rs`     | SQLite returns `SQLITE_BUSY` immediately by default. Now retries for 5 seconds. Applied in `Database::create()` and `Database::open()` |
| `fix(locations): use delete_subtree_in_txn` | `location/mod.rs` | Use `delete_subtree_in_txn(&txn)` instead of opening a separate connection                                                             |

#### 7. File Operations (all platforms)

**Trash:** The previous implementation had platform-specific `move_to_trash_windows()`, `move_to_trash_macos()`, and `move_to_trash_unix()` methods. The Windows version moved files to `%TEMP%/spacedrive_trash` — a regular directory, not the actual Recycle Bin — so files disappeared permanently without appearing in the Recycle Bin for recovery. The `trash` crate (v3.3) replaces all three with a single call that uses native OS APIs: `SHFileOperation` on Windows (real Recycle Bin), `NSFileManager` on macOS (Finder Trash), and the XDG trash spec on Linux. This also removed ~80 lines of custom trash code including `find_unique_trash_name()`.

**Hidden files:** On Windows, files are hidden via the `FILE_ATTRIBUTE_HIDDEN` filesystem attribute, not by prefixing the name with `.`. The codebase had 10 occurrences of `name.starts_with('.')` across 4 indexing files. This caused `desktop.ini` and `Thumbs.db` to appear as visible files, while `.gitignore` was incorrectly treated as hidden.

| Commit                                                          | Scope                      | Details                                                                                                                                                                        |
| --------------------------------------------------------------- | -------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `fix(files): use native trash crate`                            | `delete/strategy.rs`       | Replaced 3 platform-specific trash methods with `trash::delete()`. Removes ~80 lines. Real Recycle Bin on Windows                                                              |
| `fix(indexing): detect hidden files with FILE_ATTRIBUTE_HIDDEN` | 4 files in `ops/indexing/` | Added `is_hidden_path()` helper: `#[cfg(windows)]` uses `GetFileAttributesW()` + `FILE_ATTRIBUTE_HIDDEN`, `#[cfg(unix)]` falls back to dot-prefix. Replaced all 10 occurrences |

#### 8. Ephemeral Volume Browsing (probably all platforms)

**Problem:** Clicking on an untracked volume showed empty content, or took 30+ seconds to display files.

Root causes (3 distinct bugs fixed in one commit):

1. **Transport race condition:** `listen("core-event")` was registered after `invoke("subscribe_to_events")` → buffered events lost on first visit. Fixed by swapping registration order.
2. **Event filter too strict:** `affects_path()` with `include_descendants=false` only matched exact path equality, not direct children of the watched directory. Fixed by also checking `file_path.parent() == scope_path`.
3. **Async listing returned empty:** `query_ephemeral_directory_impl()` dispatched the indexer job and returned immediately. Since ephemeral indexing takes <500ms, it now waits for completion synchronously (polling `is_indexing()` every 5ms, 10s timeout). Subdirectories also trigger their own indexer job.

| Commit                                                             | Scope                                | Details                                                                                                                                                                              |
| ------------------------------------------------------------------ | ------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `fix(ephemeral): fix volume browsing — race, filter, sync listing` | `ephemeral/index.rs`, `transport.ts` | All three bugs above + refactored into `read_ephemeral_listing()` and `wait_for_indexing()` helpers                                                                                  |
| `fix(addressing): accept device UUID as device_slug`               | `addressing.rs`, `VolumesGroup.tsx`  | Frontend sent `volume.device_id` (UUID) instead of `device.slug` → `is_local()` returned false → "Location root path is not local" error. Backend now accepts slug, UUID, or "local" |

#### 9. Build (Windows-specific)

| Commit                                      | Scope                     | Details                                                                                                                                           |
| ------------------------------------------- | ------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| `fix(build): exclude MSI target on Windows` | `tauri.windows.conf.json` | MSI bundler rejects semver pre-release identifiers (`2.0.0-alpha.2`). CI already uses `--bundles nsis`. Added `targets: ["nsis", "app"]` to match |

#### 10. Other Fixes

| Commit                                             | Scope                                 | Details                                                                                                                                                                                                                   |
| -------------------------------------------------- | ------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `fix(locations): exact match for root paths`       | `location/mod.rs`                     | `path.starts_with("C:\\")` matched everything on C: → OneDrive flagged as "System Directory". Now uses `parent().is_none()`                                                                                               |
| `fix(networking): use temp_dir()`                  | `file_transfer.rs`, `file_sharing.rs` | Hardcoded `/tmp` → `std::env::temp_dir()`                                                                                                                                                                                 |
| `fix(watcher): downgrade startup warning to debug` | `handlers/persistent.rs`              | `warn!("FsWatcher not connected")` is expected during startup (recovery in `start()` re-registers locations)                                                                                                              |
| `fix(ui): install maplibre-gl at root`             | `package.json`                        | `maplibre-gl` was only in `packages/interface/package.json`, but Vite's dependency resolution starts from the workspace root. Build failed with `Module not found`. Adding it to the root `package.json` fixes resolution |
| `fix(tests): align progress.rs tests`              | `progress.rs`                         | Wrong completion values, exact f32 equality instead of epsilon comparison                                                                                                                                                 |
| `fix(ui): set job progress to 100% on completion`  | `useJobs.ts`                          | `JobProgress` events are throttled to 100ms. Fast jobs complete before the second tick → UI stuck at 0%. Now sets `progress: 1.0` on `JobCompleted`                                                                       |
| `chore: update Cargo.lock`                         | `Cargo.lock`                          | For `trash` crate dependency                                                                                                                                                                                              |

#### 11. Review Feedback (CodeRabbit + Tembo bot)

Fixes based on automated review feedback. One suggestion was intentionally skipped (see below).

| Commit                                                  | Scope                   | Details                                                                                                                                                                                                                                                                                                                                                  |
| ------------------------------------------------------- | ----------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `fix: address review feedback from Tembo bot`           | 8 files                 | `volume_guid()` null terminator fix (`unwrap_or(0)` → `unwrap_or(len)`), pass real `total_bytes_capacity` to classifier, subscribe error handling in transport.ts                                                                                                                                                                                        |
| `fix: address CodeRabbit review feedback`               | 15 files                | See details below                                                                                                                                                                                                                                                                                                                                        |
| `fix(volume): preserve DB UUID on cache refresh`        | `volume/manager.rs`     | When a cached volume is updated (space change, mount change), prefer the DB UUID from `tracked_volumes_map` instead of blindly copying `existing.id` — prevents ephemeral UUIDs from overwriting stable DB UUIDs                                                                                                                                         |
| `fix(volume): use stable DB UUID in volumes.list query` | `volumes/list/query.rs` | `volumes.list` returned ephemeral `Uuid::new_v4()` IDs from the VolumeManager cache (populated before library load). ResourceChanged events used DB UUIDs (set by the refresh after library load). Mismatch → frontend appended instead of updating → volume duplication in UI. Fix: query now sets `live_vol.id = tracked_vol.uuid` for tracked volumes |
| `fix(tauri): resolve target-dir from Cargo config`      | `dev-with-daemon.ts`    | Dev script hardcoded `target/debug` as daemon binary path. Breaks when `target-dir` is customized in `.cargo/config.toml`. Now uses `cargo metadata` to resolve, with fallback to default path                                                                                                                                                           |

**CodeRabbit review details (11 items fixed, 1 skipped):**

| #   | Item                                                            | Fix                                                                                                                                                                                                                                                       |
| --- | --------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | `BatteryFlag` exact equality check                              | Changed to bitmask: `flag & 0x80 != 0` (no battery) instead of `== 128`                                                                                                                                                                                   |
| 2   | Hardcoded `C:\` for system drive                                | Use `%SystemDrive%` env var with `C:\` fallback                                                                                                                                                                                                           |
| 3   | SQLite PRAGMAs applied once via `execute()`                     | Rewritten: all PRAGMAs applied per-connection via `SqliteConnectOptions` (`busy_timeout`, `journal_mode`, `synchronous`, `pragma()`) — ensures every pooled connection gets the settings                                                                  |
| 5   | `GetVolumeInformationW` on raw mount point                      | Use `GetVolumePathNameW` first to resolve actual volume mount point (handles folder-mounted volumes)                                                                                                                                                      |
| 6   | ReFS handler incomplete                                         | Proper IOCTL version detection (`FSCTL_GET_REFS_VOLUME_DATA`), real volume GUID via shared `volume_guid()`, capability stored in `volume.supports_block_cloning`. `supports_cow()` now returns `true` for ReFS only when block cloning is confirmed (v2+) |
| 7   | System volume detected by drive letter (`C:\` vs `D:\`)         | Check for `Windows\System32` directory instead                                                                                                                                                                                                            |
| 8   | Duplicated registry read code in `device/manager.rs`            | Reuse `reg_read_hklm()` (made `pub(crate)`) from `domain/device.rs`                                                                                                                                                                                       |
| 10  | `VolumesGroup.tsx` hardcoded `'local'` fallback for device slug | Removed — `volumeData` now `undefined` when device slug is unavailable                                                                                                                                                                                    |
| 11  | `useNormalizedQuery` case-sensitive path matching on Windows    | Added `.toLowerCase()` to both sides of scope/path comparison + `.catch()` on subscription promise                                                                                                                                                        |
| 12  | `volume_guid()` duplicated between `ntfs.rs` and `refs.rs`      | Extracted shared `volume_guid()` to `volume/fs/mod.rs`, both handlers use `super::volume_guid()`                                                                                                                                                          |
| —   | `Volume` struct                                                 | Added `supports_block_cloning: bool` field with `#[serde(default)]` for ReFS CoW routing                                                                                                                                                                  |

**Skipped:** Item #4 (use `to_string_lossy()` for path conversion) — the current `to_str().unwrap_or("")` behavior is intentional: lossy conversion would silently produce garbled paths with replacement characters, which is worse than returning empty and falling through to the next detection method.

---

### Known issues (pre-existing, out of scope)

These issues were discovered during testing but are not introduced by this PR. They exist on `main` and probably affect all platforms equally. They are documented here for visibility and should be addressed in separate PRs.

#### P1 — Multi-file delete fails on indexed files

Files in indexed locations have `sd_path: SdPath::Content { content_id }`. The delete pipeline never resolves Content paths to Physical paths:

- `DeleteJob::run()` passes paths directly to the strategy without calling `resolve_in_job()` (unlike `CopyJob` which does)
- `resolve_in_job()` for Content is a TODO stub → `Err(NoOnlineInstancesFound)` (`addressing.rs:686`)
- `PathResolver::resolve()` for Content → `unimplemented!()` (`ops/addressing.rs:40`) — will panic if reached
- `LocalDeleteStrategy` falls through to "Path is remote or unsupported" silently

Additionally, `RemoteDeleteStrategy` uses deprecated `device_id()` (always returns `None`) instead of `device_slug()` — leftover from the device_id → device_slug refactor in `53a722a93`. Remote deletion routing is broken for the same reason (also affects `delete/routing.rs:36` and `search/query.rs:836`).

**Plan:** Separate PR to implement `resolve_in_job()` for Content paths (following CopyJob's pattern), fix `unimplemented!()` panic, and migrate deprecated `device_id()` calls.

**Resolution:** PR #3041 (merged upstream).

#### P1 — DEL key not wired in keyboard handler

The `explorer.delete` keybind is registered in the action registry but not connected in `useExplorerKeyboard.ts`. Requires the Content path fix above before it can work reliably.

**Resolution:** PR #3041 (merged upstream).

#### P2 — `libraries.list` handler not registered in daemon

`validate_and_reset_library_if_needed()` sends a valid `DaemonRequest` (fixed in this PR), but the daemon returns `{"Error":{"OperationFailed":"Unknown method: query:libraries.list"}}`. The method is not registered on the daemon side. Non-blocking — the app functions normally, but stale libraries are not cleaned up on restart.

#### P2 — Tags filter returns no results

Tags can be created and assigned to files, but clicking a tag to filter returns empty results. The query appears to be constructed correctly on the frontend; the issue is likely in the backend query builder.

**Resolution:** PR #3044 (open, targeting spacedrive-data).

#### P2 — Security: remote actions missing device-level authorization

Path-level validation is in place (PR #2944 — canonicalization + library boundary check). The remaining gap is device-level authorization: remote file operation handlers (`file_delete.rs`, `file_transfer.rs`) do not verify that the requesting device is paired/trusted before processing. The `_from_device` parameter is explicitly ignored (`file_delete.rs:54`). A device on the local network speaking the iroh protocol could request operations on files within library locations without being paired.

Additionally:

- `file_transfer.rs` falls back to `Uuid::new_v4()` if device registry lookup fails — fail-open, not fail-closed
- The sync protocol applies DB state changes without path validation — a compromised node could inject location records pointing to system directories, expanding the allowed path set for future file operations
- `TransferMode::EphemeralShare` auto-accepts incoming files without a UI consent prompt

#### P3 — Job progress display (partially fixed by this PR)

`JobProgress` events are throttled to 100ms. Fast jobs complete before the second tick and the UI stays stuck at 0%. This PR fixes the symptom via `useJobs.ts`: progress is set to 1.0 on `JobCompleted`. The underlying cause (the delete job emits no `ctx.progress()` calls at all) is a separate upstream issue.

**Resolution:** PR #3041 (merged upstream) — delete job now emits `GenericProgress` at 4 phases.

#### P3 — Job cancel/pause not functional

**Pause:** backend logs "Job status changed to: Paused" but the UI takes minutes to reflect it. **Cancel:** returns "Job not found" — the cancel handler searches the DB for ephemeral jobs instead of in-memory state.

#### P3 — Location removal log spam

After removing a location, the watcher and indexer jobs continue running and emit continuous "Failed to apply batch: Location not found" log errors. `remove_location()` should stop the watcher and cancel active jobs.

#### P3 — Thumbnails not auto-generated on file arrival

When a file is copied into an indexed location, the watcher detects and indexes it, but the thumbnail generation job is not triggered automatically. Manual "Regenerate thumbnails" is required.

#### P3 — Search bar not functional in Overview route

The search bar has no effect when the active route is the "Overview" view — likely a missing search context provider for that route.

**Resolution:** PR #3044 (open, targeting spacedrive-data) — Search button in `OverviewTopBar.tsx` navigates to `/explorer`.

#### P4 — Slow indexation discovery phase on large NTFS volumes

Discovery stays at 0% for extended periods on volumes with 200k+ files. Root cause: recursive `tokio::fs::read_dir`. Potential fixes: MFT direct parsing, Windows Search API, or parallel directory traversal. Out of scope for this PR.

#### P4 — Window decoration flicker on Windows

Custom window decorations (`transparent: true`, `hiddenTitle: true`) revert to native Windows chrome on click or drag. Likely a Tauri v2 / Windows compositor conflict. May require a fully custom titlebar.

**Resolution:** PR #3040 (merged upstream) — DWM dark titlebar via `DwmSetWindowAttribute`.

---

### Test plan

- [x] `cargo check -p sd-core` — passes
- [x] `cargo check -p spacedrive` (Tauri app) — passes
- [x] `cargo test -p sd-core --lib` — 321 tests pass
- [x] Manual testing on Windows 11: volume detection, ephemeral browsing, subdirectory navigation, single file deletion to Recycle Bin, location addition (OneDrive, external drives), job progress display
- [x] Volume UUID stability: verified no duplication after periodic refresh (30s cycles)
- [x] Release build on Windows 11 — `cargo tauri build` completes successfully
- [ ] macOS/Linux regression testing — not tested (Windows-only dev environment). Most changes are `#[cfg(windows)]` gated or use cross-platform APIs (`sysinfo`, `trash`, `std::env::temp_dir()`). The cross-platform fixes (daemon RPC, SQLite, ephemeral browsing, job progress) should be verified on macOS.

---

## Resolved Issues (14)

14 issues resolved by 29 commits on `windows-fixes-v2`.

| #   | Issue                                     | Commit(s)                                                         | Summary                                                       |
| --- | ----------------------------------------- | ----------------------------------------------------------------- | ------------------------------------------------------------- |
| 4   | PowerShell volume detection               | `perf(volume): replace PowerShell with sysinfo`                   | `Get-Volume` → `sysinfo::Disks` (native Win32)                |
| 6   | "Location root path is not local"         | `fix(addressing): accept device UUID as device_slug`              | Frontend sent UUID instead of slug → `is_local()` false       |
| 7   | OneDrive flagged "System Directory"       | `fix(locations): exact match for root paths`                      | `starts_with("C:\\")` → `parent().is_none()`                  |
| 8   | "Database Locked" on location removal     | `fix(db): busy_timeout` + `fix(locations): delete_subtree_in_txn` | Concurrent connection → same transaction                      |
| 9   | Volume UUID unstable at restart           | `fix(volume): restore UUID from database`                         | `tracked_volumes_map` didn't store the DB UUID                |
| 10  | Ephemeral browsing broken (Windows)       | `fix(ephemeral): handle Windows path separators`                  | 3 bugs separators `b'/'` vs `b'\\'`                           |
| 11  | `/tmp` hardcoded                          | `fix(networking): use temp_dir()`                                 | `"/tmp"` → `std::env::temp_dir()`                             |
| 12  | PathBuf cross-platform                    | `fix(networking): normalize path separators`                      | Sender normalizes to `/`, receiver to `MAIN_SEPARATOR`        |
| 13  | Hidden files Unix-only                    | `fix(indexing): FILE_ATTRIBUTE_HIDDEN`                            | 10x `starts_with('.')` → `is_hidden_path()` helper            |
| 14  | `same_physical_storage` = false (Windows) | `fix(volume): Win32 APIs for same_physical_storage`               | `GetVolumeInformationW` + `GetVolumeNameForVolumeMountPointW` |

Additional issues fixed (not individually numbered, included in the PR):

- NTFS: PowerShell → hardcoded values + `canonicalize()` + dead code removal (4 commits)
- ReFS: PowerShell → `DeviceIoControl` IOCTL (1 commit)
- Device detection: wmic → registry APIs (2 commits)
- Daemon RPC: JSON-RPC 2.0 → `DaemonRequest` format + method prefix (2 commits)
- Build: MSI target exclusion for semver pre-release (1 commit)
- UI: `useNormalizedQuery` backslash, maplibre-gl root install, job progress 100% (3 commits)
- Ephemeral browsing: race condition + event filter + sync listing (1 commit)
- Trash: fake recycle bin → `trash` crate (1 commit)
- SQLite: `busy_timeout=5000` (1 commit)
- Tests: `progress.rs` float precision (1 commit)
- Watcher: startup warning → debug (1 commit)
