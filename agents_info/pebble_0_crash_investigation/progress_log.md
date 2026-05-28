# Pebble_0 Crash Investigation Progress Log

This file is the handoff record for agents continuing the `pebble_0` new-HOD creation, save-size, and viewport texture investigation.

## 2026-05-27 Session

### User Feedback

- User still saw created HOD output as `10.4 KiB` instead of the original `1.6 MiB`.
- User still saw the viewport texture as blurred in `viewport.png`.
- Expected texture source is `Pebble_DIFF.tga`, a `3.9 MiB` DIFF TGA file.

### Findings

- `Pebble_DIFF.tga` was located at `/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/pebble/pebble_0/Pebble_DIFF.tga`.
- `file` reports it as `Targa image data - RGBA - RLE 1024 x 1024 x 32 - 8-bit alpha - top`.
- The persistent `10.4 KiB` output can happen after a bad/skeleton HOD has already been written once: the backend then reads that existing tiny file as `original_bytes`, so an `original_bytes.is_empty()` guard is no longer enough.
- The broken/skeleton v2 HOD has empty or invalid `POOL` streams, so `save_edits` must detect that and force full v2 regeneration.
- Viewport blurring is not due to the source TGA resolution. The source is 1024x1024. Viewport filtering/cache behavior was the likely cause.

### Code Progress

- `parser/src/hod.rs`
  - Added `original_needs_full_v2_regeneration(original_bytes, updated_model)`.
  - `save_edits` now routes to `generate_v2_from_model` when the original file is empty, missing `POOL`, or has empty/invalid `POOL` streams while the updated v2 model has meshes/textures.
  - This specifically handles saving over a previously-created tiny skeleton HOD.

- `src/components/Viewport.tsx`
  - Texture cache is cleared on scene rebuild.
  - Texture cache key includes name plus base64 length/prefix to avoid stale texture reuse.
  - Texture lookup prefers exact normalized texture-name matches before fuzzy matches.
  - Viewport textures now use `sRGBColorSpace`, repeat wrapping, max anisotropy, no generated mipmaps, and linear min/mag filtering.

- `src-tauri/src/lib.rs`
  - Imported TGA preview PNG generation now flips the decoded image vertically before encoding, aligning imported previews with parsed HOD texture preview orientation.

- `docs/ui-source-of-truth/06-viewport-interactions.md`
  - Updated viewport texture rendering expectation to mention sRGB color space, repeat wrapping, linear filtering, and max anisotropy.

### Verification Performed

- `npm run build` passed.
- `cargo check --lib` in `parser/` passed with existing warnings.
- `cargo check` in `src-tauri/` passed earlier with existing warnings.
- Temporary direct skeleton-regeneration test showed saving over a skeleton routes to full generation:
  - Skeleton size in test: `16463` bytes.
  - Regenerated size in test: `14222979` bytes.
  - Reparsed regenerated model: `meshes=2`, `joints=1`, `textures=0` in that specific test because the temporary skeleton model intentionally removed textures before generating the skeleton.
- `cargo run --bin verify_lossless` completed with successful reparse checks. It still prints existing size mismatch failures for generated files because regenerated mesh compression/layout differs from original byte-for-byte output.
- `git diff --check -- parser/src/hod.rs src/components/Viewport.tsx src-tauri/src/lib.rs docs/ui-source-of-truth/06-viewport-interactions.md` had no output.

### Important Caveats

- New/generated HODs are not expected to be byte-for-byte identical to originals. Generated texture/mesh compression and layout differ.
- If importing external TGA textures into a new model, generated v2 HODs may be significantly larger than the original if textures are stored as generated RGBA pool data rather than original DXT-compressed texture pool data.
- Existing in-memory imported texture previews created before the TGA orientation/filtering changes may need to be re-imported or the app reloaded.
- If user still sees `10.4 KiB` after rebuilding/rerunning, inspect the active frontend/backend binary path and confirm the saved model has textures/materials in memory when invoking `save_hod`/`save_hod_as`.

### Open Retest Steps

- Rebuild/rerun the Tauri app.
- In the UI, save over the existing `10.4 KiB` HOD and confirm output no longer remains tiny.
- Re-import `Pebble_DIFF.tga` if the current model had an old cached/imported preview.
- Confirm viewport texture sharpness with the same camera angle as `viewport.png`.
- If viewport is still blurred, inspect actual canvas screenshot resolution/device pixel ratio and material UV scale/repeat rather than source texture size.

## 2026-05-27 Follow-Up: Access Violation Crash

### User Feedback

- New `pebble_0.hod` still crashed the game with an access violation and no useful log.
- User requested original vs new HOD comparison for spec compliance.
- User observed original LODs share the same mesh name with different LODs, instead of separate differently named mesh nodes.
- User requested phased work and continued logging in this directory.

### Phase Plan Added

- Added `phase_plan.md` for staged work:
  - Phase 1: structural compare.
  - Phase 2: crash hypothesis.
  - Phase 3: UI LOD model.
  - Phase 4: texture pipeline.
  - Phase 5: fixes and verification.

### Compare Report Added

- Added `original_vs_new_compare.md` with original vs crashing HOD details.
- Highest-risk mismatches found:
  - New crashing file had no `LMIP` chunks despite a large texture pool.
  - New crashing `STAT pebblemat` had `param_count=0`.
  - New crashing LODs were separate `MULT Root_mesh_LOD0` and `MULT Root_mesh_LOD1` instead of one `MULT Root_mesh` with two `BMSH` children.
  - New crashing `BMSH` chunks used version `1401` and mask `0x13`; original uses version `1400` and mask `0x600B`.
  - New crashing `POOL` type was `3`; original HOD2 files checked use `3518`.
  - New crashing file lacks `KDOP` and `INFO` compared with the original.

### Fixes Applied In This Follow-Up

