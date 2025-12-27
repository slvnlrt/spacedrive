# Security Audit: Pairing & Authentication

**Module:** `core/src/service/network/protocol/pairing/`
**Protocol:** `spacedrive/pairing/1`
**Date:** 2025-10-28
**Status:** 🔴 Critical Vulnerabilities Identified

## Executive Summary
The Pairing Protocol is responsible for establishing trust between two Spacedrive devices using a BIP39 "Pairing Code". The audit revealed that the handshake protocol does **not** cryptographically verify that the peer knows the Pairing Code. It relies on a "Trust On First Use" (TOFU) model where the simple act of completing the handshake grants "Paired" status.

While file content transfer uses keys derived from the Pairing Code (offering some protection), the **Control Plane (Messaging/RPC)** does not. This allows an attacker to pair without the code and issue privileged commands.

## Findings

### 1. Critical: Weak Pairing Handshake (Authentication Bypass)
**Severity:** Critical
**Vulnerability Type:** Broken Authentication / MITM

#### Description
The pairing handshake consists of an exchange of a Challenge and a Signature.
1.  Initiator sends random `Challenge`.
2.  Joiner signs `Challenge` with *Joiner's Ephemeral Private Key*.
3.  Initiator verifies signature against *Joiner's Public Key* (sent in the request).

**The Flaw:** The handshake proves the Joiner owns the key they just generated. It does **not** prove the Joiner knows the shared `PairingCode`.

#### Vulnerable Code
*   `core/src/service/network/protocol/pairing/security.rs`: `verify_challenge_response` only checks Ed25519 signatures.
*   `core/src/service/network/protocol/pairing/initiator.rs`:
    ```rust
    // Verifies signature
    let signature_valid = PairingSecurity::verify_challenge_response(&device_public_key, &challenge, &response)?;
    // If valid, completes pairing!
    registry.complete_pairing(...)
    ```

#### Impact
An attacker can connect to a Spacedrive instance in "Pairing Mode", send a pairing request with their own key, sign the challenge, and successfully become a "Paired Device" in the victim's registry.

### 2. High: Lack of Authentication on Control Plane
**Severity:** High
**Vulnerability Type:** Missing Authorization

#### Description
The `messaging` protocol (used for Library operations like `RegisterDevice`, `CreateSharedLibrary`) relies solely on the Iroh `NodeId` identity. It checks if the `NodeId` exists in the `DeviceRegistry`. It does **not** use the `SessionKeys` (derived from the Pairing Code) to encrypt or sign these messages.

#### Vulnerable Code
*   `core/src/service/network/protocol/messaging.rs`:
    ```rust
    // Wrapped in Message enum
    let envelope = Message::Library(message);
    // Sent directly to stream (JSON serialization) without application-layer signature
    serde_json::to_vec(&envelope)...
    ```

#### Impact
Combined with Finding #1, an attacker who has "Paired" (without the code) can issue RPC commands. They can:
*   Register new devices.
*   Query library state (`LibraryStateRequest`).
*   Potentially trigger other logic flaws.
*   *Note:* They cannot decrypt File Transfer *content* (as that uses `SessionKeys`), but they can manipulate metadata and topology.

## Recommendations

### 1. Implement PAKE (Password Authenticated Key Exchange)
Replace the current "Sign Challenge" flow with a proper PAKE (e.g., SPAKE2+ or CPace) using the `PairingCode` as the password. This ensures the handshake *cannot* complete unless both parties know the code.

### 2. Authenticate Control Plane
Require all sensitive `messaging` protocol commands to be signed with the `SessionKeys` (HMAC or Ed25519 signature derived from session secret). Reject commands from "Paired" devices that fail this signature check.

### 3. Verify Session Keys on Connect
Upon reconnection, perform a quick challenge-response using the `SessionKeys` to prove the peer still holds the correct shared secret before allowing any operations.
