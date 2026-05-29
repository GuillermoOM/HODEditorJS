/// Analyzes the first N byte differences between HODOR's and our compressed output.
/// For each difference, shows the indicator bits, match type, offset, and length.
///
/// Usage: xpress_diff_analysis <hodor_hod> [num_diffs]

use byteorder::{LittleEndian, ReadBytesExt};
use hwr_hod_parser::iff::IffChunk;
use std::io::{Cursor, Seek, SeekFrom};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: xpress_diff_analysis <hodor_hod> [num_diffs]");
        std::process::exit(1);
    }

    let num_diffs = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(5);

    let bytes = std::fs::read(&args[1])?;
    let mut cursor = Cursor::new(&bytes);

    while cursor.position() < bytes.len() as u64 {
        let chunk = IffChunk::read_chunk(&mut cursor)?;
        if chunk.id == "POOL" {
            let mut pc = Cursor::new(&chunk.data);
            let _pool_type = pc.read_u32::<LittleEndian>()?;
            let comp_mesh_len = pc.read_u32::<LittleEndian>()? as usize;
            let decomp_mesh_len = pc.read_u32::<LittleEndian>()? as usize;
            let mesh_start = pc.position() as usize;
            let hodor_comp = &chunk.data[mesh_start..mesh_start + comp_mesh_len];

            let decomp_mesh = if comp_mesh_len == decomp_mesh_len {
                hodor_comp.to_vec()
            } else {
                hwr_hod_parser::xpress::decompress(hodor_comp, decomp_mesh_len)?
            };

            let our_comp = hwr_hod_parser::xpress::compress(&decomp_mesh);

            println!("=== Mesh Pool Compression Analysis ===");
            println!("HODOR: {} bytes compressed, {} decompressed", comp_mesh_len, decomp_mesh_len);
            println!("Ours:  {} bytes compressed", our_comp.len());
            println!();

            // Find differences
            let min_len = comp_mesh_len.min(our_comp.len());
            let mut diff_count = 0;
            let mut i = 0;

            while i < min_len && diff_count < num_diffs {
                if hodor_comp[i] != our_comp[i] {
                    diff_count += 1;
                    println!("--- Difference #{} at offset 0x{:06X} ({}) ---", diff_count, i, i);

                    // Show context
                    let start = i.saturating_sub(16);
                    let end = (i + 32).min(min_len);
                    println!("HODOR bytes [0x{:06X}..0x{:06X}]:", start, end);
                    print_hex(&hodor_comp[start..end], start);
                    println!("Ours  bytes [0x{:06X}..0x{:06X}]:", start, end);
                    print_hex(&our_comp[start..end], start);

                    // Find the indicator word that covers this position
                    // Indicator words are at fixed positions in the stream
                    let (hodor_ctx, hodor_indicator_pos) = find_indicator_context(hodor_comp, i);
                    let (our_ctx, our_indicator_pos) = find_indicator_context(&our_comp, i);

                    println!();
                    println!("HODOR: indicator at 0x{:06X}, bit {}", hodor_indicator_pos, hodor_ctx.bit_in_word);
                    println!("  bit value: {} ({})", hodor_ctx.bit_value, if hodor_ctx.bit_value == 0 { "literal" } else { "match" });
                    if hodor_ctx.bit_value == 1 {
                        println!("  match type: {}", hodor_ctx.match_type);
                        println!("  match offset: {}", hodor_ctx.match_offset);
                        println!("  match length: {}", hodor_ctx.match_length);
                    }

                    println!();
                    println!("Ours:  indicator at 0x{:06X}, bit {}", our_indicator_pos, our_ctx.bit_in_word);
                    println!("  bit value: {} ({})", our_ctx.bit_value, if our_ctx.bit_value == 0 { "literal" } else { "match" });
                    if our_ctx.bit_value == 1 {
                        println!("  match type: {}", our_ctx.match_type);
                        println!("  match offset: {}", our_ctx.match_offset);
                        println!("  match length: {}", our_ctx.match_length);
                    }

                    // Show what the decompressed bytes would be
                    println!();
                    let hodor_decomp = hwr_hod_parser::xpress::decompress(hodor_comp, decomp_mesh_len).unwrap_or_default();
                    let our_decomp = hwr_hod_parser::xpress::decompress(&our_comp, decomp_mesh_len).unwrap_or_default();
                    
                    // Find where the decompressed output diverges
                    let mut decomp_diff = None;
                    for j in 0..decomp_mesh_len.min(hodor_decomp.len()).min(our_decomp.len()) {
                        if hodor_decomp[j] != our_decomp[j] {
                            decomp_diff = Some(j);
                            break;
                        }
                    }
                    match decomp_diff {
                        Some(pos) => {
                            let start = pos.saturating_sub(8);
                            let end = (pos + 16).min(decomp_mesh_len);
                            println!("Decompressed output diverges at byte {}:", pos);
                            println!("  HODOR: {:?}", &hodor_decomp[start..end]);
                            println!("  Ours:  {:?}", &our_decomp[start..end]);
                        }
                        None => {
                            println!("Decompressed output is IDENTICAL (same data, different encoding)");
                        }
                    }

                    println!();
                    i += 1;
                } else {
                    i += 1;
                }
            }

            if diff_count == 0 {
                println!("No differences found in first {} bytes!", min_len);
            } else if diff_count >= num_diffs {
                println!("... ({} differences found, showing first {})", diff_count, num_diffs);
            }

            return Ok(());
        }
    }

    eprintln!("No POOL chunk found");
    Ok(())
}

