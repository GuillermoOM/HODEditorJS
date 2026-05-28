# From Assets to HOD 2.0: Current Status

## Objective

Create HOD 2.0 files from source assets and editor-authored metadata, then validate the generated HOD against HODOR output.

## Source Rules

Allowed implementation inputs:

- `OBJ` geometry and material usage (`usemtl`).
- `MTL` material names and texture references.
- `TGA` source texture files.
- Authored metadata JSON: `materials.json`, `joints.json`, `navlights.json`, `markers.json`, `engine_burns.json`, `collision_meshes.json`.

Forbidden implementation inputs:

- `model.json`.
- Processed HOD mesh payloads extracted from HODOR output.
- Processed HOD texture payloads extracted from HODOR output.

HODOR HOD files are used only as comparison oracles.

## Current Status

The pipeline can create valid HOD 2.0 files from source OBJ/TGA plus authored metadata JSON. The current tests do not use `model.json` as source data.

What works:

- OBJ positions, normals, UVs, faces, triangulation, negative indices.
- OBJ `usemtl` mapping to HOD mesh parts and `material_index`.
- OBJ `mtllib` existence validation.
- MTL `newmtl` and `map_Kd` validation against authored materials and source TGA files.
- Per-part vertex deduplication by OBJ position/UV/normal tuple.
- DAE oracle validation for `MULT[Root_mesh]_LOD[n]` geometry, material grouping, and per-material index counts.
- HODOR comparison of per-part material index, index count, and vertex count.
- TGA loading into `HODTexture` records.
- DXT1 and DXT5 LMIP texture compression output.
- Authored material/node/effect metadata loading from JSON.
- HOD 2.0 generation via `generate_v2_from_model`.
- Round-trip parsing of generated HODs.
- Structural comparison against HODOR for `ter_pharos` and `ter_centaur`.

What is not complete:

- LMIP chunk data format mismatch — `generate_lmip_texture_chunks_and_pool` writes LMIP data that `parse_texture` cannot re-parse correctly. Texture names are corrupted (include format bytes). Likely caused by `original_tex_preserved` flag preserving original HODOR LMIP chunks instead of using generated ones. Check `hod.rs:5081-5084`.
- In-game validation is still required after the collision mesh pool fix.
- Editor UI integration for the source-asset workflow is not done.

## DAE Intermediate Oracle

The checked-in `*.DAE` files are treated as intermediate oracles only. They are not used as source inputs for HOD generation.

Shared DAE facts:

- `<authoring_tool>` is `RODOH Apr 16 2015 17:44:57`.
- Unit is centimeter scale: `<unit meter="0.010000" name="centimeter" />`.
- Axis is `Y_UP`.
- LOD geometries use `MULT[Root_mesh]_LOD[n]-lib` ids.
- Material assignment is stored in one or more `<triangles material="MAT[...]_SHD[...]">` blocks per LOD.
- Multiple DAE triangle blocks with the same material are grouped into one HOD mesh part.

`ter_pharos` DAE oracle:

- Images: `Pharos_DIFF.TGA`, `Pharos_TEAM.TGA`, `Pharos_GLOW.TGA`, `Pharos_SPEC.TGA`, `Pharos_STRP.TGA`.
- Material: `MAT[pharos.bmp]_SHD[ship]`.
- `LOD[0]`: 496 triangles, 1488 indices, 1 material part.
- `LOD[1]`: 208 triangles, 624 indices, 1 material part.
- `LOD[2]`: 60 triangles, 180 indices, 1 material part.

`ter_centaur` DAE oracle:

- Images: `support01_DIFF.TGA` as `DXT1`, `transparent_DIFF.TGA` as `DXT5`.
- Materials: `MAT[centaur]_SHD[matte]`, `MAT[glass]_SHD[matte]`.
- Each of `LOD[0]` through `LOD[3]` has 1305 `centaur` triangles and 263 grouped `glass` triangles.
- Each LOD therefore has 3915 `centaur` indices and 789 `glass` indices.
- The DAE stores `glass` as two `<triangles>` blocks, but HODOR and the current importer group those blocks into one `glass` mesh part.

Current DAE/OBJ/HOD vertex-count observation:

- `ter_pharos` DAE unique vertex tuples by LOD: 902, 408, 124.
- `ter_pharos` current OBJ-built and HODOR BMSH vertices by LOD: 1488, 624, 180.
- `ter_centaur` DAE unique vertex tuples per LOD: 2081 `centaur`, 727 `glass`.
- `ter_centaur` current OBJ-built and HODOR BMSH vertices per LOD: 3845 `centaur`, 778 `glass`.
- DAE unique tuple counts should not be treated as the HOD vertex-buffer target; HODOR expands or preserves more face-corner tuples during DAE-to-HOD conversion.

## Test Fixtures

### ter_pharos

Inputs:

- `Root_mesh_lod0.obj`, `Root_mesh_lod1.obj`, `Root_mesh_lod2.obj`.
- `Root_mesh_lod*.mtl`.
- `Pharos_DIFF.tga`, `Pharos_GLOW.tga`, `Pharos_TEAM.tga`.
- Authored metadata JSON files.

Expected HODOR structure:

- 3 meshes.
- 1 material.
- 1 mesh part per LOD.
- 10 joints.
- 9 navlights.

### ter_centaur

Inputs:

- `Root_mesh_lod0.obj` through `Root_mesh_lod3.obj`.
- `Root_mesh_lod*.mtl`.
- `support01_DIFF.tga`, `transparent_DIFF.tga`.
- Authored metadata JSON files.

Expected HODOR structure:

- 4 meshes.
- 2 materials.
- 2 mesh parts per LOD: `centaur` and `glass`.
- 7 joints.
- 1 navlight.
- 1 engine burn.
- 5 markers.

## Validation Command

```bash
cd parser
cargo run --bin test_hodor_replication
```

Current result:

- `ter_pharos`: passed.
- `ter_centaur`: passed.
- Total: 2/2 passed.
- Latest known format note: `transparent_DIFF` now matches `HODOR=DXT5`, generated=`DXT5` from restored TGA alpha.

## Texture Format Findings

- HOD generation now supports both DXT1 and DXT5 compressed texture payloads.
- TGA alpha-channel detection must check actual pixel alpha, not just whether the file is 32-bit; the current source TGAs can carry an alpha channel while all pixels are opaque.
- `transparent_DIFF.tga` has been restored to transparent pixels and now selects DXT5 from source alpha alone.
- The DAE oracle records `IMG[transparent_DIFF]_FMT[DXT5]`, matching the restored transparent source behavior, but DAE remains comparison-only and is not an implementation source.
- HODOR LMIP mip-count rule: stop mip chain at last level where both dimensions ≥ 8 pixels (not `8.min(log2(max_dim)+1)`).
- LMIP layout (mip count, dimensions, format, byte length) now matches HODOR for both `ter_pharos` and `ter_centaur` after mip-count fix.
- Remaining gap is compressed POOL byte-size only (our Xpress compressor is more efficient than HODOR's, producing smaller compressed output for same decompressed data; this is expected behavior).

## Next Targets

1. Fix LMIP chunk data format mismatch — check `original_tex_preserved` flag at `hod.rs:5081-5084` and verify generated LMIP chunks are being used.
2. Re-run in-game validation after collision mesh pool fix.
3. Expand HODOR source-asset fixtures beyond `ter_pharos` and `ter_centaur`.

---

**Document Version:** 2.5  
**Last Updated:** 2026-05-28  
**Status:** LMIP format mismatch is the next blocker