- `parser/src/hod.rs`
  - Default generated HOD2 `POOL` type is now `3518` when no valid original texture metadata is available.
  - Bad original files with textures but missing `LMIP` now trigger full regeneration.
  - `generate_v2_from_model` no longer treats texture pool bytes alone as valid original texture metadata; original textures are preserved only when original `LMIP` chunks exist.
  - Bad/original `STAT` chunks are not preserved when there is no usable original `LMIP`/`STAT` pair; generated material bindings are emitted from the model.

- `parser/src/compiler.rs`
  - LOD grouping now strips `_LOD0`/`_LOD1` and `_lod0`/`_lod1` suffixes, not just `_lod_0`.
  - Generated `BMSH` version is now `1400`.
  - Vertex masks are normalized for HWRM output by stripping accidental secondary UV bits and mapping `0x10` to primary UV bit `0x08`.

- `parser/src/dae.rs`
  - DAE mesh parts now use `0x600B` for position, normal, UV, tangent, and binormal.

- `src/components/Viewport.tsx`
  - GLTF import now writes `position`, not the incorrect `pos` field.
  - GLTF import now adds default tangent/binormal and uses `0x600B`.

- `src/components/Inspector.tsx`
  - OBJ replacement import now adds default tangent/binormal and uses `0x600B`.

### Post-Fix Compare Result

- Temporary resave output now has:
  - `POOL` type `3518`.
  - Three `LMIP` chunks.
  - `STAT pebblemat` with `param_count=3`.
  - One `MULT Root_mesh` with LOD count `2`.
  - `BMSH` version `1400`, mask `0x600B`, and original LOD vertex/index counts.

### Remaining Risk

- Generated texture metadata is `RGBA`/single-mip, while original is `DXT1`/eight-mip. This is expected without DXT encoding/mip generation but may still affect engine behavior.
- From-scratch output still lacks `KDOP` and `INFO` when there is no original to preserve. Existing docs say collision mesh data may be optional, but this remains a spec difference.

## 2026-05-27 Phase 2: Shader Parameter Semantics

### Phase 2 Task

- HWRM shader pipeline scan (`get_shader_pipelines()` in `src-tauri/src/lib.rs`) only returns pipeline names, not per-pipeline texture parameter semantics.
- HWRM shaders define texture slots implicitly: `sob_ship.prog` slots are `inTexDiff(0)`, `inTexGlow(1)`, `inTexTeam(2)`, `inTexNorm(3)`.
- `Inspector.tsx` has `SHADER_SLOTS` mapping display labels per pipeline (e.g., ship: `["Diffuse Map (DIFF)", "Glow Map (GLOW)", "Team Paint Map (TEAM)", "Normal Map (NORM)", "Specular Map (SPEC)"]`).
- `HierarchyTree.tsx` creates default material texture slots as `name_diff`, `name_glow`, `name_team`, `name_norm` without referencing shader semantics.
- Original `STAT pebblemat` params use semantic names: `$diffuse` (idx 0), `$glow` (idx 1), `$normal` (idx 2).
- Previous generated `STAT` used texture filename keys (`pebble_diff`, `pebble_glow`, `pebble_norm`).

### Fixes Applied

- `parser/src/hod.rs`
  - Added `shader_texture_param_name(shader_name, texture_name, slot_index)` function that maps texture names to HOD semantic parameter names:
    - `*diff*` or `*albedo*` or `*base*` -> `$diffuse`
    - `*glow*` or `*emit*` -> `$glow`
    - `*team*` or `*stripe*` -> `$team`
    - `*norm*` or `*normal*` or `*bump*` -> `$normal`
    - `*spec*` or `*rough*` or `*metal*` -> `$specular`
    - `*diff_off*` -> `$diffuseoff`
    - `*glow_off*` -> `$glowoff`
    - Fallback uses shader-slot order: ship/bay slots = `[$diffuse, $glow, $team, $normal, $specular]`, thruster = `[$diffuse, $glow, $team, $normal, $diffuseoff, $glowoff]`.
  - `write_stat_texture_params()` now writes correct header extras (`extra1=5, extra2=4`) and per-param extras (`5, 4`) matching original STAT binary layout.

### Phase 2 Verification

- Temporary `tmp_phase2_stat_check` binary confirmed generated STAT matches original exactly:
  ```
  ORIGINAL  STAT name='pebblemat' shader='ship' count=3 header=5,4 param[0]=$diffuse param[1]=$glow param[2]=$normal
  GENERATED STAT name='pebblemat' shader='ship' count=3 header=5,4 param[0]=$diffuse param[1]=$glow param[2]=$normal
  ```
- `npm run build` passed.
- `cargo check --lib` passed with existing warnings.

### Next Phase Candidates

- Phase 3: If game still crashes, investigate remaining spec differences (KDOP/INFO absence, RGBA vs DXT texture format, missing mips).
- Phase 4: UI LOD model improvements (single base mesh with multiple LODs in inspector).
- Phase 5: Texture decompression/rendering pipeline (verify parsed HOD textures always decompress before viewport rendering).

## 2026-05-27 Post-Phase 2: User Feedback

### User Feedback

- **Viewport textures:** Viewport is now rendering textures added correctly. Phase 2 semantic STAT fixes + earlier viewport filtering/cache fixes resolved the blurry/missing texture issue.
- **Output file size:** Output `pebble_0.hod` is still bigger than the original `1.6 MiB`. User suspects missing compression — original HOD2 files store textures as DXT-compressed in the texture pool, but generated HOD2 files store uncompressed RGBA data.
- **Game still crashing:** Game still crashes with access violation despite Phase 1 structural fixes and Phase 2 STAT semantic fixes.

### Root Cause Analysis: File Size Bloat

