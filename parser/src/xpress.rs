use byteorder::{LittleEndian, WriteBytesExt};

/// Decompresses a buffer using MS XPress LZ77 algorithm
pub fn decompress(input: &[u8], output_size: usize) -> Result<Vec<u8>, String> {
    let mut output = vec![0u8; output_size];
    let mut output_idx = 0;
    let mut input_idx = 0;
    let mut indicator = 0u32;
    let mut indicator_bit = 31; // Starts at 31 so the first iteration triggers a read (31 + 1 = 32)

    let input_len = input.len();

    while output_idx < output_size && input_idx < input_len {
        indicator_bit += 1;
        if indicator_bit == 32 {
            if input_idx + 3 >= input_len {
                break;
            }
            // Read 32-bit little-endian indicator word
            indicator = u32::from_le_bytes([
                input[input_idx],
                input[input_idx + 1],
                input[input_idx + 2],
                input[input_idx + 3],
            ]);
            input_idx += 4;
            indicator_bit = 0;
        }

        if ((indicator >> indicator_bit) & 1) == 0 {
            // Literal byte
            if output_idx >= output_size {
                break;
            }
            output[output_idx] = input[input_idx];
            input_idx += 1;
            output_idx += 1;
        } else {
            // Match copy
            if input_idx + 1 >= input_len {
                break;
            }

            let byte1 = input[input_idx];
            let mut length: usize;
            let offset: usize;

            if (byte1 & 0b11) == 0 {
                length = 3;
                offset = (byte1 >> 2) as usize;
                input_idx += 1;
            } else if (byte1 & 0b11) == 0b10 {
                length = (((byte1 >> 2) & 0b1111) + 3) as usize;
                let byte2 = input[input_idx + 1] as usize;
                offset = (byte2 << 2) | ((byte1 >> 6) as usize);
                input_idx += 2;
            } else if (byte1 & 0b11) == 0b01 {
                length = 3;
                let byte2 = input[input_idx + 1] as usize;
                offset = (byte2 << 6) | ((byte1 >> 2) as usize);
                input_idx += 2;
            } else if (byte1 & 0b111) == 0b111 {
                let byte2 = input[input_idx + 1] as usize;
                let byte3 = input[input_idx + 2] as usize;
                let byte4 = input[input_idx + 3] as usize;
                length = (((byte2 & 0b111) << 5) | ((byte1 >> 3) as usize)) + 3;
                offset = (byte4 << 13) | (byte3 << 5) | (byte2 >> 3);
                input_idx += 4;
            } else if (byte1 & 0b11) == 0b11 {
                length = ((byte1 >> 3) + 3) as usize;
                let byte2 = input[input_idx + 1] as usize;
                let byte3 = input[input_idx + 2] as usize;
                offset = (byte3 << 8) | byte2;
                input_idx += 3;
            } else {
                return Err("Invalid XPress match header".to_string());
            }

            // Copy from sliding window
            while length > 0 {
                if output_idx >= output_size {
                    break;
                }
                let copy_src = output_idx.checked_sub(offset);
                if let Some(src) = copy_src {
                    output[output_idx] = output[src];
                } else {
                    output[output_idx] = 0; // Pad out of bounds
                }
                output_idx += 1;
                length -= 1;
            }
        }
    }

    if output_idx < output_size {
        output.truncate(output_idx);
    }
    Ok(output)
}

