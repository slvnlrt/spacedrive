# Sync & Stability Session Handover - Jan 2026

**Session Goal:** Resolve critical Windows stability issues, fix build/test errors, and improve volume detection (ReFS/NTFS).

## Key Achievements

- **Volume Detection**: Fixed "No volume found" on Windows by correctly stripping `\\?\` prefixes.
- **Data Integrity**: Implemented deterministic volume UUIDs (v5) based on hardware/FS fingerprints to prevent duplicate volumes on restart.
- **Performance**: Replaced slow PowerShell spawns with native Windows API/IOCTLs for ReFS block cloning checks.
- **UI Responsiveness**: Fixed logic where ephemeral jobs (indexing) were not emitting completion events to the UI.
- **Test Suite**: Fixed 320+ unit tests, including migrating `VolumeFingerprint` tests and fixing `progress.rs` float precision issues.
- **Native Trash**: Integrated `trash` crate for reliable file deletion on Windows.

## Important Documents

- **[windows_fixes_pr.md](./windows_fixes_pr.md)**: Detailed breakdown of every fix and commit.
- **[known_issues.md](./known_issues.md)**: Documented remaining issues to tackle (DB locks, OneDrive system path false positives).
- **[activeContext.md](./activeContext.md)**: Current state of the project and immediate next steps.

## Immediate Action Items

1.  **Issue #8 (Database Locked)**: Implement retry logic for `SQLITE_BUSY` on location removal.
2.  **Issue #7 (OneDrive)**: Whitelist common user paths to prevent "System Directory" warnings.
3.  **Security Review**: Resume work on `IPC-01` and `NET-03` now that the base system is stable.

## How to Resume

1.  Read `activeContext.md` to get centered.
2.  Run `cargo test -p sd-core --lib` to ensure baseline stability.
3.  Pick an item from `known_issues.md` or continue with the Security roadmap.
