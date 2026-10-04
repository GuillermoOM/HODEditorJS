> **ARCHIVED.** Historical record; not active guidance. See docs/README.md.

# Collision Pipeline Investigation & Implementation Plan

## Executive Summary

The HOD 2.0 collision pipeline has three interconnected gaps that need investigation and correction. HODOR generates **both** COLD and KDOP from the collision mesh — they coexist in the DTRM container. Our current implementation has COLD disabled and KDOP as a simplified AABB.

---

## The Correct HOD 2.0 Collision Pipeline (from HODOR)

```
User creates collision OBJ → DAEnerys exports COL[Root] in DAE → HODOR reads collision vertices →
  ├── COLD: BBOX + BSPH + TRIS (bounding info + collision triangles)
  └── KDOP: 26-DOP convex hull tree (~1588 bytes)
Both go into DTRM as siblings alongside HIER, NAVL, SCAR
```

---

## What Needs Investigation (using HODOR.exe dump)

### Investigation 1: KDOP Binary Format (CRITICAL)

HODOR's KDOP is NOT a simple AABB. It's a detailed 26-DOP convex hull.

**What to do:**
1. Dump KDOP from HODOR-generated files (ter_pharos, ter_centaur, ter_fenris)
2. Compare against vanilla KDOP (pebble_0, ter_elysium)
3. Identify the exact binary structure:
   - How many direction vectors? (26-DOP = 13 directions × 2 sides)
   - How are vertices stored? (positions only? normals?)
   - How are faces stored? (indices? vertex references?)
   - What is the tree structure? (flat list? hierarchical?)
4. Cross-reference with Ghidra decompilation of HODOR.exe — look for KDOP generation functions
5. Document in `hod2_reverse_engineering_knowledge_base.md`

**Key question:** Does HODOR compute KDOP from the collision mesh vertices, or from the visible mesh vertices?

### Investigation 2: COLD Generation from Collision Mesh (HIGH)

COLD is currently disabled (`if false` in hod.rs). Need to understand what HODOR writes.

**What to do:**
1. Dump COLD from HODOR-generated ter_pharos.hod (124 verts, 60 tris)
2. Compare COLD TRIS data against the DAE COL[Root] vertex data
3. Verify: Does COLD TRIS contain the exact collision mesh vertices/indices from the DAE?
4. Verify: How are BBOX and BSPH computed from the collision mesh?
5. Re-enable COLD generation in `generate_v2_from_model`

### Investigation 3: HODOR.exe Collision Function (using Ghidra dump)

**What to do:**
1. Search the Ghidra decompilation (`/tmp/hodor_decomp_full.txt`) for collision-related functions
2. Look for: KDOP generation, COLD generation, COL[...] parsing from DAE
3. Identify the exact algorithm HODOR uses to build the 26-DOP convex hull
4. Key functions to find:
   - The function that reads COL[...] vertices from DAE
   - The function that computes BBOX/BSPH from vertices
   - The function that builds the KDOP tree
   - The function that writes COLD/TRIS chunks

---

## Implementation Plan (after investigation)

### Phase 1: Fix COLD Generation

**Step 1.1:** Re-enable COLD generation in `hod.rs:5478`
- Remove the `if false` guard
- Ensure COLD is generated from `model.collision_meshes` data
- COLD should contain:
  - BBOX: computed from collision mesh min/max extents
  - BSPH: computed from collision mesh center + radius
  - TRIS: collision mesh vertices and indices (position only)

**Step 1.2:** Verify COLD matches HODOR output
- Compare generated COLD against HODOR-generated COLD byte-for-byte
- Test with ter_pharos collision mesh

### Phase 2: Improve KDOP Generation

**Step 2.1:** Based on Investigation 1 findings, update `parser/src/kdop.rs`
- Replace simplified AABB with proper 26-DOP convex hull
- Match HODOR's vertex count (46-48 verts) and face count (90-95 faces)
- Implement the correct tree structure if hierarchical

**Step 2.2:** Verify KDOP matches HODOR output
- Compare generated KDOP against HODOR-generated KDOP
- Run `dump_kdop` on generated files

### Phase 3: Editor Collision Mesh Workflow

**Step 3.1:** Redesign collision mesh node in editor
- Currently: creates a node showing bounding box/sphere only
- Should: allow importing/exporting collision OBJ, show actual collision mesh
- Should: auto-generate bounding box/sphere from collision mesh vertices
- Should: show preview of BBOX and BSPH in viewport

**Step 3.2:** Add collision OBJ import/export
- Import: read OBJ file as collision mesh, compute BBOX/BSPH
- Export: write collision mesh to OBJ file
- Auto-generate: if no collision mesh, create one from visible mesh AABB

**Step 3.3:** Ensure collision mesh flows into both COLD and KDOP
- collision_meshes → COLD (BBOX+BSPH+TRIS)
- collision_meshes → KDOP (convex hull tree)
- Both written to DTRM as siblings

### Phase 4: Verification

**Step 4.1:** Run full test suite
- `cargo run --bin verify_lossless`
- `cargo run --bin test_hodor_replication`
- Compare COLD and KDOP against HODOR output

**Step 4.2:** In-game testing (if possible)
- Test collision detection with generated HOD files
- Verify both COLD and KDOP are recognized by game engine

---

## Key Files

| File | Purpose |
|------|---------|
| `parser/src/kdop.rs` | KDOP generator — needs rewrite from AABB to 26-DOP |
| `parser/src/hod.rs:5478` | COLD generation — currently disabled, needs re-enabling |
| `parser/src/hod.rs:3333` | `generate_collision_mesh()` — may need update for proper BBOX/BSPH |
| `parser/src/dae.rs:330-367` | DAE COL[...] parsing — may need to actually extract vertices |
| `parser/src/hod.rs:5048-5137` | Collision mesh POOL append — verify correct |
| `docs/hod2-reverse-engineering/hod2_reverse_engineering_knowledge_base.md` | Update with KDOP format and COLD coexistence |
| `docs/hod2-reverse-engineering/kdop-scar-pipeline-gap-plan.md` | Update with corrected plan |

---

## Reference Data

### HODOR-generated ter_pharos collision:
- DAE COL[Root]: 124 vertices, 60 triangles
- HOD output: COLD (8 bytes) + KDOP (1588 bytes) both in DTRM

### HODOR-generated ter_centaur collision:
- DAE COL[Root]: collision mesh present
- HOD output: both COLD and KDOP

### Vanilla pebble_0:
- KDOP only (no COLD) — simple model, no collision mesh in DAE

---

## Ghidra Analysis Targets

Search `/tmp/hodor_decomp_full.txt` for:
- Functions that handle "COL" or "collision"
- Functions that compute bounding volumes
- Functions that build KDOP trees
- The DAE→HOD collision pipeline (FUN_00411b90 and related)
