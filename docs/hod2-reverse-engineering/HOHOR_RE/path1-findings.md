# Path 1 Findings: Ghidra Reverse-Engineering of HODOR's Xpress Decompressor

By reverse-engineering `HODOR.exe`'s decompression routine (`FUN_00448600`), we have uncovered the exact parameters of the custom Xpress implementation used by the Homeworld Remastered game engine.

## Indicator Bit Stream
- **Exact Indicator Bit Count**: 31 bits per 32-bit word.
- **Bit Reading Order**: LSB-first (Least Significant Bit first).
- **Sentinel Bit**: The 32nd bit (MSB) of every 32-bit indicator word is always set to `1`. The decompressor shifts the word right by 1 for each token. When the word becomes exactly `1`, it knows it has exhausted the 31 bits and reads the next 32-bit word.
- **Bit Meaning**: 
  - `1` = MATCH
  - `0` = LITERAL

## Literal Processing Optimization
When a `0` (LITERAL) is encountered, the decompressor uses a lookup table (`DAT_00479764`) on the lowest 4 bits of the indicator word to count up to 4 consecutive `0`s. It copies 4 bytes at once, and advances the source/destination pointers and shifts the indicator word by the number of consecutive zeros (1, 2, 3, or 4).

## Match Types and Encoding
When a `1` (MATCH) is encountered, the decompressor reads the token. The lowest 3 bits of the token determine the match type, which points to a configuration table (`DAT_00479778`).
However, because bits are shifted, the match types effectively cluster into 4 main classes determined by the lowest 2 bits (since types 0,1,2 are identical to 4,5,6).

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

*(Note: In the layouts above, `O` = Offset bit, `L` = Length bit, `T` = Type bit. The types overlap exactly with the findings described in `xpress-compression-fix-paths.md`.)*
