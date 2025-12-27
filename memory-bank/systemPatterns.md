## System Patterns

### Architecture
Spacedrive uses a **Daemon-Client** architecture:
-   **Core (`sd-daemon`)**: Rust-based service handling VDFS, Database (SQLite), P2P (Iroh), and File Operations.
-   **Clients**: Tauri (Desktop), Mobile, CLI communicate with `sd-daemon` via RPC/IPC.
-   **Frontend**: React (TypeScript) running in WebView, communicating with Daemon via Tauri IPC or direct TCP/WS.

### Key Technical Decisions
-   **VDFS (Virtual Distributed Filesystem)**: Abstraction layer over physical filesystems. Index-based truth.
-   **Iroh**: Networking stack for QUIC-based P2P connections and NAT traversal.
    -   **Discovery**: mDNS (local), Pkarr/DHT (global/internet).
    -   **Transport**: QUIC with TLS 1.3.
-   **SeaORM**: Async ORM for type-safe SQLite interactions.
-   **CQRS**: Command Query Responsibility Segregation for separating read/write logic.
    -   **Commands**: Preview -> Commit -> Verify lifecycle.
-   **EventBus**: Centralized event system for internal communication.

### Code Structure
-   `core/`: Rust backend implementation.
    -   `domain/`: Types and business logic.
    -   `service/`: Functional services (Network, Sync, Import).
    -   `infra/`: Database, storage, and networking adapters.
    -   `ops/`: CQRS handlers.
-   `apps/tauri/`: Frontend application (React/TypeScript).
-   `packages/interface/`: Shared UI components and logic.

### Security Architecture
-   **Sandboxing**: Extensions run in WASM modules.
-   **Encryption**: P2P traffic encrypted via TLS 1.3 (QUIC) + Session Keys (Application layer).
-   **Identity**: Device-based identity keys (Ed25519).
-   **Pairing**: BIP39-based handshake (Challenge-Response).
    -   *Audit Note*: Current implementation might have TOFU/Auth Bypass issues (NET-02).
-   **IPC**: Tauri configuration currently permissive (`asset:` scope `$HOME`, `withGlobalTauri: true`).
