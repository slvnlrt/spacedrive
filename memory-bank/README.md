# Memory Bank — Spacedrive Windows Fork

Fork de [spacedriveapp/spacedrive](https://github.com/spacedriveapp/spacedrive) (Rust/Tauri + React).
Objectif : stabiliser Windows et contribuer upstream.

## Etat actuel (2026-03-27)

### PRs mergees

| PR                                                             | Contenu                                                                           |
| -------------------------------------------------------------- | --------------------------------------------------------------------------------- |
| [#3037](https://github.com/spacedriveapp/spacedrive/pull/3037) | 29 commits, Windows fixes v2 (volumes, paths, ephemeral, trash, etc.)             |
| [#3040](https://github.com/spacedriveapp/spacedrive/pull/3040) | Dark titlebar DWM + sidebar hover (2 commits)                                     |
| [#3041](https://github.com/spacedriveapp/spacedrive/pull/3041) | Content path resolution, DEL keybinds, progress, useDeleteFiles hook (5 commits)  |
| [#3043](https://github.com/spacedriveapp/spacedrive/pull/3043) | Watcher registration + canonicalize + UNC helper + unwatch on removal (4 commits) |

### PRs ouvertes (drafts)

| PR                                                             | Branche                | Statut          | Contenu                                                                       |
| -------------------------------------------------------------- | ---------------------- | --------------- | ----------------------------------------------------------------------------- |
| [#3044](https://github.com/spacedriveapp/spacedrive/pull/3044) | `tags-and-media-fixes` | CLEAN/MERGEABLE | Tags system + media context menu + FTS5 escaping + batch lookups (21 commits) |

### PRs a preparer

| Contenu           | Notes                                            |
| ----------------- | ------------------------------------------------ |
| Device-level auth | Pas implemente, plan dans `plans/device_auth.md` |

### Branches locales

| Branche                | Base                       | Contenu                                              | Remote                      |
| ---------------------- | -------------------------- | ---------------------------------------------------- | --------------------------- |
| `main`                 | upstream/main              | Synced (`be454a0b4`)                                 | origin/main                 |
| `windows-local-fixes`  | upstream/main + 2 commits  | Ephemeral streaming cherry-pick + ConfirmDialog      | pas pousse                  |
| `tags-and-media-fixes` | upstream/main + 21 commits | PR #3044                                             | origin/tags-and-media-fixes |
| `spacedrive-data`      | upstream/spacedrive-data   | Branche upstream pour tests (UI evolutions majeures) | upstream                    |
| `security-fix`         | ancien                     | Fix NET-01 path traversal (pas upstream)             | origin/security-fix         |

### Commits inedits (non couverts par une PR)

Sur `windows-local-fixes` :

```
243ef8264 fix(ephemeral): restore streaming design, fix event delivery race conditions  <- cherry-pick upstream/spacedrive-data
5d98d2e12 fix(ui): replace broken alert/confirm with custom dialog system  <- ConfirmDialog custom
```

Le commit ephemeral vient de `upstream/spacedrive-data` (commit `b7bbf29db` par James Pine). Il sera integre a main quand `spacedrive-data` sera merge upstream.

Le commit ConfirmDialog est un fix standalone pour `window.confirm()` supprime par WebView2 sous Tauri/Windows.

---

## Structure

```
memory-bank/
+-- README.md                  <- ce fichier (index + etat PRs + branches)
+-- build_guide.md             -- environnement, commandes build/test/release
+-- open_issues.md             -- issues actionnables (P1->P4), tous pre-existants upstream
|
+-- plans/                     -- plans d'implementation pour futures PRs
|   +-- content_path_resolution.md  -- implemente (PR #3041, mergee)
|   +-- device_auth.md              -- device-level auth dans les handlers remote
|
+-- archive/                   -- reference historique
    +-- pr_3037.md             -- body de la PR soumise (mergee)
    +-- pr_3037_draft.md       -- draft original de la PR #3037
    +-- resolved_issues.md     -- 14 issues resolus par PR #3037
    +-- design_decisions.md    -- choix architecturaux (ce qu'on n'a pas porte, et pourquoi)
    +-- security_audit_full.md -- audit complet des handlers remote (PR #2944 + gaps restants)
```

## Quick reference

| Action          | Commande                           |
| --------------- | ---------------------------------- |
| Tests unitaires | `cargo test -p sd-core --lib`      |
| Check core      | `cargo check -p sd-core`           |
| Check app       | `cargo check -p spacedrive`        |
| Release build   | `cargo tauri build --bundles nsis` |
| Frontend        | `bun install` (racine)             |
| Details         | `build_guide.md`                   |
