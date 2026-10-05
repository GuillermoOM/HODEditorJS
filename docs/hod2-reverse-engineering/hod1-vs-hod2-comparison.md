# HOD 1.0 vs HOD 2.0 Comparison

## Overview

This document compares the structural differences between HOD 1.0 (Homeworld Classic) and HOD 2.0 (Homeworld Remastered) file formats.

---

## High-Level Structure

### HOD 1.0 (Homeworld Classic)

```
VERS → NAME → HVMD → DTRM → INFO
```

**Key Characteristics:**
- NO POOL chunk
- Uncompressed data in LMIP chunks
- Different collision structure (BNDV + COLD)
- Multiple STAT chunks

### HOD 2.0 (Homeworld Remastered)

```
VERS → NAME → POOL → HVMD → DTRM → INFO
```

**Key Characteristics:**
- POOL chunk with compressed data
- LMIP chunks are small (metadata only)
- Different collision structure (KDOP)
- Single STAT chunk

---

## Detailed Comparison

### 1. VERS Chunk (Version)

**HOD 1.0:**
- Size: 4 bytes
- Value: 256 (0x00000100) or similar

**HOD 2.0:**
- Size: 4 bytes
- Value: 512 (0x00000200)

**Detection:**
```rust
let is_v2 = version >= 0x200;
```

### 2. NAME Chunk (Model Name)

**HOD 1.0:**
- Size: 26 bytes
- Format: ASCII string (NO null terminator)
- Example: "Homeworld2 Multi Mesh File"

**HOD 2.0:**
- Size: 26 bytes
- Format: ASCII string (NO null terminator)
- Example: "Homeworld2 Multi Mesh File"

**Note:** Identical between versions

### 3. POOL Chunk (Compressed Data)

**HOD 1.0:**
- **NOT PRESENT**
- Data is stored uncompressed in LMIP chunks

**HOD 2.0:**
- Size: Variable (compressed)
- Type: Default (not Form)
- Contains: Textures, meshes, faces (all compressed)
- Pool type identifier: 3518 (0x0DB6)

**Critical Difference:** This is the major structural difference

### 4. HVMD Container (Visual Data)

**HOD 1.0:**
```
HVMD
├── STAT (24 bytes) - Statistics 1
├── STAT (96 bytes) - Statistics 2
├── LMIP (2,796,290 bytes) - Uncompressed mesh LOD 0
├── LMIP (2,796,290 bytes) - Uncompressed mesh LOD 1
├── LMIP (2,796,290 bytes) - Uncompressed mesh LOD 2
└── MULT (646,435 bytes) - Mesh container
```

**HOD 2.0:**
```
HVMD
├── LMIP (87 bytes) - LOD metadata 0
├── LMIP (87 bytes) - LOD metadata 1
├── LMIP (87 bytes) - LOD metadata 2
├── STAT (93 bytes) - Statistics
└── MULT (109 bytes) - Mesh container
```

**Key Differences:**
- HOD 1.0 has 2 STAT chunks (24 + 96 bytes)
- HOD 2.0 has 1 STAT chunk (93 bytes)
- HOD 1.0 LMIP chunks are huge (uncompressed data)
- HOD 2.0 LMIP chunks are small (metadata only)

### 5. LMIP Chunks (LOD Metadata)

**HOD 1.0:**
- Size: 2,796,290 bytes each (example)
- Contains: Full uncompressed mesh data
- Purpose: Stores actual vertex/face data

**HOD 2.0:**
- Size: 87 bytes each
- Contains: LOD metadata only
- Purpose: References compressed data in POOL

**Critical Difference:** LMIP purpose completely changed

### 6. MULT Chunk (Mesh Container)

**HOD 1.0:**
- Size: 646,435 bytes
- Contains: Mesh hierarchy and references

**HOD 2.0:**
- Size: 109 bytes
- Contains: Mesh hierarchy and references

**Note:** Size difference due to uncompressed vs compressed data references

### 7. DTRM Container (Transform Data)

**HOD 1.0:**
```
DTRM
├── HIER (289 bytes) - Hierarchy
├── BNDV (121 bytes) - Bounding Volume
└── COLD (8 bytes) - Collision Data
    ├── BBOX (24 bytes) - Bounding Box
    ├── BSPH (16 bytes) - Bounding Sphere
    └── TRIS (13,088 bytes) - Collision Triangles
```

