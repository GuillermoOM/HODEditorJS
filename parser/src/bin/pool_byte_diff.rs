/// Byte-level comparison of decompressed POOL streams between HODOR and generated HODs.
/// Usage: cargo run --bin pool_byte_diff
///
/// This tool extracts the raw decompressed mesh, face, and texture pools from both HODs
/// and reports exactly where they diverge byte-by-byte.

use byteorder::{LittleEndian, ReadBytesExt};
use hwr_hod_parser::xpress;
use std::fs;
use std::io::{Cursor, Read};

fn extract_pools(path: &str) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>, u32), String> {
    let data = fs::read(path).map_err(|e| format!("Failed to read {}: {}", path, e))?;
    let mut cursor = Cursor::new(&data);

    // Read IFF chunks until we find POOL
    loop {
        if cursor.position() >= data.len() as u64 {
            return Err(format!("No POOL chunk found in {}", path));
        }

        let mut id_bytes = [0u8; 4];
        cursor.read_exact(&mut id_bytes).map_err(|e| e.to_string())?;
        let id = String::from_utf8_lossy(&id_bytes).to_string();

        let size = cursor.read_u32::<byteorder::BigEndian>().map_err(|e| e.to_string())?;

        if id == "POOL" {
            // Read pool header
            let pool_type = cursor.read_u32::<LittleEndian>().map_err(|e| e.to_string())?;

            // Texture pool
            let comp_tex_len = cursor.read_u32::<LittleEndian>().map_err(|e| e.to_string())? as usize;
            let decomp_tex_len = cursor.read_u32::<LittleEndian>().map_err(|e| e.to_string())? as usize;
            let mut comp_tex = vec![0u8; comp_tex_len];
            cursor.read_exact(&mut comp_tex).map_err(|e| e.to_string())?;
            let decomp_tex = if comp_tex_len == decomp_tex_len {
                comp_tex.clone()
            } else if comp_tex_len > 0 {
                xpress::decompress(&comp_tex, decomp_tex_len)?
            } else {
                Vec::new()
            };

            // Mesh pool
            let comp_mesh_len = cursor.read_u32::<LittleEndian>().map_err(|e| e.to_string())? as usize;
            let decomp_mesh_len = cursor.read_u32::<LittleEndian>().map_err(|e| e.to_string())? as usize;
            let mut comp_mesh = vec![0u8; comp_mesh_len];
            cursor.read_exact(&mut comp_mesh).map_err(|e| e.to_string())?;
            let decomp_mesh = if comp_mesh_len == decomp_mesh_len {
                comp_mesh.clone()
            } else if comp_mesh_len > 0 {
                xpress::decompress(&comp_mesh, decomp_mesh_len)?
            } else {
                Vec::new()
            };

            // Face pool
            let comp_face_len = cursor.read_u32::<LittleEndian>().map_err(|e| e.to_string())? as usize;
            let decomp_face_len = cursor.read_u32::<LittleEndian>().map_err(|e| e.to_string())? as usize;
            let mut comp_face = vec![0u8; comp_face_len];
            cursor.read_exact(&mut comp_face).map_err(|e| e.to_string())?;
            let decomp_face = if comp_face_len == decomp_face_len {
                comp_face.clone()
            } else if comp_face_len > 0 {
                xpress::decompress(&comp_face, decomp_face_len)?
            } else {
                Vec::new()
            };

            println!("  Pool type: {}", pool_type);
            println!("  Texture: compressed={} decompressed={}", comp_tex_len, decomp_tex_len);
            println!("  Mesh:    compressed={} decompressed={}", comp_mesh_len, decomp_mesh_len);
            println!("  Face:    compressed={} decompressed={}", comp_face_len, decomp_face_len);

            return Ok((decomp_tex, decomp_mesh, decomp_face, pool_type));
        } else {
            // Skip this chunk
            let mut skip = vec![0u8; size as usize];
            cursor.read_exact(&mut skip).map_err(|e| e.to_string())?;
        }
    }
}

