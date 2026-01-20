# Progress Status

## Completed Features (Windows Stability Session)

- [x] **Volume Detection**: Prefix stripping for `\\?\` paths on Windows.
- [x] **ReFS Support**: High-performance block cloning check via native API (no PowerShell).
- [x] **Volume Persistence**: Deterministic UUIDs to prevent duplication.
- [x] **Trash Support**: Native Windows recycle bin integration.
- [x] **Unit Tests**: Full pass on `sd-core` test suite (321 tests).
- [x] **UI Synchronization**: Fixed event propagation for ephemeral jobs (indexing).

## Pending Issues (See `known_issues.md`)

- [ ] **Issue #8**: Database locked error on location removal.
- [ ] **Issue #7**: OneDrive folders flagged as system directories.
- [ ] **Issue #3**: UI Latency on Job Pause/Cancel.

## Roadmap Status

- **Foundation**: Stable ✅
- **Testing**: Passing ✅
- **Security**: Pending (Next Focus)
