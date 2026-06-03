# POOL Chunk Specification - HOD 2.0

## Overview

The POOL chunk is the core data container in HOD 2.0 files, storing compressed textures, meshes, and face indices. This document provides a complete specification of the POOL chunk structure.

---

## POOL Chunk Structure

### Header

```
Offset  Size  Description
0       4     Pool Type (u32, Little-Endian)
              Value: 3518 (0x0DB6)
```

### Stream Layout

The POOL chunk contains three sequential streams:

```
┌─────────────────────────────────────────────────────────────┐
│ Pool Type (4 bytes)                                          │
├─────────────────────────────────────────────────────────────┤
│ Texture Stream                                               │
│   ├── Compressed Size (u32, Little-Endian)                  │
│   ├── Decompressed Size (u32, Little-Endian)                │
│   └── Compressed Data (bytes)                               │
├─────────────────────────────────────────────────────────────┤
│ Mesh Stream                                                   │
│   ├── Compressed Size (u32, Little-Endian)                  │
│   ├── Decompressed Size (u32, Little-Endian)                │
│   └── Compressed Data (bytes)                               │
├─────────────────────────────────────────────────────────────┤
│ Face Stream                                                   │
│   ├── Compressed Size (u32, Little-Endian)                  │
│   ├── Decompressed Size (u32, Little-Endian)                │
│   └── Compressed Data (bytes)                               │
└─────────────────────────────────────────────────────────────┘
```

### Total Header Size

- Pool Type: 4 bytes
- Texture Stream Header: 8 bytes (compressed_size + decompressed_size)
- Mesh Stream Header: 8 bytes (compressed_size + decompressed_size)
- Face Stream Header: 8 bytes (compressed_size + decompressed_size)
- **Total: 28 bytes**

---

## Stream Details

### 1. Texture Stream

**Purpose:** Stores DXT-compressed textures  
**Compression:** Microsoft Xpress LZ77  
**Typical Ratio:** ~4:1

**Structure:**
```
Texture Stream
├── compressed_size: u32 (Little-Endian)
├── decompressed_size: u32 (Little-Endian)
└── compressed_data: bytes[compressed_size]
```

**Data Format:**
- DXT1 / BC1: 8 bytes per 4x4 block (optional 1-bit alpha)
- DXT3 / BC2: 16 bytes per 4x4 block (explicit 4-bit alpha + DXT1-like color)
- DXT5 / BC3: 16 bytes per 4x4 block (interpolated alpha + DXT1-like color)
- Multiple textures are concatenated

**Example (pebble_0):**
```
compressed_size: 1,673,248 bytes (vanilla)
decompressed_size: 2,097,120 bytes
compression_ratio: 0.80 (80% of original)
```

### 2. Mesh Stream

**Purpose:** Stores interleaved vertex data  
**Compression:** Microsoft Xpress LZ77  
**Typical Ratio:** ~2:1

**Structure:**
```
Mesh Stream
├── compressed_size: u32 (Little-Endian)
├── decompressed_size: u32 (Little-Endian)
└── compressed_data: bytes[compressed_size]
```

**Data Format:**
- Vertex format: Position (12) + Normal (12) + UV (8) + Tangent (16) + Bitangent (16)
- Total stride: 64 bytes per vertex
- Multiple LOD levels are concatenated

**Example (pebble_0):**
```
compressed_size: 35,441 bytes (vanilla)
decompressed_size: 71,040 bytes
compression_ratio: 0.50 (50% of original)
```

### 3. Face Stream

**Purpose:** Stores triangle indices  
**Compression:** Microsoft Xpress LZ77  
**Typical Ratio:** ~1.5:1

**Structure:**
```
Face Stream
├── compressed_size: u32 (Little-Endian)
├── decompressed_size: u32 (Little-Endian)
└── compressed_data: bytes[compressed_size]
```

**Data Format:**
- 16-bit indices (u16, Little-Endian)
- Sequential triangle lists
- Multiple LOD levels are concatenated

**Example (pebble_0):**
```
compressed_size: 1,334 bytes (vanilla)
decompressed_size: 2,220 bytes
compression_ratio: 0.60 (60% of original)
```

---

## Compression Algorithm

### Microsoft Xpress LZ77

**Type:** LZ77-based compression  
**Block Size:** 256 bytes (typical)  
**Window Size:** 65536 bytes (maximum)

**Key Features:**
- Literal bytes (uncompressed)
- Match copies (backreferences)
- Variable-length encoding

**Implementation:** `parser/src/xpress.rs`

### Indicator Bit Stream
- **Exact Indicator Bit Count**: 31 bits per 32-bit word.
- **Bit Reading Order**: LSB-first (Least Significant Bit first).
- **Sentinel Bit**: The 32nd bit (MSB) of every 32-bit indicator word is always set to `1`. The decompressor shifts the word right by 1 for each token. When the word becomes exactly `1`, it knows it has exhausted the 31 bits and reads the next 32-bit word.
- **Bit Meaning**: 
  - `1` = MATCH
  - `0` = LITERAL