fn compare_pools(name: &str, hodor: &[u8], generated: &[u8], stride: usize) {
    println!("\n=== {} Pool Comparison ===", name);
    println!("  HODOR size:     {} bytes", hodor.len());
    println!("  Generated size: {} bytes", generated.len());

    if hodor.len() != generated.len() {
        println!("  ⚠️  SIZE MISMATCH: {} vs {} (diff={})", hodor.len(), generated.len(),
                 generated.len() as isize - hodor.len() as isize);
    }

    let compare_len = std::cmp::min(hodor.len(), generated.len());
    let mut first_diff = None;
    let mut diff_count = 0;
    let mut diff_regions: Vec<(usize, usize)> = Vec::new();

    let mut in_diff = false;
    let mut diff_start = 0;

    for i in 0..compare_len {
        if hodor[i] != generated[i] {
            diff_count += 1;
            if first_diff.is_none() {
                first_diff = Some(i);
            }
            if !in_diff {
                in_diff = true;
                diff_start = i;
            }
        } else if in_diff {
            in_diff = false;
            diff_regions.push((diff_start, i));
        }
    }
    if in_diff {
        diff_regions.push((diff_start, compare_len));
    }

    if diff_count == 0 && hodor.len() == generated.len() {
        println!("  ✅ IDENTICAL ({} bytes)", hodor.len());
        return;
    }

    println!("  ❌ {} bytes differ out of {}", diff_count, compare_len);
    println!("  First difference at byte offset: {}", first_diff.unwrap_or(0));

    // Show first 5 diff regions
    let show_count = std::cmp::min(10, diff_regions.len());
    println!("\n  First {} diff regions (of {}):", show_count, diff_regions.len());
    for (idx, (start, end)) in diff_regions.iter().take(show_count).enumerate() {
        let len = end - start;
        let vertex_idx = if stride > 0 { start / stride } else { 0 };
        let byte_in_vertex = if stride > 0 { start % stride } else { 0 };
        println!("  Region {} @ offset 0x{:06X}-0x{:06X} ({} bytes) [vertex ~{}, byte {} in vertex]",
                 idx, start, end, len, vertex_idx, byte_in_vertex);

        // Show the actual bytes for context (up to 32 bytes)
        let show_bytes = std::cmp::min(64, len);
        let ctx_start = if *start >= 16 { start - 16 } else { 0 };
        let ctx_end = std::cmp::min(*end + 16, compare_len);

        println!("    HODOR:     {:02x?}", &hodor[ctx_start..std::cmp::min(ctx_start + show_bytes + 32, compare_len)]);
        println!("    Generated: {:02x?}", &generated[ctx_start..std::cmp::min(ctx_start + show_bytes + 32, compare_len)]);

        // If mesh pool, interpret as float values
        if name == "Mesh" && stride >= 16 {
            let aligned_start = (start / 4) * 4;
            let float_count = std::cmp::min(8, (end - aligned_start + 3) / 4);
            println!("    HODOR floats:     ");
            for f in 0..float_count {
                let off = aligned_start + f * 4;
                if off + 4 <= hodor.len() {
                    let val = f32::from_le_bytes([hodor[off], hodor[off+1], hodor[off+2], hodor[off+3]]);
                    print!("      [0x{:04X}] {:12.6}", off, val);
                }
            }
            println!();
            println!("    Generated floats: ");
            for f in 0..float_count {
                let off = aligned_start + f * 4;
                if off + 4 <= generated.len() {
                    let val = f32::from_le_bytes([generated[off], generated[off+1], generated[off+2], generated[off+3]]);
                    print!("      [0x{:04X}] {:12.6}", off, val);
                }
            }
            println!();
        }
    }

    // Summarize which vertex attribute fields differ (for mesh pool)
    if name == "Mesh" && stride >= 16 {
        println!("\n  Vertex attribute diff heatmap (stride={}):", stride);
        let mut byte_diff_histogram = vec![0u32; stride];
        let vertex_count = compare_len / stride;
        for v in 0..vertex_count {
            for b in 0..stride {
                let offset = v * stride + b;
                if offset < compare_len && hodor[offset] != generated[offset] {
                    byte_diff_histogram[b] += 1;
                }
            }
        }
        // Print the heatmap
        for (b, count) in byte_diff_histogram.iter().enumerate() {
            if *count > 0 {
                let field = match b {
                    0..=15 => "position (xyz+w)",
                    16..=31 => "normal (xyz+w)",
                    32..=39 => "uv (u+v)",
                    40..=51 => "tangent (xyz)",
                    52..=63 => "binormal (xyz)",
                    _ => "unknown",
                };
                println!("    Byte {:2} ({:16}): {} vertices differ", b, field, count);
            }
        }
    }
}

