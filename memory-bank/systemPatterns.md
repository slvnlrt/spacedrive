## System Patterns

### Architecture

Spacedrive uses a **Daemon-Client** architecture:

- **Core (`sd-daemon`)**: Rust-based service handling VDFS, Database (SQLite), P2P (Iroh), and File Operations.
- **Clients**: Tauri (Desktop), Mobile, CLI communicate with `sd-daemon` via RPC/IPC.
- **Frontend**: React (TypeScript) running in WebView, communicating with Daemon via Tauri IPC or direct TCP/WS.

### Key Technical Decisions

- **VDFS (Virtual Distributed Filesystem)**: Abstraction layer over physical filesystems. Index-based truth.
- **Iroh**: Networking stack for QUIC-based P2P connections and NAT traversal.
  - **Discovery**: mDNS (local), Pkarr/DHT (global/internet).
  - **Transport**: QUIC with TLS 1.3.
- **SeaORM**: Async ORM for type-safe SQLite interactions.
- **CQRS**: Command Query Responsibility Segregation for separating read/write logic.
  - **Commands**: Preview -> Commit -> Verify lifecycle.
- **EventBus**: Centralized event system for internal communication.

### Code Structure

- `core/`: Rust backend implementation.
  - `domain/`: Types and business logic.
  - `service/`: Functional services (Network, Sync, Import).
  - `infra/`: Database, storage, and networking adapters.
  - `ops/`: CQRS handlers.
- `apps/tauri/`: Frontend application (React/TypeScript).
- `packages/interface/`: Shared UI components and logic.

### Security Architecture

- **Sandboxing**: Extensions run in WASM modules.
- **Encryption**: P2P traffic encrypted via TLS 1.3 (QUIC) + Session Keys (Application layer).
- **Identity**: Device-based identity keys (Ed25519).
- **Pairing**: BIP39-based handshake (Challenge-Response).
  - _Audit Note_: NET-02 dismissed - cryptographic binding confirmed.
- **IPC**: Tauri configuration currently permissive (`asset:` scope `$HOME`, `withGlobalTauri: true`).

---

## Deep Architecture Insights (Dec 2025)

### File Operations Routing

```
FileCopyJob → CopyStrategyRouter.select_strategy()
                    ↓
        Compare device_slugs (source vs destination)
                    ↓
    Same slug     → Local strategies (LocalMove, FastCopy, LocalStream)
    Different slug → RemoteTransferStrategy → FileTransferProtocolHandler
```

**Key files:**

- `core/src/ops/files/copy/routing.rs` - Strategy selection logic
- `core/src/ops/files/copy/strategy.rs` - All strategy implementations

### Ephemeral Browsing

- **Local only** - uses `EphemeralIndexCache` (in-memory)
- Never routed via P2P protocols
- Purpose: browse non-indexed folders without adding to Locations

### CoreContext Pattern

Central service accessor for runtime context:

```rust
ctx.library_manager.blocking_read() → list() → library
LocationManager::new(ctx.events.clone()) → list_locations(&library)
```

### SdPath Resolution

- `SdPath` = universal addressing (Physical, Cloud, Content, Sidecar)
- Resolution to local `PathBuf` happens **before** network protocols
- P2P handlers work with resolved `PathBuf`, not `SdPath`

### Protocol Handler Pattern

All P2P protocols implement `ProtocolHandler` trait:

- `protocol_name()` - identifier string
- `handle_stream()` - bidirectional stream handling
- `handle_request()` / `handle_response()` - RPC-style

### Key Files for Future Fixes

| Domain            | Path                                                 |
| ----------------- | ---------------------------------------------------- |
| Copy routing      | `core/src/ops/files/copy/routing.rs`                 |
| Copy strategies   | `core/src/ops/files/copy/strategy.rs`                |
| P2P File Transfer | `core/src/service/network/protocol/file_transfer.rs` |
| P2P File Delete   | `core/src/service/network/protocol/file_delete.rs`   |
| P2P Sync          | `core/src/service/network/protocol/sync/`            |
| SdPath addressing | `core/src/domain/addressing.rs`                      |
| Tauri IPC         | `apps/tauri/src-tauri/`                              |

### Rust Patterns Used

- `blocking_read()` for async→sync context
- `futures::executor::block_on()` for calling async from sync (use sparingly)
- `Arc<RwLock<Vec<T>>>` for thread-safe mutable config

---

## Repository Documentation Map

### Root Level Files

| File              | Purpose                                                                                            |
| ----------------- | -------------------------------------------------------------------------------------------------- |
| `AGENTS.md`       | **Development guide** - workflows, architecture, common mistakes, coding standards. **READ FIRST** |
| `TODO`            | **Dev roadmap** - Todo+ format checklist for v2.0.0-pre.1, shows current priorities                |
| `README.md`       | Project overview and setup instructions                                                            |
| `SECURITY.md`     | Security policy and vulnerability reporting                                                        |
| `CONTRIBUTING.md` | Contribution guidelines                                                                            |
| `DOCKER.md`       | Docker setup instructions                                                                          |

### Security Audit Files (Our Work)

| File                 | Purpose                                                  |
| -------------------- | -------------------------------------------------------- |
| `SECURITY_AUDIT.md`  | **Master audit tracker** - status of all vulnerabilities |
| `AUDIT_BRIEFING.md`  | High-level summary of audit scope and findings           |
| `audit_reports/*.md` | Detailed technical reports per vulnerability             |

