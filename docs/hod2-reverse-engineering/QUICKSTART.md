# HOD 2.0 Reverse Engineering - Quick Start Guide

## For New Agents

**STOP!** Read this guide before doing anything else.

### Step 1: Understand the Project

This project reverse engineers the HOD 2.0 file format used by Homeworld Remastered. The goal is to:
- Edit existing HOD 2.0 files losslessly
- Create new HOD 2.0 files from scratch
- Convert HOD 1.0 to HOD 2.0
- Import DAE files to HOD 2.0

**Read `architecture-overview.md` immediately to understand the high-level workflow.**

### Step 2: Check Current Status

**Read `PROGRESS.md` immediately.**

This document tells you:
- What phase we're in
- What's been completed
- What's pending
- Key findings
- Decisions made

**Do NOT repeat completed work.**

### Step 3: Read the Specification

**Read `hod2-creation-specification.md` next.**

This document explains:
- HOD 2.0 file structure
- Binary format for all chunks
- Vertex format
- Compression details
- Critical quirks

### Step 4: Examine Test Data

**Look at `testing/pebble_0/` directory.**

This contains:
- `pebble_0_vanilla.hod` - Original HOD 2.0
- `pebble_0_roundtrip.hod` - Parser-generated HOD
- `Root_mesh_lod0.obj` - LOD 0 mesh
- `Root_mesh_lod1.obj` - LOD 1 mesh
- `Pebble_DIFF.tga` - Diffuse texture
- `Pebble_GLOW.tga` - Glow texture
- `Pebble_NORM.tga` - Normal map
- Material definitions

### Step 5: Review Source Code

**Check these files:**
- `parser/src/hod.rs` - Parser and serializer
- `parser/src/compiler.rs` - Mesh compilation
- `parser/src/iff.rs` - IFF chunk handling
- `parser/src/xpress.rs` - Compression

### Step 6: Run Tests

**Execute:**
```bash
cd parser
cargo run --bin verify_lossless
```

This validates round-trip preservation.

---

## Critical Rules

### 1. HOD 2.0 Structure

```
VERS → NAME → POOL → HVMD → DTRM → INFO
```

**NO top-level FORM wrapper.**

### 2. Chunk Order Matters

Must be in this exact order:
1. VERS (4 bytes)
2. NAME (26 bytes)
3. POOL (variable)
4. HVMD (Form container)
5. DTRM (Form container)
6. INFO (49-50 bytes)

### 3. Endianness

- **Big-Endian:** IFF chunk headers and sizes
- **Little-Endian:** Payload data (vertices, indices, etc.)

**Exception:** BMSH uses Little-Endian for size fields!

### 4. Compression

**POOL chunk** uses Microsoft Xpress compression:
- ~4:1 compression ratio
- Textures, meshes, and faces are compressed separately
- Block size: 256 bytes

### 5. Vertex Format

**64 bytes per vertex:**
```
Position (12) + Normal (12) + UV (8) + Tangent (16) + Bitangent (16)
```

**Mask:** 0x600B (standard format)

---

## Common Pitfalls

### 1. NAME Chunk Null Terminator

**WRONG:** Write null terminator  
**RIGHT:** NO null terminator

```rust
// WRONG
writer.write_all(b"Homeworld2 Multi Mesh File\0")?;

// RIGHT
writer.write_all(b"Homeworld2 Multi Mesh File")?;
```

### 2. BMSH Endianness

**WRONG:** Use Big-Endian for BMSH size  
**RIGHT:** Use Little-Endian for BMSH size

```rust
// WRONG
chunk.write_u32::<BigEndian>(size)?;

// RIGHT
chunk.write_u32::<LittleEndian>(size)?;
```

### 3. HIER First Value

**WRONG:** Write joint count directly  
**RIGHT:** Encode as two's complement

```rust
// WRONG
writer.write_u32::<LittleEndian>(joint_count)?;

// WRONG for files with more than 255 joints
let first_val = 0xFFFFFF00 | ((-joint_count as i32) & 0xFF) as u32;
writer.write_u32::<LittleEndian>(first_val)?;

// RIGHT
let first_val = (-(joint_count as i32)) as u32;
writer.write_u32::<LittleEndian>(first_val)?;
```

### 4. MULT Lod Count

**WRONG:** Write lod_count before parent name  
**RIGHT:** Write lod_count after parent name

```rust
// WRONG
writer.write_u32::<LittleEndian>(lod_count)?;
write_len_string(writer, &mesh.name)?;
write_len_string(writer, &mesh.parent_name)?;

// RIGHT
write_len_string(writer, &mesh.name)?;
write_len_string(writer, &mesh.parent_name)?;
writer.write_u32::<LittleEndian>(lod_count)?;
```

---

## Testing

### Run All Tests

```bash
cd parser
cargo run --bin verify_lossless
```

### Run Specific Test

```bash
cd parser
cargo run --bin test_pebble
```

### Check Test Results

- **Success:** "Reparsed: Meshes=X" where X > 0
- **Failure:** Error messages or mesh count = 0

---

## Documentation Structure

```
docs/hod2-reverse-engineering/
├── README.md                      # Project overview
├── PROGRESS.md                    # Progress tracking (UPDATE REGULARLY)
├── QUICKSTART.md                  # This file
├── hod2-creation-specification.md # Complete format spec
├── phase1-summary.md              # Phase 1 summary
└── rodoh-hod-conversion-analysis.md # RODOH analysis
```

---

## Getting Help

### If You're Stuck

1. **Read PROGRESS.md** - Check current status
2. **Read hod2-creation-specification.md** - Understand the format
3. **Examine test data** - Study `testing/pebble_0/`
4. **Review source code** - Check `parser/src/`
5. **Run tests** - Execute `verify_lossless`

### Key Files

- **Specification:** `hod2-creation-specification.md`
- **Progress:** `PROGRESS.md`
- **Source:** `parser/src/hod.rs`
- **Tests:** `testing/pebble_0/`

---

## Remember

1. **Update PROGRESS.md** after every session
2. **Document everything** you discover
3. **Don't repeat work** - Check what's been done
4. **Preserve knowledge** - In case of interruptions
5. **Test everything** - Use verify_lossless

---

**Last Updated:** 2026-05-27  
**Maintained By:** HODEditorJS Team
