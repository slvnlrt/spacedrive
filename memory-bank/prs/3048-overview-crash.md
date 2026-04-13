# PR #3048 — fix(ts-client): prevent Overview crash from single-resource cache seeding

**Status:** OPEN
**Target branch:** spacedrive-data
**Branch:** `fix/overview-crash-single-resource-seeding`
**Author:** slvnlrt
**URL:** https://github.com/spacedriveapp/spacedrive/pull/3048
**Commits:** 1
**Fix size:** 3 lines

---

## Symptom

`TypeError: Cannot read properties of undefined (reading 'total_capacity')` when navigating to the Overview page.

The error occurs because the Overview component reads `.statistics` (or `.total_capacity`) on an object that has the wrong shape — a list-shaped `{ files: [...], total_count, has_more }` instead of the expected resource object.

---

## Root Cause Analysis

Race condition in `updateSingleResource` (`useNormalizedQuery.ts:530`).

The buffer replay from the daemon delivers a `ResourceChanged` event for `library` (from `recalculate_statistics()`) before the `queryFn` has resolved. When `oldData` is `null` (query hasn't returned yet), `updateSingleResource` creates a fallback object:

```js
{ files: [resource], total_count: 1, has_more: false }
```

This list-shaped object is incorrect for non-list queries (like `libraries.info`) which expect the resource directly. The Overview component reads `.statistics` on this list-shaped object and crashes.

### Impact

The old fallback `{ files: [...] }` was only correct for file listing queries. All other query types (Library, Device[], Location, tags) received an incorrect shape when their `ResourceChanged` event arrived before the initial `queryFn` resolved.

`updateBatchResources` retains its seeding behavior because batch events only concern file listings.

### Bug origin

Introduced by commit `243ef8264` (James Pine, cherry-picked onto `windows-local-fixes`). This commit is on `spacedrive-data` but not yet on `upstream/main`.

---

## Fix

`updateSingleResource` returns `undefined` (TanStack no-op) when `oldData` is null, instead of seeding with a list-only shape. The `queryFn` delivers the correct data a few milliseconds later. Subsequent events are applied normally (since `oldData` exists by then).

**File:** `packages/ts-client/src/hooks/useNormalizedQuery.ts:530-533`

**Change:** Return `undefined` instead of constructing `{ files: [resource], total_count, has_more }` when `oldData === null`.

---

## Note

This bug is NOT on `upstream/main` yet. It will appear when `spacedrive-data` merges to main.
