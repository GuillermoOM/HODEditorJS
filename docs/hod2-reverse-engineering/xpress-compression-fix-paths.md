# Xpress Compression Fix — Agent Entry Point

## Problem Statement

Our Xpress LZ77 compressor produces byte patterns that the Homeworld Remastered game engine's decompressor cannot handle. This affects ALL three POOL streams (texture, mesh, face). The model renders correctly only when using HODOR's compressed bytes or uncompressed data.

**Proven by:** Hybrid swap tests on `ter_centaur` — only `hybrid_all_from_hodor.hod` renders correctly. Bypassing compression (setting `comp_size == decomp_size`) also works but produces larger files and blocky textures.

**Current workaround:** `compress_or_raw()` in `parser/src/xpress.rs` always returns raw (uncompressed) data. Files render correctly but are ~3-4x larger.

## Working Paths

There are 5 parallel paths to fix this. Each path has its own section below with:
- Goal and approach
- Prerequisites
- Steps to follow
- Progress tracking
- How to verify success

**Agents should pick ONE path and document progress in its section.**

---

## Path 1: Ghidra Reverse-Engineering

### Goal
Disassemble `HomeworldRM.exe` to understand the exact decompression algorithm used by the game engine's `ArchiveCompressStream` class.

### Prerequisites
- Install Ghidra (free): https://ghidra-sre.org/
- File: `/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HomeworldRM/Bin/Release/HomeworldRM.exe`
- Source path hint: `c:\users\bamboo\teamcity\agent01\work\20a11588452296ae\homeworld2\src\source\fileio\archivestream.cpp`

### Steps
1. Open `HomeworldRM.exe` in Ghidra, let it auto-analyze
2. Search for the `ArchiveCompressStream` class (look for the string "ArchiveCompressStream")
3. Find the decompression method — likely called `Read`, `Decompress`, or similar
4. Trace the indicator bit reading logic:
   - Does it read 31 or 32 bits per indicator word?
   - Does it read bits LSB-first or MSB-first?
5. Trace the match type decoding:
   - What order does it check match types?
   - How does it encode length and offset for each type?
6. Document the exact algorithm in this section
7. Implement a matching compressor in `parser/src/xpress.rs`

### What to Document
- Exact indicator bit count (31 or 32)
- Bit reading order (LSB-first or MSB-first)
- Match type priority order
- Length/offset encoding for each match type
- Any differences from our current implementation

### Progress

| Date | Agent | Finding |
|------|-------|---------|
| | | |

### Verification
- Compress `ter_centaur` mesh pool with the new compressor
- Compare compressed bytes with HODOR's — should be identical or very close
- Load generated HOD in-game — should render without spikiness

---

## Path 2: Windows RtlCompressBuffer API

### Goal
Use the Windows NT `RtlCompressBuffer` API from `ntdll.dll` to compress POOL data. If the game engine uses the same API, the bytes will match.

### Prerequisites
- Windows or Wine environment (Linux can use Wine)
- Rust FFI knowledge
- The API: `RtlCompressBuffer(COMPRESSION_FORMAT_XPRESS, ...)`

### Steps
1. Research the `RtlCompressBuffer` API signature and parameters
2. Write a Rust FFI wrapper that calls `ntdll.dll!RtlCompressBuffer`
3. Test compressing the `ter_centaur` decompressed mesh pool
4. Compare output with HODOR's compressed bytes
5. If bytes match, integrate into `compress_or_raw()`
6. If bytes differ, try `COMPRESSION_FORMAT_XPRESS_HUFF` variant

### API Reference
```c
NTSTATUS RtlCompressBuffer(
    USHORT CompressionFormat,    // COMPRESSION_FORMAT_XPRESS = 3
    PUCHAR UncompressedBuffer,
    ULONG UncompressedBufferLength,
    PUCHAR CompressedBuffer,
    ULONG CompressedBufferLength,
    ULONG UncompressedChunkSize,
    PULONG FinalCompressedSize,
    PVOID WorkSpace
);
```

### What to Document
- API signature and parameters used
- Whether output bytes match HODOR's
- Any differences found
- Integration approach

### Progress

| Date | Agent | Finding |
|------|-------|---------|
| | | |

### Verification
- Compress `ter_centaur` mesh pool with `RtlCompressBuffer`
- Compare bytes with HODOR's — should be identical
- Load generated HOD in-game — should render correctly

---

## Path 3: Match HODOR's Compressor Byte-for-Byte

### Goal
Fix our compressor in `parser/src/xpress.rs` to produce identical compressed bytes as HODOR for the same decompressed data.

### Prerequisites
- Understanding of our current compressor (`parser/src/xpress.rs`)
- `xpress_compare` diagnostic tool (`cargo run --bin xpress_compare`)
- `pool_byte_diff` diagnostic tool (`cargo run --bin pool_byte_diff`)

### Steps
1. Run `cargo run --bin xpress_compare` on `ter_centaur_hodor.hod` to see current byte differences
2. Analyze each difference:
   - Is it an indicator word difference? → Fix indicator bit layout
   - Is it a match type difference? → Fix match type selection priority
   - Is it a match offset/length difference? → Fix match finder strategy