**HOD 2.0:**
```
DTRM
├── HIER (52 bytes) - Hierarchy
└── KDOP (1,588 bytes) - Collision Tree
```

**Key Differences:**
- HOD 1.0 has BNDV chunk (bounding volume)
- HOD 1.0 has COLD with BBOX/BSPH/TRIS
- HOD 2.0 has KDOP (k-discrete oriented polytope)
- HOD 2.0 has smaller HIER chunk

### 8. INFO Chunk (File Information)

**HOD 1.0:**
- Size: 49 bytes
- Format: Mixed data

**HOD 2.0:**
- Size: 49-50 bytes
- Format: Mixed data

**Note:** Similar between versions

---

## Chunk Size Comparison

### asteroid_3.hod (HOD 1.0)

| Chunk | Size | Notes |
|-------|------|-------|
| VERS | 4 bytes | Version 256 |
| NAME | 26 bytes | Model name |
| HVMD | 0 bytes | Container |
| STAT | 24 bytes | Statistics 1 |
| STAT | 96 bytes | Statistics 2 |
| LMIP | 2,796,290 bytes | Uncompressed LOD 0 |
| LMIP | 2,796,290 bytes | Uncompressed LOD 1 |
| LMIP | 2,796,290 bytes | Uncompressed LOD 2 |
| MULT | 646,435 bytes | Mesh container |
| DTRM | 0 bytes | Container |
| HIER | 289 bytes | Hierarchy |
| BNDV | 121 bytes | Bounding volume |
| COLD | 8 bytes | Collision data |
| BBOX | 24 bytes | Bounding box |
| BSPH | 16 bytes | Bounding sphere |
| TRIS | 13,088 bytes | Collision triangles |
| INFO | 49 bytes | File information |

**Total:** ~9,049,234 bytes

### pebble_0_vanilla.hod (HOD 2.0)

| Chunk | Size | Notes |
|-------|------|-------|
| VERS | 4 bytes | Version 512 |
| NAME | 26 bytes | Model name |
| POOL | 1,680,248 bytes | Compressed data |
| HVMD | 0 bytes | Container |
| LMIP | 87 bytes | LOD metadata 0 |
| LMIP | 87 bytes | LOD metadata 1 |
| LMIP | 87 bytes | LOD metadata 2 |
| STAT | 93 bytes | Statistics |
| MULT | 109 bytes | Mesh container |
| DTRM | 0 bytes | Container |
| HIER | 52 bytes | Hierarchy |
| KDOP | 1,588 bytes | Collision tree |
| INFO | 50 bytes | File information |

**Total:** ~1,682,420 bytes

**Size Reduction:** 81% smaller in HOD 2.0

---

## Conversion Rules

### HOD 1.0 → HOD 2.0

**Step 1: Parse HOD 1.0**
- Read VERS, NAME, HVMD, DTRM, INFO
- Extract uncompressed mesh data from LMIP chunks
- Extract collision data from COLD/BNDV

**Step 2: Create POOL Chunk**
- Compress textures using DXT
- Compress meshes using Xpress
- Compress faces using Xpress
- Create POOL with type=3518

**Step 3: Restructure**
- Create new HVMD with small LMIP chunks
- Create new DTRM with KDOP instead of COLD/BNDV
- Preserve INFO chunk

**Step 4: Validate**
- Verify chunk order (VERS, NAME, POOL, HVMD, DTRM, INFO)
- Verify POOL type identifier (3518)
- Verify KDOP hash (if preserved)

### HOD 2.0 → HOD 1.0

**Step 1: Parse HOD 2.0**
- Read VERS, NAME, POOL, HVMD, DTRM, INFO
- Decompress textures, meshes, faces from POOL
- Extract collision data from KDOP

**Step 2: Create Uncompressed LMIP Chunks**
- Decompress mesh data
- Create large LMIP chunks with full mesh data

**Step 3: Create COLD/BNDV Structure**
- Convert KDOP to COLD with BBOX/BSPH/TRIS
- Create BNDV chunk

**Step 4: Restructure**
- Create new HVMD with large LMIP chunks
- Create new DTRM with COLD/BNDV
- Preserve INFO chunk

---

## Critical Quirks

### 1. LMIP Purpose Change

**HOD 1.0:** LMIP contains full uncompressed mesh data  
**HOD 2.0:** LMIP contains only metadata (references POOL)

