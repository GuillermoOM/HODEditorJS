# POOL Byte Divergence & Vertex Spikiness Investigation

## Overview

Despite successfully identifying and fixing the Type 5 clashing bug in the Xpress compression algorithm, in-game models (like `ter_centaur`) still exhibit severe vertex spikiness.

We discovered that compression was a red herring. The true cause of the corrupted geometry is a **byte-level divergence in the uncompressed POOL streams** between what HODOR generates and what our `hod.rs` logic generates.

## The `pool_byte_diff` Diagnostic Tool

We built a new diagnostic binary (`parser/src/bin/pool_byte_diff.rs`) that bypasses structural checks and does a direct byte-for-byte comparison of the decompressed texture, mesh, and face pools between a HODOR reference HOD and our generated HOD.

The results for `ter_centaur` exposed massive discrepancies.

## Key Findings

### 1. Vertex Data Divergence (Tangents & Binormals)
While Vertex Positions (x, y, z) match almost perfectly, the **Normals, Tangents, and Binormals** diverge on over 10,000+ vertices.
- **Binormals** differ on ~15,000 vertices (nearly the entire mesh).
- Because the Homeworld engine uses binormals and tangents for lighting, shading, and potentially vertex shader offsets, this massive divergence in the data is almost certainly the root cause of the "spikiness".
- **Action Required:** Investigate `compiler::compute_tangent_space` (specifically the Gram-Schmidt orthogonalization and handedness calculations) to ensure we generate the exact same floating-point tangent/binormal vectors as HODOR.

### 2. Face Pool Size Mismatch
HODOR's face pool for `ter_centaur` is **65,286 bytes**, while our generated face pool is only **37,704 bytes**.
- This is a ~27KB discrepancy.
- The first 37,638 bytes of the face pools match exactly. The mismatch occurs because HODOR appends an extra ~27KB of index data at the very end of the file.
- **Action Required:** Determine what this extra index data is. Is HODOR duplicating face indices for lower LODs instead of sharing them? Is it generating collision mesh triangles differently?

### 3. Serialization Asymmetries & Bugs
While investigating the code in `hod.rs`, we found several structural asymmetries in how we write files compared to how we read them:
1.  **Face Pool Alignment:** When `save_edits` appends collision meshes, it writes directly to the face pool without 2-byte alignment. `generate_v2_from_model` handles this correctly.
2.  **Collision Vertex Stride:** The stride calculation for collision vertices in `save_edits` is missing the `0x04` (color) mask check.
3.  **`prim_group_count`:** When writing V2 BMSH chunks, we write `-1` for `prim_group_count`. When writing V1, we write `1`. When reading V2, we just read and ignore the `i16` value. This inconsistency might cause issues if the engine expects a specific value.
4.  **Skinning Padding:** `compiler::generate_pool_data` adds the length of `skinning_data` to the vertex stride. `update_mesh_chunks` does not.

## Testing Blindspot

The mandatory `verify_lossless` test has a major blindspot: it only checks if the generated file can be re-parsed without crashing, and verifies that the structure counts (number of meshes, materials, etc.) match.

It **does not** verify the actual mesh geometry or the raw pool bytes. This allowed completely broken geometry to "pass" the test suite. `pool_byte_diff` should be used as the source of truth for vertex generation accuracy until this is resolved.
