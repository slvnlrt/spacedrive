# Memory Bank — Spacedrive Windows Fork

Fork de [spacedriveapp/spacedrive](https://github.com/spacedriveapp/spacedrive) (Rust/Tauri + React).
Objectif : stabiliser Windows et contribuer upstream.

Last updated: 2026-04-13

---

## Quick Reference

| Action           | Commande                              |
| ---------------- | ------------------------------------- |
| Build            | `cargo build`                         |
| Tests unitaires  | `cargo test -p sd-core --lib`         |
| Check core       | `cargo check -p sd-core`              |
| Check app        | `cargo check -p spacedrive`           |
| Run CLI          | `cargo run --bin sd-cli -- <command>` |
| Release build    | `cargo tauri build --bundles nsis`    |
| Frontend install | `bun install` (racine)                |
| Frontend dev     | `cd apps/tauri && bun run tauri:dev`  |
| Details          | `reference/build_guide.md`            |

---

## Directory Structure

```
memory-bank/
  README.md                     -- This file (index + PR history + quick reference)
  tracking/                     -- Current state (updated regularly)
    branches.md                 -- Local branches, upstream context, external repos
    open-prs.md                 -- Open PRs status
    open-issues.md              -- Active bugs needing attention
    backlog.md                  -- Known bugs, future ideas
  prs/                          -- One file per significant PR
    3037-windows-fixes.md       -- [MERGED] Windows v2 Desktop Fixes
    3044-tags-system.md         -- [OPEN] Tag system + explorer integration
    3048-overview-crash.md      -- [OPEN] Overview crash fix
    3052-ico-svg.md             -- [OPEN] Windows icon.ico + SVG fixes
  reference/                    -- Long-term docs (change rarely)
    build_guide.md              -- Windows build environment
    security_audit.md           -- Remote actions security audit
    design_decisions.md         -- Architectural choices
    spacebot-windows-port.md    -- Spacebot Windows port reference
  plans/                        -- Implementation plans
    device_auth.md              -- [TODO] Device-level authentication
    content_path_resolution.md  -- [DONE] Content path resolution (PR #3041)
  archive/                      -- Historical snapshots
```

---

## PR History

### Merged PRs

| PR                                                             | Date | Contenu                                                                           |
| -------------------------------------------------------------- | ---- | --------------------------------------------------------------------------------- |
| [#3037](https://github.com/spacedriveapp/spacedrive/pull/3037) |      | 29 commits, Windows fixes v2 (volumes, paths, ephemeral, trash, etc.)             |
| [#3040](https://github.com/spacedriveapp/spacedrive/pull/3040) |      | Dark titlebar DWM + sidebar hover (2 commits)                                     |
| [#3041](https://github.com/spacedriveapp/spacedrive/pull/3041) |      | Content path resolution, DEL keybinds, progress, useDeleteFiles hook (5 commits)  |
| [#3043](https://github.com/spacedriveapp/spacedrive/pull/3043) |      | Watcher registration + canonicalize + UNC helper + unwatch on removal (4 commits) |

### Open PRs (drafts)

| PR                                                             | Branche                                      | Statut          | Contenu                                                                       |
| -------------------------------------------------------------- | -------------------------------------------- | --------------- | ----------------------------------------------------------------------------- |
| [#3044](https://github.com/spacedriveapp/spacedrive/pull/3044) | `tags-and-media-fixes`                       | OPEN, MERGEABLE | Tags system + media context menu + FTS5 escaping + batch lookups (21 commits) |
| [#3048](https://github.com/spacedriveapp/spacedrive/pull/3048) | `fix/overview-crash-single-resource-seeding` | OPEN            | Overview crash fix (single-resource cache seeding)                            |
| [#3052](https://github.com/spacedriveapp/spacedrive/pull/3052) | `fix/windows-ico-and-svg-gradients`          | OPEN            | icon.ico placeholder fix + Obsidian SVG gradients                             |

### PRs to Prepare

| Contenu           | Notes                                           |
| ----------------- | ----------------------------------------------- |
| Device-level auth | Not implemented, plan in `plans/device_auth.md` |

---

## External Repos

| Repo     | Path          | State                                                                      |
| -------- | ------------- | -------------------------------------------------------------------------- |
| spacebot | `E:\spacebot` | 7 modified + 1 new (server Windows port). Desktop reverted. Not committed. |
| spaceui  | `E:\spaceui`  | Clean. Cloned, built, linked. No local changes.                            |

---

## Implementation Plans

| Plan                                          | Status                  | File                               |
| --------------------------------------------- | ----------------------- | ---------------------------------- |
| Content path resolution & delete fix          | DONE (PR #3041, merged) | `plans/content_path_resolution.md` |
| Device-level authorization for remote actions | TODO (draft)            | `plans/device_auth.md`             |

---

## Branches (summary)

See `tracking/branches.md` for full details. Quick overview:

| Branche                                      | Base                     | Covered by PR? | Notes                                        |
| -------------------------------------------- | ------------------------ | -------------- | -------------------------------------------- |
| `dev` \*                                     | upstream/spacedrive-data | —              | 26 commits: all code cumulated + docs        |
| `tags-and-media-fixes`                       | upstream/spacedrive-data | PR #3044       | 22 commits                                   |
| `fix/overview-crash-single-resource-seeding` | upstream/spacedrive-data | PR #3048       | 1 commit                                     |
| `fix/windows-ico-and-svg-gradients`          | upstream/spacedrive-data | PR #3052       | 1 commit                                     |
| `spacedrive-data`                            | upstream/spacedrive-data | N/A            | In sync with upstream (reset 2026-04-13)     |
| `main`                                       | upstream/main            | N/A            | In sync with upstream                        |
| `archive/windows-local-fixes`                | upstream/main            | No             | ARCHIVED — both commits obsolete             |
| `security-fix`                               | ancien                   | No             | NET-01 vuln fix + security audit docs, no PR |

---

## Upstream Context

Jamie merged `main` into `upstream/spacedrive-data` on 2026-04-12. `spacedrive-data` = `main` + 37 exclusive commits (sources UI, sync jobs, server work, CI fixes). All PRs now target `spacedrive-data`.

---

## Known Issues (no PR yet)

1. **window.confirm() broken on WebView2/Windows** — needs re-implementation targeting `@spacedrive/primitives` (spaceui repo). Original fix on archived `windows-local-fixes` branch targeted deleted `packages/ui/`.

2. **`security-fix` branch** — 8 commits (NET-01 path traversal fix, security audit reports, memory-bank docs). Not yet submitted upstream.

See `tracking/open-issues.md` and `tracking/backlog.md` for full details.