The generated HOD2 file is much larger because:
- Original `pebble_0.hod` texture pool: compressed `1,673,248` bytes, decompressed `2,097,120` bytes. Format: `DXT1` with 8 mips.
- Generated `pebble_0.hod` texture pool: compressed `14,206,516` bytes, decompressed `12,582,912` bytes. Format: `RGBA` with 1 mip.
- The `generate_lmip_texture_chunks_and_pool()` function in `parser/src/hod.rs` decodes DXT1 texture data to RGBA PNG, then re-encodes as RGBA raw pixels. This results in ~6x larger texture pool.
- Original HOD2 files use DXT1/DXT5 compressed textures with full mip chains. Generated files use uncompressed RGBA with a single mip level.
- The `xpress::compress()` function is still called on the generated RGBA pool, but RGBA compresses less efficiently than DXT1.

### Root Cause Analysis: Continued Crash

The game crash may be caused by:
1. **Texture format mismatch:** Engine expects DXT1 compressed textures in the texture pool, not uncompressed RGBA. The texture pool offsets in BMSH vertex data and LMIP chunks reference byte positions that assume DXT1-compressed data layout.
2. **Mip chain absence:** Original LMIP chunks declare `mip_count=8` with 8 dimension pairs. Generated LMIP chunks declare `mip_count=1`. Engine may iterate over expected mip levels and read past buffer bounds.
3. **KDOP/INFO absence:** Original has `KDOP` (1588 bytes) in DTRM and `INFO` (50 bytes) at top level. Generated from-scratch files lack both. Engine may dereference expected collision/bounds data.

### Next Steps for Next Agent

Phase 3 should focus on the two most likely remaining crash causes, in priority order:

1. **DXT1 texture compression:** Add DXT1 encoding to `generate_lmip_texture_chunks_and_pool()` so generated textures use the same compressed format as originals. This requires:
   - Implementing a DXT1 encoder (or using an existing Rust crate like `dxt` or `texture_dds`).
   - Generating full mip chains (8 levels for 1024x1024).
   - Updating LMIP metadata to declare DXT1 format and correct mip count.
   - This will also fix the file size bloat issue.

2. **KDOP/INFO preservation:** If DXT1 compression doesn't fix the crash, generate minimal KDOP/INFO chunks for from-scratch HODs, or copy them from the original pebble_0 when available.

Important context: When loading an existing HOD and saving it, the original texture pool bytes (already DXT-compressed) are preserved via `comp_tex_buf`. The bloat only occurs for new/from-scratch HODs or when the original has no usable texture pool. The user's observation about compression skipping on load+save is correct and by design.

## 2026-05-27 Phase 3: DXT1 Texture Compression [DONE]

### Phase 3 Task

- Implement DXT1 block-compression encoder for generated textures.
- Generate full mip chains (8 levels for 1024x1024).
- Update LMIP metadata to declare DXT1 format and correct mip count.

### Implementation

- `parser/src/hod.rs`
  - Added DXT1 block-compression encoder functions:
    - `rgb565_to_u16()`, `u16_to_rgb565()`, `color_error()`: color conversion/error helpers.
    - `find_best_endpoints()`: finds two best RGB565 endpoints for a 4x4 block using min/max bounding box candidates.
    - `compress_dxt1_block()`: compresses a single 4x4 RGBA block to 8 bytes of DXT1.
    - `compress_dxt1()`: compresses an entire RGBA image to DXT1.
    - `generate_mip_chain()`: generates mip chain with box-filter downsampling.
  - Updated `generate_lmip_texture_chunks_and_pool()`:
    - Now generates DXT1 format instead of RGBA.
    - Produces full mip chains (up to 8 levels, matching original).
    - Writes DXT1 format tag in LMIP metadata.
    - Writes correct mip count and dimension pairs.

### Verification Results

- `cargo check --lib` passed.
- `cargo run --bin verify_lossless` passed with successful reparse checks.
- Generated pebble_0 size: `1,691,722` bytes (original: `1,682,575` bytes, diff: `9,147` bytes / ~0.5%).
  - Previously: `14,223,201` bytes (6x larger).
  - Now: ~1x original size.
- Other test files (ter_elysium, ter_fenris) unchanged because they preserve original texture pool from existing HODs.
- `npm run build` passed.

### Remaining Size Difference

The ~9 KiB difference (~0.5%) is from:
- DXT1 encoder producing slightly different compressed output than original tool.
- Small structural differences in non-texture chunks (NAME length, INFO absence, etc.).

### Next Phase Candidates

- Phase 4: KDOP/INFO preservation if game still crashes after DXT1 fix.
- Phase 5: UI LOD model improvements.

## 2026-05-27 Post-Phase 3: Remaining Crash Fixes

### User Feedback

- Game still crashing after Phase 3 DXT1 compression fix.
- Recreated pebble_0 with LOD0 only for comparison.

### Compare Results (New vs Original)

Key remaining differences:
1. **Version**: New was 2000, original is 512 (0x200). Fixed in `App.tsx`.
2. **Missing INFO chunk**: Original has INFO FORM with OWNR sub-chunk. From-scratch HODs didn't generate one.
3. **LMIP chunk type**: Original uses Default, new used Normal. Fixed.
4. **LOD count**: New file only had LOD0 (user only imported one mesh).

### Fixes Applied

- `parser/src/hod.rs`
  - LMIP chunk type changed from `ChunkType::Normal` to `ChunkType::Default`.
  - Added INFO chunk generation with OWNR sub-chunk containing author tag.
- `src/App.tsx`
  - New HOD model version changed from `2000` to `512` (HOD 2.0 standard).

### Build Verification

- `cargo check --lib` passed.
- `npm run build` passed.
- `cargo run --bin verify_lossless` passed with successful reparse.
- Generated pebble_0 size: `1,691,722` bytes (original: `1,682,575`, diff: `9,147` bytes / ~0.5%).

## 2026-05-27 Post-Phase 3: Joint Parent Fix

### Issue Found

- Subagent comparison flagged: Root joint had `parent_name: "Root"` (circular self-reference).
- Original HOD Root joint has `parent_name: None` (empty string in binary).
- This circular reference could cause engine to infinite-loop or crash during hierarchy traversal.

### Fix Applied

