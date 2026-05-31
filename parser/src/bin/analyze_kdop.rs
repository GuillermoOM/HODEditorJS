use byteorder::{LittleEndian, ReadBytesExt};
use hwr_hod_parser::iff::IffChunk;
use std::env;
use std::fs::File;
use std::io::{Cursor, Read};

fn find_chunks<'a>(chunk: &'a IffChunk, id: &str, results: &mut Vec<&'a IffChunk>) {
    if chunk.id == id {
        results.push(chunk);
    }
    for child in &chunk.children {
        find_chunks(child, id, results);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let files: Vec<String> = if args.len() > 1 {
        args[1..].to_vec()
    } else {
        vec![
            "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/uncompressed_bigs/HWRM/pebble/pebble_0/pebble_0.hod".into(),
            "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/uncompressed_bigs/HWRM/ship/kus_targetdrone/kus_targetdrone.hod".into(),
            "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/uncompressed_bigs/HWRM/ship/junk_autoguns/junk_autoguns.hod".into(),
        ]
    };

    for file_path in &files {
        let short_name = std::path::Path::new(file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let mut file = File::open(file_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let mut cursor = Cursor::new(buffer.clone());
        let mut all_chunks = Vec::new();
        while (cursor.position() as usize) < buffer.len() {
            match IffChunk::read_chunk(&mut cursor) {
                Ok(chunk) => all_chunks.push(chunk),
                Err(_) => break,
            }
        }

        let mut kdop_chunks = Vec::new();
        for chunk in &all_chunks {
            find_chunks(chunk, "KDOP", &mut kdop_chunks);
        }

        println!("\n=== {} ===", short_name);
        for kdop in &kdop_chunks {
            println!("KDOP size: {} bytes", kdop.data.len());
            dump_kdop_analysis(&kdop.data);
        }
    }

    Ok(())
}

fn dump_kdop_analysis(data: &[u8]) {
    // Print first 128 bytes as hex
    println!("First 128 bytes:");
    for i in (0..data.len().min(128)).step_by(16) {
        let end = (i + 16).min(data.len());
        let hex: Vec<String> = data[i..end].iter().map(|b| format!("{:02x}", b)).collect();
        let ascii: String = data[i..end]
            .iter()
            .map(|&b| if (32..127).contains(&b) { b as char } else { '.' })
            .collect();
        println!("  {:4}: {:48} {}", i, hex.join(" "), ascii);
    }

    // Try parsing as: [header] + vert_count(u32) + vertices(3*f32) + tri_count(u32) + tris(3*u16)
    for header_size in 0usize..64 {
        if header_size + 4 > data.len() {
            continue;
        }
        let vert_count = u32::from_le_bytes([
            data[header_size], data[header_size+1], data[header_size+2], data[header_size+3]
        ]) as usize;
        if vert_count == 0 || vert_count > 10000 {
            continue;
        }
        
        let vert_data_size = vert_count * 12;
        let after_verts = header_size + 4 + vert_data_size;
        if after_verts + 4 > data.len() {
            continue;
        }
        
        let tri_count = u32::from_le_bytes([
            data[after_verts], data[after_verts+1], data[after_verts+2], data[after_verts+3]
        ]) as usize;
        if tri_count == 0 || tri_count > 100000 {
            continue;
        }
        
        let total_3u16 = header_size + 4 + vert_data_size + 4 + tri_count * 6;
        let total_2u16 = header_size + 4 + vert_data_size + 4 + tri_count * 4;
        let total_1u16 = header_size + 4 + vert_data_size + 4 + tri_count * 2;
        
        let format = if total_3u16 == data.len() {
            "3*u16 (triangles)"
        } else if total_2u16 == data.len() {
            "2*u16 (edges)"
        } else if total_1u16 == data.len() {
            "1*u16 (indices)"
        } else {
            continue;
        };
        
        let total = if total_3u16 == data.len() { total_3u16 }
                    else if total_2u16 == data.len() { total_2u16 }
                    else { total_1u16 };
        
        println!("\n*** MATCH: header_size={}, vert_count={}, count2={}, format={} ***",
            header_size, vert_count, tri_count, format);
        
        if header_size > 0 {
            println!("  Header ({} bytes): {:02x?}", header_size, &data[..header_size]);
        }
        
        // Print first few vertices
        let mut pos = header_size + 4;
        for i in 0..vert_count.min(5) {
            let x = f32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]);
            let y = f32::from_le_bytes([data[pos+4], data[pos+5], data[pos+6], data[pos+7]]);
            let z = f32::from_le_bytes([data[pos+8], data[pos+9], data[pos+10], data[pos+11]]);
            println!("  v[{}]: ({:.6}, {:.6}, {:.6})", i, x, y, z);
            pos += 12;
        }
        if vert_count > 5 {
            println!("  ... ({} more vertices)", vert_count - 5);
        }
        
        // Print first few indices/triangles
        pos = after_verts + 4;
        let stride = if total_3u16 == data.len() { 6 }
                     else if total_2u16 == data.len() { 4 }
                     else { 2 };
        for i in 0..tri_count.min(5) {
            if stride >= 6 {
                let a = u16::from_le_bytes([data[pos], data[pos+1]]);
                let b = u16::from_le_bytes([data[pos+2], data[pos+3]]);
                let c = u16::from_le_bytes([data[pos+4], data[pos+5]]);
                println!("  tri[{}]: ({}, {}, {})", i, a, b, c);
            } else if stride >= 4 {
                let a = u16::from_le_bytes([data[pos], data[pos+1]]);
                let b = u16::from_le_bytes([data[pos+2], data[pos+3]]);
                println!("  edge[{}]: ({}, {})", i, a, b);
            } else {
                let a = u16::from_le_bytes([data[pos], data[pos+1]]);
                println!("  idx[{}]: ({})", i, a);
            }
            pos += stride;
        }
        return;
    }

    // Also try: name_len(u32) + name + vert_count + vertices + tri_count + triangles
    for name_len in 1usize..64 {
        let name_start = 4 + name_len;
        if name_start + 4 > data.len() {
            break;
        }
        
        let name_bytes = &data[4..4 + name_len];
        let is_printable = name_bytes.iter().all(|&b| (32..127).contains(&b));
        if !is_printable {
            continue;
        }
        
        let name = String::from_utf8_lossy(name_bytes);
        
        let vert_count = u32::from_le_bytes([
            data[name_start], data[name_start+1], data[name_start+2], data[name_start+3]
        ]) as usize;
        if vert_count == 0 || vert_count > 10000 {
            continue;
        }
        
        let vert_data_size = vert_count * 12;
        let after_verts = name_start + 4 + vert_data_size;
        if after_verts + 4 > data.len() {
            continue;
        }
        
        let tri_count = u32::from_le_bytes([
            data[after_verts], data[after_verts+1], data[after_verts+2], data[after_verts+3]
        ]) as usize;
        if tri_count == 0 || tri_count > 100000 {
            continue;
        }
        
        let total = 4 + name_len + 4 + vert_data_size + 4 + tri_count * 6;
        if total == data.len() {
            println!("\n*** MATCH with name: name='{}', name_len={}, vert_count={}, tri_count={} ***",
                name, name_len, vert_count, tri_count);
            let mut pos = name_start + 4;
            for i in 0..vert_count.min(5) {
                let x = f32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]);
                let y = f32::from_le_bytes([data[pos+4], data[pos+5], data[pos+6], data[pos+7]]);
                let z = f32::from_le_bytes([data[pos+8], data[pos+9], data[pos+10], data[pos+11]]);
                println!("  v[{}]: ({:.6}, {:.6}, {:.6})", i, x, y, z);
                pos += 12;
            }
            return;
        }
    }

    println!("\nNo clean match found. Analyzing structure...");
    
    // Look for patterns: what if it's a flat array of floats?
    let num_floats = data.len() / 4;
    println!("  Could be {} floats", num_floats);
    println!("  Could be {} vec3s ({} bytes leftover)", num_floats / 3, data.len() % 12);
    
    // Print last 64 bytes
    println!("\nLast 64 bytes:");
    let start = data.len().saturating_sub(64);
    for i in (start..data.len()).step_by(16) {
        let end = (i + 16).min(data.len());
        let hex: Vec<String> = data[i..end].iter().map(|b| format!("{:02x}", b)).collect();
        let ascii: String = data[i..end]
            .iter()
            .map(|&b| if (32..127).contains(&b) { b as char } else { '.' })
            .collect();
        println!("  {:4}: {:48} {}", i, hex.join(" "), ascii);
    }
}
