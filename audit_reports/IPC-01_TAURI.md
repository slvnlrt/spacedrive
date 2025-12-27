# Security Audit: Tauri IPC & Frontend Security

**Module:** `apps/tauri/`
**Date:** 2025-10-28
**Status:** 🔴 **VERIFIED CRITICAL** (Confirmed via Code Audit Dec 2025)

## Executive Summary
The Spacedrive desktop application uses a highly permissive Tauri configuration. Verification confirmed that `daemon_request` acts as a raw command proxy, and `withGlobalTauri: true` exposes it to any XSS. Any frontend vulnerability (e.g. malicious metadata) leads directly to Full RCE.

## Findings

### 1. Critical Configuration: `withGlobalTauri` Enabled
**Severity:** Critical
**Vulnerability Type:** Insecure Configuration / Privilege Escalation

#### Description
`tauri.conf.json` has `"withGlobalTauri": true`. Verification confirmed this translates to `window.__TAURI__.invoke` being accessible from any script.

### 2. Critical: `daemon_request` Raw Proxy
**Severity:** Critical
**Vulnerability Type:** Security Control Bypass / RCE

#### Description
The Rust command `daemon_request` (in `apps/tauri/src-tauri/src/main.rs`) is a generic proxy that sends raw JSON to the daemon's TCP port. It bypasses Tauri's command allowlist.

#### Impact
**Confirmed RCE.** An attacker can send any daemon command (e.g., `system.*`, `fs.*`) via this proxy.

### 3. High: Unrestricted `asset:` Protocol Scope
**Severity:** High
**Vulnerability Type:** Insecure Configuration / Information Disclosure

#### Description
`tauri.conf.json` enables the `asset:` protocol with broad scope `["$HOME/**", "/Volumes/**"]`, allowing the frontend to read any user file.

## Remediation Strategy (Revised)

Initial recommendation was to remove `daemon_request`. However, verification of `packages/ts-client/src/transport.ts` (TauriTransport) showed that the **entire application relies on this command** for communication. Removal would break the UI completely.

### Recommended Revised Strategy:
1.  **Strict IPC Allowlist:** In `daemon_request` (Rust), parse the JSON request and validate the `method` against an allowlist of permitted daemon commands. Block dangerous categories (e.g., `debug.*`, `admin.*`).
2.  **Harden Config:** Set `withGlobalTauri: false` and migrate to explicit module imports.
3.  **Restrict Asset Scope:** Limit `asset:` protocol to specific library folders only.
