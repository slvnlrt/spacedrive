# Security Audit: Extension System

**Module:** `core/src/infra/extension/`
**Date:** 2025-10-28
**Status:** 🟢 Secure (WASM), ⚪️ Not Implemented (UI)

## Executive Summary
Spacedrive's extension system uses WebAssembly (WASM) to sandbox third-party logic. The implementation follows a "One-Door" policy where all interaction with the core system happens via a single host function `spacedrive_call`, which performs permission checks before routing to the standard internal RPC.

Crucially, the **UI integration** for extensions (HTML/JS) appears to be **unimplemented** in the current codebase. This neutralizes the high-risk "XSS via Extension" vector identified in the IPC audit.

## Findings

### 1. Robust WASM Sandbox (Secure)
**Mechanism:** `wasmer` runtime with restricted host functions.
**Observations:**
*   **Memory Isolation:** Extensions cannot access host memory outside of the specific buffers passed to host functions. `host_functions.rs` uses `wasmer::MemoryView` correctly to read inputs.
*   **Permission Enforcement:** The `host_spacedrive_call` function enforces `plugin_env.permissions.authorize(method, library_id)` *before* executing any logic.
*   **Library Scoping:** Operations are scoped to a specific `library_id` (if required by the permission model), preventing cross-library data access.

### 2. UI Integration Not Implemented (Risk Avoided)
**Mechanism:** Documentation mentions `ui_manifest.json`, but no code exists to load or render it.
**Observations:**
*   Searches for `iframe`, `webview`, or extension loading logic in the frontend returned no results.
*   **Impact:** There is currently no way for an extension to load arbitrary HTML/JS into the Spacedrive frontend. This effectively mitigates the risk of an extension exploiting the permissive `withGlobalTauri` configuration found in the IPC audit.
*   **Future Warning:** When Extension UI *is* implemented, it MUST NOT run in the main DOM. It must use a restricted `<iframe>` or separate `WebView` without access to `window.__TAURI__`.

### 3. Potential Confusion Deputy in Actions (Low Risk)
**Mechanism:** Extensions pass `SdPath` objects to Actions (e.g., `files.delete`).
**Observations:**
*   While `spacedrive_call` restricts the *Action* to the allowed Library, the *Input* (`SdPath`) is just a struct.
*   If the underlying Action implementation does not verify that the `SdPath` belongs to the Library, an extension might delete files outside its scope.
*   *Mitigation:* Most Actions seem to dispatch Jobs via the Library context, which implies implicit scoping, but this relies on the correctness of every individual Action implementation.

## Recommendations

1.  **Maintain "One-Door" Policy:** Continue forcing all extension ops through `spacedrive_call`. Do not add specialized host functions that might bypass permission checks.
2.  **Strict UI Isolation (Future):** When implementing Extension UI, use `<webview>` tags (if available in Tauri v2) or sandboxed `<iframe>` elements with `sandbox="allow-scripts"` (NO `allow-same-origin`). Ensure they cannot access `window.__TAURI__`.
3.  **Action Input Validation:** Ensure core Actions (`files.delete`, etc.) explicitly validate that provided `SdPath`s belong to the active `Library`.
