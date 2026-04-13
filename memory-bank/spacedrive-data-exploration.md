# spacedrive-data Branch Exploration Log

## Date: 2026-03-29

## Branch State

`spacedrive-data` local branch: 3 commits ahead of `upstream/spacedrive-data`

### Our cherry-picks (rebased on upstream):

1. `7035abd95` - fix(ts-client): prevent Overview crash from single-resource cache seeding (PR #3048)
2. `8c3f6b409` - fix(tags): prevent duplicate tag applications on the same file
3. `315a7d5bc` - fix(migration): keep newest row (MAX id) when deduplicating tag applications

### Dropped commit:

- `6de94af1d` (chore: comment out WIP data module) - no longer needed, Jamie added the module in `907ab4e26`

## Jamie's New Commits (fetched 2026-03-29)

### `9f0685abc` - ui

- Updated React types from `~19.1.10` to `19.2.14`
- Updated `@types/react-dom` to `19.2.3`
- Removed `npm:types-react@rc` alias pattern
- Updated `bun.lockb`
- Spacebot UI changes (ChatComposer, ConversationScreen, TasksRoute)

### `907ab4e26` - Add archive sources UI with full Rust integration

- **Added `core/src/data/manager.rs`** (219 lines) - SourceManager implementation
- **Added `core/src/data/mod.rs`** - data module entry point
- Added `core/src/ops/sources/` - CRUD operations for sources (create, delete, get, list_items, sync)
- Added `core/src/ops/adapters/` - adapter operations (config, list, update)
- Added frontend components: SourceCard, SourceDataRow, SourcePathBar, SourceStatusBadge, SourceTypeIcon
- Added Sources sidebar group and routes
- Updated `packages/ts-client/src/generated/types.ts` with new types
- 43 files changed, +2737/-218 lines

## Backend Compilation

Backend (`sd-daemon`) compiles successfully in ~11m 29s. Only warnings from `sd-archive` (unused imports).

## Frontend Status: BLOCKED

The frontend cannot run on `spacedrive-data`. Three packages are referenced in `packages/interface/package.json` as `link:` dependencies but don't exist in the repo:

```json
"@spaceui/ai": "link:@spaceui/ai",
"@spaceui/primitives": "link:@spaceui/primitives",
"@spaceui/tokens": "link:@spaceui/tokens"
```

These replaced the old `@sd/ui` workspace package. `packages/ui/` directory only contains a stale `tsbuildinfo` file. Jamie likely has these packages locally or in a private registry.

## Conclusion

- Backend is fully functional on `spacedrive-data`
- Frontend requires Jamie to publish or include the `@spaceui/*` packages
- Our fixes (overview crash, tag dedup) are cleanly rebased on top of upstream
