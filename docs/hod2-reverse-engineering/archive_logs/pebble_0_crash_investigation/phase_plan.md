# Pebble_0 Follow-Up Phase Plan

## Current User Report

- Newly-created `pebble_0.hod` now crashes the game with an access violation and no useful game error log.
- User wants original vs generated pebble files compared for HOD spec compliance.
- User observed original HOD LODs are represented as repeated mesh entries with the same mesh name and different `lod`, not separate differently named mesh nodes.
- User wants UI editing to support mesh-per-LOD in the inspector and serialize the same-name/different-LOD structure correctly.
- User suspects texture rendering may need to explicitly use decompressed textures and Blender-like texture settings: Linear Interpolation, Flat projection, Repeat Extension, Single Image Source, sRGB Color Space, Straight Alpha.

## Latest User Feedback (Post-Phase 2)

- **Viewport textures working:** Viewport now renders textures added correctly.
- **File size bloat:** Generated `pebble_0.hod` is larger than original `1.6 MiB`. Root cause: generated textures use uncompressed RGBA instead of DXT1.
- **Game still crashing:** Access violation persists. Most likely cause is texture format mismatch (DXT1 expected, RGBA provided) or missing KDOP/INFO chunks.

## Phase 1: Structural Compare First [DONE]

Goal: compare original `pebble_0.hod` against the newly-created/generated `pebble_0.hod` before making more changes.

Checks:
- Top-level IFF chunk list, order, sizes, and chunk types.
- `POOL` stream compressed/decompressed lengths for texture, mesh, and face pools.
- `HVMD/MULT` entries: name, parent, LOD child count, child chunk IDs, BMSH part counts, vertex masks, vertex/index counts, mesh/face offsets.
- `DTRM` children: especially `HIER`, `KDOP`, and any missing original children.
- Materials/textures: `LMIP`, `STAT`, texture names, dimensions, formats, texture pool offsets.

## Phase 2: Crash Hypothesis [DONE]

Only after Phase 1, identify the most likely access violation cause. High-risk candidates:
- Generated `MULT` structure differs from original enough that engine LOD lookup dereferences invalid data.
- LODs are split into separate mesh nodes or names instead of same-name/different-LOD variants.
- Material texture references point to missing/mismatched texture slots or pool offsets.
- Generated chunk order or preserved/regenerated children differ from engine expectations.

### Phase 2 Active Focus: Shader Parameter Semantics [DONE]

The current leading crash hypothesis is that generated `STAT` texture parameter names are texture filename keys (`pebble_diff`, `pebble_glow`, `pebble_norm`) instead of shader parameter semantics expected by the HWRM `ship` shader (`$diffuse`, `$glow`, `$normal`).

Before changing serialization, inspect how the app loads shader pipeline definitions from `/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/uncompressed_bigs/HWRM/`, then make generated `STAT` output choose semantic parameter names from shader/config knowledge where possible. For pebble reconstruction, the output should match original `pebblemat` texture bindings:
- texture index `0` -> `$diffuse`
- texture index `1` -> `$glow`
- texture index `2` -> `$normal`

## Phase 3: DXT1 Texture Compression [DONE]

Goal: fix the file size bloat and likely crash cause by generating DXT1-compressed textures instead of uncompressed RGBA.

Root cause: `generate_lmip_texture_chunks_and_pool()` in `parser/src/hod.rs` decodes DXT1 textures to RGBA PNG, then re-encodes as raw RGBA pixels. This results in ~6x larger texture pool and wrong format for the engine.

Tasks:
1. Implement DXT1 encoder (Rust crate `dxt` or `texture_dds`, or custom block-compression).
2. Generate full mip chains (8 levels for 1024x1024) in DXT1 format.
3. Update `generate_lmip_texture_chunks_and_pool()` to:
   - Write DXT1 format tag instead of RGBA.
   - Declare `mip_count=8` with correct dimension pairs.
   - Compress RGBA mip data to DXT1 blocks.
   - Store DXT1-compressed data in the texture pool.
4. Update `LMIP` chunk format field from `RGBA` to `DXT1`.
5. Re-run original-vs-generated compare to verify texture pool size matches original.
6. Test in-game.

