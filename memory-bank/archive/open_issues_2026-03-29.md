# Issues ouverts — Spacedrive

Tous pré-existants sur `main`, découverts pendant le travail sur `windows-fixes-v2`.
Mis a jour le 2026-03-28.

---

## P1 — Critiques (PRs séparées prévues)

### ~~Delete multi-fichier cassé sur fichiers indexés (#20+22+23)~~ ✅ RÉSOLU

**Resolu** : PR #3041 (mergee upstream)

- `resolve_in_job()` pour `SdPath::Content` implemente via queries DB (addressing.rs)
- Resolve loop ajoute dans `DeleteJob::run()` avant strategy selection
- `PathResolver::resolve()` : `unimplemented!()` remplace par erreur propre
- **Reste ouvert** : `RemoteDeleteStrategy` utilise `device_id()` deprecie -> PR separee avec device auth

### Sécurité : device-level auth manquante (#24)

PR #2944 a fixé la validation des **paths** (path traversal). Reste la validation de **qui** :

- `file_delete.rs:54` : `_from_device` ignoré
- `file_transfer.rs` : fallback `Uuid::new_v4()` (fail-open)
- `sync/handler.rs` : state changes sans validation path
- EphemeralShare : auto-accept

**Plan** : `plans/device_auth.md` | **Audit complet** : `archive/security_audit_full.md`

---

## P2 — Important

### `libraries.list` handler manquant (#15)

