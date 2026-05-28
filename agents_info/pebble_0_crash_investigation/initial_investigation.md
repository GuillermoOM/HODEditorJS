# Pebble_0 Initial Investigation [RESOLVED]

> **Status:** Both issues in this document are resolved.
> - HIER crash: fixed by injecting default Root joint in `generate_v2_from_model`.
> - Blurry textures: fixed by adding sRGB, anisotropy, repeat wrapping, linear filtering in `Viewport.tsx`.
> For ongoing work, see `phase_plan.md` and `progress_log.md`.

## Background
The user attempted to recreate the `pebble_0` HOD file from scratch using the HODEditorJS. After importing the two LOD meshes and assigning materials with textures, two distinct issues occurred:
1. **Blurry Textures**: The texture rendering in the editor viewport appeared very low resolution/blurry despite the `.tga` files being full resolution.
2. **Fatal Game Crash (`Found 0 < 1 'HIER' chunks`)**: Upon loading the saved HOD in the Homeworld Remastered engine, the game crashed complaining about a missing or zero-count `HIER` chunk.

---

## 1. Zero-Joint `HIER` Chunk Crash

### Root Cause Analysis
By debugging the `save_edits` function in `parser/src/hod.rs`, we discovered a critical flaw in the chunk injection logic. 
- When the editor creates a "New File" (creating a HOD 2.0 from scratch), it generates a completely empty byte array to pass to the parser as the "original file".
- `save_edits` attempts to synthesize basic container chunks (like `DTRM`) when the original byte array is empty.
- Later in the function, it iterates over the original `DTRM` children to replace them with updated data. If a chunk (like `MRKS` or `NAVL`) isn't found in the original file, it has fallback logic at the end of the loop to append them anyway.
- **The Bug:** `save_edits` **forgot to include fallback logic for the `HIER` chunk!** Because the newly synthesized `DTRM` chunk has zero children, the loop never encounters an original `HIER` chunk to replace, and because there's no fallback `if !hier_written { new_children.push(HIER); }`, the `HIER` chunk is simply never written to the file! This is why creating a HOD from scratch drops the `Root` joint and crashes the game with `Joints: 0`.

### Proposed Fix for the Next Agent
**The goal is to fix creating a HOD 2.0 file from scratch.**
**Modify `parser/src/hod.rs`:**
1. In `save_edits`, add `let mut hier_written = false;` tracking before iterating the `DTRM` chunk children (around line 3729).
2. Inside the `match child.id.as_str() { "HIER" => ... }` block, set `hier_written = true;`.
3. At the end of the `DTRM` loop (around line 3941), add a fallback block:
   ```rust
   if !hier_written {
       new_children.push(IffChunk {
           id: "HIER".to_string(),
           chunk_type: crate::iff::ChunkType::Form,
           version: 0,
           data: hier_data.clone(),
           children: Vec::new(),
       });
   }
   ```
4. As an extra safety measure, if `updated_model.joints.is_empty()` when serializing `hier_data`, automatically inject a default `Root` joint so no HOD ever saves with zero joints.

---

## References for Further Reverse Engineering
If creating a HOD 2.0 file from scratch reveals any additional missing chunks or structural expectations that are not fully documented in the `save_edits` function, the next agent can refer to these original toolsets and binaries for reverse engineering:

1. **HODOR (Homeworld Official DAE to HOD Tool)**:
   - Directory: `/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/HODOR/`
   - Useful for: Generating baseline, 100% correct HOD 2.0 files from DAEs to hex-compare against our `compiler.rs` output when debugging "creation from scratch".

2. **Homeworld Remastered Game Engine**:
   - Directory: `/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HomeworldRM/Bin/Release/`
   - Useful for: Disassembling the engine DLLs (e.g., `HomeworldRM.exe` or `RelicCore.dll`) in tools like Ghidra or IDA Pro if a `FATAL EXIT` stack trace occurs and the exact structural requirement of a specific IFF chunk (like `HIER` or `DTRM`) needs to be definitively proven at the assembly level.

## 2. Blurry Editor Viewport Textures

### Root Cause Analysis
The editor uses `Three.js` to render the `HODModel`. When textures are loaded dynamically via `Image` and mapped into a `THREE.Texture` (see `getCachedTexture` in `Viewport.tsx`), the default filtering settings are applied.
- By default, `Three.js` does not enable `anisotropy` on manually generated textures unless explicitly asked.
- This results in textures looking incredibly muddy or low resolution when viewed at oblique angles, which is extremely noticeable on flat surfaces or spherical meshes like pebbles.

### Proposed Fix
**Modify `src/components/Viewport.tsx`:**
When creating the texture in `getCachedTexture`:
```javascript
const tex = new THREE.Texture(img);
tex.needsUpdate = true;
tex.colorSpace = THREE.SRGBColorSpace;
// Enable maximum anisotropic filtering for crisp rendering at angles
tex.anisotropy = gl.capabilities.getMaxAnisotropy();
tex.minFilter = THREE.LinearMipmapLinearFilter;
tex.magFilter = THREE.LinearFilter;
```
*(Note: A reference to `gl` via `useThree()` needs to be passed down or retrieved in `Viewport.tsx` to get the maximum anisotropy capability).*
