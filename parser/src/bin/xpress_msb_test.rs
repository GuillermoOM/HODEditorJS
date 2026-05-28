/// Test: decompress HODOR data using MSB-first indicator bit reading
/// If this matches the expected output, the game engine reads indicator bits MSB-first
use byteorder::{LittleEndian, ReadBytesExt};
use hwr_hod_parser::iff::IffChunk;
use std::io::{Cursor, Seek, SeekFrom};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: xpress_msb_test <hodor_hod_path>");
        std::process::exit(1);
    }

    let bytes = std::fs::read(&args[1])?;
    let mut cursor = Cursor::new(&bytes);

    while cursor.position() < bytes.len() as u64 {
        let chunk = IffChunk::read_chunk(&mut cursor)?;
        if chunk.id == "POOL" {
            let mut pc = Cursor::new(&chunk.data);
            let _pool_type = pc.read_u32::<LittleEndian>()?;
            let comp_mesh = pc.read_u32::<LittleEndian>()?;
            let decomp_mesh = pc.read_u32::<LittleEndian>()?;
            println!("Mesh pool: comp={}, decomp={}", comp_mesh, decomp_mesh);

            let mesh_start = pc.position() as usize;
            let mesh_end = mesh_start + comp_mesh as usize;
            let comp_mesh_data = &chunk.data[mesh_start..mesh_end];

            // Decompress with LSB-first (current)
            let decomp_lsb = decompress_lsb(comp_mesh_data, decomp_mesh as usize)?;

            // Decompress with MSB-first
            let decomp_msb = decompress_msb(comp_mesh_data, decomp_mesh as usize)?;

            // Compare
            let mut diff_count = 0;
            let mut first_diff = None;
            for i in 0..(decomp_mesh as usize).min(decomp_lsb.len()).min(decomp_msb.len()) {
                if decomp_lsb[i] != decomp_msb[i] {
                    diff_count += 1;
                    if first_diff.is_none() {
                        first_diff = Some(i);
                    }
                }
            }

            println!("LSB-first decompressed: {} bytes", decomp_lsb.len());
            println!("MSB-first decompressed: {} bytes", decomp_msb.len());
            println!("Differences: {} bytes", diff_count);

            if let Some(pos) = first_diff {
                println!("First difference at byte: {}", pos);
                let start = pos.saturating_sub(4);
                let end = (pos + 8).min(decomp_lsb.len()).min(decomp_msb.len());
                println!("LSB: {:?}", &decomp_lsb[start..end]);
                println!("MSB: {:?}", &decomp_msb[start..end]);
            }

            // Now compress with our compressor and decompress with MSB-first
            let our_compressed = hwr_hod_parser::xpress::compress(&decomp_lsb);
            let our_msb = decompress_msb(&our_compressed, decomp_mesh as usize)?;

            let mut our_diff = 0;
            for i in 0..(decomp_mesh as usize).min(decomp_msb.len()).min(our_msb.len()) {
                if decomp_msb[i] != our_msb[i] {
                    our_diff += 1;
                }
            }
            println!("\nOur compressed decompressed MSB-first: {} bytes", our_msb.len());
            println!("Differences vs HODOR MSB-first: {} bytes", our_diff);

            return Ok(());
        }
    }
    Ok(())
}

/// LSB-first indicator bit reading (current behavior)
fn decompress_lsb(input: &[u8], output_size: usize) -> Result<Vec<u8>, String> {
    let mut output = vec![0u8; output_size];
    let mut output_idx = 0;
    let mut input_idx = 0;
    let mut indicator = 0u32;
    let mut indicator_bit = 31;

    while output_idx < output_size && input_idx < input.len() {
        indicator_bit += 1;
        if indicator_bit == 32 {
            if input_idx + 4 > input.len() { break; }
            indicator = u32::from_le_bytes([input[input_idx], input[input_idx+1], input[input_idx+2], input[input_idx+3]]);
            input_idx += 4;
            indicator_bit = 0;
        }
        let bit = (indicator >> indicator_bit) & 1;
        process_bit(bit, &mut output, &mut output_idx, &mut input_idx, input, output_size)?;
    }
    output.truncate(output_idx);
    Ok(output)
}

/// MSB-first indicator bit reading (hypothesis)
fn decompress_msb(input: &[u8], output_size: usize) -> Result<Vec<u8>, String> {
    let mut output = vec![0u8; output_size];
    let mut output_idx = 0;
    let mut input_idx = 0;
    let mut indicator = 0u32;
    let mut indicator_bit = 0;

    while output_idx < output_size && input_idx < input.len() {
        if indicator_bit == 0 {
            if input_idx + 4 > input.len() { break; }
            indicator = u32::from_le_bytes([input[input_idx], input[input_idx+1], input[input_idx+2], input[input_idx+3]]);
            input_idx += 4;
            indicator_bit = 32;
        }
        indicator_bit -= 1;
        let bit = (indicator >> indicator_bit) & 1;
        process_bit(bit, &mut output, &mut output_idx, &mut input_idx, input, output_size)?;
    }
    output.truncate(output_idx);
    Ok(output)
}

fn process_bit(bit: u32, output: &mut Vec<u8>, output_idx: &mut usize, input_idx: &mut usize, input: &[u8], output_size: usize) -> Result<(), String> {
    if bit == 0 {
        // Literal
        if *input_idx >= input.len() || *output_idx >= output_size { return Ok(()); }
        output[*output_idx] = input[*input_idx];
        *input_idx += 1;
        *output_idx += 1;
    } else {
        // Match
        if *input_idx + 1 >= input.len() { return Ok(()); }
        let byte1 = input[*input_idx];
        let (length, offset, consumed) = decode_match(byte1, input, *input_idx)?;
        *input_idx += consumed;

        for _ in 0..length {
            if *output_idx >= output_size { break; }
            let src = output_idx.checked_sub(offset);
            if let Some(s) = src {
                output[*output_idx] = output[s];
            }
            *output_idx += 1;
        }
    }
    Ok(())
}

fn decode_match(byte1: u8, input: &[u8], idx: usize) -> Result<(usize, usize, usize), String> {
    if (byte1 & 0b11) == 0 {
        Ok((3, (byte1 >> 2) as usize, 1))
    } else if (byte1 & 0b11) == 0b10 {
        let byte2 = input[idx + 1] as usize;
        let length = (((byte1 >> 2) & 0b1111) + 3) as usize;
        let offset = (byte2 << 2) | ((byte1 >> 6) as usize);
        Ok((length, offset, 2))
    } else if (byte1 & 0b11) == 0b01 {
        let byte2 = input[idx + 1] as usize;
        let offset = (byte2 << 6) | ((byte1 >> 2) as usize);
        Ok((3, offset, 2))
    } else if (byte1 & 0b111) == 0b111 {
        let byte2 = input[idx + 1] as usize;
        let byte3 = input[idx + 2] as usize;
        let byte4 = input[idx + 3] as usize;
        let length = (((byte2 & 0b111) << 5) | ((byte1 >> 3) as usize)) + 3;
        let offset = (byte4 << 13) | (byte3 << 5) | (byte2 >> 3);
        Ok((length, offset, 4))
    } else if (byte1 & 0b11) == 0b11 {
        let byte2 = input[idx + 1] as usize;
        let byte3 = input[idx + 2] as usize;
        let length = ((byte1 >> 3) + 3) as usize;
        let offset = (byte3 << 8) | byte2;
        Ok((length, offset, 3))
    } else {
        Err("Invalid match header".to_string())
    }
}