**Impact:** Cannot simply copy LMIP between versions

### 2. Collision Structure Change

**HOD 1.0:** COLD with BBOX/BSPH/TRIS  
**HOD 2.0:** KDOP

**Impact:** Must convert collision structure

### 3. STAT Chunk Differences

**HOD 1.0:** 2 STAT chunks (24 + 96 bytes)  
**HOD 2.0:** 1 STAT chunk (93 bytes)

**Impact:** Must handle different STAT formats

### 4. Compression Requirement

**HOD 1.0:** No compression (uncompressed data)  
**HOD 2.0:** Required (POOL with Xpress compression)

**Impact:** Must implement compression for HOD 1.0 → 2.0

---

## Detection Methods

### Detect HOD Version

```rust
fn detect_hod_version(chunks: &[IffChunk]) -> u32 {
    // Check for POOL chunk (HOD 2.0 indicator)
    if chunks.iter().any(|c| c.id == "POOL") {
        return 2;
    }
    
    // Check VERS chunk value
    if let Some(vers) = chunks.iter().find(|c| c.id == "VERS") {
        let version = u32::from_le_bytes([vers.data[0], vers.data[1], vers.data[2], vers.data[3]]);
        if version >= 0x200 {
            return 2;
        }
    }
    
    1 // Default to HOD 1.0
}
```

### Detect Uncompressed LMIP

```rust
fn is_uncompressed_lmip(chunk: &IffChunk) -> bool {
    chunk.id == "LMIP" && chunk.data.len() > 1000 // Threshold for uncompressed
}
```

---

## Validation Checklist

### For HOD 1.0 → 2.0 Conversion

- [ ] VERS value is 512 (0x200)
- [ ] POOL chunk is present
- [ ] POOL type is 3518 (0x0DB6)
- [ ] LMIP chunks are small (< 1000 bytes)
- [ ] DTRM has KDOP (not COLD/BNDV)
- [ ] INFO chunk is preserved

### For HOD 2.0 → 1.0 Conversion

- [ ] VERS value is 256 (0x100)
- [ ] POOL chunk is removed
- [ ] LMIP chunks are large (uncompressed data)
- [ ] DTRM has COLD/BNDV (not KDOP)
- [ ] INFO chunk is preserved

---

## Testing Strategy

### Test Case 1: HOD 1.0 → 2.0

**Input:** `asteroid_3.hod` (HOD 1.0)  
**Expected:** New HOD 2.0 file with POOL chunk  
**Validation:** Game loads without crashes

### Test Case 2: HOD 2.0 → 1.0

**Input:** `pebble_0_vanilla.hod` (HOD 2.0)  
**Expected:** New HOD 1.0 file with uncompressed LMIP  
**Validation:** Game loads without crashes

### Test Case 3: Round-Trip

**Input:** HOD 1.0 → 2.0 → 1.0  
**Expected:** Structural integrity preserved  
**Validation:** Compare with original

---

## Implementation Priority

### Priority 1: HOD 1.0 Detection

**Action:** Implement version detection  
**Rationale:** Required for any conversion  
**Effort:** Low

### Priority 2: LMIP Extraction

**Action:** Extract uncompressed mesh data from HOD 1.0 LMIP  
**Rationale:** Required for HOD 1.0 → 2.0 conversion  
**Effort:** Medium

### Priority 3: COLD/KDOP Conversion

**Action:** Convert between COLD/BNDV and KDOP  
**Rationale:** Required for both conversion directions  
**Effort:** High

### Priority 4: Complete Converter

**Action:** Implement full HOD 1.0 ↔ 2.0 converter  
**Rationale:** Final goal  
**Effort:** High

---

## References

### Source Code
- `parser/src/hod.rs` - HOD parsing and generation
- `parser/src/iff.rs` - IFF chunk handling

### Test Data
- External `<HODEditorJS>/testing/pebble_0/` - HOD 2.0 test cases. See [`docs/testing-fixtures.md`](../testing-fixtures.md) for fixture acquisition.
- `uncompressed_bigs/freespace_remastered/` - HOD 1.0 files

### Documentation
- `hod2-creation-specification.md` - HOD 2.0 format
- `pool-chunk-specification.md` - POOL chunk details

---

**Document Version:** 1.0  
**Last Updated:** 2026-05-27  
**Status:** Analysis Complete
