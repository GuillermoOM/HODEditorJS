# Goal Description

Finish the reverse-engineered `generate_v2_from_model` to make it a fully lossless editor for HOD 2.0 files. This requires preserving the unparsed/unsupported chunks (like `KDOP`, `COLD`, `SCAR`, `INFO`) and perfectly re-compressing the original texture payload back into the new `POOL` chunk.

## Ultimate Goal of the project

This app is going to be a full Homeworld 2 Remastered HOD 2.0 editor/creator, aside from the current functionalities, it should be able to:
- Load an HOD 2.0 file at full and correctly interpret it (should be already done).
- On a save, it will create a new HOD 2.0 from scratch (reversed engineered logic) and must be precise (loading an existing HOD 2.0 file and saving it should output a new file created from scratch by the app that is identical, not just patch it).
- Create a HOD 2.0 file from scratch as a proper editor for new ships creation after saving.
- Load HOD 1.0 and process them as a new HOD 2.0 file (editor reflects the loaded data on any HOD type or DAE files, as a base data structure).
- Load / import .DAE files compatible that reflect a HOD correctly.

### Permanent Context & Reference Data
- **SCAR:** Represents battle scars in Homeworld 2 Remastered.
- **Collision Mesh (COLD/KDOP):** A special node that isn't strictly necessary for a HOD to run, it is optionally added into it.
- **HODOR/RODOH scripts:** Found at `/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/HODOR/`. These are the official tools that convert HOD 1.0 -> .DAE -> HOD 2.0. We are **reverse engineering** the creation of HOD 2.0 files.

- Create a dedicated walkthrough file (specific naming to the plan) where you add all steps currently done, in case of any interruptions that a new agent has to take over to continue the work seamlessly. This is important else the new agent will end up working on something else. "hod2_serialization_walkthrough.md"

## Proposed Changes

### Backend Compiler (parser/src/hod.rs)

#### [MODIFY] `hod.rs`
- **Modify `generate_v2_from_model` Signature:** Update it to `pub fn generate_v2_from_model(original_bytes: &[u8], model: &HODModel) -> Result<Vec<u8>, String>`.
- **Extract Textures:** At the start of the function, parse `original_bytes` (if not empty) using `IffChunk::read_chunk`. Locate the original `POOL` chunk, decompress it using `xpress::decompress`, and extract the exact `original_texture_pool` payload. Pass this into `generate_pool_data` instead of an empty buffer.
- **Preserve Unparsed Chunks:** 
  - Parse the tree of `original_bytes`.
  - For the **DTRM** container: Instead of manually creating a new `DTRM` chunk with only `HIER`, `MRKR`, and `BURN`, iterate over the original `DTRM` children. Replace the matched chunks (`HIER`, `MRKR`, `BURN`) with the newly compiled ones from `model`, and append all other unrecognized chunks (`KDOP`, `COLD`, `SCAR`, `NAVL`, etc.) exactly as they were.
  - For the **HVMD** container: Do the same, replacing `STAT` and `MULT` but preserving others.
  - For the **Root** container: Replace `VERS`, `NAME`, `POOL`, `HVMD`, and `DTRM`, but push any remaining chunks (like `INFO`) to the end.

### Tauri App (src-tauri/src/lib.rs)

#### [MODIFY] `lib.rs`
- **Update `save_hod` & `save_hod_as`:** Update the calls to pass `&original_bytes` into `hwr_hod_parser::hod::generate_v2_from_model(&original_bytes, &model)`. If the file is a newly imported DAE, `original_bytes` will be safely empty.

## Verification Plan

Run verification scripts that parse the file, reserialize it without edits, and verify that the output perfectly matches the original file format, contains the exact same bytes for untouched chunks, and maintains the full file size (lossless texture/collision mesh preservation).

### Automated Tests
I will build Rust bin targets (or update `test_pebble.rs`) to load and reserialize these files, verifying the outputs via hex comparisons (`cmp` and `hexdump`):

1. **HOD 2.0 (pebble_0)**: `/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/uncompressed_bigs/HWRM/pebble/pebble_0/pebble_0.hod`
   - *Test Focus*: Verify texture pool recompression matches original byte size and that `KDOP`/`INFO` chunks are preserved. Output must be equal to the input.
2. **HOD 2.0 (ter_elysium)**: `/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/ship/ter_elysium/ter_elysium.hod`
   - *Test Focus*: Ensure complex multi-mesh, multi-material ships with `SCAR` and `COLD` chunks reserialize losslessly. Output must be equal to the input.
3. **HOD 1.0 (asteroid_3)**: `/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/uncompressed_bigs/freespace_remastered/resource/asteroid/asteroid_3/asteroid_3.hod`
   - *Test Focus*: Ensure HOD 1.0 files trigger `save_edits` properly and still save losslessly without disruption from the V2 compiler changes.
   **HOD 1.0 (other HOD 1.0 files to use for further testing)** `/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/uncompressed_bigs/freespace_remastered/ship/`
4. **.DAE (ships converted from HOD 1.0 to .DAE)**: `/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/current_project_processing/ship_converted/`
   - *Test Focus*: Ensure the app handles creating a fresh HOD file from an empty `original_bytes` buffer without crashing.