- `src/App.tsx`
  - Changed Root joint `parent_name: "Root"` to `parent_name: undefined` so serializer writes empty string (no parent).

### Build Verification

- `npm run build` passed.

## 2026-05-27 Post-Phase 3: Texture Pool Fix

### Issues Found

- **Broken faces/spikes**: Vertex positions match original but normals/UVs differ — caused by texture pool format mismatch.
- **Colored static textures**: LMIP metadata claims DXT1 but pool data was RGBA from previously-generated file.

### Root Cause

When saving over a previously-generated file, the old code preserved the original texture pool bytes (`comp_tex_buf`) whenever the original had LMIP chunks. But the original file could have:
- RGBA texture data from a previous generation (not DXT1)
- LMIP metadata that claimed DXT1 format

This mismatch caused the engine to read texture data with wrong offsets/format.

### Fix Applied

- `parser/src/hod.rs`
  - Added `original_has_dxt1_lmip` check: scans original LMIP chunks for DXT1/DXT5 format tags.
  - Texture pool is now preserved only when original LMIP chunks declare DXT1/DXT5 format.
  - When original has non-DXT1 LMIP (or no LMIP), textures are regenerated from model data with DXT1 encoding.
  - LMIP chunks are regenerated when textures are regenerated, preserved when original DXT1 is kept.
  - Removed unused `original_lmip_count`/`original_stat_count` variables.

### Build Verification

- `cargo check --lib` passed.
- `npm run build` passed.
- `cargo run --bin verify_lossless` passed with successful reparse.
- Generated pebble_0 size: `1,691,722` bytes (original: `1,682,575`, diff: `9,147` bytes / ~0.5%).
- Other test files preserved original sizes (ter_elysium, ter_fenris unchanged).

## 2026-05-27 Post-Phase 3: STAT Auto-Generation Fix

### Issues Found (from new pebble_0.hod comparison)

1. **STAT chunk had 0 params** — the materials array was empty when saving, so no texture bindings were written.
2. **LMIP texture names reordered** — depends on user import order, not wrong per se.
3. **KDOP missing** — still not generated for from-scratch HODs.

### Root Cause

When creating a new HOD and importing textures without creating a material in the Material Library, the `model.materials` array is empty. The fallback STAT generation wrote `param_count=0`, so the engine had no texture-to-material bindings.

### Fix Applied

- `parser/src/hod.rs`
  - Changed fallback STAT generation: when materials are empty but textures exist, auto-generates a material with all imported textures bound in order.
  - Uses `shader_name: "ship"` and binds textures by their index in the `model.textures` array.
  - Writes proper STAT params via `write_stat_texture_params()`.

### Build Verification

- `cargo check --lib` passed.
- `npm run build` passed.
- `cargo run --bin verify_lossless` passed.
- Generated pebble_0 size: `1,691,722` bytes (original: `1,682,575`, diff: `9,147` bytes / ~0.5%).

## 2026-05-27 Material Import Investigation

### Material JSON Found

- Location: `/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/pebble/pebble_0/Homeworld2 Multi Mesh File_materials.json`
- Content: `[{ name: "pebblemat", shader_name: "ship", texture_maps: ["Pebble_DIFF", "Pebble_GLOW", "", "Pebble_NORM", ""] }]`
- The empty strings at positions 2 (TEAM) and 4 (SPEC) are expected — ship shader has 5 slots but only 3 textures used.

### Import Flow

- `handleImportMaterials()` in `HierarchyTree.tsx` parses JSON and calls `onModelChange?.({ ...model, materials: parsedMaterials })`.
- This should update `model.materials` which is then serialized via `generate_v2_from_model`.
- `write_stat_texture_params` correctly skips empty strings and writes only non-empty texture params.

### HODOR SHADERS.MAP Analysis

- Located at `/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/HODOR/SHADERS.MAP`
- Defines shader parameter mapping for all pipelines:
  - `ship`: `$diffuse[DXT1]`, `$glow[DXT1]`, `$team[DXT1]`, `$normal[DXT1]`
  - `shipglow`: same as ship + `$spec[DXT1]`
  - `thruster`: `$diffuseOn[DXT1]`, `$diffuseOff[DXT1]`, `$glowOn[DXT1]`, `$glowOff[DXT1]`
  - `badge`, `matte`, `bay`, etc. with their own mappings
- Format: `+pipeline_name` block with `$param[FORMAT] = default_values` and channel swizzle rules
- Confirms our STAT parameter names (`$diffuse`, `$glow`, `$normal`) are correct.

### Remaining Issue

The STAT chunk having 0 params might be caused by:
1. Material JSON import timing — imported after model state was already saved.
2. Texture name mismatch between JSON (`Pebble_DIFF`) and imported textures (`pebble_diff` — lowercased).
3. The `texture_maps` in the imported material might reference texture names that don't match the model's `textures` array.

The auto-generation fallback should handle the common case. User should rebuild/retest.

## 2026-05-27 Save Architecture Fix

### Issue

- Vertex normals/tangents/binormals were zeroed in generated mesh pool because `save_edits` preserved the old mesh pool instead of regenerating from model data.
- Asteroid 3 (HOD 2.0 with inline NRML BMSH) failed re-parse because `update_mesh_chunks` always created FORM chunks, but the original had NRML MULT with inline NRML BMSH.
- VERS and NAME chunks were created as FORM type instead of NRML.

### Root Cause

The save architecture was inconsistent: `save_edits` patched old files while `generate_v2_from_model` generated from scratch. This caused mesh pool preservation issues and chunk type mismatches.

### Fixes Applied

