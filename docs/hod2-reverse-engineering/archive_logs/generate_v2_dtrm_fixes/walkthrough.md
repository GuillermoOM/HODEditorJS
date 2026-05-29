# Walkthrough: Fix `generate_v2_from_model` DTRM Serialization

> Session started: 2026-05-26  
> Status: IN PROGRESS

## Context
User opened `ter_elysium.hod` (HOD 2.0) in the app, made several edits (renamed navlights, deleted duplicate joints, added turret barrel), and saved as `ter_elysium_edited.hod`. Re-opening the file showed it was completely broken.

## Investigation Method
1. Performed byte-level binary analysis comparing original vs edited files using Python struct parsing
2. Walked both files' IFF chunk trees to compare structural layout
3. Parsed all 30 original joints vs 29 edited joints from HIER payloads
4. Identified 5 bugs in `generate_v2_from_model` (the code path used for HOD 2.0 saves via Tauri)

## Key Findings

### Code Path Discovery
The Tauri app (`src-tauri/src/lib.rs` lines 208-222) uses **two different save functions**:
- `generate_v2_from_model()` for `model.is_v2 == true` (HOD 2.0)
- `save_edits()` for HOD 1.0

The `save_edits` function has correct DTRM serialization logic. The `generate_v2_from_model` function has a simplified/stub DTRM generation that was written for the initial `verify_lossless` roundtrip testing and never updated to handle real user edits.

### Structural Comparison (ter_elysium)

**Original DTRM sub-chunks:**
```
FORM HIER size=1790  (30 joints, first_val=0xFFFFFFE2)
BURN ×8 (individual, ChunkType::Default, 100 bytes each)
NRML NAVL v=3 size=921
MRKS size=346
KDOP size=1516
FORM COLD size=84
NRML SCAR ×3
```

**Edited DTRM sub-chunks (buggy output):**
```
NRML HIER v=0 size=1736  (29 joints, first_val=0xFFFFFF00)  ← WRONG type, WRONG first_val
NRML MRKR v=0 size=294  ← spurious, not in original
NRML BURN v=3 size=608  ← consolidated, WRONG structure
NRML NAVL v=3 size=921  ← stale from original, doesn't reflect user's renames
MRKS size=346
KDOP size=1516
FORM COLD size=84
NRML SCAR ×3
```

## Changes Made
1. **HIER Header (`first_val`)**: Changed hardcoded `0xFFFFFF00` to correctly use `0xFFFFFF00 | ((-joint_count) & 0xFF)` in `generate_v2_from_model`.
2. **HIER Chunk Type**: Fixed the chunk serialization type from `ChunkType::Normal` to `ChunkType::Form`.
3. **Joint Transformation Data**: Replaced hardcoded zeroes for translation and rotation with actual values from `joint.position`, `joint.rotation`, and `joint.scale` with fallback matrix decomposition.
4. **BURN Chunks**: Replaced the logic that consolidated all BURN data into one chunk with logic that outputs individual `ChunkType::Default` BURN chunks of 100 bytes each, matching the original format.
5. **NAVL and MRKS Regeneration**: Regenerated NAVL and MRKS data natively instead of preserving stale data from the original. Also fixed a bug where preserving the original DTRM children caused duplicated MRKS chunks.

## Verification Results
- **Compile Success**: Fixed all compiler errors in parser (`dae.rs`, `verify_lossless.rs`) and made `cargo check` pass correctly.
- **DTRM Structure**: Binary dump of `ter_elysium.hod_generated.hod` verified that the `DTRM` sub-chunks perfectly match the desired formatting (`FORM HIER`, multiple individual `BURN`, single `MRKS` and `NAVL` without duplicates).
- **Joint Values**: `verify_lossless` testing successfully read original values from `ter_elysium.hod`, e.g., `EngineNozzle3` `rot=Some(Vector3 { x: 0.0, y: 3.1415927, z: 0.0 })`, and generating the new V2 HOD kept the exact same values.
- **Roundtrip Parsability**: The newly generated HODs cleanly parse in the backend with the identical number of Meshes, Joints, NavLights, Markers, and EngineBurns as the original files.
