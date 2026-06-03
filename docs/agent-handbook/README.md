# OpenCode Agent Handbook

Welcome to the **HODEditorJS** codebase! This repository is built to parse, render, and edit Homeworld Remastered HOD 1.3 and 2.0 binary ship models.

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

- When a HOD 1.3, HOD 2.0, or DAE file is loaded, it is parsed into `HODModel` and the original file bytes are discarded.
- On save, `generate_v2_from_model([], &model)` is called to create a brand new HOD 2.0 binary from the model data.
- **All output files are HOD 2.0 format**, regardless of input format. This app modernizes HOD 1.3 files.
- The `save_edits` function and `original_needs_full_v2_regeneration` logic are DEPRECATED. They only exist as a fallback for edge cases and should not be relied upon.
- Unparsed chunks from the original file (like `KDOP`, `COLD`, `SCAR`, `INFO`) must be preserved in `HODModel` as raw bytes and written back to the output during generation.

This architecture is documented in `docs/hod2-reverse-engineering/architecture-overview.md`.

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
- `docs/README.md`: The central **Knowledge Graph** bridging all active specifications and legacy reverse-engineering knowledge.
- `docs/hod2-reverse-engineering/hod2-creation-specification.md`: The central file format specification.
- `.opencode/skills/hod-binary-layout/SKILL.md`: The definitive byte-layout spec for DTRM sub-chunks (NAVL, BURN, GLOW, etc.).

**CRITICAL PARSER RULE:** If you change *any* serialization logic in `parser/src/hod.rs` or `parser/src/compiler.rs`, you **MUST** run the lossless verification suite to prove you didn't break the game engine format.

```bash
cd parser
cargo run --bin verify_lossless
```
If `verify_lossless` fails to cleanly round-trip the mesh, joint, or marker counts on the test files (`pebble_0.hod`, `ter_elysium.hod`, `ter_zephyrus.hod`), your changes are destructive and must be reverted/fixed immediately.

### Useful Paths for Testing
- **HOD 1.3 files (FS mod):** `/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/uncompressed_bigs/freespace_remastered/ship/`
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
   - Binary Fix: Read `docs/hod2-reverse-engineering/hod2_reverse_engineering_knowledge_base.md`.
3. **Execution**:
   - Implement the fix.
   - Run `npm run build` (for frontend) or `cargo check` (for backend/parser).
4. **Verification**:
   - If UI: Update `docs/ui-source-of-truth/` if behavior fundamentally changed.
   - If Parser: Run `cd parser && cargo run --bin verify_lossless`.
   - **MANDATORY FOR MANUAL TESTS**: If you modify the binary parser, HOD generation logic, or any feature requiring the user to load/save in the editor and test in-game, you **MUST** document the test snapshot in `testing/manual_tests.md`. Add a new entry to the "Test Runs" section of that file specifying the SHA, Timestamp, the HOD tested, and the expected results (pass/fail for loading, meshes, textures, etc.). This ensures a persistent history of manual verifications to prevent regressions.

## 6. Pre-Flight Checklist (Preventing Regressions)

To prevent accidental feature loss (such as breaking UI loading screens or destroying UX additions), follow these rules:

1. **Do not remove `setTimeout` blocks indiscriminately.** Tauri's IPC `invoke` API serializes payloads synchronously. Heavy operations (like passing the entire `HODModel` or parsing large JSON strings) will block the React render thread. You **MUST** ensure `setIsLoading(true)` is followed by a `setTimeout(..., 50)` before a heavy `invoke` call, allowing the browser to paint the loading screen before it freezes.
2. **Leave Trap Comments.** If you add a hack or workaround (like the `setTimeout` trick), leave a comment explicitly forbidding future agents from "cleaning it up".
3. **Verify the UI State.** If you modify any frontend file that triggers heavy Rust processing (saving, importing, generating meshes), verify that the loading spinner overlay actually mounts and is visible before the application locks up during testing.

## 7. Publishing a New Release

When preparing a new GitHub release, follow these exact steps to ensure versions align and binaries are built and uploaded correctly:

1. **Bump Version Strings**:
   Update the version number (e.g., to `1.2.1`) in the following files:
   - `package.json`
   - `src-tauri/tauri.conf.json`
   - `src-tauri/Cargo.toml`

2. **Commit and Tag**:
   ```bash
   git commit -am "chore(release): bump version to X.Y.Z"
   git tag -a vX.Y.Z -m "Version X.Y.Z"
   git push origin master --tags
   ```

3. **Build Binaries (Linux Host)**:
   All official release builds **must** be performed inside the `esp-dev` distrobox to ensure cross-compilation toolchains and GTK/WebKit dependencies are present.
   ```bash
   distrobox enter esp-dev

   # Build Linux Assets (AppImage, DEB, RPM)
   NO_STRIP=1 npm run tauri build

   # Build Windows Asset (NSIS Installer)
   CARGO_TARGET_DIR=/tmp/cargo_target npm run tauri build -- --target x86_64-pc-windows-gnu --bundles nsis
   ```

4. **Create GitHub Release and Upload Assets**:
   Use the GitHub CLI (`gh`) to create the release, paste the changelog/notes, and attach the newly compiled binaries:
   ```bash
   gh release create vX.Y.Z \\
     "src-tauri/target/release/bundle/appimage/HODEditorJS_X.Y.Z_amd64.AppImage" \\
     "src-tauri/target/release/bundle/deb/HODEditorJS_X.Y.Z_amd64.deb" \\
     "src-tauri/target/release/bundle/rpm/HODEditorJS-X.Y.Z-1.x86_64.rpm" \\
     "/tmp/cargo_target/x86_64-pc-windows-gnu/release/bundle/nsis/HODEditorJS_X.Y.Z_x64-setup.exe" \\
     --title "Version X.Y.Z" \\
     --notes "- First bullet point of changelog\\n- Second bullet point"
   ```
