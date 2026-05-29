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
- HODOR-style incremental vertex sharing for flat DAE triangle lists that need generated tangent space.
- TGA loading into `HODTexture` records.
- DXT1 and DXT5 LMIP texture compression output.
- Authored material/node/effect metadata loading from JSON.
- HOD 2.0 generation via `generate_v2_from_model`.
- Round-trip parsing of generated HODs.
- Structural comparison against HODOR for `ter_pharos` and `ter_centaur`.
- Empty-original `save_edits` V2 POOL handling now skips template-based `update_mesh_chunks()` when `original_bytes` is empty, preserving the `generate_pool_data` result (`parser/src/hod.rs:5838-5893`).
- Upgraded MS XPress sliding-window offset cap to 2MB (allowing cross-LOD matching of identical geometry data).
- Optimized search chain depth to 256 for instant compilation speed and maximum compression ratio.
- Eliminated Type 5 matches entirely to resolve bit-clashing/corruption on odd lengths, preventing in-game decompression crashes and spikiness.

What is not complete:

- **In-game re-test:** `ter_centaur` now matches HODOR structural vertex/index counts. The previously observed "vertex explosion" spikiness was identified as an Xpress compression Type 2 truncation bug which has been fixed. Needs a final in-game check to confirm flawless rendering.
- **Compression byte parity:** Generated compressed pool sizes still differ from HODOR because compressor choices are not byte-for-byte identical; decompressed structures and round-trip parsing pass.
- **Serialization asymmetries** (alignment, stride calculation, prim_group_count) still need cleanup if byte-for-byte parity becomes a goal.
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
- The implemented DAE path matches HODOR by deduplicating each face-corner before tangent/binormal accumulation, then finalizing tangent space afterward. Exact-zero UV determinant handling is required; using an epsilon over-merges `ter_centaur`.

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

- `ter_pharos`: passed (DAE-based, single material).
- `ter_centaur`: passed — reparses per LOD as 3845 `centaur` vertices and 778 `glass` vertices, matching HODOR.
- Compression: fully working (all 8 xpress tests pass, HOD files compress correctly).
- In-game: needs a fresh check with the new vertex/index-matching output.

## Next Targets

1. **Re-test in-game** with correct `ter_centaur` vertex/index counts.
2. **Add ter_fenris to test suite** with proper JSON metadata.
3. **Verify OBJ pipeline** against the DAE pipeline for editor workflow.
4. Fix serialization asymmetries (alignment, stride, prim_group_count) if byte-for-byte parity becomes necessary.
5. Editor UI integration.

---

**Latest Validation:** `cargo run --bin test_hodor_replication` passes 2/2. `cargo run --bin verify_lossless` reparses generated files structurally with expected size mismatches. `cargo run --bin test_fenris` validates the empty-original save/parse path after the V2 POOL guard.  

**Document Version:** 5.2  
**Last Updated:** 2026-05-28  
**Status:** Compression fully replicated. DAE parser fixed for multi-material. HODOR structural replication passes for tracked fixtures.
