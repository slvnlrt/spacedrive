# Security Audit: Tauri IPC & Frontend Security

**Module:** `apps/tauri/`
**Date:** 2025-10-28
**Status:** 🟠 High Risk Configuration (Hard to Exploit)

## Executive Summary
The Spacedrive desktop application uses a highly permissive Tauri configuration that grants the frontend full control over the Core/Daemon and unrestricted access to the filesystem via the `asset:` protocol. While the frontend code (React) currently appears free of Obvious XSS vectors (due to standard React escaping), the **impact** of any future XSS vulnerability would be catastrophic (Full RCE).

## Findings

### 1. Critical Configuration: `withGlobalTauri` Enabled
**Severity:** Critical
**Vulnerability Type:** Insecure Configuration / Privilege Escalation

#### Description
`tauri.conf.json` has `"withGlobalTauri": true`. This injects the `window.__TAURI__` object into the global scope of the WebView.

#### Impact
If an attacker finds *any* Cross-Site Scripting (XSS) vulnerability in the frontend (e.g., via a malicious file name, extension, or markdown preview):
1.  They can access `window.__TAURI__.invoke`.
2.  They can call `daemon_request` (see below).
3.  They gain full control over the Spacedrive Daemon.

### 2. High: `daemon_request` Bypass
**Severity:** High
**Vulnerability Type:** Security Control Bypass

#### Description
The Rust command `daemon_request` (in `apps/tauri/src-tauri/src/main.rs`) acts as a generic proxy. It accepts a JSON payload and sends it directly to the Daemon via TCP port 6969.

#### Vulnerable Code
```rust
#[tauri::command]
async fn daemon_request(request: serde_json::Value, ...) {
    // Connects to TCP 127.0.0.1:6969
    // Writes raw JSON
}
```

#### Impact
This effectively bypasses Tauri's IPC allowlist system. Even if you disable specific Tauri commands (like `fs:write`), an attacker can simply send a JSON-RPC message to the daemon to perform the same action (e.g., `action:files.delete`) via this proxy.

### 3. High: Unrestricted `asset:` Protocol Scope
**Severity:** High
**Vulnerability Type:** Insecure Configuration / Information Disclosure

#### Description
`tauri.conf.json` enables the `asset:` protocol with scope `["$HOME/**", "/Volumes/**"]`.

#### Impact
The frontend can read **any file** on the user's system using `fetch('asset:///Users/me/.ssh/id_rsa')` or `<img src="...">`. Combined with the `fs:allow-home-read-recursive` capability, the frontend is not sandboxed from the host filesystem.

## Mitigation Factors
*   **React Safety:** A code audit of `packages/interface` found **zero** usages of `dangerouslySetInnerHTML`. Filenames are rendered using standard React bindings `{file.name}`, which auto-escapes HTML entities.
*   **Exploit Difficulty:** Currently, an attacker cannot easily inject the JavaScript required to trigger the dangerous IPC calls.

## Recommendations

1.  **Disable `withGlobalTauri`:** Set to `false`. Use the explicit `import { invoke } from '@tauri-apps/api'` syntax in the frontend, which is harder for injected scripts to access (requires module hijacking).
2.  **Restrict `asset:` Scope:** Limit the `asset:` protocol to specific "Library" folders rather than the entire `$HOME`.
3.  **Strict CSP:** Define a strict `script-src` in `tauri.conf.json` that disallows `'unsafe-inline'` (if possible) and restricts sources to `'self'`.
4.  **Remove `daemon_request` Proxy:** Instead of a generic proxy, define specific, typed Tauri commands for the actions the frontend actually needs. This allows the Tauri layer to act as a firewall/validator.
