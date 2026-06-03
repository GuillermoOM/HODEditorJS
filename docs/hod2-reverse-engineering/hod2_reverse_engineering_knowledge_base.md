# HOD 2.0 Reverse Engineering Knowledge Base

This document serves as the central knowledge repository for the effort to reverse engineer and implement parsing and serialization for Homeworld Remastered HOD 2.0 files. Any future agent or developer picking up this project should refer to this document for context, methodology, and testing requirements.

## 1. Project Goal
The primary objective of this effort was to evolve the tool into a fully functional HOD editor capable of modifying and reliably re-serializing HOD 2.0 files without crashing the Homeworld Remastered game engine. Prior to this effort, saving a HOD 2.0 file resulted in corrupted output chunks that caused `Found 0 < 1 'VERS' chunks` crashes or completely wiped mesh geometries upon loading.

## 2. Key Discoveries & Technical Knowledge
Throughout the reverse engineering process, several critical quirks regarding the HOD 2.0 architecture were identified and addressed:

* **Flat File Structure**: Unlike standard IFF files or HOD 1.0, HOD 2.0 files **are not** wrapped inside a top-level `FORM` container. The file consists of a flat, sequential list of base IFF chunks starting directly with `FORM ... VERS` and `FORM ... NAME`. Wrapping the file in a top-level `FORM HOD ` causes engine crashes.
* **Endianness Inconsistencies**: 
  * Standard IFF chunk headers and sizes (e.g., `VERS`, `NAME`, `HVMD`) use **Big-Endian**.
  * However, inner binary payload data and specifically `BMSH` (Basic Mesh) structures utilize **Little-Endian** formatting. The `VERS` chunk payload is Little-Endian (e.g., version 512 is `[0x00, 0x02, 0x00, 0x00]`).
* **The `NAME` Chunk & Byte Alignment**:
  * In HOD 2.0, the `NAME` chunk is formatted as a flat `FORM` container whose payload is exactly the character length of the model name string (e.g., exactly 26 bytes for `Homeworld2 Multi Mesh File`). **Do not append a trailing null byte (`\0`)** when serializing it. Doing so will shift the byte-alignment of all subsequent chunks (like `POOL` and `DTRM`), silently corrupting the parser in the game engine and causing immediate crashes upon load.
* **Node Hierarchy & Cascade Deletions**:
  * Joints, `engine_burns`, `nav_lights`, `engine_glows`, and `dockpaths` are structurally intertwined via the `parent_name` property (or just `name` for navlights). If a parent joint is deleted in the UI or modified, **all dependent child nodes must be cascade-deleted** (or explicitly re-parented). Leaving a dependent node pointing to a non-existent parent joint will cause the game engine to crash silently.
* **The `MULT` Chunk & Sub-Meshes**:
  * The `MULT` container is highly volatile. It begins with a custom binary payload: `name` (u32 length prefix + string), `parent_name` (u32 length prefix + string), and `lod_count` (u32 Little-Endian).
  * Following this payload, it expects `NRML`-wrapped `BMSH` children. Some HOD 2.0 `MULT` chunks also include a leading `FORM TAGS` child (`DoScars`), but this is not universal: original `pebble_0.hod` omits it while `ter_elysium.hod` and `ter_fenris.hod` include it. Preserve and round-trip whether each parsed mesh had `FORM TAGS`; do not unconditionally synthesize it for from-scratch pebble-style meshes.
  * **`FORM TAGS` Size**: The `FORM TAGS` chunk (which holds variables like `DoScars`) uses a payload size of 15 bytes in original HOD 2.0 files: real id `TAGS` + little-endian string length + `DoScars`. Do not include a counted padding byte in this size; doing so makes generated `MULT` payloads one byte larger than the originals and shifts the following `NRML` data.
  * **`NRML` Wrappers**: In HOD 2.0, the `BMSH` mesh data chunk does not exist on its own. It is the payload *inside* a normal chunk designated as `NRML`.
* **POOL Compression**: HOD 2.0 files store all mesh geometries, vertices, faces, and textures inside a giant `POOL` chunk compressed using Microsoft's `xpress` compression algorithm. 
  * *Xpress Compression Limits*: The Homeworld Remastered internal Xpress decompressor appears to fail or corrupt memory if fed a fully uncompressed stream disguised as an Xpress chunk, or if blocks exceed standard MS Xpress 64KB chunk boundaries without proper framing. Proper match-copy emission is required.
  * *Note on Size Parity*: A generated `POOL` chunk will rarely match the byte-for-byte size of the original due to varying block compression ratios. Structural integrity is verified by ensuring the decompressed meshes parse correctly, not by comparing exact file sizes.
