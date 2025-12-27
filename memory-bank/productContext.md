# Product Context

## Problem
Computing has shifted to cloud-centric silos, sacrificing data ownership for multi-device sync. Users own powerful devices but are forced to use central servers to move files between them.

## Solution
Spacedrive is a distributed file manager built on a Virtual Distributed Filesystem (VDFS). It enables:
-   **Local-First Sync**: Peer-to-peer file synchronization without central servers.
-   **Content Identity**: Files are identified by content (CAS), not path, allowing deduplication and effective backup.
-   **Multi-Device Index**: A unified view of all files across all devices.

## User Experience Options
-   **Desktop App**: Main interface (Tauri/React) for file management.
-   **Mobile App**: (Planned) iOS/Android access.
-   **CLI**: Headless server management and automation.

## Core Capabilities
-   No internet required for local network sync.
-   End-to-End Encryption (E2EE) for all transfers.
-   WASM-based extension system for safe community plugins.
