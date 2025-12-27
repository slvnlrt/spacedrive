# Tech Context

## Technology Stack

- **Language**: Rust (Core), TypeScript (Frontend).
- **Frontend Framework**: React, TailwindCSS, Radix UI.
- **Desktop Runtime**: Tauri (v1/v2 transition likely).
- **Database**: SQLite (SQLCipher support), SeaORM.
- **Networking**: Iroh, generic QUIC.
- **Hashing**: BLAKE3.

## Development Environment

- **Package Manager**: Bun (Frontend), Cargo (Rust).
- **Build System**: `cargo xtask` for orchestration.
- **Linting**: `cargo clippy`, `eslint`.

### Windows Specifics

- **Linkers**: Requires both MSVC (`link.exe`) and LLVM (`lld-link.exe`) in PATH.
- **MSVC Version**: 14.44.35207 (Community 2022).
- **LLVM**: Required for `lld-link.exe` as configured in `.cargo/config.toml`.

## Dependencies

- `iroh`: Core networking.
- `sea-orm`: Database interaction.
- `tauri`: Desktop application framework.
- `specta`: Type-safe Rust-to-TS export.