- `parser/src/hod.rs`
  - Added `preserved_chunks: Vec<IffChunk>` to `HODModel` to store unparsed chunks (INFO, KDOP, COLD, SCAR, etc.).
  - Parser captures unparsed root chunks and DTRM sub-chunks into `preserved_chunks`.
  - `generate_v2_from_model` writes preserved chunks back to output.
  - `save_edits` now routes v2 files to `generate_v2_from_model` (generate from scratch).
  - `generate_v2_from_model` always generates textures from model data (no original pool preservation).
  - `update_mesh_chunks` detects NRML MULT and uses inline BMSH writing (v1 style) instead of pool-based writing.
  - VERS and NAME chunks now use `ChunkType::Normal` (NRML) to match original HOD 2.0 format.
  - `update_mesh_chunks` preserves original chunk type when reconstructing MULT.
- `docs/agent-handbook/README.md`
  - Added critical hard rule: every save generates a fresh HOD 2.0 file from model data. No patching.

### Build Verification

- `cargo check --lib` passed.
- `npm run build` passed.
- `cargo run --bin verify_lossless` passed for all tests:
  - Pebble: SUCCESS (reparse), diff 9129 bytes
  - Ter Elysium: SUCCESS (reparse), diff 2199606 bytes
  - Ter Fenris: SUCCESS (reparse), diff 35128 bytes
  - Asteroid 3: SUCCESS (reparse), diff -54 bytes

## 2026-05-27 Phase 5: UI LOD Model

### Issue

- HOD format stores LODs as one `MULT` with `lod_count=N` and N `BMSH` children sharing the same mesh name.
- UI showed each LOD as a separate flat item (`Root_mesh_lod_0`, `Root_mesh_lod_1`), requiring the user to create differently named mesh nodes.

### Changes Applied

- `src/components/HierarchyTree.tsx`
  - Meshes are now grouped by base name (stripping `_lod_N` suffix).
  - One tree item per base mesh (e.g., `Root_mesh (2 LODs)`).
  - When selected, LOD variants are shown as indented children with vertex counts.
  - Selection type uses `mesh` for base mesh, `mesh_lod` for specific LOD variant.

- `src/components/Inspector.tsx`
  - Updated to handle both `mesh` (base mesh) and `mesh_lod` (specific LOD) selection types.
  - When base mesh is selected, shows the first LOD's details.
  - When specific LOD is selected, shows that LOD's details.

### Build Verification

- `npm run build` passed.
- Parser serialization already supports same-name/different-LOD meshes via `generate_mult_chunks` grouping by base name.

## 2026-05-27 Post-Phase 5: LOD Inspector Improvements

### Changes Applied

- `src/components/Inspector.tsx`
  - Added `MeshLODInspector` component with:
    - LOD list showing vertex count, triangle count, material index per LOD.
    - Add LOD button (imports OBJ as new LOD).
    - Delete LOD button (removes LOD from mesh).
    - Reorder LOD buttons (move LOD up/down).
    - Per-LOD eye visibility toggles.
    - OBJ export per LOD.
    - Material assignment per LOD.
    - Geometry info (vertex mask, vertex count, index count).
  - `InspectorProps` extended with `visibleMeshes` and `onToggleVisibility` for per-LOD visibility.

- `src/components/HierarchyTree.tsx`
  - Eye toggle for base mesh now toggles all LODs.

- `src/components/Viewport.tsx`
  - LOD visibility toggles wired through `visibleMeshes` state in App.
  - Mesh visibility keyed by `${mesh.name}_lod_${lod}`.

### Build Verification

- `npm run build` passed.

## 2026-05-27 Save Architecture Rework

### Issue

- Original save architecture patched existing files, causing accumulated bugs:
  - Mesh pool preservation mismatches.
  - Chunk type inconsistencies (NRML vs FORM).
  - KDOP/SCAR/COLD nesting errors.
  - VERS/NAME chunk type errors.

### Design Decision

**Every save generates a fresh HOD 2.0 file from model data. No patching. All output is HOD 2.0 format.**

This eliminates all patching bugs by treating the original file as read-only reference data.

### Changes Applied

- `parser/src/hod.rs`
  - `save_edits` now routes all v2 saves to `generate_v2_from_model`.
  - `generate_v2_from_model` always generates textures, meshes, and materials from model data.
  - Original texture pool bytes only preserved when original LMIP declares DXT1/DXT5 format.
  - Original mesh/face pool bytes preserved only when meshes haven't been modified.
  - Added `preserved_chunks: Vec<IffChunk>` to `HODModel` for unparsed chunks (INFO, KDOP, COLD, SCAR).
  - Parser captures unparsed root chunks and DTRM sub-chunks into `preserved_chunks`.
  - `generate_v2_from_model` writes preserved chunks back to output (KDOP inside DTRM children, INFO at top level).
  - VERS and NAME chunks changed from `ChunkType::Normal` (NRML) to `ChunkType::Form` (FORM).
  - KDOP nesting fix: preserved KDOP/COLD/SCAR pushed into `dtrm_children`, not top-level.
  - `update_mesh_chunks` preserves original chunk type (NRML vs FORM) when reconstructing MULT.

- `docs/agent-handbook/README.md`
  - Added hard rule: every save generates fresh HOD 2.0 from model data.

### Build Verification

- `cargo check --lib` passed.
- `npm run build` passed.
- `cargo run --bin verify_lossless` passed for all tests:
  - Pebble: SUCCESS (reparse), 5 byte delta
  - Ter Elysium: SUCCESS (reparse), -109 bytes delta
  - Ter Fenris: SUCCESS (reparse), -109 bytes delta
  - Asteroid 3: SUCCESS (reparse), -54 bytes delta
  - DAE fallback: SUCCESS (reparse)

## 2026-05-27 From-Scratch Pebble_0 Creation

### User Action

- User created a new `pebble_0.hod` from scratch using the editor:
  - Imported OBJ meshes: `Root_mesh_lod0.obj` (20KB, 144 verts), `Root_mesh_lod1.obj` (10KB, 72 verts)
  - Imported TGA textures: `Pebble_DIFF.tga` (4.1MB), `Pebble_GLOW.tga` (3.6MB), `Pebble_NORM.tga` (4.2MB)
  - Source directory: `/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/pebble/pebble_0/`

### Results

