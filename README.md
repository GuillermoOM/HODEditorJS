# Homeworld Remastered HOD Editor (`HODEditorJS`)

A modern, cross-platform visual editor for Homeworld Remastered (HWRM) `.hod` ship skeleton and mesh files. Built with **Tauri**, **React**, **TypeScript**, **Three.js**, and a custom standalone **Rust parser (`hwr_hod_parser`)**.

This tool lets you inspect, edit, and save `.hod` files with a visual interface. It supports migrating legacy **HOD 1.0** files to **HOD 2.0**, building weapon assemblies, editing collision hulls, managing materials and textures, and creating animations.

![Main Editor](screenshots/Main_Editor.png)

---

## Table of Contents
1. [Screenshots](#screenshots)
2. [Key Features](#key-features)
3. [Known Limitations](#known-limitations)
4. [OBJ Import Guide](#obj-import-guide)
5. [DAE Import (WIP)](#dae-import-wip)
6. [Feature Reference](#feature-reference)
7. [Architecture](#architecture)
8. [Installation & Running](#installation--running)
9. [Development & Testing](#development--testing)
10. [License](#license)

---

## Screenshots

| | |
|:---:|:---:|
| ![Main Menu](screenshots/Main_Menu.png) | ![Ship Loaded](screenshots/Ship_Loaded.png) |
| ![Node Hierarchy](screenshots/Node_hierarchy.png) | ![Node Assembly](screenshots/Node_Assembly.png) |
| ![Material Manager](screenshots/Material_Manager.png) | ![Animation Editor](screenshots/Animation_editor.png) |
| ![Collision Mesh](screenshots/collision_mesh_creation.png) | ![Engine Assembly](screenshots/engine_assembly.png) |
| ![Target Box Editor](screenshots/target_box_editor.png) | |

---

## Key Features

**File I/O**
- Open and save HOD files (native file dialog or manual path entry)
- Create new blank HOD 2.0 models
- Import/export OBJ meshes (per-LOD mesh, collision hull, engine glow, engine shape)
- Import/export GLTF models
- Import/export TGA textures (batch import with auto DXT1/DXT5 detection)
- Import/export material libraries as JSON
- Compile animations to binary `.mad` files
- Import DAE/Collada models (WIP, see limitations)

**3D Viewport**
- 6 render modes: Textured Shaded (Raw / Team Paint), Textured Flat (Raw / Team Paint), Untextured Solid, Wireframe
- Move and Rotate transform gizmos with visual feedback
- Adaptive 3-tier grid that adjusts with camera zoom
- Cinematic 4-light setup (deep space ambient, distant sun, cool nebula fill, warm dust bounce)
- Togglable layers: NavLights, Collision Hulls, Dockpaths, Engine Burns, Node Hierarchy Lines
- Auto-camera focus on model load
- Joint spheres, bone lines, marker cones, NavLight pulsing sprites, dockpath pyramids
- Engine burn/glow/shape and collision hull visualization

**Hierarchy Tree (4 Tabs)**
- **Hierarchy**: Full recursive joint tree with collapsible folders, drag-and-drop reparenting, search/filter, per-node visibility toggles with cascading propagation, LOD mutual exclusion
- **Materials**: Material list with shader info, texture thumbnail grid with dimensions and format, add/delete materials, search/filter
- **Animations**: Animation list with duration/track/keyframe counts, expandable to show joint channels and timestamps
- **Target Boxes**: Add/edit/delete target boxes with live dimension readout in meters, LUA code export with copy-to-clipboard

**Inspector Panel**
- Per-node property editing for all 13 node types: Joint, Marker, NavLight, Dockpath, Collision, Mesh (LOD), Engine Glow, Engine Burn, Engine Shape, Weapon/Turret/Hardpoint/Capture/Repair/Salvage groups, Material, Keyframe
- Collision hull: center, radius, extents, OBJ import/export, convex hull generation from visual mesh, box bounds calculation
- Material editor: shader pipeline selector, per-shader texture slot mapping (15 shader types), texture dropdown with thumbnail preview, TGA import
- Weapon assembly: group rename (cascading), structure completeness check, auto-repair missing joints, weapon-to-turret conversion

**Animation Timeline**
- Playback controls: rewind, play/pause, stop, loop, speed (0.25x/0.5x/1x/2x)
- Timeline ruler with click/drag scrubbing
- Per-joint track lanes with color-coded keyframe diamonds
- Keyframe drag to retime, hover tooltip with position/rotation/time data, click to select and seek, right-click to delete
- Record keyframe captures current joint transform
- Compile to binary `.mad` format
- Real-time LERP/SLERP interpolation in the 3D viewport

**Node Creation**
- 13 node types available from the Add Node modal
- Standard assembly templates (Weapon, Turret, Hardpoint, Capture Point, Repair Point, Salvage Point) with correct HWRM joint hierarchy and coordinate offsets
- 0-indexed naming conventions matching game engine rules
- Unified group folding in the tree (assemblies shown as single nodes)
- Self-healing integrity engine: real-time diagnostics with one-click structure repair

**Migration Assistant**
- HOD 1.0 to 2.0 schema migration with 7 reclassification targets (Standard Joint, Marker, Weapon Assembly, Collision Hull, NavLight, Engine Burn, Engine Glow)
- Coordinate cloning from any other joint in the model

**Settings**
- Shader directory configuration (persistent JSON storage)
- keeper.txt path management for texture resolution and shader pipeline scanning
- Configurable via Inspector's Configure button

---

## Known Limitations

This editor is actively developed and has known gaps. Please be aware before using it in production modding workflows:

**Editing Limitations**
- **No Undo/Redo**: There is no undo or redo system. Any change is permanent once made. Save frequently and keep backups.
- **No Multi-Select**: Only one node can be selected at a time.
- **No Copy/Paste/Duplicate**: There is no clipboard or duplication functionality. Nodes must be created manually.
- **No Keyboard Shortcuts**: All operations must be performed through the UI. No hotkey bindings exist.
- **No Batch Operations**: No batch rename, batch reparent, or batch property editing.
- **No Scale Gizmo**: Only Move and Rotate transform gizmos are available in the viewport. Scale transform is not exposed.

**Animation (WIP)**
- Animation editing is a work in progress. Keyframes serialize and export correctly to `.mad` files, but the in-viewport playback rendering is incomplete. Mesh delta transforms propagate via joint animation, but complex multi-joint animations may not preview accurately.

**File Format Issues**
- **Saving as HOD 1.3**: The editor saves files using the HOD 2.0 binary format. The version header written may be misinterpreted by some tools as HOD 1.3 rather than pure 2.0.
- **Opening HOD 1.0 (HW2 Classic)**: While the parser handles HOD 1.0 binary decompression and two-pass material/texture resolution, some legacy HW2 Classic files may exhibit parsing edge cases. The migration assistant can help convert 1.0 structures to 2.0.
- **DAE Import is Unstable**: The Collada/DAE importer is a work in progress. It may cause crashes, produce incorrect hierarchy structures, or fail on complex models. Use OBJ import for reliable mesh replacement. Back up your work before attempting DAE import.

**Textures**
- **No Texture Filtering**: The viewport does not apply mipmapping or texture filtering. Textures may appear aliased at distance.
- **Missing Texture Filtering**: Texture previews in the material inspector are raw PNG conversions without filtering.

**General**
- **No Dedicated Settings Panel**: Shader directories and keeper.txt paths are configured through the Inspector's Configure button and toolbar inputs rather than a centralized settings UI.
- **No Preferences Persistence**: Window size, panel widths, and last-opened paths are not persisted between sessions.

---

## OBJ Import Guide

The editor supports importing Wavefront `.obj` files to replace mesh geometry at any LOD level, as well as collision hulls, engine glow meshes, and engine shape meshes.

### Axis Orientation

Homeworld Remastered uses a specific coordinate system. When exporting OBJ files from Blender, Maya, or other 3D tools, ensure your model is oriented as follows:

| Axis | Direction | Description |
|:---:|:---:|:---|
| **Z** | Front | The forward direction of the ship (bow/nose) |
| **Y** | Vertical | The up direction (top of the ship) |
| **X** | Horizontal | The lateral direction (port/starboard) |

```
        Y (Up)
        |
        |
        +---- Z (Front)
       /
      /
     X (Horizontal)
```

**Blender Export Settings:**
- Forward axis: **-Z** (or adjust to match Z-Front convention)
- Up axis: **Y Up**
- Apply transforms before export (Ctrl+A > All Transforms)
- Ensure your ship model faces the Z-positive direction and is upright on Y

**Maya Export Settings:**
- Up axis: **Y**
- Forward axis: **Z**

### Importing Mesh LODs

1. Select a **Mesh** node in the hierarchy tree
2. In the Inspector panel, find the LOD variant list
3. Select the LOD level you want to replace
4. Click **Import OBJ** to load your `.obj` file
5. The mesh geometry replaces the selected LOD in-place, reconstructing positions, normals, and UV coordinates

### Importing Collision Hulls

1. Select a **Collision** node in the hierarchy tree
2. In the Inspector panel, click **Import OBJ** under the collision mesh section
3. The OBJ geometry is imported into the collision hull mesh parts
4. You can also use **Generate Convex Hull** to auto-generate a collision mesh from the visual mesh, or **Calculate Box Bounds** for a simple bounding box

### Importing Engine Glow/Shape

1. Select an **Engine Glow** or **Engine Shape** node
2. In the Inspector panel, click **Import OBJ**
3. The mesh replaces the engine glow/shape geometry

### Exporting

All importable mesh types can also be exported as `.obj` files via the corresponding **Export OBJ** buttons. Exported files include `.mtl` material references and TGA textures are written to the same folder.

---

## DAE Import (WIP)

> **Warning**: The DAE/Collada importer is experimental and unstable. It may crash the editor, produce incorrect results, or generate HOD files that cause in-game crashes. Use OBJ import for reliable workflows. Back up your work before using this feature.

The editor can import Collada `.dae` files through the toolbar **Import DAE** button. The Rust backend parses the XML structure and attempts to:

- Auto-repair assembly naming conventions
- Clean hierarchy structures
- Deduplicate node names
- Normalize collision meshes

This feature is under active development. Complex DAE files with advanced features (skin weights, morph targets, multiple scenes) are not supported.

---

## Feature Reference

### File I/O

| Operation | Description |
|:---|:---|
| Open HOD (dialog) | Native OS file picker filtered to `.hod` files |
| Open HOD (path) | Enter an absolute filesystem path manually |
| Save HOD | Overwrite current file + auto-save companion `.mad` |
| Save HOD As | Native save dialog for new path + companion `.mad` |
| New HOD | Create blank HOD 2.0 template with single Root joint |
| Import DAE | Collada import via Rust XML parser (WIP) |
| Import OBJ (mesh) | Per-LOD mesh replacement via Three.js OBJLoader |
| Export OBJ (mesh) | Per-LOD mesh export with `.mtl` and TGA textures |
| Import OBJ (collision) | Import geometry into collision hull mesh parts |
| Export OBJ (collision) | Export collision hull as OBJ |
| Import OBJ (engine glow) | Import geometry into engine glow mesh parts |
| Export OBJ (engine glow) | Export engine glow mesh as OBJ |
| Import OBJ (engine shape) | Import geometry into engine shape mesh parts |
| Export OBJ (engine shape) | Export engine shape mesh as OBJ |
| Export GLTF | Export all visible 3D meshes as `.gltf` |
| Import GLTF | Import `.gltf` meshes and append to model |
| Import TGA | Multi-file TGA import with PNG preview, auto DXT1/DXT5 detection |
| Export TGA | Batch export all model textures as `.tga` files |
| Export Materials (JSON) | Export material library as JSON + TGA textures |
| Import Materials (JSON) | Import material library from JSON file |
| Compile .MAD | Export animation to binary `.mad` format |

### Node Types (Add Node Modal)

| Type | Description |
|:---|:---|
| Joint | Standard skeleton bone |
| Marker | Attachment point with position and rotation |
| Mesh | Visual mesh with LOD support |
| NavLight | Pulsing navigation light with color, frequency, phase |
| Dockpath | Docking path with tolerance and speed per point |
| Collision | Collision hull with bounding box and mesh |
| Weapon Template | Weapon assembly (Position/Direction/Rest/Muzzle) |
| Turret Template | Turret assembly (Position/Direction/Rest/Latitude/Barrel/Muzzle) |
| Engine Nozzle | Engine nozzle with burn plume, glow, and shape sub-nodes |
| Repair Point | Repair point assembly (Base/Heading/Left/Up) |
| Capture Point | Capture point assembly (Base/Heading/Left/Up) |
| Hardpoint | Hardpoint assembly (Position/Direction/Rest) |
| Salvage Point | Salvage point assembly (Base/Heading/Left/Up) |

### Viewport Render Modes

| Mode | Description |
|:---|:---|
| Textured Shaded (Raw) | Standard textured rendering with lighting |
| Textured Shaded (Team Paint) | Textured with team color overlay (primary + stripe) |
| Textured Flat (Raw) | Textured with flat/unlit shading |
| Textured Flat (Team Paint) | Flat shading with team color overlay |
| Untextured Solid | Solid color rendering without textures |
| Wireframe | Wireframe overlay on all meshes |

### Viewport Layer Toggles

| Layer | Visualization |
|:---|:---|
| NavLights | Pulsing sprites with frequency/phase animation, distance range spheres |
| Collision Hulls | Red transparent mesh + wireframe, bounding box and sphere helpers |
| Dockpaths | Orange pyramid markers at dockpoints, connecting lines |
| Engine Burns | Red line loops for burn vertices, orange particle spawn indicators |
| Node Hierarchy | Green joint spheres, blue bone connection lines |

### Hierarchy Tree Features

| Feature | Description |
|:---|:---|
| Search/Filter | Real-time text filter with recursive descendant matching |
| Drag-and-Drop | Reparent joints, markers, meshes, navlights, engine elements |
| Auto-Scroll | Edge-proximity auto-scrolling during drag operations |
| Visibility Toggle | Per-node eye icon with cascading to all LODs and descendants |
| LOD Exclusion | Only one LOD per mesh base name visible at a time |
| Context Menu | Right-click for Rename, Delete (with confirmation), Toggle Y-Flip, Remove Texture |
| Diagnostics | Resizable panel showing validation warnings (missing joints, engine burn limit, etc.) |

### Inspector Node Editing

| Node Type | Editable Properties |
|:---|:---|
| Joint | Position (XYZ), Rotation (Euler degrees XYZ), Parent |
| Engine Nozzle | Joint properties + Add/Remove Burn Plume, Glow, Shape |
| Marker | Position (XYZ), Rotation (Euler degrees XYZ) |
| NavLight | Root coordinate (XYZ), Section ID, Size, Phase, Frequency, Style, Distance, Color, Sprite Visible, High End Only |
| Dockpath | Parent joint, Add/Remove dockpoints, point list with tolerance/speed |
| Collision | Center (XYZ), Radius, Min/Max Extents (XYZ), OBJ import/export, Generate Convex Hull, Calculate Box Bounds |
| Mesh (LOD) | LOD variant list (add/delete/reorder/visibility), OBJ import/export, material assignment, geometry stats |
| Engine Glow | LOD variant list, OBJ import/export, mesh statistics |
| Engine Burn | Parent joint, Divisions, Flames, burn vertex list (add/remove/XYZ) |
| Engine Shape | Parent joint, mesh statistics, OBJ import/export, material assignment |
| Weapon/Turret Group | Group rename (cascading), structure check, auto-repair, weapon-to-turret conversion, per-joint position/rotation |
| Material | Name, shader pipeline, texture slot mapping (15 shader types), texture dropdown with thumbnails, TGA import |
| Keyframe | Time, Position delta (XYZ), Rotation (Euler: Pitch/Yaw/Roll), raw quaternion |

---

## Architecture

```
┌────────────────────────────────────────────────────────┐
│                     FRONTEND (UI)                      │
│         React 19 + TypeScript 5.8 + Vite 7            │
│  - 4-Tab Hierarchy Tree (Hierarchy/Materials/          │
│    Animations/Target Boxes)                            │
│  - Inspector Panel (per-node property editing)         │
│  - Animation Timeline & Playback Controllers           │
├────────────────────────────────────────────────────────┤
│                     VIEWPORT (3D)                      │
│                    Three.js 0.184                      │
│  - 6 Render Modes & Team Paint                         │
│  - Transform Gizmos (Move/Rotate)                      │
│  - Real-Time NavLight Pulse Animations                 │
│  - LERP/SLERP Animation Playback                       │
├────────────────────────────────────▲───────────────────┤
│                                    │ IPC (Tauri 2)     │
│                                    ▼                   │
├────────────────────────────────────────────────────────┤
│                 TAURI DESKTOP BRIDGE                   │
│                       Rust API                         │
│  - Native File Dialogs (rfd)                           │
│  - TGA Import/Export (image crate)                     │
│  - Convex Hull Generation                              │
│  - Diagnostic Logger                                   │
├────────────────────────────────────▲───────────────────┤
│                                    │ Rust FFI          │
│                                    ▼                   │
├────────────────────────────────────────────────────────┤
│                  STANDALONE PARSER                     │
│                hwr_hod_parser (Rust)                   │
│  - MS XPress 31-bit LZ77 Decompressor/Compressor       │
│  - HOD 1.0 & 2.0 IFF Binary Reader/Writer              │
│  - Absolute Euler & Float Preservation                 │
│  - DAE/Collada XML Parser                              │
└────────────────────────────────────────────────────────┘
```

---

## Installation & Running

### Prerequisites
- [Node.js](https://nodejs.org/) v18.x or newer
- [Rust & Cargo](https://rustup.rs/) v1.75.0 or newer
- Platform-specific Tauri dependencies ([Tauri Prerequisites Guide](https://v2.tauri.app/start/prerequisites/))

### Install Dependencies
```bash
git clone <repository-url>
cd HODEditorJS
npm install
```

### Run in Development Mode
```bash
npm run tauri dev
```

### Build Production Executable
```bash
npm run build
npm run tauri build
```

---

## Development & Testing

```bash
# Check parser compilation
cargo check --manifest-path parser/Cargo.toml

# Run parser unit tests and LZ77 roundtrip verifications
cargo test --manifest-path parser/Cargo.toml
```

---

## License

Distributed under the **Creative Commons Attribution-NonCommercial 4.0 International (CC BY-NC 4.0)** license.

- **You are free to**: Share, copy, redistribute, adapt, remix, transform, and build upon the material.
- **Under the following terms**:
  - **Attribution**: You must give appropriate credit, provide a link to the license, and indicate if changes were made.
  - **Non-Commercial**: You may NOT use the material for commercial purposes (the software cannot be sold).

See the `LICENSE` file for full terms.
