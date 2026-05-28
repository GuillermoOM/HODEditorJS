# HODOR Replication Test Status

## Objective

Validate that HODEditorJS can generate HOD 2.0 files from source assets plus editor-authored metadata, matching HODOR structure for the selected fixtures.

## Source Inputs

Allowed inputs used by this test:

- OBJ files for mesh geometry and `usemtl` material assignment.
- MTL files for source material and texture references.
- TGA files for source texture image data.
- Authored JSON files for editor-created metadata: materials, joints, navlights, markers, engine burns, and collision meshes.

Forbidden inputs not used by this test:

- `model.json`.
- Processed mesh payloads extracted from HODOR HOD files.
- Processed texture payloads extracted from HODOR HOD files.

HODOR HOD files are parsed only as comparison oracles.

## Current Result

**Test Cases:** `ter_pharos`, `ter_centaur`  
**Result:** 0/2 passed — LMIP format mismatch  
**Command:** `cargo run --bin test_hodor_replication`

ter_pharos fails with corrupted texture name: `'Pharos_DIFFDXT1     '` vs `'Pharos_DIFF'`.  
ter_centaur fails with texture count mismatch: 1 vs 4.

Root cause: `generate_lmip_texture_chunks_and_pool` writes LMIP chunk data that `parse_texture` cannot re-parse correctly. Likely the `original_tex_preserved` flag at `hod.rs:5081` causes original HODOR LMIP chunks to be preserved instead of using the newly generated ones.

**verify_lossless** (separate test) passes structurally for all 4 fixtures:
- `pebble_0`: byte-for-byte identical
- `ter_elysium`: size diff 67629 bytes (expected — collision mesh added, compression diff)
- `ter_fenris`: size diff 76911 bytes (expected — collision mesh added, compression diff)
- `asteroid_3`: size diff -54 bytes (expected — compression efficiency)

**Collision mesh pool appending**: Confirmed working — decomp_mesh grew from 146688 to 146816 bytes (128 bytes for 8 vertices × 16 bytes each).

## What The Test Verifies

- HODOR HOD parses successfully.
- Source asset model builds successfully.
- Generated HOD parses successfully.
- Mesh count matches HODOR.
- Material count matches HODOR.
- Texture count matches HODOR.
- Joint count matches HODOR.
- Mesh part count matches HODOR.
- Mesh part material indices match HODOR.
- Mesh part index counts match HODOR.
- OBJ `mtllib` references exist.
- MTL `newmtl` names match authored material JSON.
- MTL `map_Kd` texture references match source TGA files and material texture maps.
- Stable texture metadata comparison: name presence, dimensions, and format mismatch reporting.
- LMIP texture layout comparison: mip count, dimensions, format, and byte length.
- Generated HOD round-trips through parse/generate/parse.

## Fixture Summary

| Test Case | HODOR Size | Latest Generated Size | Meshes | Materials | Notes |
|-----------|------------|-----------------------|--------|-----------|-------|
| `ter_pharos` | 236,648 bytes | 179,110 bytes | 3 | 1 | 1 part per LOD; LMIP layout OK |
| `ter_centaur` | 232,860 bytes | 475,256 bytes | 4 | 2 | 2 parts per LOD; LMIP layout OK |

Generated file size is not expected to match HODOR yet because texture and compression behavior still differ.

Texture-format result:

- `transparent_DIFF`: HODOR emits `DXT5`, and generated output now emits `DXT5` after restoring transparent source pixels.
- Alpha-pixel detection selects DXT5 directly from the TGA source.

## Completed Work

- Built a HODOR comparison harness.
- Loaded source OBJ files instead of processed HOD mesh data.
- Loaded source TGA files instead of processed HOD texture payloads.
- Loaded authored metadata JSON for editor-created values.
- Implemented OBJ `usemtl` to HOD mesh part/material-index mapping.
- Implemented per-part OBJ vertex deduplication matching HODOR part counts for `ter_centaur`.
- Implemented OBJ/MTL/material/TGA consistency validation.
- Implemented DXT5 texture compression output path.
- Refined TGA import format detection to use actual non-opaque alpha pixels rather than alpha-channel presence alone.
- Restored `transparent_DIFF.tga` and `.TGA` to transparent source pixels and verified the format mismatch disappeared.
- Added LMIP texture layout diagnostic (`compare_texture_layouts`) reporting per-texture mip count, dimensions, format, and byte length for HODOR vs generated.
- Identified HODOR LMIP mip-count rule: stop mip chain at last level where both dimensions ≥ 8 pixels.
- Updated `parser/src/hod.rs` LMIP mip-count generation to match HODOR rule.
- After mip-count fix: LMIP layout now matches HODOR for both fixtures (mip count, dimensions, format, byte length all OK).
- Remaining gap is compressed POOL byte-size only (our Xpress compressor is more efficient than HODOR's, producing smaller compressed output for same decompressed data; this is expected behavior).

## Next Steps

1. Fix LMIP chunk data format mismatch — check `original_tex_preserved` at `hod.rs:5081-5084`.
2. Re-run `cargo run --bin test_hodor_replication` — should pass 2/2.
3. Re-run in-game validation after collision mesh pool fix.
4. Expand HODOR fixture coverage.

---

**Document Version:** 2.4  
**Last Updated:** 2026-05-28  
**Status:** LMIP format mismatch is the next blocker
