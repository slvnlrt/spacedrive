# Active Context

## Current Focus

**Windows Stability & Test Suite Repair Complete.**
The project is now in a stable state on Windows, with all critical volume detection, file operation, and UI synchronization bugs resolved. The test suite is passing (321 tests).

## Recent Changes

- **Jan 20, 2026**: Completed major Windows stability overhaul.
- **Unit Tests**: Fixed all breakages in `sd-core` tests (`manager.rs`, `progress.rs`, `volume_fingerprint`).
- **Performance**: Removed PowerShell dependency for ReFS checks in favor of native Win32 API.
- **Bug Fixes**: resolved Volume duplication, UI latency on file operations, and ephemeral job event propagation.

## Next Steps

1.  **Address Database Lock (Issue #8)**: Implement retry logic for location removal.
2.  **Fix False Positive Paths (Issue #7)**: Allow standard OneDrive/User folders in onboarding.
3.  **Resume Security Roadmap**: Now that the build is stable, proceed with `IPC-01` (Daemon IPC security) and `NET-03` (P2P Handshake).

## Active Decisions

- **Deterministic UUIDs**: Volumes now use v5 UUIDs derived from hardware/FS fingerprints to ensure persistence across restarts.
- **Native IOCTLs**: We prefer `windows-sys` and direct IOCTL calls over spawning shell processes for performance and stability.
- **Memory Bank**: Documentation for this session is stored in `memory-bank/` including `windows_fixes_pr.md` and `known_issues.md`.
