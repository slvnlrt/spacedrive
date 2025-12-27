# Security Audit: Database & Data Storage

**Module:** `core/src/infra/db/`
**Date:** 2025-10-28
5: **Status:** 🔴 **VERIFIED AT-RISK** (Confirmed Dec 2025)

## Executive Summary
The database layer uses `sea-orm` over SQLite. The audit focused on two critical areas: At-Rest Encryption and SQL Injection vulnerability. While the application follows best practices for query construction (preventing SQLi), it completely lacks encryption for the database file, leaving sensitive metadata exposed if the host file system is compromised.

## Findings

### 1. Critical: Missing At-Rest Encryption
**Severity:** Critical (Contextual)
**Vulnerability Type:** Missing Encryption of Sensitive Data

#### Description
The application initializes SQLite databases using the standard `sqlite://` scheme without any encryption extensions (like SQLCipher).

#### Vulnerable Code
*   `core/src/infra/db/mod.rs`:
    ```rust
    let db_url = format!("sqlite://{}?mode=rwc", path.display());
    // ...
    let conn = SeaDatabase::connect(opt).await?;
    // PRAGMAs are set (WAL, etc.), but no 'PRAGMA key' or sqlcipher setup
    ```

#### Impact
If an attacker gains access to the user's filesystem (e.g., via the IPC-01 vulnerability or NET-01 Path Traversal), they can simply copy the `.db` file and open it with any SQLite viewer.
*   **Data Exposed:** File metadata, file paths, thumbnails, tags, user notes, and potentially cached cloud credentials (if stored in DB).
*   *Note:* This confirms the open task `SEC-001`.

### 2. Robust SQL Injection Protection (Secure)
**Severity:** None (Positive Finding)
**Status:** Verified

#### Description
The codebase primarily uses `sea-orm` which handles parameter binding automatically. Where raw SQL is required (e.g., Full Text Search), the code correctly uses parameterized queries.

#### Observation
*   `core/src/ops/search/query.rs` manually constructs SQL for FTS5 but uses `?` placeholders and `Statement::from_sql_and_values` to bind user input.
*   FTS query strings are also manually escaped (`replace('"', r#"\""#)`) to prevent logic errors in the search syntax, adding a second layer of safety.

## Recommendations

1.  **Implement SQLCipher:** Switch from `sqlite` to `sqlcipher` (or `sea-orm` with `sqlcipher` backend feature).
    *   Derive the database key from the user's master key (managed by `KeyManager`).
    *   Ensure the key is never stored on disk in plaintext.
2.  **Audit Future Raw Queries:** While current usage is safe, ensure any future "performance optimizations" involving `execute_unprepared` are strictly reviewed to forbid variable interpolation.
