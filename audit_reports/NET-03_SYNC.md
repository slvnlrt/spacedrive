# Security Audit: Synchronization Logic

**Module:** `core/src/service/sync/`
**Protocol:** `spacedrive/sync/1`
**Date:** 2025-10-28
**Status:** 🔴 **VERIFIED CRITICAL** (Confirmed via Code Audit Dec 2025)

## Executive Summary
The synchronization logic uses a hybrid approach: "State-Based" for device-owned data and "Log-Based" (HLC) for shared data. Deep verification confirmed that the handling of device-owned data is critically flawed. It lacks origin validation, allowing any paired peer to spoof updates for any other device by simply specifying a target `device_id` in the message payload.

## Findings

### 1. Critical: Device Impersonation (Spoofing)
**Severity:** Critical
**Vulnerability Type:** Missing Authorization / Broken Access Control

#### Description
The sync protocol accepts `StateChange` messages containing a `device_id` field. Verification confirmed that `SyncProtocolHandler` trusts this field blindly and does not cross-check it against the authenticated `from_device` ID of the peer connection.

#### Vulnerable Code
*   `core/src/service/network/protocol/sync/handler.rs`:
    ```rust
    SyncMessage::StateChange { device_id, ... } => {
        // VERIFIED: Payload device_id is used directly for DB updates
        peer_sync.on_state_change_received(StateChangeMessage { device_id, ... })
    }
    ```

#### Impact
A malicious peer (Peer B) can send a sync message claiming to be the User's Laptop (Peer A). The system will accept this update and overwrite Peer A's data (e.g., file locations, index status) in the local database.

### 2. High: Time Travel Attack (Logical DoS)
**Severity:** High
**Vulnerability Type:** Improper Input Validation (Timestamp)

#### Description
The "State-Based" sync uses Last-Write-Wins (LWW) based on the timestamp provided in the message. There is no check to reject timestamps significantly in the future.

#### Vulnerable Code
*   `core/src/service/sync/peer.rs`: `apply_state_change` updates the watermark and applies the change to the DB.
*   `core/src/infra/sync/mod.rs` (implied): Likely uses `UPSERT` where `new.timestamp > old.timestamp`.

#### Impact
An attacker can spoof Peer A and send a state update with `timestamp: 9999-12-31`. This update will be accepted as "newest".
*   **Permanent Corruption:** Any subsequent *legitimate* updates from the real Peer A (with current timestamps) will be rejected as "older".
*   **Result:** The attacker can permanently lock the state of target files/locations.

### 3. High: HLC Clock Drift Injection
**Severity:** Medium/High
**Vulnerability Type:** Logic Flaw

#### Description
While `apply_shared_change` uses HLC (Hybrid Logical Clock), it updates the local `hlc_generator` with the received HLC (`self.hlc_generator.lock().await.update(entry.hlc)`).
If a peer sends a shared change with an HLC having a massive physical time component, it forces the local HLC to fast-forward.

#### Impact
This can desynchronize the logical clock from physical time, potentially causing issues with conflict resolution logic that relies on wall-clock proximity or "now" checks.

## Recommendations

1.  **Enforce Origin Validation:** In `handle_sync_message`, reject any `StateChange` where `message.device_id != from_device`. A peer should *only* be allowed to sync its own state.
2.  **Bound Timestamps:** Reject `StateChange` timestamps that are more than X minutes in the future (e.g., 5 minutes + clock skew allowance).
3.  **Bound HLC Drift:** Reject incoming HLCs where the physical component is significantly ahead of local wall-clock time.
