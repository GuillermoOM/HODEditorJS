# OpenCode Agent Handbook

Welcome to the **HODEditorJS** codebase! This repository is built to parse, render, and edit Homeworld Remastered HOD 1.0 and 2.0 binary ship models.

This guide provides the required context for any OpenCode agent to successfully work on this project without breaking existing features or repeating past mistakes.

## 1. Project Architecture

This is a **Tauri** desktop application with three major structural boundaries:

1. **`src/` (Frontend)**: React + TypeScript + Vite.
   - Responsible for UI state, rendering the 3D viewport (Three.js), and managing user interaction (the Hierarchy tree, inspector, and toolbars).
   - Core files: `App.tsx` (top-level state and IPC), `components/HierarchyTree.tsx` (complex node tree), `components/Viewport.tsx` (WebGL rendering).
   - **Important**: The React UI acts as the state manager. The Rust backend does not maintain persistent state between calls.
2. **`src-tauri/` (Backend Host)**: Rust + Tauri.
   - Responsible for native OS bridging (file dialogs) and defining IPC commands like `load_hod`, `save_hod`, `export_textures_tga`.
   - Core file: `src/lib.rs`.
3. **`parser/` (Core Engine)**: Pure Rust library (`hwr_hod_parser`).
   - Responsible for the heavy lifting: binary parsing, struct mapping, Microsoft Xpress compression (`xpress.rs`), and binary serialization of HOD files (`hod.rs`, `iff.rs`, `compiler.rs`).
   - Includes multiple utility bins (`src/bin/*`) for diagnostic dumping and validation testing.

## 2. HOD Save Architecture (CRITICAL HARD RULE)

**Every save operation MUST generate a fresh HOD 2.0 file from the in-memory model. Never patch the original file.**

- When a HOD 1.0, HOD 2.0, or DAE file is loaded, it is parsed into `HODModel` and the original file bytes are discarded.
- On save, `generate_v2_from_model([], &model)` is called to create a brand new HOD 2.0 binary from the model data.
- **All output files are HOD 2.0 format**, regardless of input format. This app modernizes HOD 1.0 files.
- The `save_edits` function and `original_needs_full_v2_regeneration` logic are DEPRECATED. They only exist as a fallback for edge cases and should not be relied upon.
- Unparsed chunks from the original file (like `KDOP`, `COLD`, `SCAR`, `INFO`) must be preserved in `HODModel` as raw bytes and written back to the output during generation.

This architecture is documented in `agents_info/implementation_plan.md`.

## 2. Reading the UI Source of Truth

When modifying the React frontend, **never rely on `TODO.md` or aspirational docs.**
Instead, read the exact behavior specifications located in `docs/ui-source-of-truth/`.

The `docs/ui-source-of-truth/README.md` file dictates strict rules:
- Verify behavior against the listed source files before changing it.
- If a spec and source disagree, treat source as canonical.
- When changing behavior, update the relevant spec in the same work item.

## 3. Modifying the Rust Parser (The HOD 2.0 Quirks)

Working with the binary parser is highly sensitive. The HOD 2.0 format has several extremely volatile architectural quirks.

Before making changes to the parser, read:
- `agents_info/hod2_reverse_engineering_knowledge_base.md`: Contains the foundational reverse-engineering knowledge (Flat file structure, Endianness inconsistencies, MULT chunk padding, POOL compression, etc.).
- `agents_info/hod2_serialization_walkthrough.md`: A historical log of how the HOD 2.0 serialization was implemented.
- `.opencode/skills/hod-binary-layout/SKILL.md`: The definitive byte-layout spec for DTRM sub-chunks (NAVL, BURN, GLOW, etc.).

**CRITICAL PARSER RULE:** If you change *any* serialization logic in `parser/src/hod.rs` or `parser/src/compiler.rs`, you **MUST** run the lossless verification suite to prove you didn't break the game engine format.

```bash
cd parser
cargo run --bin verify_lossless
```
If `verify_lossless` fails to cleanly round-trip the mesh, joint, or marker counts on the test files (`pebble_0.hod`, `ter_elysium.hod`, `ter_zephyrus.hod`), your changes are destructive and must be reverted/fixed immediately.

### Useful Paths for Testing
- **HOD 1.0 files (FS mod):** `/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/uncompressed_bigs/freespace_remastered/ship/`
- **HOD 2.0 files (FS mod):** `/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/ship/`
- **DAE files (FS mod):** `/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/current_project_processing/ship_converted/`
- **HOD 2.0 files (Vanilla game):** `/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/uncompressed_bigs/HWRM/ship/`
- **HOD 2.0 files (pebbles):** `/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/uncompressed_bigs/HWRM/pebble/`

## 4. UI Scripts and Automated Refactoring

You may notice files like `src/components/inject.py` or `fix_tree.py`.
The `HierarchyTree.tsx` is an extremely massive component (~3000 lines). When adding cross-cutting concerns (like dragging-and-dropping to all 10+ node types, or adding a right-click context menu to every single node), manual text replacement often fails.

It is an established pattern in this repo for agents to write temporary, throwaway Python scripts utilizing `re` (Regex) to safely inject code blocks or `onClick` handlers into the massive TSX files, verify the build (`npm run build`), and then commit. Do not be afraid to write Python AST/Regex scripts to modify the TSX files safely if AST-Grep is insufficient.

## 5. Typical Workflow for an Agent

1. **Understand Request**: Is this a UI feature or a binary format fix?
2. **Context Gathering**:
   - UI Feature: Read `docs/ui-source-of-truth/` to see the current constraints.
   - Binary Fix: Read `agents_info/hod2_reverse_engineering_knowledge_base.md`.
3. **Execution**:
   - Implement the fix.
   - Run `npm run build` (for frontend) or `cargo check` (for backend/parser).
4. **Verification**:
   - If UI: Update `docs/ui-source-of-truth/` if behavior fundamentally changed.
   - If Parser: Run `cd parser && cargo run --bin verify_lossless`.