### Memory Bank (Session Persistence)

| File                            | Purpose                            |
| ------------------------------- | ---------------------------------- |
| `memory-bank/activeContext.md`  | Current focus and recent changes   |
| `memory-bank/progress.md`       | Overall audit/remediation progress |
| `memory-bank/systemPatterns.md` | Architecture insights (this file)  |
| `memory-bank/techContext.md`    | Tech stack and environment setup   |
| `memory-bank/tasks.md`          | Windows build/test commands        |
| `memory-bank/projectbrief.md`   | Audit objectives                   |
| `memory-bank/productContext.md` | Product context for Spacedrive     |

### Crate-Level Documentation

| Path                   | Purpose                                           |
| ---------------------- | ------------------------------------------------- |
| `core/AGENTS.md`       | Core-specific development guide (similar to root) |
| `core/README.md`       | Core crate overview                               |
| `xtask/README.md`      | Build orchestration tool docs                     |
| `extensions/README.md` | Extension system docs                             |
| `docs/README.md`       | General documentation index                       |

### Implementation Notes

| File                                   | Purpose                                    |
| -------------------------------------- | ------------------------------------------ |
| `FILE_OPENING_IMPLEMENTATION_NOTES.md` | Cross-platform file opening implementation |
| `TABS-IMPLEMENTATION.md`               | Tab system implementation notes            |

---

## Whitepaper Insights (v2 Architecture)

_Source: `whitepaper/spacedrive.pdf`_

### Core Concepts

| Concept              | Description                                                                      |
| -------------------- | -------------------------------------------------------------------------------- |
| **VDFS**             | Virtual Distributed File System - unified virtual layer over all storage sources |
| **Entry Model**      | Every file/folder = Entry with UUID, persistent across renames/moves             |
| **SdPath**           | URI scheme `sd://...` with 3 modes: Physical, Cloud, Content-Aware               |
| **Closure Tables**   | Hierarchical indexing with O(1) lookups for millions of entries                  |
| **Virtual Sidecars** | Derivative data (thumbnails, OCR) stored in `.sdlibrary`, linked by Entry ID     |

### Security Model

| Aspect                    | Implementation                                                         |
| ------------------------- | ---------------------------------------------------------------------- |
| **Cryptographic Pairing** | Ed25519 signatures + BIP39 mnemonic codes                              |
| **E2E Encryption**        | P2P with end-to-end encryption (QUIC/TLS 1.3)                          |
| **WASM Sandbox**          | Extensions with granular permissions (e.g., read-only per folder)      |
| **Two-Tier Hashing**      | BLAKE3 sampling (fast ID) + full hash (integrity)                      |
| **Spacedrop**             | ECDH key exchange for perfect forward secrecy between unpaired devices |

### Sync Protocol (Relevant for NET-03)

- **Leaderless Hybrid Model** - no distributed consensus (Raft/Paxos)
- **Device-Authoritative Data** - each device has sole authority over its local index
- **Shared Metadata** - HLC (Hybrid Logical Clock) ordered logs for deterministic merge
- **Libraries = Trust Boundaries** - Locations can be scoped to specific devices

### Networking Stack

- **Iroh** - Single QUIC endpoint per device
- **Protocol Multiplexing** - sync, transfer, pairing on same connection
- **Spacedrop** - ephemeral sharing between unpaired devices with PFS

### Storage Tiering

- **PhysicalClass** - hardware reality (SSD, HDD, Cloud Archive)
- **LogicalClass** - user intent (Hot, Cold storage)
- **Transactional Actions** - "preview-then-commit" workflow before any file operation

---

## Lessons Learned (NET-01 Fix - Dec 2025)

### ⚠️ Rust Async/Sync Patterns

**NEVER hold a sync `RwLockReadGuard` across `.await` points.**

```rust
// ❌ WRONG - Guard held across async call
let guard = lock.blocking_read();
some_async_fn().await; // Future becomes !Send

// ✅ CORRECT - Clone data, drop guard before await
let data = {
    let guard = lock.read().await;
    guard.clone()
}; // Guard dropped here
some_async_fn(data).await;
```

**Why it matters:**

- Sync guards are not `Send`
- Async futures must be `Send` to work across threads
- Compiler error: `future cannot be sent between threads safely`

**Fix patterns:**

1. Make the method fully async with `.read().await`
2. Clone data and drop guard before any async call
3. Restructure to avoid holding locks during async operations

### ⚠️ Testing Requirements

**ALWAYS run full compilation and tests before submitting:**

```powershell
# Minimum verification before any PR
cargo check -p sd-core           # Compile check
cargo test -p sd-core            # All tests
cargo test -p sd-core --test <integration_test>  # Integration tests
```

**Why unit tests may pass but integration fails:**

- Unit tests may not exercise all code paths (e.g., `if let Some(ctx) = ...`)
- Integration tests use real `CoreContext` and reveal async issues
- The compiler only complains when the problematic code is actually called from async

### ⚠️ Code Review Checklist

Before submitting security fixes:

1. [ ] **Compile check passed** (`cargo check`)
2. [ ] **All unit tests pass** (`cargo test`)
3. [ ] **Integration tests pass** (find them with `find . -name "*_test*"`)
4. [ ] **No sync guards held across await** (grep for `blocking_read`, `write().unwrap()`)
5. [ ] **All entry points covered** (grep for message handlers, not just one)
6. [ ] **No `futures::executor::block_on`** in async context