* **Normal.W Handedness**:
  * The 4th component of the Normal vector (`Normal.W`) in `0x600B` vertices MUST be set to `1.0` (or the appropriate Tangent Handedness sign). If set to `0.0`, the vertex shader calculates a singular TBN matrix leading to NaNs and a massive vertex explosion in-game.
* **HOD 1.0 Retrofit Caveats**:
  * HOD 1.0 `BMSH` parts can contain multiple primitive groups after the vertex buffer. Do not read only one `(prim_type, index_count, indices)` block; consume all groups or the cursor desynchronizes and later parts fail with `failed to fill whole buffer` (observed on `ter_zephyrus.hod`).
  * **Texture Orientation & DXT Block Order**: HOD 2.0 `POOL` chunks store DXT-compressed textures in **top-down** block order (block row 0 = image top). Our DXT decompressor (`decompress_dxt1`/`decompress_dxt5`) writes pixels at `pixel_offset = (py * width + px) * 4` with `py = by * 4 + y`, producing top-down RGBA that matches standard image convention. **No flip is needed** when compressing top-down RGBA back to DXT for the POOL. The frontend uses `tex.flipY = true` (Viewport.tsx:1288), which makes Three.js flip the top-down PNG to bottom-up on GPU upload — matching the DirectX rendering convention. For the editor UI thumbnail previews (hierarchy tree, inspector panels), flip the RGBA before PNG encoding so they display correctly as standard images. **Critical pitfall**: `encode_b64_png_thumbnail` must NOT flip internally — it previously did, causing a double-flip when combined with `flipY = true`.
  * **HOD 1.0 Inline LMIP/TEXM Mip Dimensions**: Legacy HOD 1.0 inline texture chunks store only the base texture width/height before the compressed mip byte stream. They do **not** include `(width,height)` pairs for every remaining mip level. HOD 2.0 POOL-backed LMIP chunks do include per-mip dimensions in the LMIP header. Therefore `parse_texture` must skip remaining mip dimensions only when `context.is_v2 == true`; doing so for HOD 1.0 shifts the inline DXT byte cursor into compressed data by 8 bytes per extra mip, corrupting DXT5 alpha/color data and also subtly corrupting multi-mip DXT1 textures.
  * **HIER `sx, sy, sz` Fields Are Bounds, Not Scale**: The three floats after rotation in the HIER chunk per-joint record are **vector bounds** (gimbal limits or joint extents), NOT scale multipliers. Writing actual scale values (or the parsed bounds) into these fields causes the game engine to misinterpret them as scale, corrupting joint transforms and rotating the ship away from its forward vector. Always write `(1.0, 1.0, 1.0)` for these fields on save. The `compose_transform_matrix` function should also use `(1.0, 1.0, 1.0)` for scale when building `local_transform` from parsed HIER data.
  * HOD 1.0 `DOCK` chunks are valid editor data and must not be skipped just because `context.is_v2 == false`. Use legacy/extended layout fallback because observed ship files can include extra path metadata such as compatible ship strings.
  * HOD 1.0 files may use companion `.mad` files for animation even when embedded `MRKR/KEYF` chunks are empty. Load the companion MAD before falling back to embedded KEYF.
* **Non-Essential Chunks**:
  * `SCAR` chunks: Relate to visual battle scars; rebuilding these is not strictly required.
  * `Collision Mesh` (`COLD`/`KDOP`): HOD 2.0 ships can store the renderable collision hull in DTRM `KDOP` while the following `COLD` chunk is only a name payload such as `u32 len + "Root"`. Do not treat a tiny name-only `COLD` as an empty collision mesh; associate it with the preceding KDOP data for editor COL node display. KDOP uses a 444-byte header: `radius + min_extents + max_extents` (7 floats / 28 bytes), then 13 direction records of 8 floats each (416 bytes), followed by `vertex_count(u32)`, `vertex_count * vec3(f32)`, `face_count(u32)`, `face_count * 3*u16`, and usually 8 trailing padding bytes.
  * `Engine Burns` (`BURN`): In HOD 2.0, BURN chunks are stored individually inside DTRM as `ChunkType::Default` (100 bytes each). Do not consolidate them into a single `NRML BURN` chunk.