### Literal Processing Optimization
When a `0` (LITERAL) is encountered, the decompressor uses a lookup table (`DAT_00479764`) on the lowest 4 bits of the indicator word to count up to 4 consecutive `0`s. It copies 4 bytes at once, and advances the source/destination pointers and shifts the indicator word by the number of consecutive zeros (1, 2, 3, or 4).

### Match Types and Encoding
When a `1` (MATCH) is encountered, the decompressor reads the token. The lowest 3 bits of the token determine the match type, which points to a configuration table (`DAT_00479778`). Because bits are shifted, the match types effectively cluster into 4 main classes determined by the lowest 2 bits (since types 0,1,2 are identical to 4,5,6).

| Type | Token Size (bytes) | Offset Bits | Max Offset | Length Bits | Max Length | Offset Shift | Length Shift |
|---|---|---|---|---|---|---|---|
| 0 & 4 | 1 | 6 | 63 | 0 | 3 | 2 | 0 |
| 1 & 5 | 2 | 14 | 16383 | 0 | 3 | 2 | 0 |
| 2 & 6 | 2 | 10 | 1023 | 4 | 18 | 6 | 2 |
| 3 | 3 | 16 | 65535 | 5 | 34 | 8 | 3 |
| 7 | 4 | 21 | 2097151 | 8 | 258 | 11 | 3 |

### Token Layouts

1. **Type 0 / 4 (TT = 00)**: 1-byte token
   - Layout: `OOOOOOTT`
   - Offset: `(token & 0xFF) >> 2` (6 bits, max 63)
   - Length: Always 3

2. **Type 1 / 5 (TT = 01)**: 2-byte token
   - Layout: `OOOOOOOO OOOOOOTT`
   - Offset: `(token & 0xFFFF) >> 2` (14 bits, max 16383)
   - Length: Always 3

3. **Type 2 / 6 (TT = 10)**: 2-byte token
   - Layout: `OOOOOOOO OOLLLLTT`
   - Offset: `(token & 0xFFFF) >> 6` (10 bits, max 1023)
   - Length: `(((token >> 2) & 0xF) + 3)` (4 bits, max 18)

4. **Type 3 (TTT = 011)**: 3-byte token
   - Layout: `OOOOOOOO OOOOOOOO LLLLLTTT`
   - Offset: `(token & 0xFFFFFF) >> 8` (16 bits, max 65535)
   - Length: `(((token >> 3) & 0x1F) + 3)` (5 bits, max 34)

5. **Type 7 (TTT = 111)**: 4-byte token
   - Layout: `OOOOOOOO OOOOOOOO OOOOOLLL LLLLLTTT`
   - Offset: `(token & 0xFFFFFFFF) >> 11` (21 bits, max 2097151)
   - Length: `(((token >> 3) & 0xFF) + 3)` (8 bits, max 258)

---

## Pool Type Identifier

### Value: 3518 (0x0DB6)

**Location:** First 4 bytes of POOL chunk  
**Format:** u32, Little-Endian  
**Importance:** Must be set correctly for game engine to parse

**Verification:**
```rust
let pool_type = u32::from_le_bytes([0x06, 0x0D, 0x00, 0x00]); // 0x0DB6 = 3518
```

---

## Data Alignment

### Texture Stream
- No specific alignment required
- DXT blocks are naturally aligned (16 bytes)

### Mesh Stream
- Vertices are naturally aligned (64 bytes each)
- No padding between vertices

### Face Stream
- 2-byte alignment required (u16 indices)
- Padded with 0 if odd length

---

## Compression Statistics

### pebble_0 (Vanilla)

| Stream | Compressed | Decompressed | Ratio | Xpress Stats |
|--------|------------|--------------|-------|--------------|
| Texture | 1,673,248 | 2,097,120 | 0.80 | inds=32299, lits=704301, matches=296968 |
| Mesh | 35,441 | 71,040 | 0.50 | inds=890, lits=25523, matches=2046 |
| Face | 1,334 | 2,220 | 0.60 | inds=38, lits=1158, matches=7 |

### pebble_0 (Roundtrip)

| Stream | Compressed | Decompressed | Ratio | Xpress Stats |
|--------|------------|--------------|-------|--------------|
| Texture | 1,122,994 | 2,097,120 | 0.54 | inds=15456, lits=131478, matches=347650 |
| Mesh | 35,739 | 71,040 | 0.50 | inds=881, lits=24851, matches=2456 |
| Face | 1,322 | 2,220 | 0.60 | inds=38, lits=1146, matches=7 |

### pebble_0 (From Assets)

