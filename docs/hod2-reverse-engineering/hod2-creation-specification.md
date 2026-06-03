# HOD 2.0 Creation Specification - Phase 1: Knowledge Consolidation

## Executive Summary

This document consolidates all known information about creating HOD 2.0 files for Homeworld Remastered. It is based on analysis of:
- Vanilla HOD 2.0 files (pebble_0, pebble_1, pebble_2)
- RODOH-generated HOD files (ter_pharos)
- HODEditorJS parser implementation
- Existing knowledge base documents

## 1. HOD 2.0 File Structure

### 1.1 High-Level Layout

```
┌─────────────────────────────────────────────────────────────┐
│ VERS (4 bytes)                                              │
├─────────────────────────────────────────────────────────────┤
│ NAME (26 bytes)                                             │
├─────────────────────────────────────────────────────────────┤
│ POOL (variable) - COMPRESSED DATA                           │
│   ├── Texture Pool (compressed textures)                    │
│   ├── Mesh Pool (interleaved vertex data)                   │
│   └── Face Pool (triangle indices)                          │
├─────────────────────────────────────────────────────────────┤
│ HVMD (Form container)                                       │
│   ├── LMIP (87 bytes each) × LOD count                     │
│   ├── STAT (92-93 bytes)                                    │
│   └── MULT (variable) - Mesh container                      │
├─────────────────────────────────────────────────────────────┤
│ DTRM (Form container)                                       │
│   ├── HIER (52-733 bytes) - Hierarchy                       │
│   ├── NAVL (580 bytes) - Navigation lights (optional)       │
│   ├── KDOP (1588 bytes) - Collision tree                    │
│   ├── COLD (8 bytes) - Collision data (optional)            │
│   └── SCAR (variable) - Battle scars (optional)             │
├─────────────────────────────────────────────────────────────┤
│ INFO (49-50 bytes)                                          │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 Critical Rules

1. **NO top-level FORM wrapper** - HOD 2.0 files are flat sequences of chunks
2. **Chunk order matters** - VERS, NAME, POOL, HVMD, DTRM, INFO
3. **Endianness** - Big-Endian for IFF headers, Little-Endian for payload data
4. **POOL compression** - Microsoft Xpress algorithm for textures and meshes

## 2. Chunk Specifications

### 2.1 VERS Chunk (Version)

**Size:** 4 bytes  
**Payload:** Little-Endian u32  
**Value:** 512 (0x00000200)

```
Offset  Size  Description
0       4     Version (512 = HOD 2.0)
```

### 2.2 NAME Chunk (Model Name)

**Size:** 26 bytes  
**Payload:** ASCII string (NO null terminator)

```
Offset  Size  Description
0       26    "Homeworld2 Multi Mesh File"
```

### 2.3 POOL Chunk (Compressed Data)

**Size:** Variable  
**Type:** Default (not Form)

**Internal Structure:**
```
POOL Header
├── Texture Pool
│   ├── Compressed size (u32)
│   ├── Decompressed size (u32)
│   └── DXT-compressed texture data
├── Mesh Pool
│   ├── Compressed size (u32)
│   ├── Decompressed size (u32)
│   └── Interleaved vertex data
└── Face Pool
    ├── Compressed size (u32)
    ├── Decompressed size (u32)
    └── 16-bit triangle indices
