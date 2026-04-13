# Local Branches & Repos

Updated: 2026-04-13

## Upstream Context

Jamie merged `main` into `upstream/spacedrive-data` on 2026-04-12 (commit `74cb8c46b`). `upstream/main` is fully contained in `upstream/spacedrive-data`. `spacedrive-data` = `main` + 37 exclusive commits (sources UI, sync jobs, server work, CI fixes). All PRs now target `spacedrive-data`.

## Local Branches

| Branch                                       | Ahead of upstream | Covered by PR? | Remote                        | Notes                                                                                                                                                                                       |
| -------------------------------------------- | ----------------- | -------------- | ----------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `dev` \*                                     | 26 commits        | —              | —                             | Current branch. All code cumulated: 22 tags + 1 overview crash + 1 ico/svg + 3 docs.                                                                                                        |
| `tags-and-media-fixes`                       | 22 commits        | PR #3044       | `origin/tags-and-media-fixes` | Rebased on spacedrive-data. Tags system + media context menu + FTS5 escaping + batch lookups.                                                                                               |
| `fix/overview-crash-single-resource-seeding` | 1 commit          | PR #3048       | —                             | Rebased on spacedrive-data. Overview crash fix.                                                                                                                                             |
| `fix/windows-ico-and-svg-gradients`          | 1 commit          | PR #3052       | —                             | Cherry-picked from spacedrive-data local. icon.ico + SVG gradients.                                                                                                                         |
| `spacedrive-data`                            | 0 (in sync)       | N/A            | upstream                      | Reset to `upstream/spacedrive-data` on 2026-04-13. All local commits were already in PRs.                                                                                                   |
| `security-fix`                               | 8 commits         | No PR          | `origin/security-fix`         | NET-01 vuln fix (`b7f75ba91`) + security audit reports + memory-bank docs + delete .gitmodules.                                                                                             |
| `archive/windows-local-fixes`                | 2 commits         | No (archived)  | not pushed                    | Both commits obsolete. `243ef8264` ephemeral fix integrated by Jamie upstream. `5d98d2e12` ConfirmDialog targets deleted `packages/ui/`, needs re-implementation for `@spaceui/primitives`. |
| `main`                                       | 0                 | N/A            | `origin/main`                 | In sync with upstream/main.                                                                                                                                                                 |

### windows-local-fixes Archive Detail (2026-04-13)

Both exclusive commits are obsolete:

- `243ef8264` fix(ephemeral) — Jamie integrated the same fixes upstream
- `5d98d2e12` fix(ui) dialog system — targets `packages/ui/` which no longer exists (replaced by `@spaceui/primitives`). Problem still valid (window.confirm() broken on WebView2), needs re-implementation.

## External Repos

| Repo          | Branch    | State                                                                                                                                                                                                                                                                                           |
| ------------- | --------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `E:\spacebot` | `spaceui` | 7 modified + 1 new (server Windows port only). Desktop changes reverted to Jamie's originals. Not committed yet. See `memory-bank/spacebot-windows-port.md`.                                                                                                                                    |
| `E:\spaceui`  | `main`    | Clean. Cloned, built (`bun install && bun run build && bun link` each package). No local changes. Required for spacedrive-data frontend: `@spaceui/ai`, `@spaceui/primitives`, `@spaceui/tokens`. Spacedrive's `vite.config.ts` has source aliases pointing to `../../../spaceui/packages/...`. |

## Untracked Files

- `nul` — Windows reserved device name artifact, harmless

## Stashes

None. All 6 stashes extracted (patches in `archive/`) and dropped on 2026-04-13.

## Not Yet in Any PR

1. **`security-fix`** — 8 commits:
   - `b7f75ba91` fix(security): NET-01 - Prevent arbitrary file write in P2P transfers
   - Security audit reports and docs
   - Memory bank documentation
   - Delete .gitmodules

2. **Device-level auth** — not implemented, plan in `memory-bank/plans/device_auth.md`