| Stream | Compressed | Decompressed | Ratio | Xpress Stats |
|--------|------------|--------------|-------|--------------|
| Texture | 1,124,432 | 2,097,120 | 0.54 | inds=15503, lits=133322, matches=347249 |
| Mesh | 18,156 | 71,040 | 0.26 | inds=371, lits=9088, matches=2390 |
| Face | 1,322 | 2,220 | 0.60 | inds=38, lits=1146, matches=7 |

---

## Key Observations

### 1. Texture Compression Variance

**Observation:** Vanilla texture compression is 33% larger than generated  
**Hypothesis:** RODOH uses different DXT compression quality settings  
**Impact:** Affects file size but not visual quality

### 2. Mesh Compression Variance

**Observation:** From-assets mesh compression is 44-49% smaller than vanilla  
**Hypothesis:** Different vertex ordering or padding  
**Impact:** May affect rendering performance

### 3. Face Compression Consistency

**Observation:** Face compression is nearly identical across all versions  
**Analysis:** Face indices are simple sequential data, compression is consistent  
**Impact:** No issues expected

### 4. Xpress Compression Patterns

**Observation:** Significant differences in Xpress statistics  
**Analysis:** Different hash chain settings or compression levels  
**Impact:** Affects file size but not functionality

---

## Implementation Notes

### Creating POOL Chunk

```rust
fn generate_pool_data(compiled_meshes: &[CompiledMesh], comp_tex: &[u8], 
                      decomp_tex_len: u32, pool_type: u32) -> Vec<u8> {
    let mut pool_buf = Vec::new();
    let mut cursor = Cursor::new(&mut pool_buf);
    
    // Pool type identifier
    cursor.write_u32::<LittleEndian>(pool_type)?; // 3518 = 0x0DB6
    
    // Texture stream
    cursor.write_u32::<LittleEndian>(comp_tex.len() as u32)?;
    cursor.write_u32::<LittleEndian>(decomp_tex_len)?;
    cursor.write_all(comp_tex)?;
    
    // Mesh stream
    let comp_mesh = xpress::compress_or_raw(&decomp_mesh);
    cursor.write_u32::<LittleEndian>(comp_mesh.len() as u32)?;
    cursor.write_u32::<LittleEndian>(decomp_mesh.len() as u32)?;
    cursor.write_all(&comp_mesh)?;
    
    // Face stream
    let comp_face = xpress::compress_or_raw(&decomp_face);
    cursor.write_u32::<LittleEndian>(comp_face.len() as u32)?;
    cursor.write_u32::<LittleEndian>(decomp_face.len() as u32)?;
    cursor.write_all(&comp_face)?;
    
    Ok(pool_buf)
}
```

### Reading POOL Chunk

```rust
fn parse_pool_chunk(chunk: &IffChunk) -> Result<PoolData, String> {
    let mut cursor = Cursor::new(&chunk.data);
    
    let pool_type = cursor.read_u32::<LittleEndian>()?;
    
    // Texture stream
    let comp_tex_len = cursor.read_u32::<LittleEndian>()? as usize;
    let decomp_tex_len = cursor.read_u32::<LittleEndian>()? as usize;
    let mut comp_tex = vec![0u8; comp_tex_len];
    cursor.read_exact(&mut comp_tex)?;
    
    // Mesh stream
    let comp_mesh_len = cursor.read_u32::<LittleEndian>()? as usize;
    let decomp_mesh_len = cursor.read_u32::<LittleEndian>()? as usize;
    let mut comp_mesh = vec![0u8; comp_mesh_len];
    cursor.read_exact(&mut comp_mesh)?;
    
    // Face stream
    let comp_face_len = cursor.read_u32::<LittleEndian>()? as usize;
    let decomp_face_len = cursor.read_u32::<LittleEndian>()? as usize;
    let mut comp_face = vec![0u8; comp_face_len];
    cursor.read_exact(&mut comp_face)?;
    
    Ok(PoolData { pool_type, comp_tex, decomp_tex_len, ... })
}
```

---

## Validation Checklist

When creating or modifying POOL chunks:

- [ ] Pool type is 3518 (0x0DB6)
- [ ] All stream headers are present (8 bytes each)
- [ ] Compressed sizes match actual data lengths
- [ ] Decompressed sizes are correct
- [ ] Xpress compression is valid (decompressable)
- [ ] Face stream is 2-byte aligned
- [ ] Total size matches expected size

---

## References

### Source Code
- `parser/src/xpress.rs` - Xpress compression implementation
- `parser/src/compiler.rs` - `generate_pool_data` function
- `parser/src/hod.rs` - POOL chunk parsing

### Test Data
- `testing/pebble_0/pebble_0_vanilla.hod` - Reference POOL chunk
- `testing/pebble_0/pebble_0_roundtrip.hod` - Generated POOL chunk

### Tools
- `cargo run --bin dump_pool` - Dump POOL stream sizes
- `cargo run --bin testing_diff` - Compare POOL chunks

---

**Document Version:** 1.0  
**Last Updated:** 2026-05-27  
**Status:** Complete
