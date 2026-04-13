# Décisions de design — windows-fixes-v2

## Ce qu'on a délibérément PAS porté de `build-fix`

### `SPACEDRIVE_VOLUME_NAMESPACE` + `to_deterministic_uuid()`

Non re-appliqué.

La méthode `to_deterministic_uuid()` (UUID v5 basé sur le fingerprint) était définie mais **jamais appelée** dans la construction de volumes. `volume/manager.rs` utilisait toujours `Uuid::new_v4()`. La méthode existait dans le code mais n'était pas branchée dans le flux réel.

Le fix réel (commit `fix(volume): restore UUID from database at startup`) restaure le UUID de la DB au restart, ce qui est plus fiable qu'un UUID déterministe car il respecte l'UUID historique déjà en base.

### Context menus (Quick/Full Rescan, Remove Location)

Non re-appliqué. Feature UI utile mais pas un fix de stabilité. Peut être ajouté dans une PR séparée.

## Approche architecturale : pas de résolution frontend

On a explicitement revert un fix qui résolvait Content → Physical côté frontend (`useExplorerKeyboard.ts`, `useFileContextMenu.ts`). Raison : c'est une incohérence architecturale — le backend (CopyJob) fait déjà cette résolution, le DeleteJob devrait faire pareil. Résoudre côté frontend contourne le problème au lieu de le résoudre correctement, et ne marcherait pas pour les fichiers distants.

→ La bonne solution est d'implémenter `resolve_in_job()` côté backend. Voir `plans/content_path_resolution.md`.
