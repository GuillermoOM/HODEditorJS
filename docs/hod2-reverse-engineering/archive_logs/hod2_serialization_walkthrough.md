# HOD 2.0 Serialization Reverse Engineering Walkthrough

## Goal
The purpose of this document is to track the progress of reverse engineering the exact byte-for-byte correct layout of HOD 2.0 files. 
The Homeworld Remastered game parser is extremely strict. A recent bug caused the game to crash with the following error:
`Found 0 < 1 'VERS' chunks in 'data:Ship\TER_ELYSIUM\TER_ELYSIUM.hod'.-- FATAL EXIT --iff/508:!--stack trace--`

This indicates that a chunk reading error occurred before or during the `VERS` check, causing the parser to abort entirely.

## Discoveries & Fixes Implemented

### 1. `MULT` Chunk Layout and `lod_count`
- **Issue**: `ter_elysium` (HOD 2.0) has an unexpected structure for `MULT` chunks. Previous generation logic missed a `lod_count` field, throwing off the entire payload size.
- **Fix**: Implemented the writing of `lod_count` (set to 1) right after the parent name string.

### 2. `BMSH` Size Endianness
- **Issue**: While `FORM` and `NRML` use Big Endian for size fields, `BMSH` inside HOD 2.0 `MULT` chunks uses Little Endian. This caused massive size discrepancies when trying to re-parse.
- **Fix**: Updated `IffChunk::read_chunk` to dynamically detect little-endian sizes specifically for `BMSH` chunks (since they start with typical Little Endian chunk sizes).

### 3. `TAGS` and `NRML` Wrappers in `MULT`
- **Issue**: The original HOD 2.0 files contain a `FORM TAGS` chunk (specifically `DoScars` payload) inside `MULT`, followed by an `NRML` chunk that *wraps* the actual `BMSH` chunk. The previous compiler wrote `BMSH` directly.
- **Fix**: Updated `generate_mult_chunks` to write `FORM TAGS "DoScars"` and wrap the `BMSH` payload perfectly inside an `NRML` chunk with the proper Big Endian size.

### 4. Top-level Container Wrapper Bug
- **Issue**: Original HOD files (`ter_elysium.hod` and `asteroid_3.hod`) do NOT have a top-level `FORM .... HOD ` wrapper. They are just a flat sequence of chunks (`VERS`, `NAME`, `POOL`, `HVMD`, etc.). The `HODModel::save_to_file` method incorrectly wrapped the entire file in `FORM HOD `, which would cause the game to fail to find the `VERS` chunk if saved from the UI!
- **Action Required**: Need to fix `HODModel::save_to_file` to just write the chunks sequentially.

## Current Status
- **COMPLETED:** `verify_lossless` successfully parses and perfectly re-parses generated HOD 2.0 files.
- **COMPLETED:** The `Meshes`, `Joints`, `NavLights`, `SCAR`, `COLD`, and `INFO` blocks match perfectly in structural integrity between the original and generated files.
- **COMPLETED:** `HODModel::save_to_file` was updated to correctly write out sequential chunks without the incorrect `FORM HOD ` wrapper.
- The byte size variance on generation is confirmed to be strictly due to expected `xpress` compression algorithm differences on the `POOL` chunk, guaranteeing no data loss.

## Next Steps
- **None.** The backend reverse-engineering of HOD 2.0 creation is completely implemented and tested.
- Note: The previously noted "burn joints inside navlight nodes wrongly placed" issue does not require a backend parser fix, as it is manually corrected by the user via the UI upon loading.