fn main() -> Result<(), String> {
    let test_cases = vec![
        ("ter_centaur", "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/mod-tools/HODEditorJS/testing/ter_centaur"),
    ];

    for (name, dir) in &test_cases {
        println!("\n{}", "=".repeat(60));
        println!("=== {} ===", name);
        println!("{}", "=".repeat(60));

        let hodor_path = format!("{}/{}_hodor.hod", dir, name);
        let generated_path = format!("{}/{}_generated.hod", dir, name);

        println!("\nExtracting HODOR pools from: {}", hodor_path);
        let (hodor_tex, hodor_mesh, hodor_face, hodor_pool_type) = extract_pools(&hodor_path)?;

        println!("\nExtracting Generated pools from: {}", generated_path);
        let (gen_tex, gen_mesh, gen_face, gen_pool_type) = extract_pools(&generated_path)?;

        println!("\nPool types: HODOR={} Generated={}", hodor_pool_type, gen_pool_type);

        // For ter_centaur, vertex mask is 0x600B which means:
        // bit 0 (pos) = 16, bit 1 (normal) = 16, bit 3 (uv) = 8, bit 13 (tangent) = 12, bit 14 (binormal) = 12
        // stride = 64
        compare_pools("Texture", &hodor_tex, &gen_tex, 0);
        compare_pools("Mesh", &hodor_mesh, &gen_mesh, 64);
        compare_pools("Face", &hodor_face, &gen_face, 2);

        // Also dump first 3 vertices from each for visual inspection
        println!("\n=== First 3 Vertices (stride=64) ===");
        for (label, pool) in [("HODOR", &hodor_mesh), ("Generated", &gen_mesh)] {
            println!("\n  {}:", label);
            for v in 0..std::cmp::min(3, pool.len() / 64) {
                let off = v * 64;
                let pos_x = f32::from_le_bytes([pool[off], pool[off+1], pool[off+2], pool[off+3]]);
                let pos_y = f32::from_le_bytes([pool[off+4], pool[off+5], pool[off+6], pool[off+7]]);
                let pos_z = f32::from_le_bytes([pool[off+8], pool[off+9], pool[off+10], pool[off+11]]);
                let pos_w = f32::from_le_bytes([pool[off+12], pool[off+13], pool[off+14], pool[off+15]]);
                let nor_x = f32::from_le_bytes([pool[off+16], pool[off+17], pool[off+18], pool[off+19]]);
                let nor_y = f32::from_le_bytes([pool[off+20], pool[off+21], pool[off+22], pool[off+23]]);
                let nor_z = f32::from_le_bytes([pool[off+24], pool[off+25], pool[off+26], pool[off+27]]);
                let nor_w = f32::from_le_bytes([pool[off+28], pool[off+29], pool[off+30], pool[off+31]]);
                let uv_u = f32::from_le_bytes([pool[off+32], pool[off+33], pool[off+34], pool[off+35]]);
                let uv_v = f32::from_le_bytes([pool[off+36], pool[off+37], pool[off+38], pool[off+39]]);
                let tan_x = f32::from_le_bytes([pool[off+40], pool[off+41], pool[off+42], pool[off+43]]);
                let tan_y = f32::from_le_bytes([pool[off+44], pool[off+45], pool[off+46], pool[off+47]]);
                let tan_z = f32::from_le_bytes([pool[off+48], pool[off+49], pool[off+50], pool[off+51]]);
                let bin_x = f32::from_le_bytes([pool[off+52], pool[off+53], pool[off+54], pool[off+55]]);
                let bin_y = f32::from_le_bytes([pool[off+56], pool[off+57], pool[off+58], pool[off+59]]);
                let bin_z = f32::from_le_bytes([pool[off+60], pool[off+61], pool[off+62], pool[off+63]]);
                println!("    V{}: pos=({:.4}, {:.4}, {:.4}, {:.4}) nor=({:.4}, {:.4}, {:.4}, {:.4}) uv=({:.4}, {:.4}) tan=({:.4}, {:.4}, {:.4}) bin=({:.4}, {:.4}, {:.4})",
                         v, pos_x, pos_y, pos_z, pos_w, nor_x, nor_y, nor_z, nor_w, uv_u, uv_v, tan_x, tan_y, tan_z, bin_x, bin_y, bin_z);
            }
        }

        // Compare BMSH chunk data between the two HODs
        println!("\n=== BMSH Chunk Comparison ===");
        compare_bmsh_chunks(&hodor_path, &generated_path)?;
    }

    Ok(())
}

fn compare_bmsh_chunks(hodor_path: &str, generated_path: &str) -> Result<(), String> {
    let hodor_bmsh = extract_bmsh_data(hodor_path)?;
    let gen_bmsh = extract_bmsh_data(generated_path)?;

    println!("  HODOR BMSH chunks: {}", hodor_bmsh.len());
    println!("  Generated BMSH chunks: {}", gen_bmsh.len());

    for (idx, (h, g)) in hodor_bmsh.iter().zip(gen_bmsh.iter()).enumerate() {
        if h == g {
            println!("  BMSH[{}]: ✅ IDENTICAL ({} bytes)", idx, h.len());
        } else {
            println!("  BMSH[{}]: ❌ DIFFER", idx);
            println!("    HODOR:     {:02x?}", h);
            println!("    Generated: {:02x?}", g);
        }
    }

    Ok(())
}

fn extract_bmsh_data(path: &str) -> Result<Vec<Vec<u8>>, String> {
    let data = fs::read(path).map_err(|e| e.to_string())?;
    let mut results = Vec::new();

    // Simple scan for "BMSH" followed by version bytes
    let bmsh_tag = b"BMSH";
    for i in 0..data.len().saturating_sub(8) {
        if &data[i..i+4] == bmsh_tag {
            // Read size (big-endian u32 right after the tag, preceded by NRML wrapper)
            // Actually for MULT children, BMSH appears as: NRML <size_be> BMSH <version_be>
            // The BMSH data follows the version
            if i >= 8 && &data[i-8..i-4] == b"NRML" {
                let nrml_size = u32::from_be_bytes([data[i-4], data[i-3], data[i-2], data[i-1]]) as usize;
                let bmsh_version = u32::from_be_bytes([data[i+4], data[i+5], data[i+6], data[i+7]]);
                let bmsh_data_start = i + 8;
                let bmsh_data_end = std::cmp::min(bmsh_data_start + nrml_size - 8, data.len());
                if bmsh_data_end > bmsh_data_start {
                    results.push(data[bmsh_data_start..bmsh_data_end].to_vec());
                }
            }
        }
    }

    Ok(results)
}
