> **ARCHIVED.** Historical record; not active guidance. See docs/README.md.

# STAT Count Mismatch Crash Fix

## Root Cause

The DAE parser (`dae.rs:122-129`) creates a fallback "nameplate.bmp" material for any `<triangles>`/`<polylist>` without a `material` attribute. This fires for the `COL[Root]` collision mesh geometry, which doesn't have a material attribute. Result: 3 STATs for 2 visible mesh parts → engine OOB crash.

## The Fix

**File:** `parser/src/dae.rs`

**Line 122-129:** The check `doc.descendants().any(|node| (node.has_tag_name("triangles") || node.has_tag_name("polylist")) && node.attribute("material").is_none())` scans ALL geometries including `COL[Root]`. 

**Change:** Filter out `COL[...]` geometries from this check. Only check visible mesh geometries (those starting with `MULT[...]` or without a `COL[`/`ROOT_COL` prefix).

**Alternative (simpler):** Move the fallback material creation to AFTER the COL[...] handling, so collision geometries are already skipped by the `continue` at line 456. But the current check fires on `doc.descendants()` before any geometry-specific handling, so it needs a filter.

**Simplest fix:** Change the condition to exclude nodes inside `COL[` geometries:

```rust
// Before (line 122):
if doc.descendants().any(|node| (node.has_tag_name("triangles") || node.has_tag_name("polylist")) && node.attribute("material").is_none()) {

// After:
if doc.descendants().any(|node| {
    (node.has_tag_name("triangles") || node.has_tag_name("polylist"))
        && node.attribute("material").is_none()
        && !node.ancestors().any(|a| {
            a.has_tag_name("geometry")
                && a.attribute("id")
                    .map_or(false, |id| id.starts_with("COL[") || id.starts_with("ROOT_COL"))
        })
}) {
```

Or more robustly, track which geometries are collision during parsing and skip them in the fallback check.

## Verification

After the fix:
1. Run `cargo run --bin verify_lossless`
2. Run `cargo run --bin test_hodor_replication`
3. Re-generate `ter_centaur_from_dae.hod` — should have 2 STATs, not 3
4. Test in-game — should no longer crash

## Files to Modify

- `parser/src/dae.rs` lines 122-129

## Update

- `docs/hod2-reverse-engineering/PROGRESS.md` — document the fix
