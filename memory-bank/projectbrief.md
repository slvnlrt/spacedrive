# Project Brief

## Vision
Spacedrive is a distributed file manager designed to connect devices and files across a peer-to-peer network. This specific fork has been created to conduct a security audit of the core architecture, specifically Rust Core (`sd-daemon`), Tauri Interface, and P2P networking in Spacedrive v2.

## Objectives
1.  **Architecture Discovery**: Understand Daemon-Client model, Iroh-based P2P, and VDFS.
2.  **Vulnerability Identification**: Identify "Real and Exploitable" vulnerabilities (RCE, Auth Bypass, etc.).
3.  **Documentation**: Establish a formal audit trail.

## Scope
-   **Network & Trust**: File Transfer, Pairing, Sync protocols.
-   **Interface & IPC**: Tauri Configuration, Frontend Logic.
-   **Core Security**: Extension Sandbox, Database.

## Success Criteria
-   Verification of identified vulnerabilities (NET-01, NET-02, NET-03, IPC-01).
-   Documentation of findings in `verification_report.md`.
-   Integration of security context into the Memory Bank.
