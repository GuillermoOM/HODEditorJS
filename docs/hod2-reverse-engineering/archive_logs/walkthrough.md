# Walkthrough - Homeworld HOD Vertex Rendering, Multi-Joint Assembly Templates, and Drag-and-Drop Reparenting

This document outlines the visual rendering resolution, advanced skeletal assembly architectures, structural validation engines, and interactive drag-and-drop enhancements implemented in the Homeworld Remastered HOD Editor.

---

## 1. Core 3D Viewport Bug: Bounding Box and Zoom Inflation

During deep analysis of `ter_orion.hod`'s skinned meshes, we discovered why joints were rendering in the sidebar tree but the 3D canvas remained empty/invisible:

1. **Extreme Vertex Coordinates Outliers**: The skinned mesh vertex pools contain skeletal dummy/anchor vertices with raw file stream uninitialized or non-geometric coordinates ranging from $10^{11}$ to $10^{38}$ units.
2. **Static Render Limitations**: The Three.js interactive previewer renders meshes statically. Without the GPU bone-skinning matrices applied to bring these offset coordinates back into world space, these outliers remain at their astronomical values.
3. **Camera Fitting Auto-Collapse**: The `Viewport.tsx` calculates the 3D bounding box of the ship using these indices. Because of the astronomical coordinates, the bounding box dimensions expanded to $10^{38}$ units. The auto-focus script then positioned the camera at a distance of $10^{38}$ units to attempt to fit the entire volume. At this distance, the actual ship geometry (100–1500 units) shrank to less than a single sub-pixel, making the viewport look completely blank.

* **Resolution**: Any coordinate whose magnitude exceeds `10,000.0` units (or is NaN/non-finite) is automatically sanitized on load to `0.0`. Normal vectors with invalid length ranges are reconstructed mathematically.

---

## 2. Advanced Multi-Joint Assembly Templates

We designed and extended `src/components/HierarchyTree.tsx` (`handleAddNode`) to support creating multi-joint linked families with precise **parent-child nesting** and **`+5.0` local coordinate offsets** along primary defining axes:

*   **Hardpoint (`hardpoint_template`):**
    *   `Hardpoint_name_Position` (Pivot) $\rightarrow$ `_Direction` at `+5.0y` (`[0, 5, 0, 1]`) and `_Rest` at `+5.0z` (`[0, 0, 5, 1]`) parented directly under `Position`.
*   **Capture Point (`capture_point_template`):**
    *   `CapturePoint#` (Base) $\rightarrow$ `Heading` at `+5.0z` (`[0, 0, 5, 1]`), `Left` at `+5.0z` (`[0, 0, 5, 1]`), and `Up` at `+5.0y` (`[0, 5, 0, 1]`) parented under `CapturePoint#`.
*   **Repair Point (`repair_point_template`):**
    *   `RepairPoint#` (Base) $\rightarrow$ `Heading` at `+5.0z` (`[0, 0, 5, 1]`), `Left` at `+5.0x` (`[5, 0, 0, 1]`), and `Up` at `+5.0y` (`[0, 5, 0, 1]`) parented under `RepairPoint#`.
*   **Salvage Point (`salvage_point_template`):**
    *   `SalvagePoint#` (Base) $\rightarrow$ `Heading` at `+5.0z` (`[0, 0, 5, 1]`), `Left` at `+5.0z` (`[0, 0, 5, 1]`), and `Up` at `+5.0y` (`[0, 5, 0, 1]`) parented under `SalvagePoint#`.
*   **Weapon Turret (`turret_template`):**
    *   `Weapon_name_Position` $\rightarrow$ `_Direction` at `+5.0y` (`[0, 5, 0, 1]`), `_Latitude` at `+5.0z` (`[0, 0, 5, 1]`) under `Position`, `_Barrel` at `+5.0y` under `Latitude`, `_Muzzle` at `+5.0y` under `Barrel`, and `_Rest` at `+5.0z` under `Position`.
*   **Weapon Gimbal (`weapon_template`):**
    *   `Weapon_name_Position` $\rightarrow$ `_Direction` at `+5.0y` (`[0, 5, 0, 1]`), `_Muzzle` at `+5.0z` (`[0, 0, 5, 1]`), and `_Rest` at `+5.0z` (`[0, 0, 5, 1]`) parented under `Position`.

---

## 3. Naming Conventions (0-Indexed, No Underscores for Hash Nodes)

The creation modals automatically pre-fill suggested names utilizing modern 0-indexed count logic without underscores for hash symbols, supporting:
*   `Weapon_name`, `Hardpoint_name`
*   `NavLight#` $\rightarrow$ `NavLight0`, `NavLight1` (no underscores)
*   `marker#` $\rightarrow$ `marker0`, `marker1` (lowercase, no underscores)
*   `EngineShape#`, `EngineNozzle#`, `EngineGlow#`, `EngineBurn#` (no underscores)
*   `Root_mesh (LOD #)` $\rightarrow$ `Root_mesh_LOD0`
*   `EngineGlow#_LOD#` $\rightarrow$ `EngineGlow0_LOD0`

---

