# Original vs New Pebble_0 HOD Compare

## Files Compared

- Original/reference: `/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/pebble/pebble_0/pebble_0_original.hod`
- User-created crashing file: `/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/pebble/pebble_0/pebble_0.hod`

## Initial Compare Findings

### Top-Level Structure

- Original size: `1,682,575` bytes, `6` root chunks.
- New crashing size: `14,217,188` bytes, `5` root chunks.
- Original root chunks: `VERS`, `NAME`, `POOL`, `HVMD`, `DTRM`, `INFO`.
- New crashing root chunks: `VERS`, `NAME`, `POOL`, `HVMD`, `DTRM`.
- New file is missing top-level `INFO`.

### POOL

- Original `POOL` type: `3518`.
- New crashing `POOL` type: `3`.
- Original texture pool: compressed `1,673,248`, decompressed `2,097,120`.
- New crashing texture pool: compressed `14,206,516`, decompressed `12,582,912`.
- The new file is using generated RGBA texture pool data, which is much larger than original DXT data.

### HVMD Textures And Material

- Original has three `LMIP` texture chunks:
  - `Pebble_DIFF`, `DXT1`, `8` mips, `1024x1024`.
  - `Pebble_GLOW`, `DXT1`, `8` mips, `1024x1024`.
  - `Pebble_NORM`, `DXT1`, `8` mips, `1024x1024`.
- New crashing file had no `LMIP` chunks at all, despite having a large texture pool.
- Original `STAT pebblemat` has `param_count=3` with texture indices and param names `$diffuse`, `$glow`, `$normal`.
- New crashing `STAT pebblemat` had `param_count=0`, so the material had no texture bindings.

### MULT / LOD Layout

- Original has one `MULT Root_mesh` with declared LOD count `2` and two nested `BMSH` children:
  - LOD `0`, material `0`, mask `0x600B`, vertices `144`, indices `144`.
  - LOD `1`, material `0`, mask `0x600B`, vertices `72`, indices `72`.
- New crashing file had two separate `MULT` chunks:
  - `Root_mesh_LOD0`, declared LOD count `1`.
  - `Root_mesh_LOD1`, declared LOD count `1`.
- New crashing `BMSH` chunks used version `1401` and mask `0x13`; original uses version `1400` and mask `0x600B`.

### DTRM

- Original `DTRM` has `HIER` and `KDOP`.
- New crashing `DTRM` only has `HIER`.
- `KDOP` may be optional according to existing project notes, but it remains a difference from the original and should be considered if crashes persist after fixing texture/material/LOD structure.

## Fixes Applied After Compare

- `parser/src/hod.rs`
  - New v2 generation now defaults `POOL` type to `3518` instead of `3` when no valid original texture metadata is available.
  - Saving over bad v2 files with texture data but missing `LMIP` metadata now forces full regeneration.
  - If the original/bad file has texture pool bytes but no `LMIP`, generated `LMIP` chunks and generated texture pool are used instead of preserving the invalid texture pool-only state.
  - If the original/bad file has no usable `LMIP`/`STAT` pair, generated `STAT` chunks are emitted from the model instead of preserving empty material bindings.

- `parser/src/compiler.rs`
  - Mesh grouping now recognizes suffixes like `_LOD0` and `_LOD1`, not only `_lod_0` style keys.
  - This makes imported/new meshes named `Root_mesh_LOD0` and `Root_mesh_LOD1` serialize as one `MULT Root_mesh` with multiple LOD `BMSH` children.
  - Generated `BMSH` version changed to `1400` to match original HOD2 pebble structure.
  - Vertex masks are normalized by stripping accidental `0x10/0x20` UV-set bits and mapping `0x10` to the supported primary UV bit `0x08`.

- `parser/src/dae.rs`, `src/components/Viewport.tsx`, `src/components/Inspector.tsx`
  - Imported DAE/OBJ/GLTF mesh parts now use HWRM-style mask `0x600B` when possible.
  - GLTF import now writes `position` instead of the wrong `pos` field.
  - Imported OBJ/GLTF vertices now include default tangent/binormal fields.

## Post-Fix Test Compare

A temporary resave test used the original parsed model, renamed its two meshes to `Root_mesh_LOD0`/`Root_mesh_LOD1`, and saved over the bad new file bytes into a temporary output.

