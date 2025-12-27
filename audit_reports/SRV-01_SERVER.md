# Security Audit: Server RPC Exposure

**Module:** `apps/server/src/main.rs`
**Protocol:** `HTTP /rpc`
**Date:** 2025-12-27
**Status:** 🔴 **CRITICAL** (Discovered during Verification Audit)

## Executive Summary
The standalone `spacedrive-server` component exposes the full daemon RPC interface via an HTTP POST endpoint (`/rpc`). Deep audit revealed that authentication can be completely disabled via the `SD_AUTH=disabled` environment variable. This allows unauthenticated Remote Code Execution (RCE) via the same vector as IPC-01, but reachable over the network.

## Findings

### 1. Critical: Unauthenticated RPC Proxy
**Severity:** Critical
**Vulnerability Type:** Broken Authentication / Remote Code Execution

#### Description
The server component provides a `/rpc` endpoint that proxies JSON-RPC requests directly to the Spacedrive daemon's TCP socket.

#### Vulnerable Code
In `apps/server/src/main.rs`:
```rust
// auth can be disabled
if input == "disabled" {
    return (HashMap::new(), true); // true = disabled
}

// /rpc endpoint
let app = Router::new()
    .route("/rpc", post(daemon_rpc))
    .layer(middleware::from_fn_with_state(state.clone(), basic_auth));
```

#### Impact
If auth is disabled (or misconfigured), any network-adjacent attacker can:
1.  Send a POST request to `/rpc`.
2.  Execute *any* daemon command (e.g., `action:files.delete`, `action:system.execute`).
3.  Gain full control over the Spacedrive instance and the host filesystem.

## Recommendations

1.  **Remove "disabled" option:** Do not allow disabling authentication in production builds.
2.  **Enforce Safe Defaults:** If `SD_AUTH` is not provided, generate a random strong password and print it to the console (standard practice for headless servers).
3.  **Localhost Binding:** Ensure the server binds to `127.0.0.1` by default unless explicitly overridden by the user.
4.  **CORS Policy:** Implement a strict CORS policy to prevent browser-based attacks against the RPC endpoint.