```

**Compression:** Microsoft Xpress (LZ77-based)

### 2.4 HVMD Container (Visual Data)

**Type:** Form container (size = 0)

Children:
1. **LMIP** (87 bytes each) - LOD definitions
2. **STAT** (92-93 bytes) - Statistics
3. **MULT** (variable) - Mesh container

### 2.5 MULT Chunk (Mesh Container)

**Structure:**
```
MULT Payload
├── name: String (u32 length + UTF-8)
├── parent_name: String (u32 length + UTF-8)
├── lod_count: u32 (Little-Endian)
├── FORM TAGS (optional) - "DoScars"
└── NRML BMSH - Mesh data
```

**Critical:** BMSH uses Little-Endian size fields!

### 2.6 BMSH Chunk (Basic Mesh)

**Structure:**
```
BMSH Payload (Little-Endian)
├── lod: u32
├── part_count: u32
├── reserved: u32
├── vertex_format: u32 (0x600B = standard)
├── vertex_count: u32
├── index_count: u32
└── mesh_data: bytes
```

### 2.7 DTRM Container (Transform Data)

**Type:** Form container (size = 0)

Children:
1. **HIER** (52-733 bytes) - Hierarchy
2. **NAVL** (580 bytes) - Navigation lights (optional)
3. **KDOP** (1588 bytes) - Collision tree
4. **COLD** (8 bytes) - Collision data (optional)
5. **SCAR** (variable) - Battle scars (optional)

### 2.8 HIER Chunk (Hierarchy)

**Structure:**
```
HIER Payload (Little-Endian)
├── first_val: u32
│   └── Encoding: full two's-complement signed negative joint count, i.e. (-joint_count as i32) reinterpreted as u32
├── joints: Array of Joint structures
└── ...
```

**Critical:** `first_val` encodes joint count as a full signed two's-complement value. Do not mask to the low byte; files with more than 255 joints legitimately use values like `0xFFFFFED6` for `-298`.

### 2.9 INFO Chunk (File Information)

**Size:** 49-50 bytes  
**Payload:** Mixed data (metadata, checksums, etc.)

## 3. Vertex Format

### 3.1 Standard Vertex (64 bytes)

```
Offset  Size  Description
0       12    Position (X, Y, Z) - float32 × 3
12      12    Normal (X, Y, Z) - float32 × 3
24      8     UV (U, V) - float32 × 2
32      16    Tangent (X, Y, Z, W) - float32 × 4
48      16    Bitangent (X, Y, Z, W) - float32 × 4
```

### 3.2 Vertex Format Mask

**Standard mask:** 0x600B

This mask indicates:
- Position present
- Normal present
- UV0 present
- Tangent/Bitangent present

## 4. LOD System

### 4.1 LMIP Chunk (LOD Definition)

**Size:** 87 bytes each  
**Count:** One per LOD level

**Structure:**
```
LMIP Payload
├── LOD level (u32)
├── Distance thresholds (floats)
└── Quality settings
```

### 4.2 LOD Levels

**Typical LOD count:** 2-3 levels

**Example (pebble_0):**
- LOD 0: 144 vertices (highest detail)
- LOD 1: 72 vertices (medium detail)
- LOD 2: (not present in pebble_0)

## 5. Material System

### 5.1 Material Naming Convention

```
MAT[TextureName]_SHD[ShaderType]
```

**Example:** `MAT[pebblemat]_SHD[ship]`

### 5.2 Shader Types

Common shaders:
- `ship` - Standard ship shader
- `matte` - Matte shader
- `thruster` - Engine thruster
- `background` - Background objects

### 5.3 Texture Roles

| Role | Description | Source |
|------|-------------|--------|
| DIFF | Diffuse texture | Texture name suffix |
| GLOW | Glow/emission | Texture name suffix |
| SPEC | Specular | Texture name suffix |
| TEAM | Team color mask | Texture name suffix |
| STRP | Stripe mask | Texture name suffix |
| NORM | Normal map | Texture name suffix |

## 6. Compression Details

### 6.1 Microsoft Xpress Algorithm

- **Type:** LZ77-based compression
- **Block size:** 256 bytes
- **Typical ratio:** ~4:1 for textures, ~4:1 for meshes

### 6.2 Compression Differences

**Vanilla vs Roundtrip (pebble_0):**

| Pool | Vanilla | Roundtrip | Difference |
|------|---------|-----------|------------|
| Texture | 1,528,396 bytes | 977,396 bytes | -36% |
| Mesh | 114,516 bytes | 114,516 bytes | 0% |
| Face | 344 bytes | 332 bytes | -3% |

**Note:** Texture compression varies based on settings/quality.

## 7. Testing Data

### 7.1 Available Test Cases

**Directory:** `testing/pebble_0/`

**Files:**
- `pebble_0_vanilla.hod` - Original HOD 2.0
- `pebble_0_roundtrip.hod` - Parser-generated HOD
- `pebble_0_from_assets.hod` - Created from assets
- `Root_mesh_lod0.obj` - LOD 0 mesh
- `Root_mesh_lod1.obj` - LOD 1 mesh
- `Pebble_DIFF.tga` - Diffuse texture
- `Pebble_GLOW.tga` - Glow texture
- `Pebble_NORM.tga` - Normal map
- `Homeworld2 Multi Mesh File_materials.json` - Material definitions

### 7.2 Test Case Structure

**pebble_0:**
- 2 LOD levels (LOD 0: 144 verts, LOD 1: 72 verts)
- 1 material (pebblemat with ship shader)
- 3 textures (DIFF, GLOW, NORM)
- No navlights, no dockpaths, no collision mesh

**pebble_1:**
- 3 LOD levels
- 1 material
- 3 textures

**pebble_2:**
- Similar structure to pebble_0/pebble_1

## 8. Known Quirks & Pitfalls

### 8.1 Critical Issues

1. **NAME chunk** - Do NOT append trailing null byte
2. **MULT lod_count** - Must be written after parent name string
3. **BMSH endianness** - Uses Little-Endian, not Big-Endian
4. **TAGS chunk** - Optional in MULT, preserve if present
5. **HIER first_val** - Encodes the full signed negative joint count as two's complement, not just the low byte

### 8.2 Node Hierarchy

- Joints, navlights, engine burns are structurally intertwined
- Deleting a parent must cascade-delete children
- Leaving orphaned references causes game crashes

## 9. Current Implementation Status

### 9.1 HODEditorJS Parser

**Status:** Phase 5 COMPLETED (As of May 2026)

- ✅ Parsing HOD 2.0 files
- ✅ Serializing HOD 2.0 files
- ✅ Round-trip verification
- ✅ HOD 1.0 → HOD 2.0 conversion (Auto-upgrade on save)
- ✅ DAE → HOD 2.0 conversion (Native multi-material import)
- ✅ Texture compression pipeline (Preserves 8888 or compresses via Xpress)
- ✅ Tangent/bitangent calculation (Byte-matching HODOR deterministic math)
- ✅ Microsoft Xpress LZ77 algorithm completely replicated natively in Rust

### 9.2 Outstanding Capabilities

- ⏳ DAE complex node mapping (Handling advanced DAEnerys output hierarchies)
- ⏳ Parameterized mathematical generation of `KDOP` and `SCAR` chunks from raw geometry (Currently preserves existing chunks but relies on engine fallback for from-scratch DAE imports)

---

**Document Version:** 2.0  
**Last Updated:** 2026-05-29  
**Status:** Format Specification Complete. Parser Replicated.