struct MatchContext {
    bit_in_word: usize,
    bit_value: u32,
    match_type: String,
    match_offset: usize,
    match_length: usize,
}

fn find_indicator_context(data: &[u8], target_offset: usize) -> (MatchContext, usize) {
    let mut stream_pos = 0; // position in the data stream (literals + match bytes)
    let mut data_idx = 0; // position in the compressed data

    // Read first indicator
    if data_idx + 4 > data.len() {
        return (MatchContext {
            bit_in_word: 0, bit_value: 0,
            match_type: "unknown".to_string(),
            match_offset: 0, match_length: 0,
        }, 0);
    }

    let mut indicator = u32::from_le_bytes([data[data_idx], data[data_idx+1], data[data_idx+2], data[data_idx+3]]);
    data_idx += 4;
    let mut bit_idx = 0;

    while data_idx < data.len() && stream_pos < target_offset + 10 {
        if bit_idx >= 32 {
            // Read next indicator
            if data_idx + 4 > data.len() { break; }
            indicator = u32::from_le_bytes([data[data_idx], data[data_idx+1], data[data_idx+2], data[data_idx+3]]);
            data_idx += 4;
            bit_idx = 0;
        }

        let bit = (indicator >> bit_idx) & 1;
        let indicator_pos = data_idx - 4;

        if bit == 0 {
            // Literal
            if stream_pos == target_offset {
                return (MatchContext {
                    bit_in_word: bit_idx,
                    bit_value: 0,
                    match_type: "literal".to_string(),
                    match_offset: 0,
                    match_length: 0,
                }, indicator_pos);
            }
            data_idx += 1;
            stream_pos += 1;
        } else {
            // Match
            if data_idx >= data.len() { break; }
            let byte1 = data[data_idx];
            let (length, offset, consumed) = decode_match(byte1, data, data_idx);

            if stream_pos <= target_offset && stream_pos + length > target_offset {
                let match_type = classify_match(byte1);
                return (MatchContext {
                    bit_in_word: bit_idx,
                    bit_value: 1,
                    match_type: match_type.to_string(),
                    match_offset: offset,
                    match_length: length,
                }, indicator_pos);
            }

            data_idx += consumed;
            stream_pos += length;
        }

        bit_idx += 1;
    }

    (MatchContext {
        bit_in_word: 0, bit_value: 0,
        match_type: "unknown".to_string(),
        match_offset: 0, match_length: 0,
    }, 0)
}

fn classify_match(byte1: u8) -> &'static str {
    if (byte1 & 0b11) == 0 {
        "Type 0 (1-byte, offset<64, len=3)"
    } else if (byte1 & 0b11) == 0b10 {
        "Type 1 (2-byte, offset<1024, len 3-18)"
    } else if (byte1 & 0b11) == 0b01 {
        "Type 2 (2-byte, offset<16384, len=3)"
    } else if (byte1 & 0b111) == 0b111 {
        "Type 3 (4-byte, offset up to 2MB)"
    } else if (byte1 & 0b11) == 0b11 {
        "Type 4 (3-byte, offset up to 65535)"
    } else {
        "UNKNOWN"
    }
}

fn decode_match(byte1: u8, input: &[u8], idx: usize) -> (usize, usize, usize) {
    if (byte1 & 0b11) == 0 {
        (3, (byte1 >> 2) as usize, 1)
    } else if (byte1 & 0b11) == 0b10 {
        let byte2 = input[idx + 1] as usize;
        let length = (((byte1 >> 2) & 0b1111) + 3) as usize;
        let offset = (byte2 << 2) | ((byte1 >> 6) as usize);
        (length, offset, 2)
    } else if (byte1 & 0b11) == 0b01 {
        let byte2 = input[idx + 1] as usize;
        let offset = (byte2 << 6) | ((byte1 >> 2) as usize);
        (3, offset, 2)
    } else if (byte1 & 0b111) == 0b111 {
        let byte2 = input[idx + 1] as usize;
        let byte3 = input[idx + 2] as usize;
        let byte4 = input[idx + 3] as usize;
        let length = (((byte2 & 0b111) << 5) | ((byte1 >> 3) as usize)) + 3;
        let offset = (byte4 << 13) | (byte3 << 5) | (byte2 >> 3);
        (length, offset, 4)
    } else if (byte1 & 0b11) == 0b11 {
        let byte2 = input[idx + 1] as usize;
        let byte3 = input[idx + 2] as usize;
        let length = ((byte1 >> 3) + 3) as usize;
        let offset = (byte3 << 8) | byte2;
        (length, offset, 3)
    } else {
        (0, 0, 0)
    }
}

fn print_hex(data: &[u8], base_offset: usize) {
    for (i, chunk) in data.chunks(16).enumerate() {
        print!("  {:06X}: ", base_offset + i * 16);
        for b in chunk {
            print!("{:02X} ", b);
        }
        // Print ASCII
        print!(" | ");
        for b in chunk {
            if *b >= 0x20 && *b < 0x7F {
                print!("{}", *b as char);
            } else {
                print!(".");
            }
        }
        println!();
    }
}