* **DTRM Serialization (HOD 2.0)**:
  * The `HIER` chunk inside `DTRM` must be `ChunkType::Form` (`FORM HIER`), not `NRML`.
  * The `first_val` in the `HIER` chunk encodes the full signed negative joint count as a two's-complement `i32` reinterpreted as `u32` (e.g., 298 joints -> `-298` -> `0xFFFFFED6`). Do not mask to the low byte with `0xFFFFFF00 | ((-joint_count) & 0xFF)`; that only works for counts up to 255 and breaks large HIER chunks.
  * Preserving original DTRM sub-chunks (`MRKS`, `KDOP`, `COLD`, `SCAR`) when re-saving is fine, but you MUST exclude chunks you actively regenerate (`HIER`, `BURN`, `NAVL`, `MRKS`, `MRKR`) to avoid duplicating them in the output file, which can double marker counts and corrupt node mapping.
* **Animation Rotation & Euler Interpolation**:
  * HODKeyframes map rotations using both Quaternions (`rotation`) and Euler angles (`rotation_euler`, usually YXZ order).
  * **Critical UI Trap:** Quaternions cannot natively represent a rotation greater than 180 degrees (they take the shortest path). If a user inputs a 360-degree rotation (or continuous spin) in the UI, converting it to a Quaternion and back to Euler will wrap the value (e.g., 360 -> 0). The UI must always read from and display `kf.rotation_euler` (converted to degrees) if available, bypassing the quaternion decomposition, to allow animations that exceed 360 degrees or continuous spinning.

## 3. Testing Methodology
All new serialization techniques, chunk modifications, or parsing updates **MUST** be verified against a predefined suite of test files. 

### The `verify_lossless` Script
A dedicated testing environment is set up at `parser/src/bin/verify_lossless.rs`. 
Run it using:
```bash
cargo run --bin verify_lossless
```
**Mechanism:** The script reads original HOD files natively into memory (`HODModel::parse`), re-compiles them using our custom backend (`generate_v2_from_model`), saves them to `<filename>_generated.hod`, and immediately parses the *newly generated bytes* back into memory to confirm that all structural hierarchies (Mesh counts, Joints, NavLights) survive the round-trip without loss.

### Mandatory Test Suite Files
When making changes, ensure `verify_lossless` successfully evaluates the following core models:
1. **HOD 2.0 Standard Fallback Test (`pebble_0.hod`)**:
   `/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/uncompressed_bigs/HWRM/pebble/pebble_0/pebble_0.hod`
2. **HOD 2.0 Multi-Mesh Test (`ter_elysium.hod`)**:
   `/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/ship/ter_elysium/ter_elysium.hod`
3. **HOD 2.0 Complex Animation Test (`ter_fenris.hod`)**:
   `/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/ship/ter_fenris/ter_fenris.hod`
4. **HOD 1.0 Retro-compatibility Test (`asteroid_3.hod`)**:
   `/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/uncompressed_bigs/freespace_remastered/resource/asteroid/asteroid_3/asteroid_3.hod`
5. **.DAE Fallback parsing test (`galaxymapgalaxy.dae`)**:
   `/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/uncompressed_bigs/HWRM/effect/galaxymap/hodsunpacked/mapgalaxy/galaxymapgalaxy.dae`

*Rule of Thumb: The script output `Reparsed: Meshes=X` must evaluate to a number greater than 0, and match the original mesh counts as closely as possible.*

## 4. Recommendations for Future Agents
* **Verify First, Code Second**: If you encounter a bug during HOD generation, do not blindly rewrite chunk data. Run `verify_lossless` to reproduce it. Once reproduced, use terminal hex dumps or inline `println!` statements of the binary stream (around the `Cursor` position) to find out exactly where the structural misalignment happens.
* **Reference HODOR Scripts**: If you are stuck trying to map out how a specific chunk behaves, inspect the original `HODOR`/`RODOH` conversion scripts provided by Gearbox:
  `/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/HODOR/`
* **Maintain the Knowledge Base**: This document and the `hod2_serialization_walkthrough.md` file should be treated as living artifacts. Update them whenever new quirks about the HOD 2.0 file format are uncovered.
