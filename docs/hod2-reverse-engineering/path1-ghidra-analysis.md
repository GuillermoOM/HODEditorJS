# Path 1: Ghidra Reverse-Engineering

## Purpose

Disassemble `HomeworldRM.exe` to understand the exact decompression algorithm used by the game engine's `ArchiveCompressStream` class.

## Status

**Started:** 2026-05-29  
**Agent:** OpenCode Agent  
**Phase:** Setup

## Prerequisites

1. **Ghidra** — free reverse engineering tool from NSA
2. **Java JDK 17+** — required by Ghidra
3. **HomeworldRM.exe** — the game executable

## Installation

### Option A: Download Ghidra directly
```bash
# Download from GitHub
cd /tmp
wget https://github.com/NationalSecurityAgency/ghidra/releases/download/Ghidra_11.3.2_build/ghidra_11.3.2_PUBLIC_20250415.zip
unzip ghidra_11.3.2_PUBLIC_20250415.zip
# Run Ghidra
./ghidra_11.3.2_PUBLIC/ghidraRun
```

### Option B: Install via Flatpak (if available)
```bash
flatpak install flathub org.ghidra_sre.Ghidra
```

### Option C: Use distrobox
```bash
distrobox-enter esp-dev
# Then install Java and Ghidra manually
```

## Step-by-Step Guide

### Step 1: Create a Ghidra Project

1. Launch Ghidra
2. File → New Project → Non-Shared Project
3. Name: `HomeworldRM`
4. Location: `~/ghidra_projects`

### Step 2: Import the Executable

1. File → Import File
2. Select: `/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HomeworldRM/Bin/Release/HomeworldRM.exe`
3. Format: PE (Portable Executable) — should auto-detect
4. Click OK

### Step 3: Auto-Analysis

1. Double-click `HomeworldRM.exe` in the project to open the CodeBrowser
2. When prompted, click **Yes** to auto-analyze
3. This will take several minutes — let it complete

### Step 4: Find ArchiveCompressStream

1. **Search → For Strings**
2. Search for: `ArchiveCompressStream`
3. This will find the class name in the binary's RTTI or string data
4. Note the address

### Step 5: Find the Decompression Method

1. **Window → Defined Strings** — search for strings related to compression
2. Look for: `Compress Stream`, `ArchiveCompressStream`, `compression method`
3. Find the function that references these strings
4. This is likely the decompression entry point

### Step 6: Analyze the Decompression Code

The decompression code should have this structure:

```
// Pseudocode for MS Xpress LZ77 decompression
while (output_pos < output_size) {
    // Read indicator word every 32 bits
    if (indicator_bit == 32) {
        indicator = read_u32_le();
        indicator_bit = 0;
    }
    
    bit = (indicator >> indicator_bit) & 1;
    indicator_bit++;
    
    if (bit == 0) {
        // Literal byte
        output[output_pos++] = input[input_pos++];
    } else {
        // Match copy
        byte1 = input[input_pos++];
        
        if ((byte1 & 0x03) == 0) {
            // Type 0: 1-byte, offset < 64, length = 3
            length = 3;
            offset = byte1 >> 2;
        } else if ((byte1 & 0x03) == 0x02) {
            // Type 1: 2-byte, offset < 1024, length 3-18
            byte2 = input[input_pos++];
            length = ((byte1 >> 2) & 0x0F) + 3;
            offset = (byte2 << 2) | (byte1 >> 6);
        } else if ((byte1 & 0x03) == 0x01) {
            // Type 2: 2-byte, offset < 16384, length = 3
            byte2 = input[input_pos++];
            length = 3;
            offset = (byte2 << 6) | (byte1 >> 2);
        } else if ((byte1 & 0x07) == 0x07) {
            // Type 3: 4-byte, offset up to 2MB
            byte2 = input[input_pos++];
            byte3 = input[input_pos++];
            byte4 = input[input_pos++];
            length = ((byte2 & 0x07) << 5 | (byte1 >> 3)) + 3;
            offset = (byte4 << 13) | (byte3 << 5) | (byte2 >> 3);
        } else if ((byte1 & 0x03) == 0x03) {
            // Type 4: 3-byte, offset up to 65535
            byte2 = input[input_pos++];
            byte3 = input[input_pos++];
            length = (byte1 >> 3) + 3;
            offset = (byte3 << 8) | byte2;
        }
        
        // Copy from sliding window
        for (i = 0; i < length; i++) {
            output[output_pos] = output[output_pos - offset];
            output_pos++;
        }
    }
}
```

### Step 7: Look for Differences

Compare the disassembled code with our implementation in `parser/src/xpress.rs`:

1. **Indicator bit count** — does it use 31 or 32 bits?
2. **Bit reading order** — does it read LSB-first or MSB-first?
3. **Match type priority** — what order does it check types?
4. **Offset/length encoding** — does it use the same bit layouts?
5. **End-of-stream handling** — how does it handle partial indicator words?

### Step 8: Document Findings

Update this document with the exact algorithm found in the disassembly.

## Key Addresses to Find

| Item | Address | Notes |
|------|---------|-------|
| `ArchiveCompressStream` class | | RTTI or string reference |
| Decompression function | | Called from `Read` method |
| Indicator word reading | | Where `read_u32_le` happens |
| Match type dispatch | | Switch/if-else on byte1 |

## What to Look For

The game engine might differ from our implementation in:

1. **Indicator word size** — 31 vs 32 bits per word
2. **Bit reading order** — LSB-first vs MSB-first
3. **Match type checking order** — the order matters because some types overlap
4. **Offset encoding** — different bit layouts for the same type
5. **Length encoding** — different formulas for calculating length
6. **Sliding window size** — might be different from our implementation

## Progress Log

| Date | Step | Result | Notes |
|------|------|--------|-------|
| 2026-05-29 | Install Java | ✅ Complete | Java 21 OpenJDK in distrobox esp-dev |
| 2026-05-29 | Download Ghidra | ✅ Complete | Ghidra 11.3.2 extracted to /tmp/ghidra_11.3.2_PUBLIC/ |
| 2026-05-29 | Launch Ghidra | ✅ Running | GUI should be visible |
| 2026-05-29 | Import HomeworldRM.exe | Pending | User action required |
| 2026-05-29 | Auto-analyze | Pending | User action required |
| 2026-05-29 | Find decompression code | Pending | User action required |

## How to Launch Ghidra

```bash
# From distrobox (Java is installed there)
distrobox-enter esp-dev
JAVA_HOME=/usr/lib/jvm/java-21-openjdk /tmp/ghidra_11.3.2_PUBLIC/ghidraRun
```

Or use the GUI launcher if Ghidra is already running.

## Next Steps

1. Ghidra should be open — create a new project
2. Import `HomeworldRM.exe` from:
   `/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HomeworldRM/Bin/Release/HomeworldRM.exe`
3. Let auto-analysis complete (takes several minutes)
4. Search for `ArchiveCompressStream` string
5. Find the decompression function
6. Document the algorithm in this file