`validate_and_reset_library_if_needed()` → `{"Error":{"OperationFailed":"Unknown method: query:libraries.list"}}`. Handler absent côté daemon. Non bloquant (l'app fonctionne), mais les bibliothèques supprimées ne sont pas nettoyées au restart.

### ~~Tags filter sans résultats (#19)~~ ✅ RÉSOLU

**Resolu** : PR #3044 (en attente de merge, MERGEABLE)

- `find_entry_ids_for_tag()` + `resolve_tag_filter()` dans `query.rs` pour `search.files`
- `files.by_tag` retourne `Vec<File>` complets (meme format que `directory_listing`)
- Tag view integree a l'Explorer standard (mode tag dans context, `useExplorerFiles`)
- Prevention doublons tags, DEL key branchee, search bar Overview fonctionnelle

---

## P3 — Modéré

### ~~Fichiers supprimes restent visibles dans l'UI (#26)~~ RESOLU

**Cause root :** Bug upstream -- `LocationAddAction::execute()` ne registrait pas la location aupres du `FsWatcherService` apres creation.

**Fixes : PR #3043 (mergee upstream)**

1. `location/manager.rs` : `tokio::fs::canonicalize()` + `strip_windows_extended_prefix()`
2. `locations/add/action.rs` : appel `fs_watcher.watch_location(meta)` apres `add_location()`
3. `locations/remove/action.rs` : appel `watcher.unwatch_location()` symetrique
4. UNC normalization extraite dans `common::utils::strip_windows_extended_prefix()` -- DRY
5. Volume GUIDs (`\\?\Volume{...}`) preserves -- ne strip `\\?\` que sur les lettres de lecteur

### Job cancel/pause non fonctionnel (#3)

Pause : backend OK mais UI met des minutes. Cancel : "Job not found" (cherche en DB au lieu de mémoire).

### Location removal log spam (#2)

Watcher + indexer continuent après suppression DB → `"Failed to apply batch: Location not found"` en boucle.

### ~~Job progress UI (#16)~~ ✅ RÉSOLU

**Resolu** : PR #3041 (mergee upstream)

- `useJobs.ts` : progress=1.0 sur `JobCompleted`
- `DeleteJob::run()` : emet `GenericProgress` a 4 phases (Preparing, Deleting, Complete)

### ~~DEL key non branché (#21)~~ ✅ RÉSOLU

**Resolu** : PR #3041 (mergee upstream)

- `explorer.delete` (DEL / Cmd+Backspace) et `explorer.permanentDelete` (Shift+DEL) branches dans `useExplorerKeyboard.ts`
- Hook partage `useDeleteFiles` (DRY entre keybind et context menu, guard `isPending`)

### ~~Search bar cassée en Overview (#17)~~ ✅ RÉSOLU

**Resolu** : PR #3044 (en attente de merge, MERGEABLE)

- Bouton Search dans `OverviewTopBar.tsx` navigue vers `/explorer`

### ~~Navigation dossier depuis vue tag (#25)~~ RESOLU

**Resolu** : PR #3044 (en attente de merge, MERGEABLE)

- `SdPath::Physical` utilisait `"unknown-device"` comme fallback
- Fix : fallback sur `get_current_device_slug()` dans `files.by_tag` et `search.files`

### Thumbnails pas auto-générés (#18) — 🟡 PARTIELLEMENT FIXÉ

Nouveau fichier indexé par le watcher mais thumbnail generation non déclenchée. Nécessite "Regenerate thumbnails" manuel.

**Fixé (2026-03-21) :**

- Context menu : remplacé `useJobDispatch` (cassé, `jobs.dispatch` n'existe pas) par des mutations directes (`media.thumbnail.regenerate`, `media.ocr.extract`, etc.)
- MIME type : fallback par extension quand content_identity ne fournit pas le MIME
- `useJobDispatch.ts` supprimé (commit `6880ccc23`, 2026-03-25)

**Limitation connue :**

- Thumbnails sur fichiers ephemeral (non indexés) impossibles : le sidecar system requiert un `content_identity` (FK constraint). Nécessiterait soit un refactor du pipeline sidecar, soit la création d'un content_identity à la volée.

**Architecture media (clarifiée 2026-03-24) :**
Tous les media processors suivent un design intentionnel à 3 niveaux :

1. **Watcher** (automatique) : fichier modifié → processor réactif (ex: `ThumbnailProcessor`)
2. **Single-file action** (UI) : `media.thumbnail.regenerate` etc. → processor direct, retour immédiat
3. **Bulk job** : `ThumbnailJob` → batch avec phases, persistence, progress
   Le context menu utilise le (2), ce qui est le pattern prévu. Le `useJobDispatch` cassé appelait un endpoint générique `jobs.dispatch` qui n'existe pas — c'était lui le bug, pas un design inachevé.

---

## P4 — Low priority

### Indexation lente sur gros volumes (#1)

Discovery à 0% longtemps sur 200k+ fichiers. `tokio::fs::read_dir` récursif. Pistes : MFT parsing, Windows Search API, discovery parallèle.

### ~~Window decoration flicker (#5)~~ ✅ RÉSOLU

**Resolu** : PR #3040 (mergee upstream)

- `DwmSetWindowAttribute` avec `DWMWA_CAPTION_COLOR` (#1E1E1E) + `DWMWA_USE_IMMERSIVE_DARK_MODE`
- Force la barre de titre sombre independamment de la couleur d'accent Windows
- Applique sur la fenetre main (startup) et les fenetres creees dynamiquement

---

## Audit architectural (2026-03-25)

Audit complet de tous nos commits post-PR #3037, verifiant la conformite avec les patterns architecturaux du projet.

### Resultat : pas de workaround type "ephemeral"

Notre seul workaround avere (wait_for_indexing synchrone dans directory_listing.rs, PR #3037) a ete reverte par le maintainer (commit `b7bbf29db` sur `spacedrive-data`). Cherry-picke sur `windows-local-fixes`. Le streaming design est restaure avec 3 race conditions fixees a la source.

### Corrections suite a l'audit

| Fix                    | Commit      | PR    | Description                                                       |
| ---------------------- | ----------- | ----- | ----------------------------------------------------------------- |
| UNC helper DRY         | `dbbe52bb4` | #3043 | 3 duplications → `common::utils::strip_windows_extended_prefix()` |
| Volume GUID protection | `48af496c0` | #3043 | Ne strip `\\?\` que sur lettres de lecteur                        |
| Unwatch on removal     | `dbbe52bb4` | #3043 | `unwatch_location()` symétrique au `watch_location()`             |
| FTS5 escaping          | `72ce78c0e` | #3044 | Tokens wrappés en guillemets, bloque injection opérateurs         |
| Batch entry lookups    | `72ce78c0e` | #3044 | 2 branches de apply/action.rs: N queries → 1 query                |
| Dead code removal      | `6880ccc23` | #3044 | `useJobDispatch.ts` supprimé (0 imports)                          |
| Redundant imports      | `45c1141e1` | #3044 | Inline `use sea_orm::` supprimé (déjà au top-level)               |

### Points identifiés mais non corrigés (acceptables)

- **useRefetchTagQueries** : même pattern workaround que `LocationInspector` (refetch manuel). Le backend émet déjà les events corrects. Accepté comme pattern temporaire du projet.
- **Validation dans `from_input()` vs `validate()` trait method** : fonctionne mais ne supporte pas `RequiresConfirmation`. Acceptable pour les opérations tag actuelles.
- **DWM dark titlebar** : approche Win32 directe, pas de wrapper Tauri. C'est la seule option sur Windows.

### Analyse upstream : commit `b7bbf29db` (James Pine)

Le maintainer a reverté notre approche synchrone pour l'ephemeral et fixé les vrais bugs :

1. `subscriptionManager.ts` : listener pré-enregistré avant `transport.subscribe()` (events du buffer replay perdus)
2. `useNormalizedQuery.ts` : cache TanStack seedé au lieu de dropper les events si `oldData` undefined
3. `directory_listing.rs` : subdirectories sans children indexés font fallthrough vers un nouveau job

Notre workaround (polling 25ms, timeout 10s) masquait ces 3 races. Le maintainer construit Spacebot (assistant IA avec EventSource streaming) et un column view — il a besoin du pipeline event fiable.

---

## CodeRabbit review PR #3044 — etat au 2026-03-27

**Note :** PR #3044 rebased sur upstream/main apres merge de #3041 (conflit dans useFileContextMenu.ts resolu). Statut : CLEAN/MERGEABLE.

### Findings résolus

| Finding                                     | Commit      | Fix                                        |
| ------------------------------------------- | ----------- | ------------------------------------------ |
| TOCTOU race metadata/manager.rs             | `04a181535` | ON CONFLICT atomic upsert                  |
| MIN(id) → MAX(id) migration                 | `8e4a7d6bd` | Garde la row la plus récente               |
| children vs descendants                     | `04a181535` | `get_direct_children()` (depth=1)          |
| Delete cascade non atomique                 | `04a181535` | Transaction                                |
| include_children/min_confidence ignorés     | `04a181535` | Implémentés                                |
| Champs File fabricés                        | `5bf5584f8` | Skip row + warn si champs requis manquants |
| Tags entry+content non fusionnés            | `04a181535` | extend + dedup                             |
| Unapply notification content-scoped         | `04a181535` | Collecte via content_id                    |
| Version non incrémenté sur upsert           | `3d078f410` | `version + 1` dans ON CONFLICT             |
| Unapply sur-reporte entries_affected        | `3d078f410` | Skip si 0 rows deleted                     |
| Optimistic update Inspector cassé           | `d4ae1cc78` | Supprimé, refetch suffit                   |
| alert() dans TagsGroup                      | `d4ae1cc78` | Supprimé                                   |
| Pastilles tag restent après delete          | `5c4608654` | Emit "file" events avant delete            |
| Inspector stale après tag mutation          | `5c4608654` | Refetch files.by_id                        |
| tagModeActive vs mode (faux positif)        | `99c9d272a` | Revert — concepts indépendants             |
| Inline imports redondants (apply/action.rs) | `45c1141e1` | Supprimé sea_orm inline, top-level suffit  |
| FTS5 injection risk (manager.rs)            | `72ce78c0e` | Tokens wrappés en guillemets doubles       |
| Per-entry UUID lookups (apply/action.rs)    | `72ce78c0e` | Batch query via WHERE IN                   |
| Dead useJobDispatch hook                    | `6880ccc23` | Fichier supprimé                           |
| UNC strip trop agressif (volume GUIDs)      | `48af496c0` | Conditionné sur lettre de lecteur          |

### Findings ouverts — sync (PR séparée)

Le sync insert fonctionne, mais ni `tags.delete` ni `tags.unapply` ne propagent `ChangeType::Delete`. Documenté via `TODO(sync)` dans `delete/action.rs` et `unapply/action.rs`.

Lié : exposer insert vs update dans le retour de `apply_semantic_tags()` pour que les callers émettent le bon ChangeType.

### Findings ouverts — bugs réels non fixés

**~~Extension perdue sur fichiers racine (#16)~~** ✅ RÉSOLU — commit `f9d254994`

**~~Valider EntryUuid avant tagging (#4)~~** ✅ RÉSOLU — commit `f9d254994`

**~~Performance O(n²) tag merge (#NEW)~~** ✅ RÉSOLU — commit `7bf06c579`, pré-indexation HashMap

**Tag filter backend non branché dans l'UI** : `search.files` accepte un `TagFilter` côté backend (`search/query.rs`) mais aucun composant frontend ne l'utilise. Le filtre par tag dans la barre de recherche n'est pas accessible à l'utilisateur. À brancher dans l'UI search quand les filtres avancés seront implémentés.

**Tag filter + pagination FTS (#13)** : `search/query.rs` applique `LIMIT/OFFSET` avant le filtrage tags via `retain()`. Bug pré-existant dans la logique search, pas dans le code tags.

**~~`window.confirm()` supprime par WebView2~~** RESOLU — branche `windows-local-fixes` (commit `5d98d2e12`)

- WebView2 supprime silencieusement `alert()`/`confirm()`/`prompt()` sous Tauri/Windows
- Ajout `ConfirmDialog` dans `@sd/ui` : utilise `dialogManager` + `Dialog` existants, retourne `Promise<boolean>`
- Tous les `confirm()` production -> `confirmDialog()`, tous les `alert()` -> `toast.error/success/info`
- `platform.confirm()` (Tauri) mis a jour pour utiliser `confirmDialog()`
- **Note :** commit standalone local, pas encore en PR. TagsGroup.tsx et useFileContextMenu.ts exclus (dependent de PR #3044).

### Findings ouverts — mineurs

| Finding                                  | Fichier                         | Impact                                 |
| ---------------------------------------- | ------------------------------- | -------------------------------------- |
| Filtrer par media type dans context menu | `useFileContextMenu.ts:355-388` | Backend ignore les types non-supportés |
| `type="button"` manquant sur Tag remove  | `Tag.tsx:42`                    | Pas de formulaire parent en pratique   |

### Nitpicks (style/doc)

| Finding                                            | Fichier                                                |
| -------------------------------------------------- | ------------------------------------------------------ |
| DB errors silencieux dans MIME resolution          | `thumbnail/action.rs:127-149`                          |
| Import ordering                                    | `thumbnail/action.rs:3-15`                             |
| Split tag queries en modules standard              | `ancestors.rs`, `children.rs`                          |
| Rustdoc manquant                                   | `unapply/output.rs:7-10`                               |
| Cast `any`                                         | `TagSelector.tsx:44-50`, `useExplorerFiles.ts:146-150` |
| Indexer rows par content_identity_uuid avant merge | `files_by_tag.rs:321-342`                              |

---

### ~~Overview crash — `total_capacity` undefined~~ ✅ RÉSOLU

**Cause root :** Race condition dans `updateSingleResource` (`useNormalizedQuery.ts:530`). Le buffer replay du daemon livrait un `ResourceChanged` pour `library` (de `recalculate_statistics()`) avant que le `queryFn` ait résolu. Le fallback `oldData === null` créait `{ files: [Library], total_count: 1, has_more: false }`, mauvaise shape pour une query single-resource. Le composant Overview lisait `.statistics` sur cet objet list-shaped et crashait.

**Fix :** `updateSingleResource` retourne `undefined` (no-op TanStack) quand `oldData` est null, au lieu de seeder avec une shape list-only. Le `queryFn` livre les données correctes quelques ms après. Les events suivants sont appliqués normalement (oldData existe).

**Impact vérifié :** L'ancien fallback `{ files: [...] }` n'était correct que pour les file listing queries. Toutes les autres queries (Library, Device[], Location, tags) recevaient une shape incorrecte. `updateBatchResources` garde son seeding car les batch events ne concernent que les file listings.

**Fichier :** `packages/ts-client/src/hooks/useNormalizedQuery.ts:530-533`

**PR :** #3048 (contre `spacedrive-data`, OPEN). Bug introduit par commit `243ef8264` (James Pine, cherry-pick sur `windows-local-fixes`). Pas encore sur upstream/main.

---

## Issues mineurs (non numérotés)

| Issue                                | Fichier                                      | Impact                           |
| ------------------------------------ | -------------------------------------------- | -------------------------------- |
| Inode `None` dans persistent indexer | `persistent.rs:651-661`                      | Renames non détectés sur Windows |
| Tests avec paths Unix hardcodés      | `snapshot.rs:287`, `sidecar/path.rs:120,145` | Tests échouent sur Windows       |
| Copy strategy `ends_with('/')`       | `copy/strategy.rs:582`                       | Fallback mineur                  |
| SdPath `C://` parsé comme scheme     | `addressing.rs:413`                          | Théorique (`C:\` pas affecté)    |
