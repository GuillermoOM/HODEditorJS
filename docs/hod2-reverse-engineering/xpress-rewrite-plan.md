# MS Xpress Rewrite Plan

## Problem

Our MS Xpress compressor/decompressor produces bytes incompatible with the game engine. HODOR's compressed bytes work; ours don't. Both decompress to the same data using our decompressor, but the game engine uses a different decompressor (HODOR's).

## Root Cause

HODOR's decompressor (`FUN_00448600`, 316 bytes at VA `0x448600` in `HODOR.exe`) uses a fundamentally different bit-processing scheme than ours:

1. **8 match types** (3-bit selector via `word & 7`), not 5
2. **Batch literal processing** (1-4 bytes at once via lookup table)
3. **Different indicator bit consumption** (starts at `uVar11=1`, shifts right)

## HODOR's Decompressor Lookup Tables

### Match Decode Table (`DAT_00479778`, 8 entries × 8 bytes)

| Type | Mask       | ShiftLo | Mask2 | ShiftHi | Consumed |
|------|------------|---------|-------|---------|----------|
| 0    | 0x000000FF | 2       | 0x00  | 0       | 1 byte   |
| 1    | 0x0000FFFF | 2       | 0x00  | 0       | 2 bytes  |
| 2    | 0x0000FFFF | 6       | 0x0F  | 2       | 2 bytes  |
| 3    | 0x00FFFFFF | 8       | 0x1F  | 3       | 3 bytes  |
| 4    | 0x000000FF | 2       | 0x00  | 0       | 1 byte   |
| 5    | 0x0000FFFF | 2       | 0x00  | 0       | 2 bytes  |
| 6    | 0x0000FFFF | 6       | 0x0F  | 2       | 2 bytes  |
| 7    | 0xFFFFFFFF | 11      | 0xFF  | 3       | 4 bytes  |

**Decode formula:**
- `offset = (word & mask) >> shift_lo`
- `length = ((word >> shift_hi) & mask2) + 3`

### Literal Count Table (`DAT_00479764`, 16 entries)

| Index | Count |
|-------|-------|
| 0     | 4     |
| 1     | 0*    |
| 2     | 1     |
| 3     | 0*    |
| 4     | 2     |
| 5     | 0*    |
| 6     | 1     |
| 7     | 0*    |
| 8     | 3     |
| 9     | 0*    |
| 10    | 1     |
| 11    | 0*    |
| 12    | 2     |
| 13    | 0*    |
| 14    | 1     |
| 15    | 0*    |

*`0` means "treat as 1" (single literal byte)

**Usage:** `count = literal_table[indicator & 0xF]`; if count==0, count=1. Copy `count` bytes from input to output, shift indicator right by `count`.

## HODOR's Decompressor Algorithm (Ghidra pseudocode)

```c
puVar5 = output + output_size;
uVar11 = 1;  // Will trigger read on first iteration

while (output < puVar5) {
    if (uVar11 == 1) {
        uVar11 = *input++;  // Read new 32-bit indicator word
    }
    
    if ((uVar11 & 1) == 0) {
        // Literal(s)
        count = literal_table[uVar11 & 0xF];
        if (count == 0) count = 1;
        memcpy(output, input, count);
        output += count;
        input += count;
        uVar11 >>= count;
    } else {
        // Match
        word = *input;
        type = word & 7;
        offset = (word & table[type].mask) >> table[type].shift_lo;
        length = ((word >> table[type].shift_hi) & table[type].mask2) + 3;
        input += table[type].consumed;
        
        // Copy from sliding window
        src = output - offset;
        if (offset < 4) {
            // Byte-by-byte copy first 3 bytes
            for (i = 0; i < 3; i++) output[i] = src[i];
            src -= 2 + (offset & 1);
            // Then 4-byte copy for remaining
            for (i = 3; i < length; i++) output[i] = src[i];
        } else {
            // Direct 4-byte copy
            for (i = 0; i < length; i++) output[i] = src[i];
        }
        output += length;
        uVar11 >>= 1;
    }
}
```

## Implementation Steps

### Step 1: Rewrite Decompressor (`parser/src/xpress.rs`)

Replace current `decompress()` function with HODOR's algorithm:
- Add `MATCH_TABLE` constant (8 entries)
- Add `LITERAL_TABLE` constant (16 entries)
- Implement batch literal processing
- Implement 8 match types with correct decode formula
- Handle `offset < 4` special case

### Step 2: Rewrite Compressor (`parser/src/xpress.rs`)

Replace current `compress()` function to produce HODOR-compatible bytes:
- Use 3-bit match type selector (types 0-7)
- Encode matches using HODOR's table format
- Process literals in batches (1-4 at once)
- Set indicator bits correctly for batch literals

### Step 3: Restore Compression in `compress_or_raw()`

Remove the bypass workaround once compressor produces compatible bytes.

### Step 4: Test Round-Trip

```bash
cargo test --lib xpress::tests
```

### Step 5: Test In-Game

Run `test_hodor_replication` and verify HOD files render correctly in-game.

## Files to Modify

- `parser/src/xpress.rs` — Main file to rewrite
- `parser/src/hod.rs` — May need updates if pool format changes
- `parser/src/compiler.rs` — May need updates

## Key References

- Ghidra project: `/tmp/ghidra_project/HODOR`
- Full decompilation: `/tmp/hodor_decomp_full.txt`
- Decompressor: `FUN_00448600` at VA `0x448600` (316 bytes)
- Match table: `DAT_00479778` at VA `0x479778`
- Literal table: `DAT_00479764` at VA `0x479764`
- Test data: `testing/ter_centaur/rtl_test/`
