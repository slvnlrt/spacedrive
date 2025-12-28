# Security Audit: Peer-to-Peer File Transfer

**Module:** `core/src/service/network/protocol/file_transfer.rs`
**Protocol:** `spacedrive/filetransfer/1`
**Date:** 2025-10-28
**Status:** 🟢 **REMEDIATED** (Fixed Dec 28, 2025 - PR submitted)

## Executive Summary

The file transfer module allows Spacedrive nodes to exchange files. Deep verification confirmed that `TransferRequest` allows arbitrary file writes because it trusts the `destination_path` from remote peers. The `validate_path_access` function, while present, is ineffective as it relies on simple existence checks rather than enforcing a sandboxed root (Location).

## Findings

### 1. Critical: Arbitrary File Write (Path Traversal) via TransferRequest

**Severity:** Critical
**Vulnerability Type:** Path Traversal / Arbitrary File Write

#### Description

The `TransferRequest` message accepts a `destination_path` string from the remote peer. Verification confirmed that the receiver uses this path directly with `fs::File::create`, enabling writes to any user-writable directory.

#### Vulnerable Code

In `core/src/service/network/protocol/file_transfer.rs`:

```rust
// Message definition
FileTransferMessage::TransferRequest { destination_path, .. }

// In handler
let file_path = PathBuf::from(&session.destination_path); // Used directly
tokio::fs::create_dir_all(parent)...
tokio::fs::OpenOptions::new()...open(&file_path)...
```

#### Impact

A malicious or compromised peer can overwrite **any file writable by the user running Spacedrive**.

- **Remote Code Execution (RCE):** Overwriting `~/.ssh/authorized_keys`, `~/.zshrc`, `~/.bash_profile`, or adding scripts to startup folders.
- **System Destabilization:** Overwriting configuration files or binaries.

### 2. High: Arbitrary File Read (Path Traversal) via PullRequest

**Severity:** High
**Vulnerability Type:** Path Traversal / Arbitrary File Read

#### Description

The `PullRequest` message allows a peer to request a file from the host. The validation logic uses `canonicalize()` to resolve paths but **fails to enforce a restricted root directory**. It returns `true` (allowed) for any file that exists on the filesystem.

#### Vulnerable Code

```rust
fn validate_path_access(&self, path: &std::path::Path, ...) -> bool {
    let normalized = match path.canonicalize() { ... };
    // Only checks existence, not location!
    if !normalized.exists() ... { return false; }
    true
}
```

#### Impact

A malicious or compromised peer can read **any file readable by the user**.

- **Information Disclosure:** Stealing SSH private keys (`~/.ssh/id_rsa`), AWS credentials, the Spacedrive database, or system password hashes (if accessible).

## Risk Analysis: The "Trusted Peer" Fallacy

Spacedrive uses a pairing model where devices are "trusted" after an initial exchange. However, relying on this trust for file system access is a critical security flaw for several reasons:

1.  **Lateral Movement:** If _one_ of a user's devices is compromised (e.g., a laptop infected with malware), the attacker immediately gains RCE on _all_ other paired Spacedrive devices (desktops, servers) via the File Transfer vulnerability. The "Trust" boundary becomes a highway for malware propagation.
2.  **Zero Trust Architecture:** A secure distributed system should assume that peers, even authenticated ones, might send malformed or malicious commands. Capabilities should be least-privilege. A file transfer capability should only allow writing to a designated "Inbox" and reading from designated "Shared" folders, regardless of who is asking.
3.  **Insider Threat / Shared Devices:** If a device is temporarily shared or accessed by another user, they could leverage the active Spacedrive session to pull private files from other connected nodes without authentication.

## Recommendations

1.  **Enforce Managed Locations:** Spacedrive already has a concept of `Location` (indexed directories). File transfers should be strictly limited to these roots.
2.  **Sanitize Paths:**
    - **Push:** Force all incoming files to a specific download directory (e.g., `~/Downloads/Spacedrive/Incoming`). Ignore the sender's directory structure or sanitize it to remove `..` components.
    - **Pull:** Verify that the canonicalized requested path starts with the canonicalized path of a valid `Location`.
3.  **UI Confirmation:** Do not auto-accept transfers (even "TrustedCopy") without user configuration or explicit prompt, especially for sensitive file overwrites.

---

## Remediation (Dec 28, 2025)

### Fix Implemented

Path validation added to both `FileTransferProtocolHandler` and `FileDeleteProtocolHandler`:

- **`is_path_allowed()`:** Validates paths against registered Library Locations via `CoreContext`.
- **Canonicalization:** Resolves symlinks and `../` before comparison.
- **Fail-Safe:** Denies all access if no Locations are configured.

### Files Modified

- `core/src/service/network/protocol/file_transfer.rs`
- `core/src/service/network/protocol/file_delete.rs`
- `core/src/lib.rs` (CoreContext injection)

### Tests Added

7 security regression tests covering:

- Rejection of paths outside allowed Locations
- Acceptance of paths inside allowed Locations
- Path traversal (`../`) blocking
- Fail-safe behavior with no config

### PR Status

Submitted to upstream repository.