## 4. Tree Group Folder Rendering & Suffix Validation Warnings

*   **Group Folding Visualizer**: Hardpoints, Capture Points, Repair Points, and Salvage Points render as unified collapsible group folders in the tree view (matching the visual layout used for weapons and turrets). This prevents accidental joint deletion or broken parent-child hierarchies.
*   **getWarnings() Validation**: The automated error-checking pipeline validates the structural integrity of all 6 joint group templates, verifying that required component joints are present, correctly linked, and parented. If any structural flaws are found, warning messages and alert badges are rendered.

---

## 5. Group Inspector, Mass-Rename, and Self-Healing Repair Tool

*   **Point-Group Inspector UI**: Selecting any point-group displays a dedicated inspector layout with validation status boxes (Complete/Incomplete) and lists of component joint status cards. Missing component joints are visually flagged with red-dashed borders.
*   **One-Click "Repair Structure"**: Modders can repair invalid assemblies with a single click. The tool automatically recreates missing joint entities, restores their correct parent linkages, and snaps their transformations back to standard `+5.0` offsets.
*   **Group Mass-Rename**: Renaming a point-group automatically propagates the rename across all of its component joints, markers, and meshes simultaneously (`oldName_suffix` $\rightarrow$ `newName_suffix`), preventing broken linkages in the engine.

---

## 6. Drag-and-Drop NavLight Reparenting

*   **Drag-and-Drop Support**: Allowed NavLight nodes in both standard hierarchy and root lists in the sidebar to be fully draggable.
*   **Automated Backing Joint Sync**: Dragging and dropping a NavLight node onto a joint triggers `handleReParentNode`, updating the parent of the underlying joint in `model.joints` with the exact same name. This keeps the physical light source and its skeletal anchor perfectly aligned in 3D space!

---

## 7. MS XPress LZ77 Compressor Bug Resolving (HOD v2.0 Saved Rotations Resolution)

Modders discovered that saving a HOD v2.0 ship model (e.g. `ter_elysium.hod`) and loading it in-game would result in the ship rendering rotated 90 degrees left and tilting 30 degrees, even though its parsed bone matrices matched the original perfectly. We traced this to corruption in the compressed mesh and texture data streams inside the `POOL` chunk due to three critical bugs in the MS XPress LZ77 encoder:

1. **Incorrect Bit Grouping Limit**: The XPress compressor bitstream grouped code blocks every **32 bits**, while the game engine's decompressor parsed indicators every **31 bits** (as it sets `indicator_bit = 0` on entering the `if indicator_bit == 31` branch on the 31st loop iteration). This off-by-one difference caused the decompressor to treat raw data bytes as indicator blocks, resulting in desynchronization and corrupted texture/mesh streams.
2. **Offset Bit Shift Bug**: During 2-byte match copy encoding, the bottom 2 bits of the match offset were merged directly via bitwise OR (`(best_offset & 3) | 0b10`) instead of being shifted left by 6 bits (`((best_offset & 3) << 6) | 0b10`), which corrupted the offset values and caused incorrect vertex stream references.
3. **Off-by-One Distance Offset**: The compressor calculated match offset distances as `input_idx - start - 1` instead of `input_idx - start`, resulting in off-by-one pixel replication errors on decompression.

* **Resolution**: Corrected the compressor to group bits every 31 bits, applied the correct bit-shift of 6 for the 2-byte match copy offset, and used direct distance offsets. The entire roundtrip is now validated 100% mathematically correct by Cargo unit tests.

---

## 8. HOD 1.0 Materials and Texture Parsing

Opened legacy HOD 1.0 ships (e.g., `shi_cain.hod`) previously failed to load or display materials or textures in the frontend canvas:

1. **Sequential Parsing Dependency**: The `HVMD` sub-chunks are parsed sequentially in the file. HOD 1.0 models write their `STAT` material chunks *before* the `LMIP` texture chunks. When `parse_stat_material` was called on the `STAT` chunk, the `textures` array was empty, resulting in unsuccessful texture mappings.
2. **Ignored Legacy Chunks**: The `LMIP` texture chunks and `STAT` material chunks had strict `if context.is_v2` guards, meaning they were completely skipped when loading a legacy HOD 1.0 file.
3. **Single Mip Dimension Header**: HOD 1.0 `LMIP` chunks only store the width and height of Level 0 in their headers, followed directly by the raw pixel streams of all mips, rather than explicitly enumerating the dimensions of all subsequent levels. This caused arithmetic integer overflow panics when attempting to read sub-levels.

* **Resolution**: Rebuilt `HVMD` parsing with a **two-pass sequential reader** (Pass 1 loads all textures from `LMIP` and `TEXM` blocks; Pass 2 loads all materials from `STAT` and `MATT` blocks and maps them to the resolved textures). Programmatically computed subsequent mip levels by dividing dimensions recursively for HOD 1.0. Legacy ships now render perfectly with full previews, base64 previews, and materials!

---

## 9. Absolute Bone and Marker Rotation & Scale Preservation (Phase 1 Final Resolution)

