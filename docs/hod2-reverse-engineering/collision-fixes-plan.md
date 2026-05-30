# Collision Pipeline Fixes — Implementation Plan

## Executive Summary

Three issues need fixing:
1. Editor-created collision mesh stubs should auto-generate a decimated version of the visible mesh, not an 8-vertex box
2. `ter_centaur_from_dae.hod` crashes the game — need to isolate which chunk causes it
3. Auto-generate collision mesh button should be available even when no collision geometry exists yet

---

## Fix 1: Editor Collision Mesh Stub Improvement

### Current State
- `hod.rs:3335-3361` — `generate_collision_mesh()` creates default collision mesh with hardcoded `[-10, 10]` extents and empty parts
- `hod.rs:3363-3479` — For collision meshes with empty parts, generates 8-vertex AABB box
- `HierarchyTree.tsx:390-400` — UI "Add Collision" creates empty stub

### What Should Happen
When user adds a collision node in the editor, it should auto-generate a **decimated mesh** from the visible LOD-0 mesh. This means:
1. Take the LOD-0 visible mesh vertices and indices
2. Decimate/simplify to create a low-poly collision hull
3. Compute BBOX/BSPH from the decimated vertices
4. Store as collision mesh parts

### Implementation

**Step 1:** Create a mesh decimation function in Rust
- File: `parser/src/collision.rs` (new) or inside `hod.rs`
- Input: vertices + indices from visible LOD-0 mesh
- Output: decimated vertices + indices
- Algorithm: Simple vertex clustering or quadric error metric (QEM) decimation
- Target: ~5-10% of original vertex count (e.g., 1000 visible verts → 50-100 collision verts)

**Step 2:** Add Tauri command `auto_generate_collision_from_mesh`
- Input: model + source mesh name
- Output: updated model with collision mesh populated
- Call decimation function on the selected mesh
- Compute BBOX/BSPH from decimated vertices

**Step 3:** Update `generate_collision_mesh()` in `hod.rs`
- Instead of hardcoded box, use visible mesh LOD-0 for decimation
- Only fallback to box if no visible meshes exist

**Step 4:** Update Inspector.tsx
- Make "Auto-Generate BBOX/BSPH from Mesh" button visible even when parts are empty
- Add "Auto-Generate Collision Mesh from Visible Mesh" button that calls the new Tauri command

---

## Fix 2: Game Crash Investigation

### Current State
- `ter_centaur_from_dae.hod` (266,216 bytes) crashes the game on load
- Multiple potential causes identified in audit

### Investigation Method
The agent should systematically isolate the crash cause:

**Step 1:** Run `cargo run --bin verify_lossless` on the test file to check structural integrity

**Step 2:** Create a binary stripper tool that removes one chunk at a time:
- Strip KDOP → test in game
- Strip COLD → test in game
- Strip SCAR → test in game
- Strip collision mesh from POOL → test in game
- Strip NAVL → test in game

**Step 3:** Check STAT count vs mesh part count
- Count STAT chunks in HVMD
- Count BMSH parts across all MULT chunks
- They must match exactly — mismatch causes OOB crash

**Step 4:** Check pool compression sizes
- Verify `decomp_mesh_len` in POOL header matches actual decompressed buffer size
- Verify `decomp_face_len` matches actual decompressed buffer size
- Compare against HODOR-generated ter_centaur_hodor.hod pool sizes

**Step 5:** Check KDOP binary format
- Compare generated KDOP against HODOR-generated KDOP byte-for-byte
- Verify vertex count, face count, and trailing padding match

### Most Likely Crash Causes (Priority Order)
1. STAT count mismatch (documented)
2. Pool decompression size mismatch (new compression)
3. KDOP degenerate geometry
4. TRIS format change

---

## Fix 3: Auto-Generate Button Availability

### Current State
- `Inspector.tsx:1872-1889` — "Auto-Generate BBOX/BSPH from Mesh" button only visible when `col.mesh.parts.some(p => p.vertices.length > 0)`
- When loading a HOD with no collision geometry, button is hidden

### What Should Happen
The button should be visible **always** when a collision node is selected, with different behavior:
- If collision mesh has vertices: recompute BBOX/BSPH from existing vertices
- If collision mesh is empty: generate collision mesh from visible mesh first, then compute BBOX/BSPH

### Implementation

**Step 1:** Remove the guard condition on the button
- Always show "Auto-Generate" button when collision node is selected

**Step 2:** Add "Generate from Visible Mesh" section above the existing buttons
- Dropdown to select source mesh (already exists)
- Button: "Generate Collision Mesh from [mesh name]"
- This calls the new Tauri command from Fix 1

**Step 3:** Flow after generation
1. User selects collision node
2. User picks source visible mesh from dropdown
3. User clicks "Generate Collision Mesh"
4. Collision mesh is populated with decimated vertices
5. BBOX/BSPH are auto-computed
6. "Auto-Generate BBOX/BSPH from Mesh" button becomes active for tweaking

---

## Implementation Order

| Priority | Task | Effort | Impact |
|----------|------|--------|--------|
| 1 | Investigate game crash (strip chunks) | Medium | CRITICAL |
| 2 | Fix crash root cause | Low-High (depends on finding) | CRITICAL |
| 3 | Add mesh decimation function | High | HIGH |
| 4 | Add Tauri command for auto-generation | Low | HIGH |
| 5 | Update Inspector UI buttons | Medium | HIGH |
| 6 | Update `generate_collision_mesh()` fallback | Low | MEDIUM |

---

## Key Files to Modify

| File | Changes |
|------|---------|
| `parser/src/hod.rs` | Fix `generate_collision_mesh()`, fix pool compression, fix COLD duplication |
| `parser/src/kdop.rs` | Verify KDOP format matches HODOR |
| `parser/src/collision.rs` (new) | Mesh decimation algorithm |
| `src-tauri/src/lib.rs` | Add `auto_generate_collision_from_mesh` command |
| `src/components/Inspector.tsx` | Update button visibility, add generation flow |
| `docs/hod2-reverse-engineering/hod2_reverse_engineering_knowledge_base.md` | Document findings |

---

## Testing Strategy

1. **Before changes:** Run `cargo run --bin verify_lossless` on all test files to establish baseline
2. **After crash fix:** Test `ter_centaur_from_dae.hod` in-game
3. **After decimation:** Compare decimated mesh vertex count against HODOR's collision mesh
4. **After all changes:** Run full test suite, test in-game with Homeworld Remastered

---

## Reference: HODOR Collision Mesh Sizes

From the RODOH analysis:
- ter_pharos COL[Root]: 124 vertices, 60 triangles
- ter_centaur COL[Root]: similar scale
- Visible mesh LOD-0: thousands of vertices
- Ratio: collision mesh is ~2-5% of visible mesh vertex count
