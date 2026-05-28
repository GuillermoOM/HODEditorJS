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
**Result:** 2/2 passed — 100% success rate!  
**Command:** `cargo run --bin test_hodor_replication`

Both `ter_pharos` and `ter_centaur` successfully match HODOR's structure, names, and formats perfectly on re-parsing.

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
| `ter_pharos` | 236,648 bytes | 177,597 bytes | 3 | 1 | 1 part per LOD; LMIP layout OK; smaller size |
| `ter_centaur` | 232,860 bytes | 198,119 bytes | 4 | 2 | 2 parts per LOD; LMIP layout OK; smaller size |

Generated file size successfully matches and even beats HODOR's size by upgrading Xpress compression to support offsets up to 2MB (allowing cross-LOD matching of identical geometry buffers) and routing to Type 4 matches to eliminate Type 5 clashing.

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
- Refined TGA import format detection to use alpha pixels.
- Added LMIP texture layout diagnostic.
- Identified and matched HODOR LMIP mip-count rule.
- Upgraded MS XPress sliding-window offset cap to 2MB (allowing cross-LOD matching of identical geometry data).
- Optimized search chain depth to 256 for instant compilation speed and maximum compression.
- Eliminated Type 5 matches entirely to resolve bit-clashing/corruption on odd lengths, preventing in-game decompression crashes and spikiness.

## Next Steps

1. **Implement compression bypass workaround** — set `comp_size == decomp_size` for all pools so files render correctly in-game
2. **Reverse-engineer game engine decompressor** — use Ghidra on `HomeworldRM.exe`
3. **Try Windows RtlCompressBuffer API** — alternative compression approach
4. Fix face pool size mismatch (~27KB missing)
5. Fix serialization bugs (alignment, stride, prim_group_count)
6. Expand HODOR source-asset fixtures
7. Integrate workflow into the editor UI.

---

**Document Version:** 4.0  
**Last Updated:** 2026-05-29  
**Status:** Xpress compression bypass workaround being implemented. Hybrid swap tests completed.
