# Tech Context

## Technology Stack
-   **Language**: Rust (Core), TypeScript (Frontend).
-   **Frontend Framework**: React, TailwindCSS, Radix UI.
-   **Desktop Runtime**: Tauri (v1/v2 transition likely).
-   **Database**: SQLite (SQLCipher support), SeaORM.
-   **Networking**: Iroh, generic QUIC.
-   **Hashing**: BLAKE3.

## Development Environment
-   **Package Manager**: Bun (Frontend), Cargo (Rust).
-   **Build System**: `cargo xtask` for orchestration.
-   **Linting**: `cargo clippy`, `eslint`.

## Dependencies
-   `iroh`: Core networking.
-   `sea-orm`: Database interaction.
-   `tauri`: Desktop application framework.
-   `specta`: Type-safe Rust-to-TS export.
