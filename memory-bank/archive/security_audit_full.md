# Security Audit: Remote Actions & Library Boundary Enforcement

**Date**: 2026-03-15
**Scope**: All network protocol handlers in `core/src/service/network/protocol/`
**Context**: User concern about lateral movement if a Spacedrive node is compromised

---

## Principle: Library = Security Boundary

**Proposed rule**: Only files within registered library locations should be accessible to remote operations. Ephemeral (non-library) file access should be local-only.

**Current state**: Partially implemented, with critical gaps.

---

## What Works (Positive Findings)

### Path validation via canonicalization (PR #2944 by slvnlrt, merged Dec 2025)
- `file_delete.rs:165-192` — `is_path_allowed()` uses `path.canonicalize()` to resolve symlinks and `..` sequences, then checks against allowed paths
- `file_transfer.rs:404-459` — Same pattern, robust against path traversal attacks
- Allowed paths = registered library locations queried dynamically from `LocationManager` via `CoreContext`
- **Fail-safe**: if no context or no paths configured → all access denied (line 178)
- **4 regression tests** included (lines 337-406)

### Ed25519 pairing
- `pairing/mod.rs` — Device pairing uses cryptographic signatures
- Establishes trust at connection time via ALPN routing

### Library location whitelisting
- Both delete and transfer handlers query `LocationManager::list_locations()` to build the allowed path set
- Paths outside these locations are rejected
- This effectively makes library = security boundary for file operations (path level)

---

## Remaining Vulnerabilities (not covered by PR #2944)

PR #2944 fixed **path-level** authorization (WHERE operations can target). What remains is **device-level** authorization (WHO can request operations). These are complementary: even with path validation, an untrusted device shouldn't be able to delete files in your library.

### 1. Delete handler ignores device identity (HIGH)

**File**: `core/src/service/network/protocol/file_delete.rs`
- **Line 54**: `_from_device: Uuid` — prefixed with `_`, intentionally unused
- **Line 273**: `let device_id = Uuid::nil()` — placeholder, always nil

**Impact**: Any device on the network can send delete commands for files **within library locations**. Path traversal is blocked (PR #2944), but an unpaired device could still delete authorized files.

**Fix**: Verify `from_device` is in `DeviceState::Paired` before processing. Reject unknown devices.

### 2. File transfer falls back to random UUID (CRITICAL)

**File**: `core/src/service/network/protocol/file_transfer.rs`
- **Lines 1529-1546**: If device registry lookup fails → `Uuid::new_v4()` (random)
- **Lines 1543-1545**: If device registry unavailable → same random fallback

**Impact**: If registry is corrupted or temporarily unavailable, any connection gets accepted with a random device ID. No fail-safe.

**Fix**: Fail-closed: reject transfer if device cannot be verified. Never assign random UUID.

### 3. Sync protocol could bypass path validation (HIGH)

**File**: `core/src/service/network/protocol/sync/handler.rs`
- **Lines 110-120**: `on_state_change_received(change)` applies DB changes directly

**Impact**: A compromised node could sync location records pointing to system directories (e.g., `C:\Windows\System32`). Once in the DB as a "location", file operations against that path would pass `is_path_allowed()` validation.

**Fix**: Validate that synced location paths don't include system directories. Apply the same `is_system_directory()` check used in location creation.

### 4. EphemeralShare auto-accept (MEDIUM)

**File**: `core/src/service/network/protocol/file_transfer.rs`
- **Lines 698-702**: `TransferMode::EphemeralShare { .. } => true` — auto-accepts without UI prompt

**Impact**: Unwanted files can be pushed to a device without user consent.

**Fix**: Implement UI consent prompt via Tauri event. Queue transfers pending approval.

### 5. No per-handler device re-verification (HIGH)

**File**: `core/src/service/network/core/event_loop.rs`
- **Lines 228-242**: Pairing is checked at connection time only

**Impact**: If a device is un-paired after connection, in-flight messages are still processed. Also, the pairing check result is not passed down to handlers.

**Fix**: Pass trust level to each handler. Handlers should reject operations from untrusted devices.

---

## Recommendations (Priority Order)

### P0: Fail-closed on unknown devices
- Delete handler: verify `from_device` is paired
- Transfer handler: reject if device not in registry (no random UUID fallback)

### P1: Sync path validation
- Add `is_system_directory()` check when syncing location records
- Reject location paths that point to OS-critical directories

### P2: Per-handler trust propagation
- Pass `DeviceState` to protocol handlers
- Each handler checks required trust level (Paired for delete/transfer, any for discovery)

### P3: EphemeralShare consent
- Implement UI prompt for incoming ephemeral transfers
- Queue pending transfers until user accepts/rejects

### P4: Library-only remote operations
- Enforce that remote operations can ONLY target files within the requesting library
- Cross-library operations should require explicit grant

---

## Relationship to Content Path Resolution

The security boundary enforcement is **orthogonal** to the Content path resolution fix. However:

- Once Content paths are resolved to Physical paths in the delete job, the resolved paths will pass through `is_path_allowed()` validation
- This is correct behavior: the resolved Physical path must be within a library location
- Content paths that resolve to paths outside library locations would be rejected (defense in depth)

---

## Files Reference

| File | Lines | Issue |
|------|-------|-------|
| `service/network/protocol/file_delete.rs` | 54, 273 | Device identity ignored |
| `service/network/protocol/file_transfer.rs` | 1529-1546, 698-702 | Random UUID fallback, auto-accept |
| `service/network/protocol/sync/handler.rs` | 110-120 | No path validation on sync |
| `service/network/core/event_loop.rs` | 228-242 | Trust check not propagated |
