# Issues résolus — PR #3037

14 issues résolus par 29 commits sur `windows-fixes-v2`.
PR : https://github.com/spacedriveapp/spacedrive/pull/3037

---

| # | Issue | Commit(s) | Résumé |
|---|-------|-----------|--------|
| 4 | PowerShell volume detection | `perf(volume): replace PowerShell with sysinfo` | `Get-Volume` → `sysinfo::Disks` (Win32 natif) |
| 6 | "Location root path is not local" | `fix(addressing): accept device UUID as device_slug` | Frontend envoyait UUID au lieu de slug → `is_local()` false |
| 7 | OneDrive flaggé "System Directory" | `fix(locations): exact match for root paths` | `starts_with("C:\\")` → `parent().is_none()` |
| 8 | "Database Locked" on location removal | `fix(db): busy_timeout` + `fix(locations): delete_subtree_in_txn` | Connexion concurrente → même transaction |
| 9 | Volume UUID instable au restart | `fix(volume): restore UUID from database` | `tracked_volumes_map` ne stockait pas le DB UUID |
| 10 | Ephemeral browsing cassé (Windows) | `fix(ephemeral): handle Windows path separators` | 3 bugs séparateurs `b'/'` vs `b'\\'` |
| 11 | `/tmp` hardcodé | `fix(networking): use temp_dir()` | `"/tmp"` → `std::env::temp_dir()` |
| 12 | PathBuf cross-platform | `fix(networking): normalize path separators` | Sender normalise en `/`, receiver en `MAIN_SEPARATOR` |
| 13 | Hidden files Unix-only | `fix(indexing): FILE_ATTRIBUTE_HIDDEN` | 10 × `starts_with('.')` → `is_hidden_path()` helper |
| 14 | `same_physical_storage` = false (Windows) | `fix(volume): Win32 APIs for same_physical_storage` | `GetVolumeInformationW` + `GetVolumeNameForVolumeMountPointW` |

Issues supplémentaires fixés (pas numérotés individuellement, inclus dans la PR) :
- NTFS : PowerShell → hardcoded values + `canonicalize()` + dead code removal (4 commits)
- ReFS : PowerShell → `DeviceIoControl` IOCTL (1 commit)
- Device detection : wmic → registry APIs (2 commits)
- Daemon RPC : JSON-RPC 2.0 → `DaemonRequest` format + method prefix (2 commits)
- Build : MSI target exclusion pour semver pre-release (1 commit)
- UI : `useNormalizedQuery` backslash, maplibre-gl root install, job progress 100% (3 commits)
- Ephemeral browsing : race condition + event filter + sync listing (1 commit)
- Trash : fake recycle bin → `trash` crate (1 commit)
- SQLite : `busy_timeout=5000` (1 commit)
- Tests : `progress.rs` float precision (1 commit)
- Watcher : startup warning → debug (1 commit)
