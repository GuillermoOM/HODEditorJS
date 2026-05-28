use std::fs;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;

fn decompress_31(input: &[u8], output_size: usize) -> Result<Vec<u8>, String> {
    let mut output = vec![0u8; output_size];
    let mut output_idx = 0; let mut input_idx = 0;
    let mut indicator = 0u32; let mut indicator_bit = 30;
    let input_len = input.len();
    while output_idx < output_size && input_idx < input_len {
        indicator_bit += 1;
        if indicator_bit == 31 {
            if input_idx + 3 >= input_len { break; }
            indicator = u32::from_le_bytes([input[input_idx], input[input_idx+1], input[input_idx+2], input[input_idx+3]]);
            input_idx += 4; indicator_bit = 0;
        }
        if ((indicator >> indicator_bit) & 1) == 0 {
            if output_idx >= output_size { break; }
            output[output_idx] = input[input_idx];
            input_idx += 1; output_idx += 1;
        } else {
            if input_idx + 1 >= input_len { break; }
            let byte1 = input[input_idx];
            let mut length: usize; let offset: usize;
            if (byte1 & 0b11) == 0 { length = 3; offset = (byte1 >> 2) as usize; input_idx += 1;
            } else if (byte1 & 0b11) == 0b10 { length = (((byte1 >> 2) & 0b1111) + 3) as usize; let byte2 = input[input_idx + 1] as usize; offset = (byte2 << 2) | ((byte1 >> 6) as usize); input_idx += 2;
            } else if (byte1 & 0b11) == 0b01 { length = 3; let byte2 = input[input_idx + 1] as usize; offset = (byte2 << 6) | ((byte1 >> 2) as usize); input_idx += 2;
            } else if (byte1 & 0b111) == 0b111 { let byte2 = input[input_idx + 1] as usize; let byte3 = input[input_idx + 2] as usize; let byte4 = input[input_idx + 3] as usize; length = (((byte2 & 0b111) << 5) | ((byte1 >> 3) as usize)) + 3; offset = (byte4 << 13) | (byte3 << 5) | (byte2 >> 3); input_idx += 4;
            } else if (byte1 & 0b11) == 0b11 { length = ((byte1 >> 3) + 3) as usize; let byte2 = input[input_idx + 1] as usize; let byte3 = input[input_idx + 2] as usize; offset = (byte3 << 8) | byte2; input_idx += 3;
            } else { return Err("Invalid".to_string()); }
            while length > 0 {
                if output_idx >= output_size { break; }
                let copy_src = output_idx.checked_sub(offset);
                if let Some(src) = copy_src { output[output_idx] = output[src]; } else { output[output_idx] = 0; }
                output_idx += 1; length -= 1;
            }
        }
    }
    if output_idx < output_size { output.truncate(output_idx); }
    Ok(output)
}

fn decompress_32(input: &[u8], output_size: usize) -> Result<Vec<u8>, String> {
    let mut output = vec![0u8; output_size];
    let mut output_idx = 0; let mut input_idx = 0;
    let mut indicator = 0u32; let mut indicator_bit = 31;
    let input_len = input.len();
    while output_idx < output_size && input_idx < input_len {
        indicator_bit += 1;
        if indicator_bit == 32 {
            if input_idx + 3 >= input_len { break; }
            indicator = u32::from_le_bytes([input[input_idx], input[input_idx+1], input[input_idx+2], input[input_idx+3]]);
            input_idx += 4; indicator_bit = 0;
        }
        if ((indicator >> indicator_bit) & 1) == 0 {
            if output_idx >= output_size { break; }
            output[output_idx] = input[input_idx];
            input_idx += 1; output_idx += 1;
        } else {
            if input_idx + 1 >= input_len { break; }
            let byte1 = input[input_idx];
            let mut length: usize; let offset: usize;
            if (byte1 & 0b11) == 0 { length = 3; offset = (byte1 >> 2) as usize; input_idx += 1;
            } else if (byte1 & 0b11) == 0b10 { length = (((byte1 >> 2) & 0b1111) + 3) as usize; let byte2 = input[input_idx + 1] as usize; offset = (byte2 << 2) | ((byte1 >> 6) as usize); input_idx += 2;
            } else if (byte1 & 0b11) == 0b01 { length = 3; let byte2 = input[input_idx + 1] as usize; offset = (byte2 << 6) | ((byte1 >> 2) as usize); input_idx += 2;
            } else if (byte1 & 0b111) == 0b111 { let byte2 = input[input_idx + 1] as usize; let byte3 = input[input_idx + 2] as usize; let byte4 = input[input_idx + 3] as usize; length = (((byte2 & 0b111) << 5) | ((byte1 >> 3) as usize)) + 3; offset = (byte4 << 13) | (byte3 << 5) | (byte2 >> 3); input_idx += 4;
            } else if (byte1 & 0b11) == 0b11 { length = ((byte1 >> 3) + 3) as usize; let byte2 = input[input_idx + 1] as usize; let byte3 = input[input_idx + 2] as usize; offset = (byte3 << 8) | byte2; input_idx += 3;
            } else { return Err("Invalid".to_string()); }
            while length > 0 {
                if output_idx >= output_size { break; }
                let copy_src = output_idx.checked_sub(offset);
                if let Some(src) = copy_src { output[output_idx] = output[src]; } else { output[output_idx] = 0; }
                output_idx += 1; length -= 1;
            }
        }
    }
    if output_idx < output_size { output.truncate(output_idx); }
    Ok(output)
}

fn main() {
    let orig_path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/pebble/pebble_0/pebble_0_original.hod";
    let bytes = fs::read(orig_path).unwrap();
    let mut pool_start = 0;
    for i in 0..bytes.len()-4 {
        if &bytes[i..i+4] == b"POOL" {
            pool_start = i;
            break;
        }
    }
    let mut cursor = Cursor::new(&bytes[pool_start+8..]);
    let _pool_type = cursor.read_u32::<LittleEndian>().unwrap();
    let comp_tex_len = cursor.read_u32::<LittleEndian>().unwrap();
    let decomp_tex_len = cursor.read_u32::<LittleEndian>().unwrap();
    let mut comp_tex = vec![0u8; comp_tex_len as usize];
    std::io::Read::read_exact(&mut cursor, &mut comp_tex).unwrap();
    
    let out31 = decompress_31(&comp_tex, decomp_tex_len as usize).unwrap();
    let out32 = decompress_32(&comp_tex, decomp_tex_len as usize).unwrap();
    
    // Find first differing byte
    let mut diff_idx = 0;
    while diff_idx < out31.len() && diff_idx < out32.len() {
        if out31[diff_idx] != out32[diff_idx] { break; }
        diff_idx += 1;
    }
    println!("First differing byte at index: {}", diff_idx);
    
    // Print around diff
    if diff_idx > 10 {
        println!("out31: {:?}", &out31[diff_idx-10..diff_idx+10]);
        println!("out32: {:?}", &out32[diff_idx-10..diff_idx+10]);
    }
}