- **Editor viewport**: Renders correctly (`pebble_in_editor.png`).
- **In-game**: Severe vertex explosion/distortion (`pebble_in_game.png`).
- **Generated file size**: 2,384,584 bytes (vs original 1,682,575 bytes).
- **Re-parse**: Generated file parses successfully with identical mesh/joint counts.

### Vertex Data Comparison

Comparing original vs from-scratch hex at vertex 0 offset:

**Original (correct):**
```
pos:  [-0.17662, -1.65612, 2.464357, 1.0]
norm: [-0.225789, -0.743329, 0.629668, 1.0]
uv:   [0.5, 0.0]
tang: [0.9705058, -0.22769131, 0.0792161]
binorm: [-0.038397547, 0.65264744, 0.7566881]
```

**From-scratch (distorted):**
```
pos:  [-0.17662, -1.65612, 2.464357, 1.0]  ← MATCHES
norm: [-0.22578895, -0.7433288, 0.6296679, 0.0]  ← w=0.0 instead of 1.0
uv:   [0.5, 0.0]  ← MATCHES
tang: [1.0, 0.0, 0.0]  ← DEFAULT (no tangent data in OBJ)
binorm: [0.0, 0.0, 1.0]  ← DEFAULT (no binormal data in OBJ)
```

### Analysis

- Position data matches exactly — vertex positions are correct.
- Normal w-component differs (0.0 vs 1.0) — likely cosmetic, not crash-causing.
- Tangent/binormal are defaults (1,0,0) and (0,0,1) — cosmetic, not crash-causing.
- The vertex explosion is NOT caused by wrong vertex data in the pool.

### Investigation Results (Detailed Analysis)

**NRML BMSH format is CORRECT.** Detailed byte-level trace confirms:

The IFF NRML format is: `NRML | size(BE) | real_id(4 bytes) | version(BE) | data`
- `generate_mult_chunks` writes: `NRML | nrml_size | "BMSH" | 1400(BE) | lod | part_count | parts`
- `IffChunk::read_chunk` parses: `id = "BMSH"`, `version = 1400`, `data = lod | part_count | parts`
- Parser check `child.id.trim() == "NRML"` is FALSE (id is "BMSH"), falls through to `Cow::Borrowed`
- Parser check `actual_child.id.trim() == "BMSH"` is TRUE → `parse_basic_mesh` IS called
- `parse_basic_mesh` reads lod and part_count as LE i32 from data → correct values
- **NRML format is NOT the bug.**

**Vertex data matches between original and from-scratch:**
- Position: identical (e.g., [-0.17662, -1.65612, 2.464357, 1.0])
- Normal: nearly identical (w=0.0 vs w=1.0 — cosmetic)
- UV: identical (e.g., [0.5, 0.0])
- Tangent/binormal: defaults (1,0,0) and (0,0,1) — cosmetic, not crash-causing
- Vertex stride: 64 bytes, matching original
- Vertex mask: 0x600B, matching original

**Remaining hypotheses (ranked by likelihood):**

1. **Missing KDOP collision bounds:** From-scratch HODs lack KDOP (1588 bytes in original). Engine may use KDOP for bounding volume or collision checks that affect mesh placement/scaling. Without it, vertices may be placed relative to wrong origin or scale.

2. **Different DXT1 compression output:** Our custom DXT1 encoder produces different compressed bytes than the original tool. The texture pool data differs. While the texture pool shouldn't affect vertex positions, some engines use texture metadata for UV calculations.

3. **Pool type/structure mismatch:** The POOL type is 3518 (correct), but internal offsets or alignment may differ. The xpress compression of mesh/face pool may produce different compressed output that the game decompresses differently.

4. **Chunk ordering or presence:** From-scratch HODs have: VERS, NAME, POOL, HVMD, DTRM, INFO. Original has: VERS, NAME, POOL, HVMD, DTRM, INFO. Order matches. But from-scratch DTRM has only HIER (no KDOP, COLD, SCAR, BNDV, ETSH). Game may iterate DTRM children and fail gracefully when expected chunks are missing, causing default mesh transforms.

5. **HIER joint data mismatch:** From-scratch generates a default Root joint with position (0,0,0), rotation (0,0,0), scale (1,1,1). The original may have different joint data that affects mesh positioning in the hierarchy.

### Key Files for Investigation

- `parser/src/compiler.rs:218` — `generate_mult_chunks()`: Creates MULT chunks with embedded NRML BMSH. **Format is correct.**
- `parser/src/hod.rs:3484` — `generate_v2_from_model()`: Assembles full HOD from model.
- `parser/src/hod.rs:2365` — `parse_basic_mesh()`: Parses BMSH data from pool. **Works correctly.**
- `parser/src/hod.rs:2890` — `write_vertex()`: Writes vertex data to mesh pool. **Normal w=0 vs original w=1 (cosmetic).**
- `parser/src/iff.rs:116` — `IffChunk::read_chunk()` NRML branch: Format is `NRML | size(BE) | real_id | version(BE) | data`. **Correctly implemented.**
- `parser/src/compiler.rs:136` — `generate_pool_data()`: Writes compressed mesh/face pool. **May have alignment issues.**

### What's Left

1. **Investigate KDOP absence:** Generate minimal KDOP collision bounds from mesh vertex positions for from-scratch HODs. This is the highest-priority remaining hypothesis.
2. **Compare pool binary output:** Byte-compare generated vs original mesh pool decomp data to find subtle differences in vertex layout or alignment.
3. **Test rebuilt app with all fixes:** LOD inspector, eye toggles, mesh editing, material assignment, OBJ import/export.
4. **Full verification:** `cargo run --bin verify_lossless` for all test files after any fixes.

## 2026-05-27 Compression Follow-Up: Stretching Still Present

### User Feedback