Post-fix output now has:
- `POOL` type `3518`.
- Three generated `LMIP` chunks for `pebble_diff`, `pebble_glow`, `pebble_norm`.
- `STAT pebblemat` with `param_count=3`.
- One `MULT Root_mesh` with declared LOD count `2`.
- Two nested `BMSH` children with version `1400`, mask `0x600B`, and original vertex/index counts.

Remaining differences:
- Generated textures are RGBA with one mip, not original DXT1 with eight mips.
- `KDOP` and `INFO` are still absent for from-scratch files when there is no valid original to preserve.

## Post-Phase 2 Update

- `STAT` param names are now fixed: generated `STAT` uses semantic names (`$diffuse`, `$glow`, `$normal`) matching original exactly.
- Viewport textures now render correctly.
- File size remains bloated: `14.2 MiB` generated vs `1.6 MiB` original.
- Game still crashes with access violation.

## Remaining Crash Causes (Priority Order) [UPDATED: Phase 3 DXT1 compression is now done]

1. ~~Texture format mismatch~~ **FIXED in Phase 3**: Generated textures now use DXT1 with 8 mips matching original format.
2. **Vertex explosion in from-scratch HODs (ACTIVE):** From-scratch HODs render correctly in editor but show severe vertex distortion in-game. Vertex position data matches — the issue is likely in MULT/BMSH serialization format or pool offset alignment.
3. **Missing KDOP/INFO:** From-scratch HODs lack KDOP collision bounds. Engine may dereference expected collision/bounds data.
4. ~~Mip chain absence~~ **FIXED in Phase 3**: LMIP now declares `mip_count=8` with correct dimension pairs.

## From-Scratch Pebble_0 Comparison

### Files Compared

- Original/reference: `pebble_0_original.hod` (1,682,575 bytes)
- From-scratch generated: `pebble_0.hod` (2,384,584 bytes)

### Key Differences

| Aspect | Original | From-Scratch |
|--------|----------|--------------|
| Size | 1,682,575 | 2,384,584 |
| Reparse | N/A | SUCCESS (5 byte delta) |
| Mesh count | 2 | 2 |
| Joint count | 1 | 1 |
| Texture count | 3 | 3 |
| Normal w-component | 1.0 | 0.0 |
| Tangent data | computed | default (1,0,0) |
| Binormal data | computed | default (0,0,1) |
| KDOP | present (1588 bytes) | absent |
| COLD | present | absent |
| SCAR | present | absent |
| In-game render | correct | severe vertex explosion |
| Editor render | N/A | correct |

### Root Cause Analysis

The vertex explosion is NOT caused by wrong chunk structure or missing mesh counts. The NRML BMSH serialization format is also correct (byte-level trace confirmed).

The most likely remaining causes are:
1. **Face pool / index topology encoding**: Generated face pool is a plain sequential `u16` stream (`0,1,2,3,...`). Original face pool contains repeated zeroes, degenerate-looking entries, and flag/restart-looking values while keeping the same decompressed length. This best explains long stretched render triangles if the game treats the stream as a strip-like primitive list.
2. **KDOP absence**: From-scratch HODs lack KDOP bounds. This may affect visibility/culling/bounds, but is less likely than index topology to produce stretched triangle geometry.
3. **Missing DTRM children**: Current original pebble reference parses as `HIER` + `KDOP`; generated parses as `HIER` only. `COLD` does not appear in the current original pebble parse.
4. **HIER joint data**: Default Root joint (0,0,0) position/rotation may differ from original's actual joint data.

## Compression Follow-Up Update

- Original total size: `1,682,575` bytes.
- Generated total size after 32-bit compression changes: `1,125,695` bytes.
- Original texture pool: compressed `1,673,248`, decompressed `2,097,120`.
- Generated texture pool: compressed `1,122,492`, decompressed `2,097,120`.
- Original mesh pool: compressed `6,628`, decompressed `13,824`.
- Generated mesh pool: compressed `2,103`, decompressed `13,824`.
- Original face pool: compressed `344`, decompressed `432`.
- Generated face pool: compressed `332`, decompressed `432`.
- Compression is no longer sufficient as the root cause because the game still stretches after the compression fix.
