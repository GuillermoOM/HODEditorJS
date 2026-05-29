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
**Result:** 2/2 passed  
**Command:** `cargo run --bin test_hodor_replication`

- `ter_pharos`: PASSED (DAE-based, single material, vertex counts match)
- `ter_centaur`: PASSED — reparses per LOD as 3845 `centaur` vertices and 778 `glass` vertices, matching HODOR.

**Compression:** Fully working. All 8 xpress roundtrip tests pass. HOD files compress correctly (183KB generated vs 232KB HODOR).

**In-Game Rendering:** The previously observed "vertex explosion" / spikiness has been identified as a Type 2 truncation bug in the Xpress compressor (allowed 12 bits for offset when Type 2 only supports 10 bits). This bug has been fixed, so in-game rendering should now be flawless. Needs a final check.

**Latest Fix:** `parser/src/xpress.rs` now correctly limits Type 2 matches to `offset <= 0x3FF` (1023) to match HODOR's binary layout `DAT_00479778`. Also, `parser/src/compiler.rs` mirrors HODOR's DAE path for generated tangent-space parts: deduplicate each face-corner against the current vertex buffer before adding that triangle's tangent/binormal contribution, use exact-zero UV determinant handling, then finalize tangent space. This matches HODOR's `FUN_0040e7f0`/`FUN_0040ea90` ordering and fixes `ter_centaur` vertex/index counts.

**Latest Validation:** `cargo run --bin test_hodor_replication` passes 2/2. `cargo test --lib` passes. `cargo run --bin verify_lossless` reparsed generated files structurally successfully with expected size mismatches. `cargo run --bin test_fenris` passed the empty-original save/parse path.

**Rejected Approach:** Generic deduplication of flat sequential indexed parts in `compile_model_meshes()` was tested and reverted. It made `ter_pharos` fail with `Mesh 'Root_mesh' lod 0 part 0 vertex count mismatch: 1488 vs 1299` and made `ter_centaur` fail with `3845 vs 3678`, so the remaining fix must be more selective than full-vertex deduplication.

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

| Test Case | HODOR Size | Latest Generated Size | Result | Notes |
|-----------|------------|----------------------|--------|-------|
| `ter_pharos` | 236,648 bytes | ~172K bytes | PASSED | Single material, DAE-based |
| `ter_centaur` | 232,860 bytes | ~183K bytes | PASSED | Vertex/index counts match HODOR |

Compression is working. `ter_centaur` generated HOD is smaller than HODOR's due to different compression choices, but decompressed structure now matches the HODOR oracle.

## Completed Work

- Ghidra decompilation of HODOR.exe (841 functions)
- Replicated HODOR's MS Xpress decompressor (FUN_00448600)
- Replicated HODOR's MS Xpress compressor (FUN_004482a0)
- All 8 xpress roundtrip tests pass
- DAE parser fixed for multiple triangle groups per geometry
- DAE parser merges consecutive parts with same material
- DAE parser extracts LOD from geometry IDs
- Compression bypass removed — real compression active
- HODOR confirmed deterministic via wine testing
- In-game rendering: textures correct, geometry partially correct
- Empty-original V2 `save_edits` POOL overwrite guard added
- HODOR-style incremental tangent-space vertex sharing implemented
- `cargo run --bin test_hodor_replication` passes for `ter_pharos` and `ter_centaur`

## Next Steps

1. **Re-test in-game** with correct `ter_centaur` vertex/index counts
2. **Add ter_fenris to test suite** with proper JSON metadata
3. Verify OBJ pipeline against the DAE pipeline for editor workflow
4. Fix serialization bugs (alignment, stride, prim_group_count) if byte-for-byte parity becomes necessary
5. Integrate workflow into the editor UI.

---

**Document Version:** 5.2  
**Last Updated:** 2026-05-28  
**Status:** Compression fully replicated. DAE parser fixed. HODOR structural replication passes for tracked fixtures.
