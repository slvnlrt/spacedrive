# Spacedrive Development Tasks (Windows)

## Environment Setup

These commands must be run in your PowerShell session before building the core or the desktop app.

```powershell
# Complete PATH setup (MSVC + LLVM + Native DLLs)
$env:PATH = "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC\14.44.35207\bin\Hostx64\x64;C:\Program Files\LLVM\bin;E:\spacedrive\apps\.deps\bin;E:\spacedrive\apps\.deps\lib;" + $env:PATH

# 2. Verify linkers are found
where lld-link.exe
where link.exe
```

## Building and Running

### Desktop App (Dev Mode)

Requires Bun to be installed.

```powershell
cd apps/tauri
bun run tauri:dev
```

### Setup & Native Dependencies

If you need to re-run the initialization and native dependency downloads.

```powershell
cargo run -p xtask -- setup
```

## Testing

### Core Tests

Run these from the project root.

```powershell
# All core tests
cargo test -p sd-core

# Specific security tests for NET-01
cargo test -p sd-core is_path_allowed -- --nocapture
```
