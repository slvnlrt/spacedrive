# Development Workflow

This document outlines the strategy for maintaining this fork, syncing with the official Spacedrive repository (**upstream**), and preparing contributions.

## Git Strategy

### 1. Tracking Upstream

Maintain a clean `main` branch that mirrors the official repository.

```powershell
# Setup upstream remote (once)
git remote add upstream https://github.com/spacedriveapp/spacedrive.git

# Sync main
git checkout main
git pull upstream main
```

### 2. Feature Branches

Always work in specialized branches created from a stable base (e.g., `build-fix`).

```powershell
git checkout build-fix
git checkout -b fix/db-lock
```

### 3. Syncing Your Work with Upstream

To bring new upstream features/fixes into your work:

```powershell
git checkout main
git pull upstream main
git checkout build-fix
git merge main
# Resolve conflicts if necessary
```

## Preparing Clean Pull Requests

If your working branch (like `build-fix`) has "mixed" commits containing multiple unrelated changes, use the following techniques to create "Atomic PRs" for the Spacedrive team.

### Option A: Selective Staging (`git checkout -p`)

This is the most precise method to extract specific logic from your messy branch into a clean PR branch.

1.  **Start from clean upstream**:
    ```powershell
    git checkout main
    git checkout -b pr/refs-native-api
    ```
2.  **Pull specific changes interactively**:
    ```powershell
    # Interactively select "hunks" of code from build-fix
    git checkout -p build-fix -- core/src/volume/fs/refs.rs
    ```
3.  **Commit and pushing**:
    ```powershell
    git commit -m "perf(refs): replace PowerShell with native Windows API"
    git push origin pr/refs-native-api
    ```

### Option B: Temporary Rebase/Split

If a commit is truly mixed and you want to split it:

1.  `git rebase -i <commit-before-target>^`
2.  Mark the commit as `edit`.
3.  `git reset HEAD^` (This keeps your changes but "uncommits" them).
4.  Stage specific files/lines and commit them separately.
5.  `git rebase --continue`

## Synchronization Frequency

- **Daily**: Pull `upstream main` to stay aware of core changes.
- **Before PR**: Always merge/rebase with the latest `upstream main` and run `cargo test -p sd-core --lib` to ensure no regressions were introduced by the merge.