3. Fix one difference at a time, re-running the comparison after each fix
4. When bytes match for texture pool, repeat for mesh and face pools
5. Test in-game after each significant fix

### Current Differences (as of 2026-05-29)
- First byte difference at offset 45 in texture pool
- HODOR uses Type 2 (2-byte, offset<16384, len=3) where we use Type 0 (1-byte, offset<64, len=3)
- Match selection strategy differs — HODOR picks different offsets when multiple matches exist

### Known Fixes Already Applied
- 32-bit indicator words (was 31-bit)
- Type 4 match handling (was missing)

### What to Document
- Each byte difference found and its root cause
- Each fix applied and its effect on the byte comparison
- Remaining differences after each fix

### Progress

| Date | Agent | Finding | Fix Applied |
|------|-------|---------|-------------|
| | | | |

### Verification
- `xpress_compare` shows zero byte differences for all three pools
- Generated HOD renders correctly in-game

---

## Path 4: Use a Known-Good MS Xpress Library

### Goal
Replace our custom compressor with an existing, correct MS Xpress implementation.

### Prerequisites
- Research existing implementations:
  - ReactOS: `modules/rostests/winetests/ntdll/test_compress.c`
  - Wine: `dlls/ntdll/rtl.c`
  - Windows SDK samples
  - Python `ms-compress` library
  - C# `ManagedLzma`

### Steps
1. Research existing MS Xpress LZ77 compressor implementations
2. Port the best one to Rust (or call via FFI)
3. Test compressing `ter_centaur` decompressed mesh pool
4. Compare output with HODOR's compressed bytes
5. If bytes match, integrate into `compress_or_raw()`
6. If bytes differ, investigate differences and adjust

### Known Implementations to Check
- `ms-compress` (Python): https://github.com/AresS31/ms-compress
- ReactOS `RtlCompressBuffer` implementation
- Wine `RtlCompressBuffer` implementation
- `lzss` crate (Rust) — may have Xpress variant

### What to Document
- Implementation source and license
- Whether output bytes match HODOR's
- Any differences found
- Integration approach

### Progress

| Date | Agent | Finding |
|------|-------|---------|
| | | |

### Verification
- Compress `ter_centaur` mesh pool with the library
- Compare bytes with HODOR's — should be identical or very close
- Load generated HOD in-game — should render correctly

---

## Path 5: Ship Uncompressed (Current Workaround)

### Goal
Keep the current workaround — ship with uncompressed POOL streams. This is the fallback if other paths fail.

### Current State
- `compress_or_raw()` in `parser/src/xpress.rs` always returns raw data
- Files render correctly but are ~3-4x larger
- Textures look blocky (DXT encoder quality issue, separate from Xpress)

### Known Issues
- File size: `ter_centaur` is ~1.2MB uncompressed vs ~200KB with HODOR's compression
- Textures blocky: our DXT encoder produces lower quality than HODOR's
- Face pool missing ~27KB of data (HODOR has 65,286 bytes vs our 37,704 bytes)

### What to Document
- File size comparisons
- Any rendering issues found
- DXT encoder quality findings

### Progress

| Date | Agent | Finding |
|------|-------|---------|
| | | |

---

## Diagnostic Tools

| Tool | Command | Purpose |
|------|---------|---------|
| `xpress_compare` | `cargo run --bin xpress_compare -- <hod>` | Compare compressed bytes with HODOR's |
| `pool_byte_diff` | `cargo run --bin pool_byte_diff` | Compare decompressed pool bytes |
| `pool_swap_test` | `cargo run --bin pool_swap_test -- <hodor> <gen> <outdir>` | Create hybrid HODs for testing |
| `xpress_msb_test` | `cargo run --bin xpress_msb_test -- <hod>` | Test MSB-first indicator reading |
| `xpress_decomp_test` | `cargo run --bin xpress_decomp_test -- <hod>` | Test 31-bit vs 32-bit indicators |

## Key Files

| File | Purpose |
|------|---------|
| `parser/src/xpress.rs` | Xpress compressor/decompressor |
| `parser/src/hod.rs` | HOD parser/generator (calls `compress_or_raw`) |
| `parser/src/compiler.rs` | Mesh pool generation |
| `testing/ter_centaur/ter_centaur_hodor.hod` | HODOR reference file |
| `testing/ter_centaur/ter_centaur_generated.hod` | Our generated file |

## In-Game Testing

To test a generated HOD in-game:
1. Copy the `.hod` file to the game's mod directory
2. Launch Homeworld Remastered
3. Load the model and check for:
   - Spikiness / vertex explosion (mesh pool issue)
   - Rainbow textures (face pool issue)
   - Blocky textures (DXT encoder quality issue)
   - Correct rendering (success)

---

**Document Version:** 1.0
**Created:** 2026-05-29
**Purpose:** Agent entry point for Xpress compression fix
