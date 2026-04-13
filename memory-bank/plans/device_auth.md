# Plan : Device-Level Authorization for Remote Actions

**Status** : Draft — PR séparée à proposer upstream
**Audit complet** : `../archive/security_audit_full.md`
**Contexte** : PR #2944 (slvnlrt, déc 2025) a fixé la validation path-level. Ce plan couvre le device-level.

---

## Principe

Library = security boundary. Les remote operations ne doivent être possibles que par des devices **paired** sur des fichiers **dans des library locations**.

## Vulnérabilités identifiées (par priorité)

### P0 : Fail-closed on unknown devices

**file_delete.rs:54** — `_from_device: Uuid` ignoré (prefixé `_`)
- Fix : vérifier `from_device` est dans `DeviceState::Paired` avant traitement
- Rejeter les devices inconnus

**file_transfer.rs:1529-1546** — Fallback `Uuid::new_v4()` si device registry lookup échoue
- Fix : rejeter le transfer, pas de random UUID. Fail-closed.

### P1 : Sync path validation

**sync/handler.rs:110-120** — `on_state_change_received(change)` applique les changes DB directement
- Fix : valider que les locations syncées ne pointent pas vers des répertoires système
- Appliquer le même `is_system_directory()` check que lors de la création de location

### P2 : Per-handler trust propagation

**event_loop.rs:228-242** — Pairing vérifié à la connexion uniquement, pas propagé aux handlers
- Fix : passer `DeviceState` à chaque handler
- Chaque handler vérifie le trust level requis (Paired pour delete/transfer, any pour discovery)

### P3 : EphemeralShare consent

**file_transfer.rs:698-702** — `TransferMode::EphemeralShare { .. } => true` (auto-accept)
- Fix : UI consent prompt via Tauri event, queue pending transfers

---

## Fichiers à modifier

| Fichier | Changement |
|---------|------------|
| `protocol/file_delete.rs` | Vérifier `from_device` paired |
| `protocol/file_transfer.rs` | Supprimer random UUID fallback, ajouter device check |
| `protocol/sync/handler.rs` | Valider paths dans les state changes |
| `core/event_loop.rs` | Propager trust level aux handlers |

## Risques

- Breaking change si des workflows existants dépendent du comportement permissif actuel
- Nécessite de comprendre le flow complet de pairing (ALPN routing, `PairingManager`)
- Le sync protocol est complexe — validation path pourrait avoir des effets de bord sur le sync initial