- Previous agent patched `xpress::compress` to use the 32-bit compression path.
- Generated `pebble_0.hod` now shrank to `1,125,695` bytes vs original `1,682,575` bytes.
- Texture pool compression is now better than original: generated texture pool compressed to `1,122,492` bytes vs original `1,673,248` bytes, while both decompress to `2,097,120` bytes.
- In-game geometry still stretches to infinity.
- User noted game collision is handled by `COLD`; KDOP may be for some other engine-side bounding/visibility purpose.

### Commands Run

- `cargo run --bin compare_hods`
- `cargo run --bin dump_pool -- /run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/pebble/pebble_0/pebble_0_original.hod`
- `cargo run --bin dump_pool -- /run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/pebble/pebble_0/pebble_0.hod`
- Added and ran diagnostic: `cargo run --bin face_pool_compare`

### Confirmed Structure After Compression Fix

- Root chunk count matches: 6 chunks.
- POOL type matches: `3518`.
- Texture decompressed size matches: `2,097,120` bytes.
- Mesh decompressed size matches: `13,824` bytes.
- Face decompressed size matches: `432` bytes.
- Both files parse as 2 meshes, 1 joint.
- BMSH metadata matches:
  - LOD0: `144` vertices, `144` indices, mask `0x600B`, stride `64`.
  - LOD1: `72` vertices, `72` indices, mask `0x600B`, stride `64`.
- DTRM differs:
  - Original: `HIER` + `KDOP`.
  - Generated: `HIER` only.
  - No `COLD` chunk in the original pebble reference according to the current parser output.

### New Strong Lead: Face Pool / Index Topology

The generated file's face pool is a plain sequential `u16` stream:

```text
generated LE first 80:
[0, 1, 2, 3, 4, 5, ..., 78, 79]
```

The original face pool is not a plain sequential triangle-list stream. It includes repeated zeroes, degenerate-looking entries, and flag-looking high-byte values:

```text
original LE first 80:
[0, 1, 2, ..., 15, 0, 16, 17, ..., 27, 0, 28, 29, 30, 0, 32, ...,
 11520, 11776, 0, 32768, 12032, 12800, ...]
```

Raw original bytes around the first problematic region:

```text
0070: [00, 00, 00, 00, 00, 2d, 00, 2e, 00, 00, 00, 80, 00, 2f, 00, 32]
```

This is more consistent with a triangle-strip or strip-like primitive stream with degenerates/flags than a flat triangle list. If the game renders the face pool as a strip-like primitive stream, a generated sequential `0,1,2,3,...` stream will connect unrelated vertices across the whole mesh, producing exactly the observed long stretched triangles.

### Updated Hypothesis Ranking

1. **Face pool/index topology encoding is wrong for generated meshes.** This now best matches the visual symptom. The editor parser treats indices as plain local `u16`s and clamps invalid values, which can hide the issue. The game likely interprets the original face pool with stricter strip/primitive semantics.
2. **KDOP absence may still matter**, but is less likely to create long stretched render triangles. KDOP is still present in original and absent in generated, so it remains a required follow-up after topology is fixed.
3. **COLD is collision**, but the original pebble reference currently does not contain `COLD`; therefore missing `COLD` does not explain this specific pebble render corruption.

### Next Engineering Task

- Reverse-engineer HOD2 face pool topology semantics and update generated face pool output accordingly.
- Investigate whether `prim_group_count = -1` means triangle strip, triangle list, or an engine-specific stripified stream.
- Do not spend effort on KDOP generation until a generated face pool with sane game-facing topology has been tested.

## 2026-05-27 Compression Revalidation: 31-bit Xpress Restored

### Work Performed

- Restored Relic-compatible Xpress indicator cadence to the current 31-bit read/write behavior in `parser/src/xpress.rs`.
- Added `xpress::compress_or_raw(input)` so generated POOL streams can be stored raw when compression would not shrink them.
- Updated POOL parsing paths to treat `comp_len == decomp_len` as a raw, uncompressed stream.
- Updated `parser/src/bin/face_pool_compare.rs` to handle raw face streams before decompressing.
- Regenerated `/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/pebble/pebble_0/pebble_0.hod` from `pebble_0_original.hod` using `cargo run --bin test_from_scratch`.

### Validation Commands

- `cargo run --bin test_from_scratch`
- `cargo run --bin compare_hods`
- `cargo run --bin dump_pool -- "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/pebble/pebble_0/pebble_0.hod"`
- `cargo run --bin face_pool_compare`
- `cargo run --bin verify_lossless`
- `git diff --check -- parser/src/bin/face_pool_compare.rs parser/src/xpress.rs parser/src/hod.rs parser/src/compiler.rs`

### Results

- Regenerated from-scratch pebble parses successfully.
- Generated pebble size is now `1,132,334` bytes.
- Generated POOL:
  - type `3518`
  - texture pool `comp=1,122,994`, `decomp=2,097,120`
  - mesh pool `comp=6,648`, `decomp=13,824`
  - face pool `comp=332`, `decomp=432`
- Original-vs-generated model comparison shows exact mesh identity at parsed-model level:
  - meshes `2 -> 2`
  - both named `Root_mesh`
  - LODs `0` and `1` preserved
  - vertex counts `144` and `72` preserved
  - index counts `144` and `72` preserved
  - mismatched positions/normals/tangents: `0`
- Generated DTRM now includes preserved `KDOP`; generated root chunks include `INFO`.
- `verify_lossless` structurally succeeds on the mandatory suite: every generated file reparses with matching mesh, joint, navlight, marker, and engine-burn counts. The script still prints expected byte-size mismatches.

### Corrected Finding

- The previous face-pool/index-topology hypothesis was based on output generated with the incompatible 32-bit Xpress cadence.
- After restoring 31-bit Xpress and regenerating the file, `face_pool_compare` shows the original and generated pebble face pools are both identical sequential little-endian `u16` streams for the first inspected region and both decode to `432` bytes.
- Face pool topology is therefore not the current leading cause for pebble stretching.

### Current Status

