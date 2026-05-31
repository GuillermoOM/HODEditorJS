# HODEditorJS Gap Fix Implementation Plan

This document outlines the actionable, step-by-step project plan to fix the critical and medium gaps preventing HODEditorJS from fully substituting the `DAEnerys` -> `HODOR` pipeline. 

Based on the Pipeline Audit, the critical gaps are:
1. **No OBJ Import:** Requires a backend parser.
2. **No TGA → DDS Conversion:** Missing DDS compression and mipmaps.
3. **Texture Slot Resolution:** DAE import only maps `_DIFF`, leaving `_GLOW`, `_NORM`, etc., missing.
4. **Multi-channel Packing:** Packing `REFL`+`GLOW`+`SPEC` into the `GLOW` slot per shader type.

---

## Phase 1: Enabling Complete DAE Texture Import (Critical)
*Focus: Ensure DAE imported ships render correctly by resolving all textures and packing them properly.*

### Step 1.1: Extended Texture Slot Resolution
- **Goal:** Update `parser/src/dae.rs` so that when it finds a `_DIFF` texture, it searches the directory for the 18 other possible DAEnerys texture suffixes (e.g., `_GLOW`, `_SPEC`, `_TEAM`, `_NORM`).
- **Action:** 
  - Add logic in `parse_dae` texture handling to probe the filesystem for matching files.
  - Store these paths in an extended `HODMaterial` struct (add `glow_path`, `spec_path`, `team_path`, etc., to `HODMaterial` in `hod.rs`).

### Step 1.2: Shader-Specific Multi-Channel Packing
- **Goal:** Emulate the packing rules found in DAEnerys (`HWMaterial.cs`).
- **Action:**
  - Create a new module `parser/src/texture_packer.rs`.
  - Depending on the shader (e.g., `ship`, `thruster`), read the raw RGBA pixels from the discovered TGAs.
  - Pack them into new raw RGBA buffers (e.g., `GLOW` map = `REFL(R)` + `GLOW(G)` + `SPEC(B)` + `Alpha(A)`).
  - Pass the packed raw buffers forward to the DDS converter.

---

## Phase 2: TGA → DDS Conversion Pipeline (Critical)
*Focus: Ensure textures embedded in HOD files are valid DXT compressed DDS with Mipmaps.*

### Step 2.1: Implement DXT Compression & Mipmaps
- **Goal:** Convert raw RGBA buffers into DXT1 or DXT5 compressed DDS blobs with mipmap chains.
- **Action:**
  - Add `image-dds` (or `squish` / `intel-tex-rs-2` + custom header) to `parser/Cargo.toml`.
  - Create `parser/src/dds_encode.rs`.
  - Implement a function that takes raw RGBA pixels, generates mipmaps down to 1x1, compresses each level via DXT5 (or DXT1 if no alpha), and writes a standard DDS header.

### Step 2.2: Integrate DDS into HOD Save
- **Goal:** Write the compressed DDS bytes into the `TEXR` chunks instead of storing raw PNG or failing.
- **Action:**
  - Update `src-tauri/src/lib.rs` and `hod.rs` texture pipeline.
  - Make sure that when `generate_v2_from_model` writes the POOL, it uses the generated DDS blobs and compresses them with Xpress.

---

## Phase 3: OBJ Import Capability (Critical)
*Focus: Allow users to bring raw OBJ files into the tool, bypassing DAEnerys entirely.*

### Step 3.1: Rust Backend OBJ/MTL Parser
- **Goal:** Parse OBJ geometry and materials.
- **Action:**
  - Add the `tobj` crate to `parser/Cargo.toml` (handles OBJ + MTL efficiently, triangulates faces).
  - Create `parser/src/obj.rs`.
  - Parse the OBJ. Triangulate faces and group them by material (similar to what DAEnerys does via Assimp).

### Step 3.2: Tauri IPC Command & UI Integration
- **Goal:** Wire the backend parser to the frontend.
- **Action:**
  - Create `import_obj_file` in `src-tauri/src/lib.rs`.
  - Update frontend (`Inspector.tsx`, `App.tsx` or toolbar) to call `import_obj_file` which returns a complete `HODModel`.
  - Ensure the imported OBJ model generates default joints and proper scene graph structure (e.g., `Root` -> `Root_mesh`).

### Step 3.3: Mesh Processing
- **Goal:** Compute Tangents, apply vertex deduplication, and handle multi-indexed formats.
- **Action:**
  - Ensure the `compiler.rs` logic cleanly accepts the output of the `obj.rs` parser, applies `compile_hodor_style_tangent_part()`, and generates collision geometry automatically.

---

## Phase 4: Polish & Secondary Gaps (Medium)
- **Step 4.1:** Mesh auto-splitting. Add logic in `compiler.rs` or `dae.rs` / `obj.rs` to detect if a mesh has > 65,535 vertices and split it automatically into multiple parts.
- **Step 4.2:** Fix the phantom `nameplate.bmp` material bug in `dae.rs` to eliminate the extra `STAT` chunk.
- **Step 4.3:** Resume SCAR (battle scars) chunk generation reverse engineering.