Expected outcome: generated `pebble_0.hod` should be ~1.6 MiB (matching original) and game should no longer crash due to texture format mismatch.

## Phase 4: KDOP/INFO Preservation [DONE - INFO ADDED]

Goal: if DXT1 compression doesn't fix the crash, add missing chunks.

Tasks:
1. ~~Generate minimal `KDOP` collision bounds from mesh vertex positions.~~ Not yet implemented.
2. Generate INFO chunk with OWNR sub-chunk containing author tag. **DONE.**
3. Test in-game.

Note: KDOP is still missing in from-scratch HODs. If game still crashes, this is the next priority.

## Phase 5: UI LOD Model [DONE]

Goal: inspect current React state assumptions before editing UI.

User Notes: Meshes in HODs can have multiple linked LODs, so the Nodetree should show a main node for the mesh while the inspector has options to add LOD meshes to it. Of course we need to output the HOD file properly to spec regaring LOD meshes.

Checks:
- Current `HODMesh` already has `name` and `lod`, but tree/inspector uses synthetic keys `${name}_lod_${lod}`.
- Confirm whether import/new-file flows create duplicate names with LODs or force unique names.
- Design inspector controls so a selected base mesh can expose/edit LOD variants without requiring user-created differently named mesh nodes.

## Phase 6: From-Scratch Vertex Explosion [IN PROGRESS]

Goal: debug why from-scratch HODs render correctly in editor but show severe vertex distortion in-game.

### Investigation Results

**Compression is no longer sufficient as the root cause.** The 32-bit Xpress fix reduced the generated pebble from `~14 MiB`/`~2.38 MiB` down to `1,125,695` bytes, and all POOL streams decompress to expected sizes, but the user still sees stretching in-game.

**NRML BMSH format is NOT the bug.** Byte-level trace confirmed:
- IFF NRML format: `NRML | size(BE) | real_id(4 bytes) | version(BE) | data`
- `generate_mult_chunks` writes correct NRML BMSH that parses correctly
- `parse_basic_mesh` receives correct LOD/part counts
- Vertex mask `0x600B`, stride `64`, vertex counts, and index counts match the original

**Current strongest lead: face pool/index topology encoding.**
- Original and generated face pools both decompress to `432` bytes.
- Generated face pool is plain sequential LE `u16`: `0,1,2,3,...`.
- Original face pool contains degenerate/restart/flag-looking entries and is not a plain triangle-list stream.
- If the game interprets HOD2 face pools as triangle strips or strip-like primitive streams, sequential indices would produce long stretched triangles.

**KDOP/COLD status:**
- Original pebble reference currently parses as DTRM children: `HIER` + `KDOP`.
- Generated parses as DTRM children: `HIER` only.
- Current parser output does not show a `COLD` chunk in the original pebble reference, so missing `COLD` does not explain this specific visual corruption.
- KDOP remains a follow-up for bounds/collision/visibility, but it is less likely than index topology to create stretched render triangles.

### Next Tasks

1. Reverse-engineer HOD2 face pool topology semantics.
2. Determine whether `prim_group_count = -1` means triangle strip, triangle list, or an engine-specific stripified stream.
3. Update generated face pool output to match game-facing topology encoding.
4. Test in-game before moving to KDOP generation.

### 2026-05-27 Revalidation Update

The face-pool hypothesis above is no longer supported after regenerating with Relic-compatible 31-bit Xpress. `face_pool_compare` now shows original and generated pebble face pools both decode to sequential little-endian `u16` streams with matching `432` decompressed bytes, and `compare_hods` reports `0` mismatched positions, normals, and tangents.

The regenerated file also preserves `KDOP` and includes `INFO`. Current next step is an in-game retest of the regenerated `/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/pebble/pebble_0/pebble_0.hod`. If stretching persists, investigate runtime-facing byte-level metadata or chunk ordering rather than basic parsed geometry/topology.

## Phase 7: Full Verification [PENDING]

Goal: verify all fixes across all test files.

Tasks:
1. Rebuild app with all fixes.
2. Test LOD inspector, eye toggles, mesh editing, material assignment, OBJ import/export.
3. Re-run `cargo run --bin verify_lossless` for all test files.
4. Test from-scratch HOD creation in-game.
5. Generate KDOP collision bounds for from-scratch HODs if needed.