/// Compresses a buffer using MS XPress LZ77 algorithm
pub fn compress(input: &[u8]) -> Vec<u8> {
    let mut compressed = Vec::new();
    let mut input_idx = 0;
    let input_len = input.len();

    let mut indicator = 0u32;
    let mut indicator_bit = 0;
    let mut indicator_pos = 0;

    compressed.write_u32::<LittleEndian>(0).unwrap();

    let mut head = vec![-1i32; 65536];
    let mut prev = vec![-1i32; input_len];

    while input_idx < input_len {
        if indicator_bit == 32 {
            let bytes = indicator.to_le_bytes();
            compressed[indicator_pos..indicator_pos + 4].copy_from_slice(&bytes);

            indicator_pos = compressed.len();
            compressed.write_u32::<LittleEndian>(0).unwrap();
            indicator = 0;
            indicator_bit = 0;
        }

        let mut best_length = 0;
        let mut best_offset = 0;

        if input_idx + 3 <= input_len {
            let hash = (((input[input_idx] as usize) << 10)
                ^ ((input[input_idx + 1] as usize) << 5)
                ^ (input[input_idx + 2] as usize))
                & 0xFFFF;

            let mut start = head[hash];
            let mut chain_len = 0;

            while start != -1 && chain_len < 256 {
                let start_idx = start as usize;
                let offset = input_idx - start_idx;
                if offset > 2097151 {
                    break;
                }

                let mut len = 0;
                while input_idx + len < input_len
                    && input[start_idx + len] == input[input_idx + len]
                {
                    len += 1;
                    if len >= 258 {
                        break;
                    }
                }

                if len >= 3 && len > best_length {
                    best_length = len;
                    best_offset = offset;
                }

                start = prev[start_idx];
                chain_len += 1;
            }

            prev[input_idx] = head[hash];
            head[hash] = input_idx as i32;
        }

        if best_length >= 3 {
            indicator |= 1 << indicator_bit;
            indicator_bit += 1;

            if best_offset < 64 && best_length == 3 {
                let byte = (best_offset << 2) as u8;
                compressed.push(byte);
                for i in 1..best_length {
                    let idx = input_idx + i;
                    if idx + 3 <= input_len {
                        let hash = (((input[idx] as usize) << 10)
                            ^ ((input[idx + 1] as usize) << 5)
                            ^ (input[idx + 2] as usize))
                            & 0xFFFF;
                        prev[idx] = head[hash];
                        head[hash] = idx as i32;
                    }
                }
                input_idx += best_length;
            } else if best_offset < 1024 && best_length >= 3 && best_length <= 18 {
                let byte1 = (((best_length - 3) << 2) | ((best_offset & 3) << 6) | 0b10) as u8;
                let byte2 = (best_offset >> 2) as u8;
                compressed.push(byte1);
                compressed.push(byte2);
                for i in 1..best_length {
                    let idx = input_idx + i;
                    if idx + 3 <= input_len {
                        let hash = (((input[idx] as usize) << 10)
                            ^ ((input[idx + 1] as usize) << 5)
                            ^ (input[idx + 2] as usize))
                            & 0xFFFF;
                        prev[idx] = head[hash];
                        head[hash] = idx as i32;
                    }
                }
                input_idx += best_length;
            } else if best_offset < 16384 && best_length == 3 {
                let byte1 = (((best_offset & 0x3F) << 2) | 0b01) as u8;
                let byte2 = (best_offset >> 6) as u8;
                compressed.push(byte1);
                compressed.push(byte2);
                for i in 1..best_length {
                    let idx = input_idx + i;
                    if idx + 3 <= input_len {
                        let hash = (((input[idx] as usize) << 10)
                            ^ ((input[idx + 1] as usize) << 5)
                            ^ (input[idx + 2] as usize))
                            & 0xFFFF;
                        prev[idx] = head[hash];
                        head[hash] = idx as i32;
                    }
                }
                input_idx += best_length;
            } else if best_offset < 65536 && best_length <= 34 {
                // Type 4: 3-byte, offset up to 65535, length 3-34
                let byte1 = (((best_length - 3) << 3) | 0b11) as u8;
                let byte2 = (best_offset & 0xFF) as u8;
                let byte3 = ((best_offset >> 8) & 0xFF) as u8;
                compressed.push(byte1);
                compressed.push(byte2);
                compressed.push(byte3);
                for i in 1..best_length {
                    let idx = input_idx + i;
                    if idx + 3 <= input_len {
                        let hash = (((input[idx] as usize) << 10)
                            ^ ((input[idx + 1] as usize) << 5)
                            ^ (input[idx + 2] as usize))
                            & 0xFFFF;
                        prev[idx] = head[hash];
                        head[hash] = idx as i32;
                    }
                }
                input_idx += best_length;
            } else {
                let byte1 = ((((best_length - 3) & 0x1F) << 3) | 0b111) as u8;
                let byte2 = ((((best_length - 3) >> 5) & 0x07) | ((best_offset & 31) << 3)) as u8;
                let byte3 = ((best_offset >> 5) & 0xFF) as u8;
                let byte4 = ((best_offset >> 13) & 0xFF) as u8;
                compressed.push(byte1);
                compressed.push(byte2);
                compressed.push(byte3);
                compressed.push(byte4);
                for i in 1..best_length {
                    let idx = input_idx + i;
                    if idx + 3 <= input_len {
                        let hash = (((input[idx] as usize) << 10)
                            ^ ((input[idx + 1] as usize) << 5)
                            ^ (input[idx + 2] as usize))
                            & 0xFFFF;
                        prev[idx] = head[hash];
                        head[hash] = idx as i32;
                    }
                }
                input_idx += best_length;
            }
        } else {
            indicator_bit += 1;
            compressed.push(input[input_idx]);
            input_idx += 1;
        }
    }

    if indicator_bit == 0 {
        compressed.truncate(indicator_pos);
    } else {
        let bytes = indicator.to_le_bytes();
        compressed[indicator_pos..indicator_pos + 4].copy_from_slice(&bytes);
    }

    compressed
}

/// Compresses when useful, otherwise returns the input unchanged.
///
/// HOD POOL streams store both compressed and decompressed sizes. Existing
/// save code uses equal sizes to mean an uncompressed/raw stream, avoiding the
/// game-side decompressor path for incompressible data.
///
/// WORKAROUND: Currently returns raw (uncompressed) data always, because our
/// Xpress compressor produces byte patterns incompatible with the game engine's
/// decompressor. This makes files larger but renders correctly in-game.
/// TODO: Fix compressor to match HODOR's byte patterns, then restore compression.
pub fn compress_or_raw(input: &[u8]) -> Vec<u8> {
    // Bypass compression — game engine's decompressor can't handle our bytes
    input.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xpress_roundtrip_basic() {
        let original =
            b"Hello World! Hello World! Hello World! This is MS XPress LZ77 algorithm test!"
                .to_vec();
        let compressed = compress(&original);
        let decompressed = decompress(&compressed, original.len()).unwrap();
        assert_eq!(original.as_slice(), decompressed.as_slice());
    }
}
