# HOD 2.0 Reverse Engineering Project

## Overview

This directory contains all documentation and analysis for reverse engineering the HOD 2.0 file format used by Homeworld Remastered. The current project objective is to create HOD 2.0 files from source assets and editor-authored metadata, then validate the generated HOD against HODOR output. This enables:

1. **Lossless editing** of existing HOD 2.0 files
2. **Creation** of new HOD 2.0 files from scratch
3. **Conversion** from HOD 1.3 to HOD 2.0
4. **Import** of DAE files to HOD 2.0

## Directory Structure

```
docs/hod2-reverse-engineering/
├── README.md                          # This file
├── architecture-overview.md           # High-level editor workflow and pipeline objective
├── PROGRESS.md                        # Progress tracking (UPDATE REGULARLY)
├── QUICKSTART.md                      # Quick start guide for new agents
├── testing-guide.md                   # How to run pebble tests
├── progress-reporting-standard.md     # Rules for status updates and source inputs
├── hod2-creation-specification.md     # Complete HOD 2.0 format specification
├── phase1-summary.md                  # Phase 1 completion summary
├── phase2-gap-analysis.md             # Phase 2 gap analysis
└── rodoh-hod-conversion-analysis.md   # RODOH conversion analysis
```

## Related Files

### Primary Source Code
- `parser/src/hod.rs` - HOD parser and serializer
- `parser/src/compiler.rs` - Mesh compilation logic
- `parser/src/iff.rs` - IFF chunk handling
- `parser/src/xpress.rs` - Microsoft Xpress compression

### Test Data
- `testing/pebble_0/` - Simple test case (2 LODs, 1 material)
- `testing/pebble_1/` - Test case with 3 LODs
- `testing/pebble_2/` - Additional test case

### Reference Documentation
- `hod2_reverse_engineering_knowledge_base.md` - Core knowledge base (Migrated from `agents_info`)
- `archive_logs/hod2_serialization_walkthrough.md` - Implementation history (Archived)
- `archive_logs/implementation_plan.md` - Legacy project goals and plan (Archived)
- `../../../.opencode/skills/hod-binary-layout/SKILL.md` - Binary layout specs

### External Tools
- `GBXTools/HODOR/RODOH.exe` - Official conversion tool
- `GBXTools/HODOR/HODOR.exe` - Official conversion tool
- `GBXTools/HODOR/SHADERS.MAP` - Shader mapping configuration

## Current Status

**Phase:** Phase 1 Complete  
**Last Updated:** 2026-05-28  
**Next Phase:** HODOR source-asset replication validation

### Completion Status

| Phase | Status | Description |
|-------|--------|-------------|
| Phase 1 | ✅ COMPLETE | Knowledge consolidation, specification creation |
| Phase 2 | IN PROGRESS | Source asset ingestion and HODOR comparison |
| Phase 3 | IN PROGRESS | Validation suite and documentation cleanup |
| Phase 4 | PENDING | Editor integration and in-game validation |

## Quick Start

### For New Agents

1. **Read this README** - Understand project scope
2. **Read PROGRESS.md** - Understand current status
3. **Read progress-reporting-standard.md** - Understand valid source inputs and update format
4. **Read hod2-creation-specification.md** - Understand the format
5. **Check related files** - Review source code and test data
6. **Continue from current phase** - Don't repeat completed work

### For Developers

1. **Review specification** - Understand HOD 2.0 structure
2. **Examine test cases** - Study `testing/pebble_0/`
3. **Run verify_lossless** - Test round-trip preservation
4. **Check compiler.rs** - Understand serialization logic

## Key Concepts

### HOD 2.0 File Structure

```
VERS → NAME → POOL → HVMD → DTRM → INFO
```

- **VERS**: Version (512 = HOD 2.0)
- **NAME**: Model name ("Homeworld2 Multi Mesh File")
- **POOL**: Compressed data (textures, meshes, faces)
- **HVMD**: Visual data container (LODs, statistics, meshes)
- **DTRM**: Transform data container (hierarchy, collision, navlights)
- **INFO**: File information

### Critical Rules

1. **NO top-level FORM wrapper** - Flat sequence of chunks
2. **POOL compression** - Microsoft Xpress algorithm
3. **Vertex format** - 64 bytes (Position + Normal + UV + Tangent + Bitangent)
4. **Endianness** - Big-Endian headers, Little-Endian payloads
5. **Chunk order** - Must be VERS, NAME, POOL, HVMD, DTRM, INFO
6. **Source asset rule** - HODOR replication must ingest OBJ/MTL/TGA and editor-authored metadata; never use `model.json` or processed HOD mesh/texture payloads as implementation source data

## Testing

### Test Cases Available

**pebble_0:**
- 2 LOD levels (144 vertices, 72 vertices)
- 1 material (pebblemat with ship shader)
- 3 textures (DIFF, GLOW, NORM)
- No navlights, dockpaths, or collision mesh

**pebble_1:**
- 3 LOD levels
- 1 material
- 3 textures

**pebble_2:**
- Similar structure to pebble_0/pebble_1

### Running Tests

See [testing-guide.md](testing-guide.md) for the full test suite reference.

Quick start:

```bash
cd parser
cargo run --bin replicate_testing   # Regenerate from-assets HODs
cargo run --bin testing_diff         # Compare vanilla vs roundtrip vs from_assets
cargo run --bin verify_lossless      # Mandatory roundtrip + DAE verification
```

## Known Issues

### Current Limitations

1. **HOD 1.0 → 2.0 conversion** - Not fully implemented
2. **DAE → HOD 2.0 conversion** - Not implemented
3. **Texture compression** - DXT1/DXT3/DXT5 supported; encoder output may differ byte-for-byte from original tools
4. **Editor integration** - Source-asset HOD creation is still driven by test binaries

### Critical Quirks

1. **NAME chunk** - No trailing null byte
2. **MULT lod_count** - Written after parent name string
3. **BMSH endianness** - Little-Endian (not Big-Endian)
4. **HIER first_val** - Encodes the full signed negative joint count as two's complement
5. **TAGS chunk** - Optional in MULT, preserve if present

## Progress Tracking

**IMPORTANT:** Update `PROGRESS.md` regularly to preserve knowledge in case of interruptions. Use `progress-reporting-standard.md` for the required update format and source-input rules.

### What to Track

- Current phase and status
- Completed tasks
- Pending tasks
- Key findings
- Decisions made
- Issues encountered
- Next steps
- Source inputs used by each test, including an explicit note that `model.json` is not used

### Update Frequency

- After completing each task
- When discovering important information
- When making decisions
- At end of each session

## References

### Documentation

- [HOD 2.0 Creation Specification](hod2-creation-specification.md)
- [Testing Guide](testing-guide.md)
- [Phase 1 Summary](archive_logs/phase1-summary.md)
- [RODOH Conversion Analysis](archive_logs/rodoh-hod-conversion-analysis.md)
- [Knowledge Base](hod2_reverse_engineering_knowledge_base.md)
- [Serialization Walkthrough](archive_logs/hod2_serialization_walkthrough.md)

### Source Code

- [HOD Parser](../../parser/src/hod.rs)
- [Compiler](../../parser/src/compiler.rs)
- [IFF Handler](../../parser/src/iff.rs)
- [Xpress Compression](../../parser/src/xpress.rs)

### Test Data

- [Pebble 0](../../testing/pebble_0/)
- [Pebble 1](../../testing/pebble_1/)
- [Pebble 2](../../testing/pebble_2/)

---

**Project Started:** 2026-05-27  
**Last Updated:** 2026-05-28  
**Maintained By:** HODEditorJS Team
