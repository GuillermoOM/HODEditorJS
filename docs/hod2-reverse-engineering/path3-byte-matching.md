# Path 3: Byte-Match HODOR's Compressor

## Purpose

Fix our Xpress LZ77 compressor (`parser/src/xpress.rs`) to produce identical compressed bytes as HODOR for the same decompressed data.

## Status

**Started:** 2026-05-29  
**Agent:** OpenCode Agent  
**Phase:** Analysis Complete — Path 3 is extremely difficult

## Key Findings

1. **Decompressed outputs are IDENTICAL** — both compressors produce the same decompressed data. The difference is only in the compressed byte encoding.

2. **HODOR uses all 5 match types** — Type 0 (34), Type 1 (31), Type 2 (16), Type 3 (14), Type 4 (16) in first 10 indicators.

3. **Our compressor is 4x more efficient** — 16,771 bytes vs HODOR's 65,852 bytes for the same 87,424 bytes of texture data.

4. **The game engine expects HODOR's exact encoding** — not just valid Xpress data. This means we must replicate HODOR's hash table, search strategy, and match selection priority exactly.

5. **Matching HODOR's behavior requires reverse-engineering their compressor** — this is essentially the same as Path 1 (Ghidra), but harder because we're doing it from the outside.

## Recommendation

Path 3 is extremely difficult because:
- Our compressor finds different (better) matches than HODOR
- The match selection depends on hash table implementation, search depth, and match priority rules
- We'd need to replicate HODOR's exact algorithm, not just fix encoding

**Better alternatives:**
- Path 1 (Ghidra) — disassemble the game engine's decompressor to understand what it expects
- Path 4 (known-good library) — find an existing MS Xpress implementation that matches HODOR's behavior
- Path 5 (ship uncompressed) — keep the current workaround

## Analysis Details

### HODOR's Match Type Distribution (first 10 indicators)
| Type | Count | Description |
|------|-------|-------------|
| Type 0 | 34 | 1-byte, offset<64, len=3 |
| Type 1 | 31 | 2-byte, offset<1024, len 3-18 |
| Type 2 | 16 | 2-byte, offset<16384, len=3 |
| Type 3 | 14 | 4-byte, offset up to 2MB |
| Type 4 | 16 | 3-byte, offset up to 65535 |

### Compression Ratio Comparison
| Pool | Decompressed | HODOR Compressed | Our Compressed | HODOR Ratio | Our Ratio |
|------|-------------|-----------------|----------------|-------------|-----------|
| Texture | 87,424 | 65,852 | 16,771 | 75.3% | 19.2% |
| Mesh | 1,218,456 | 125,480 | 104,560 | 10.3% | 8.6% |
| Face | 65,286 | 28,135 | 11,484 | 43.1% | 17.6% |

## Approach

1. Run `xpress_compare` to identify byte differences
2. At each difference, decode both HODOR's match and our match
3. Understand why HODOR picks a different match (different offset? different length? different type?)
4. Adjust the match finder to match HODOR's behavior
5. Re-run comparison after each fix
6. When bytes match for all three pools, test in-game

## Key Files

| File | Purpose |
|------|---------|
| `parser/src/xpress.rs` | Our compressor/decompressor |
| `parser/src/bin/xpress_compare.rs` | Byte comparison tool |
| `testing/ter_centaur/ter_centaur_hodor.hod` | HODOR reference |
| `testing/ter_centaur/rtl_test/decomp_mesh.bin` | Decompressed mesh pool |
| `testing/ter_centaur/rtl_test/hodor_comp_mesh.bin` | HODOR's compressed mesh pool |

## Commands

```bash
# Compare our compressed output with HODOR's
cargo run --bin xpress_compare -- testing/ter_centaur/ter_centaur_hodor.hod

# Extract decompressed pools (if needed)
cargo run --bin extract_pool_streams -- testing/ter_centaur/ter_centaur_hodor.hod testing/ter_centaur/rtl_test
```

## Known Fixes Already Applied

| Fix | Effect |
|-----|--------|
| 32-bit indicator words | Moved first diff from byte 3 → byte 45 |
| Type 4 match handling | Further improved alignment |

## Current State

First byte difference at offset 45 in texture pool:
- HODOR: Type 2 (2-byte, offset<16384, len=3)
- Ours: Type 0 (1-byte, offset<64, len=3)

## Analysis Template

For each byte difference, document:

| Offset | HODOR Byte | Our Byte | HODOR Match | Our Match | Root Cause | Fix |
|--------|-----------|----------|-------------|-----------|------------|-----|
| | | | | | | |

## Progress Log

| Date | Step | Result | Notes |
|------|------|--------|-------|
| | | | |
