# PR #3052 — fix(windows): correct icon.ico and SVG gradient decimal separators

**Status:** OPEN
**Target branch:** spacedrive-data
**Branch:** `fix/windows-ico-and-svg-gradients`
**Author:** slvnlrt
**URL:** https://github.com/spacedriveapp/spacedrive/pull/3052
**Commits:** 1

---

## Issue 1: Broken icon.ico placeholder

### Problem

The upstream `icon.ico` file is a 103-byte broken placeholder. This causes the Windows Resource Compiler to fail with:

```
error RC2175: resource file icon.ico is not in 3.00 format
```

This error breaks `cargo tauri build` on Windows because the Tauri build process embeds `icon.ico` into the Windows executable via the resource compiler (`rc.exe`). A valid `.ico` file must contain at least one image entry with proper ICO header structure; the 103-byte placeholder does not satisfy this.

### Fix

Replaced with a valid `icon.ico` generated from the Spacedrive logo. This is a functional placeholder that allows the build to succeed. It can be replaced with an official icon later.

---

## Issue 2: Obsidian SVG gradient decimal separators

### Problem

The Obsidian file type icon SVG uses comma decimal separators (e.g., `0,5` instead of `0.5`) in `<linearGradient>` stop-offset values. This is likely due to locale-dependent number formatting in the tool that generated the SVG.

While some SVG renderers are lenient about this, the SVG specification requires dot decimal separators. Strict parsers (and some browsers/renderers) may misinterpret `0,5` as two separate values (`0` and `5`) rather than the decimal `0.5`.

### Fix

Replaced comma decimal separators with dots in the Obsidian SVG gradient definitions.

---

## Notes

- The `icon.ico` commit was originally an orphan on the local `spacedrive-data` branch. It was cherry-picked to `fix/windows-ico-and-svg-gradients` and submitted as PR #3052.
- Both fixes are cosmetic/build-related and have no runtime behavior changes.