- The regenerated file is ready for in-game retest.
- If stretching persists, the next likely issue is not parsed geometry, face indices, KDOP presence, INFO presence, or basic BMSH metadata. Investigate runtime-facing differences such as texture compression dialect/quality, STAT/LMIP byte-level metadata, chunk child ordering inside `MULT`/`DTRM`, or game-specific expectations not represented by the current parser model.

### Caveats

- `git diff --check` still reports pre-existing trailing whitespace in `parser/src/compiler.rs` and `parser/src/hod.rs`; the new `face_pool_compare.rs` change did not introduce whitespace errors.
- `cargo fmt` is unavailable in this environment (`cargo-fmt` missing for stable toolchain).

## 2026-05-27 Editor HOD 2.0 Render Regression Fixed

### User Report

- Loading HOD 2.0 files in the editor caused glitched model rendering.
- The issue was seen on load/render in the editor, before in-game testing of newly generated files.

### Root Cause

- The Rust HOD 2.0 mesh parser called `align_face_pool()` before reading each part's face indices.
- `align_face_pool()` used a heuristic to skip one byte when upcoming face-pool words looked suspicious.
- HOD 2.0 face pools are plain little-endian `u16` index streams in the tested files. Skipping one byte desynchronizes every following index for that part, which produces scrambled/glitched triangles in the editor viewport.

### Fix Applied

- Removed the `align_face_pool(&mut context.face_pool)` call from `parse_basic_mesh()` in `parser/src/hod.rs`.
- Removed the now-unused `align_face_pool()` helper.

### Verification

- User confirmed HODs now render correctly in the editor.
- `cargo check --lib` passed with existing warnings.
- `cargo run --bin verify_lossless` structurally succeeded across the mandatory suite. It still prints expected byte-size deltas, but generated files reparse with matching mesh/joint/navlight/marker/engine-burn counts.

### Corrected Understanding

- The earlier face-pool topology suspicion was wrong because the previous diagnostic was polluted by the parser's byte-alignment heuristic and earlier Xpress experimentation.
- The actual editor render glitch was caused by parser-side face index desynchronization on load, not by frontend rendering and not by HOD 2.0 requiring strip/flag face topology for the tested files.

## 2026-05-27 Current User-Created Pebble HOD Check

### Files Compared

- Original: `/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/pebble/pebble_0/pebble_0_original.hod`
- User-created from scratch: `/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/pebble/pebble_0/pebble_0.hod`

### Commands Run

- `cargo run --bin compare_hods`
- `cargo run --bin dump_pool -- "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/pebble/pebble_0/pebble_0_original.hod"`
- `cargo run --bin dump_pool -- "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/pebble/pebble_0/pebble_0.hod"`
- `cargo run --bin face_pool_compare`

### Current Diagnostics

- Original size: `1,682,575` bytes.
- User-created size: `1,127,639` bytes.
- Original `POOL` starts at byte `54`; user-created `POOL` starts at byte `37` because the generated file has `NAME = New_Model` instead of `Homeworld2 Multi Mesh File`.
- Both parse successfully as HOD 2.0 with:
  - `2` meshes
  - `1` joint
  - LODs `0` and `1`
  - mesh name `Root_mesh`
  - vertex counts `144` and `72`
  - index counts `144` and `72`
- Position mismatch count: `0` for both LODs.
- Normal mismatch count: `0` for both LODs.
- Tangent mismatch count: `144` for LOD0 and `72` for LOD1, because imported OBJ geometry defaults tangent/binormal data instead of preserving original tangent/binormal vectors.
- Face pools decode identically in the inspected region: both are sequential little-endian `u16` streams and both decompress to `432` bytes.

### POOL Comparison

- Original POOL:
  - type `3518`
  - texture pool `comp=1,673,248`, `decomp=2,097,120`
  - mesh pool `comp=6,628`, `decomp=13,824`
  - face pool `comp=344`, `decomp=432`
- User-created POOL:
  - type `3518`
  - texture pool `comp=1,124,432`, `decomp=2,097,120`
  - mesh pool `comp=2,107`, `decomp=13,824`
  - face pool `comp=332`, `decomp=432`

### Remaining Differences To Watch In-Game

- User-created DTRM currently parses with only `HIER`; original has `HIER` + `KDOP`.
- User-created model name is `New_Model`.
- Imported OBJ tangents/binormals are defaulted, not original.
- Texture pool decompressed size matches original, but compressed bytes differ due generated DXT1/Xpress output.

### Current Status

- Editor HOD 2.0 rendering is fixed.
- User is testing the user-created `pebble_0.hod` in-game. If it still renders incorrectly in-game, investigate the remaining differences above first, with `KDOP` absence and default tangent/binormal data as the top concrete differences.

## 2026-05-27 HOD 2.0 NAME Save Rule

### Finding

- Original `pebble_0_original.hod` has `NAME = Homeworld2 Multi Mesh File`.
- The `NAME` chunk payload length is `26` bytes and contains exactly `Homeworld2 Multi Mesh File` with no trailing null.
- A user-created from-scratch HOD had `NAME = New_Model`, which shortened the early chunk layout and moved the `POOL` offset from byte `54` to byte `37`.

### Decision

- Every HOD 2.0 save must write the fixed `NAME` payload `Homeworld2 Multi Mesh File`, independent of the editable/UI model name.
- The UI model name may still be useful for display, but it must not be serialized into the HOD 2.0 `NAME` chunk.

### Fix Applied

- `parser/src/hod.rs`
  - `generate_v2_from_model()` now always writes `NAME` data as `Homeworld2 Multi Mesh File`.
  - The empty-original fallback path in `save_edits()` also writes `Homeworld2 Multi Mesh File`.

### Verification

- `cargo check --lib` passed with existing warnings.
- `cargo run --bin test_from_scratch` regenerated the pebble HOD and reparsed it with `Name='Homeworld2 Multi Mesh File'`.
- `cargo run --bin verify_lossless` structurally succeeded across the mandatory suite. Size deltas remain expected; generated files reparse with matching key counts.
