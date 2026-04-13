# Spacebot Windows Port (2026-04-11)

## PR and commits

**Draft PR:** https://github.com/spacedriveapp/spacebot/pull/558
**Target branch:** `spaceui` (against PR #555)
**Source:** `slvnlrt:spaceui`

| Commit     | Message                                                                 |
| ---------- | ----------------------------------------------------------------------- |
| `1268b940` | add Windows platform support for server                                 |
| `449ca5be` | add Windows desktop build support                                       |
| `ff0b2330` | fix sidecar scope: use shell:allow-spawn instead of shell:allow-execute |
| `c9f7156e` | auto-copy sidecar binary in dev mode on Windows                         |
| `f79c6c14` | capture and display sidecar stderr on crash                             |

PR #555 review (by vsumner) was checked: none of our bugs were identified. The review focused on React/TypeScript patterns and Rust backend correctness, with no platform testing or Tauri integration review.

## Bugs fixed (server-side)

### 1. Build failure: `daemonize` crate is Unix-only

- `Cargo.toml`: moved `daemonize` and `libc` from `[dependencies]` to `[target.'cfg(unix)'.dependencies]`
- Added `windows-sys = { version = "0.59", features = ["Win32_Storage_FileSystem"] }` to `[target.'cfg(windows)'.dependencies]`

### 2. Build failure: `daemon.rs` uses Unix-only APIs

- `src/daemon.rs`: split into `unix_impl` and `windows_impl` modules with `#[cfg]` guards
- Unix module preserves all original functionality
- Windows module: Ctrl+C shutdown handler, stubs for IPC/PID check (TODO: named pipes)

### 3. Build failure: `ssh.rs` uses Unix permissions

- `src/api/ssh.rs`: wrapped `set_permissions` in `#[cfg(unix)]` / `#[cfg(not(unix))]` (no-op on non-unix)

### 4. Build failure: `system.rs` uses `libc::statvfs`

- `src/api/system.rs`: added `#[cfg(windows)]` implementation using `GetDiskFreeSpaceExW`
- Real implementation, not a stub

### 5. Server doesn't start in foreground on Windows

- `src/main.rs`: added `let foreground = foreground || cfg!(windows);` to force foreground mode

### 6. Build target directory on SSD

- `.cargo/config.toml`: added `target-dir = "F:/spacebot-target"` (shared by all crates in repo)

### 7. Missing `icon.ico` for Windows desktop build

- `desktop/src-tauri/icons/icon.ico`: generated via `bunx tauri icon` from Spacebot-iOS-Default asset
- Without this file, the Tauri Windows build fails: `error RC2175: resource file icon.ico is not in 3.00 format`
- Note: the upstream repo has NO icon.ico at all (macOS only had icon.icns)

## Bugs fixed (desktop app)

### 8. Sidecar "Start Local Server" scope error

- **Symptom:** "Scoped command binaries/spacebot not found" when clicking "Start Local Server"
- **Root cause:** `desktop/src-tauri/capabilities/default.json` had the sidecar scope under `shell:allow-execute`, but `interface/src/platform.ts:110` calls `command.spawn()` (not `command.execute()`). The scope must be under `shell:allow-spawn`.
- **Fix:** Moved sidecar scope `{"name": "binaries/spacebot", "sidecar": true, "args": ["start", "--foreground"]}` from `shell:allow-execute` to `shell:allow-spawn`. Removed bare `shell:allow-execute`.
- **Note:** Could affect all platforms, not just Windows.

### 9. Sidecar launches wrong binary (exit code 101)

- **Symptom:** After fixing the scope, clicking "Start Local Server" shows "Process exited with code 101" with stderr: `PluginInitialization("global-shortcut", "HotKey already registered")`
- **Root cause:** Binary name collision on Windows NTFS (case-insensitive filesystem).
  - Desktop `Cargo.toml` had `[[bin]] name = "Spacebot"`, producing `Spacebot.exe`
  - Sidecar is named `spacebot.exe`
  - Both crates share `target-dir = "F:/spacebot-target"` via `.cargo/config.toml`
  - `tauri-plugin-shell` resolves sidecar path by extracting the last path component from the sidecar name (`"binaries/spacebot"` -> `"spacebot"`) and joining it with `current_exe().parent()` (see `tauri-plugin-shell-2.3.3/src/process/mod.rs:118-128`)
  - On NTFS, `spacebot.exe` resolves to `Spacebot.exe` (the desktop binary, 19MB), not the server sidecar (346MB)
  - The desktop binary launches itself as sidecar with `start --foreground`, re-initializes Tauri, tries to register Alt+Space hotkey (already claimed by first instance) -> panic
- **Evidence:** MD5 hash confirmed sidecar in `desktop/src-tauri/binaries/` matches the real server binary. Python binary search confirmed `"error running Spacebot"` string found only in desktop binary, not server binary.
- **Fix:** Renamed desktop binary from `Spacebot` to `spacebot-desktop` in `desktop/src-tauri/Cargo.toml`. Deleted old `Spacebot.exe`/`.pdb`/`.d` from `F:\spacebot-target\debug\`.

### 10. Sidecar binary not found in dev mode

- **Symptom:** After renaming desktop binary, sidecar reports "file not found (os error 2)" because `spacebot.exe` doesn't exist next to the desktop exe in the target dir.
- **Root cause:** In dev mode, Tauri resolves sidecars relative to `current_exe().parent()`. The sidecar binary lives in `desktop/src-tauri/binaries/` but needs to be at `F:\spacebot-target\debug\spacebot.exe`. In production builds, the bundler handles this automatically.
- **Fix:** Added sidecar auto-copy logic to `desktop/src-tauri/build.rs`:
  - Uses `OUT_DIR` to derive the target debug/release directory
  - Copies `binaries/spacebot-{triple}.exe` to `{target_dir}/spacebot.exe`
  - Compares source/dest modification timestamps to only copy when source is newer
  - Runs on Windows only (`#[cfg(windows)]`)

### Diagnostic instrumentation added

- `interface/src/platform.ts`: Added `onStderr` handler to `BundledProcessHandlers` interface, wired `command.stderr.on("data", ...)` in `spawnBundledProcess()`
- `interface/src/components/ConnectionScreen.tsx`: Added `stderrLines` array collection (last 20 lines), displays stderr in error messages on crash, added `console.log("[sidecar stdout/stderr]", line)` for DevTools visibility

## Working state

- **Server binary:** Builds and runs correctly on Windows (`cargo build --release` from `E:\spacebot`, binary at `F:\spacebot-target\release\spacebot.exe`)
- **Web UI:** Works via `http://localhost:19898` when server is started manually
- **Desktop app:** Fully functional. "Start Local Server" sidecar works correctly after fixes #8, #9, #10.
- **Config:** `C:\Users\slvnl\.spacebot\config.toml` configured with OpenRouter, Claude Sonnet 4 channel, Haiku worker, agent `spacedrive-agent`

## Files changed (spacebot, `spaceui` branch)

### Server-side

| File                               | Change                              |
| ---------------------------------- | ----------------------------------- |
| `.cargo/config.toml`               | `target-dir = "F:/spacebot-target"` |
| `Cargo.toml`                       | Platform-conditional deps           |
| `Cargo.lock`                       | Auto-updated                        |
| `src/daemon.rs`                    | Unix/Windows platform modules       |
| `src/main.rs`                      | Force foreground on Windows         |
| `src/api/ssh.rs`                   | Platform guard on permissions       |
| `src/api/system.rs`                | Windows filesystem usage impl       |
| `desktop/src-tauri/icons/icon.ico` | New file, needed for Windows build  |

### Desktop-side

| File                                            | Change                                          |
| ----------------------------------------------- | ----------------------------------------------- |
| `desktop/src-tauri/capabilities/default.json`   | Sidecar scope moved to `shell:allow-spawn`      |
| `desktop/src-tauri/Cargo.toml`                  | Binary renamed `Spacebot` -> `spacebot-desktop` |
| `desktop/src-tauri/build.rs`                    | Auto-copy sidecar binary in dev mode            |
| `interface/src/platform.ts`                     | Added stderr handler for sidecar                |
| `interface/src/components/ConnectionScreen.tsx` | Added stderr display in crash error messages    |