Modders discovered that saving a HOD v2.0 ship model (e.g. `ter_elysium.hod`) and loading it in-game would cause the entire ship and its skeletal bones to render rotated 90 degrees left and tilted 30 degrees. We discovered that while the parsed 3D matrices were mathematically correct, Homeworld Remastered reads the raw float parameters (`px, py, pz, rx, ry, rz, sx, sy, sz` for joints and `px, py, pz, rx, ry, rz` for markers) directly from the `HIER` and `MRKS` chunks instead of recomposing them from a 4x4 matrix. 

Because Euler angle representations are non-unique and matrix-to-Euler decomposition is lossy, converting the file's original floats into a 4x4 matrix and then decomposing it back upon serialization produced different Euler angle sets and scale values (e.g., converting scale `0.0` to `1.0` or changing rotation `[0.0, 3.1415927, 0.0]` to `[3.1415927, 0.0, 3.1415927]`). The game engine composes these floats using its own internal conventions, leading to corrupted, tilted joint hierarchies in-game.

* **Resolution**: Extended the `HODJoint` and `HODMarker` structures with optional raw float tracking vectors (`position`, `rotation`, and `scale` for joints; `rotation_euler` for markers). During parsing, these raw floats are preserved exactly as read from the binary streams. During serialization, `save_edits` bypasses matrix decomposition for all unmodified joints and markers and serializes the exact raw floats back.
* **Result**: Achieved **100% byte-for-byte identical roundtripping** for both `HIER` and `MRKS` chunks on unmodified models, guaranteeing flawless visual alignment and zero orientation corruption in-game.

---

## 10. Unified Animation Engine & Companion MAD Serialization (Phase 3 Resolution)

With Phase 3 complete, we have enabled full loading, editing, playback visualization, and complete serialization of skeleton and joint animations:

1. **Pure-Rust Animation Data Pipeline**:
   - Implemented binary companion `.mad` file decoding in `parser/src/hod.rs`, mapping nine Euler YXZ channels of joint movement tracks to unified high-level `HODAnimation` track descriptors containing quaternion rotations, position offsets, and scale coordinates.
   - Restored classic HOD 1.0 embedded keyframe marker parsing (`MRKR` -> `KEYF` -> `ANIM` IFF chunks), synthesizing them into unified animation tracks mapping to keyframe timestamps.
2. **Mathematically Precise Roundtrip Conversions**:
   - Coded mathematically precise conversions between Three.js quaternions and Euler YXZ radians matching the Homeworld game engine's rotation Palace/conventions.
   - Recreated natural linear and cubic tangent interpolation vectors for exported binary keyframe datasets to ensure buttery-smooth skeletal movement in-game.
3. **Frontend-to-Backend State Synchronization**:
   - Connected React timeline playback loops and gizmo keyframe addition buttons in `src/components/Viewport.tsx` to the parent application's model controller state using `onModelChange={updateModel}`.
   - Clicking "Save" or "Save As" automatically triggers the Tauri backend commands to write/patch the corresponding `.mad` companion file or embed the modified keyframe streams into legacy HOD 1.0 markers.
4. **Mesh Geometry Animation Propagation** *(Phase 3 Polish)*:
   - **Root Cause Identified**: The Viewport.tsx previously rendered ship meshes as flat, position-baked objects in `meshesGroup` (using `applyMatrix4` to snapshot the joint world matrix at load time). When `evaluateAnimation` animated joint sphere markers in `jointsGroup`, the ship geometry remained static.
   - **Resolution**: Two complementary changes were made:
     - At mesh creation time: each `THREE.Mesh` object now stores `userData.parentJointName` (the HOD joint bone name) and `userData.baseJointMatrix` (the original baked world matrix snapshot).
     - In `evaluateAnimation`: after computing the LERP/SLERP interpolation for each animated joint, a **delta transform** (`deltaPos` + `deltaQuat`) is computed from the original joint sphere position/rotation vs the animated one. This delta matrix is applied to all `meshesGroup` children whose `userData.parentJointName` matches the animated joint, re-positioning and re-rotating the geometry in real time.
   - **Result**: Ship mesh parts animated by the `.mad` file (e.g., `RadarDish` mesh on `ter_fenris.hod`) now visually rotate and translate in the 3D viewport as the timeline scrubs.
5. **Animation Status Badge** *(Phase 3 Polish)*:
   - A pulsing green pill badge ("N Animations Loaded — Use Timeline Below ↓") now appears at the top center of the 3D viewport when the loaded model has animations. The badge uses the new `anim-ready-badge` CSS keyframe class for a smooth glow animation, helping users immediately discover the timeline player at the bottom.
6. **Diagnostics Panel Drag Handle CSS** *(Phase 3 Polish)*:
   - Added `.diagnostics-handle-hover:hover` and `:active` styles to `src/App.css` with a glowing cyan highlight and `box-shadow: 0 0 8px var(--accent-cyan)` to provide clear visual feedback during resize drag operations.
7. **Validation and Build Parity**:
   - TypeScript build (`npm run build`) completes in ~2s with **0 errors and 0 TypeScript warnings**.
   - Rust parser crate (`cargo build` in `parser/`) compiles with **0 errors** (only pre-existing unused variable hints).

