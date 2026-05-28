use std::fs;

fn main() {
    let pebble_orig = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/uncompressed_bigs/HWRM/pebble/pebble_0/pebble_0.hod";
    let pebble_gen = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/uncompressed_bigs/HWRM/pebble/pebble_0/pebble_0.hod_generated.hod";

    let orig = fs::read(pebble_orig).expect("Cannot read original");
    let gen = fs::read(pebble_gen).expect("Cannot read generated");

    println!("=== PEBBLE_0 DETAILED COMPARISON ===");
    println!("Original: {} bytes", orig.len());
    println!("Generated: {} bytes", gen.len());
    println!("Diff: {} bytes\n", gen.len() as isize - orig.len() as isize);

    // Parse original
    println!("=== ORIGINAL ===");
    let orig_chunks = parse_chunks(&orig);
    for c in &orig_chunks {
        print_chunk(c, &orig, 0);
    }

    println!("\n=== GENERATED ===");
    let gen_chunks = parse_chunks(&gen);
    for c in &gen_chunks {
        print_chunk(c, &gen, 0);
    }

    // Compare POOL internals
    println!("\n=== POOL INTERNALS ===");
    let orig_pool = orig_chunks.iter().find(|c| c.id == "POOL").unwrap();
    let gen_pool = gen_chunks.iter().find(|c| c.id == "POOL").unwrap();
    println!("Original POOL: offset=0x{:06X}, data_len={}", orig_pool.data_offset, orig_pool.data_len);
    dump_pool(&orig, orig_pool.data_offset);
    println!();
    println!("Generated POOL: offset=0x{:06X}, data_len={}", gen_pool.data_offset, gen_pool.data_len);
    dump_pool(&gen, gen_pool.data_offset);
}

struct ChunkInfo {
    id: String,
    chunk_type: String, // FORM, NRML, DEF
    form_type: Option<String>,
    data_offset: usize, // offset of payload data in file
    data_len: usize,
    children: Vec<ChunkInfo>,
}

fn parse_chunks(data: &[u8]) -> Vec<ChunkInfo> {
    let mut chunks = Vec::new();
    let mut pos = 0;
    while pos + 8 <= data.len() {
        let tag = &data[pos..pos+4];
        let size_bytes: [u8; 4] = data[pos+4..pos+8].try_into().unwrap();
        let size_be = u32::from_be_bytes(size_bytes) as usize;
        let size_le = u32::from_le_bytes(size_bytes) as usize;

        if tag == b"FORM" {
            if pos + 12 > data.len() { break; }
            let form_type = std::str::from_utf8(&data[pos+8..pos+12]).unwrap_or("????").to_string();
            let total_size = if size_be < 0x01000000 { size_be } else { size_le };
            
            // Check if it's a container
            let is_container = matches!(form_type.as_str(), "HVMD" | "DTRM" | "INFO");
            
            if is_container {
                let mut children = Vec::new();
                let mut sub_pos = pos + 12;
                let end = pos + 8 + total_size;
                while sub_pos + 8 <= end && sub_pos + 8 <= data.len() {
                    let sub_tag = &data[sub_pos..sub_pos+4];
                    let sub_size_bytes: [u8; 4] = data[sub_pos+4..sub_pos+8].try_into().unwrap();
                    let sub_size_be = u32::from_be_bytes(sub_size_bytes) as usize;
                    let sub_size_le = u32::from_le_bytes(sub_size_bytes) as usize;
                    
                    if sub_tag == b"FORM" {
                        if sub_pos + 12 > data.len() { break; }
                        let sub_form_type = std::str::from_utf8(&data[sub_pos+8..sub_pos+12]).unwrap_or("????").to_string();
                        let sub_total = if sub_size_be < 0x01000000 { sub_size_be } else { sub_size_le };
                        
                        let sub_is_container = matches!(sub_form_type.as_str(), "HVMD" | "DTRM" | "INFO");
                        if sub_is_container {
                            // Recurse for nested containers
                            let mut sub_children = Vec::new();
                            let mut ss_pos = sub_pos + 12;
                            let ss_end = sub_pos + 8 + sub_total;
                            while ss_pos + 8 <= ss_end && ss_pos + 8 <= data.len() {
                                let ss_tag = &data[ss_pos..ss_pos+4];
                                let ss_size_bytes: [u8; 4] = data[ss_pos+4..ss_pos+8].try_into().unwrap();
                                let ss_size_be = u32::from_be_bytes(ss_size_bytes) as usize;
                                let ss_size_le = u32::from_le_bytes(ss_size_bytes) as usize;
                                if ss_tag == b"FORM" {
                                    if ss_pos + 12 > data.len() { break; }
                                    let ss_ft = std::str::from_utf8(&data[ss_pos+8..ss_pos+12]).unwrap_or("????").to_string();
                                    let ss_total = if ss_size_be < 0x01000000 { ss_size_be } else { ss_size_le };
                                    sub_children.push(ChunkInfo {
                                        id: ss_ft.clone(),
                                        chunk_type: "FORM".to_string(),
                                        form_type: Some(ss_ft),
                                        data_offset: ss_pos + 12,
                                        data_len: ss_total.saturating_sub(4),
                                        children: Vec::new(),
                                    });
                                    ss_pos += 8 + ss_total;
                                } else {
                                    let actual = if ss_size_be > 0x01000000 && ss_size_le <= 0x01000000 { ss_size_le } else { ss_size_be };
                                    let ss_id = std::str::from_utf8(&data[ss_pos..ss_pos+4]).unwrap_or("????").to_string();
                                    let ct = if ss_tag == b"NRML" { "NRML" } else { "DEF" };
                                    sub_children.push(ChunkInfo {
                                        id: ss_id,
                                        chunk_type: ct.to_string(),
                                        form_type: None,
                                        data_offset: ss_pos + if ct == "NRML" { 12 } else { 8 },
                                        data_len: actual,
                                        children: Vec::new(),
                                    });
                                    ss_pos += 8 + actual;
                                }
                            }
                            children.push(ChunkInfo {
                                id: sub_form_type.clone(),
                                chunk_type: "FORM".to_string(),
                                form_type: Some(sub_form_type),
                                data_offset: sub_pos + 12,
                                data_len: sub_total.saturating_sub(4),
                                children: sub_children,
                            });
                        } else {
                            children.push(ChunkInfo {
                                id: sub_form_type.clone(),
                                chunk_type: "FORM".to_string(),
                                form_type: Some(sub_form_type),
                                data_offset: sub_pos + 12,
                                data_len: sub_total.saturating_sub(4),
                                children: Vec::new(),
                            });
                        }
                        sub_pos += 8 + sub_total;
                    } else {
                        let actual = if sub_size_be > 0x01000000 && sub_size_le <= 0x01000000 { sub_size_le } else { sub_size_be };
                        let sub_id = std::str::from_utf8(sub_tag).unwrap_or("????").to_string();
                        let ct = if tag == b"NRML" || sub_tag == b"NRML" { "NRML" } else { "DEF" };
                        children.push(ChunkInfo {
                            id: sub_id,
                            chunk_type: ct.to_string(),
                            form_type: None,
                            data_offset: sub_pos + if ct == "NRML" { 12 } else { 8 },
                            data_len: actual,
                            children: Vec::new(),
                        });
                        sub_pos += 8 + actual;
                    }
                }
                chunks.push(ChunkInfo {
                    id: form_type.clone(),
                    chunk_type: "FORM".to_string(),
                    form_type: Some(form_type),
                    data_offset: pos + 12,
                    data_len: total_size.saturating_sub(4),
                    children,
                });
            } else {
                chunks.push(ChunkInfo {
                    id: form_type.clone(),
                    chunk_type: "FORM".to_string(),
                    form_type: Some(form_type),
                    data_offset: pos + 12,
                    data_len: total_size.saturating_sub(4),
                    children: Vec::new(),
                });
            }
            pos += 8 + total_size;
        } else {
            let actual = if size_be > 0x01000000 && size_le <= 0x01000000 { size_le } else { size_be };
            let id_str = std::str::from_utf8(tag).unwrap_or("????").to_string();
            let ct = if tag == b"NRML" { "NRML" } else { "DEF" };
            chunks.push(ChunkInfo {
                id: id_str,
                chunk_type: ct.to_string(),
                form_type: None,
                data_offset: pos + if ct == "NRML" { 12 } else { 8 },
                data_len: actual,
                children: Vec::new(),
            });
            pos += 8 + actual;
        }
    }
    chunks
}

fn print_chunk(c: &ChunkInfo, data: &[u8], indent: usize) {
    let pad = "  ".repeat(indent);
    if let Some(ref ft) = c.form_type {
        println!("{}FORM ({}) payload_len={}", pad, ft, c.data_len);
    } else {
        println!("{}{} {} size={}", pad, c.chunk_type, c.id, c.data_len);
    }
    for child in &c.children {
        print_chunk(child, data, indent + 1);
    }
}

fn dump_pool(data: &[u8], pool_data_offset: usize) {
    let mut pos = pool_data_offset;
    if pos + 16 > data.len() { println!("  POOL too short"); return; }

    let pool_type = u32::from_le_bytes(data[pos..pos+4].try_into().unwrap());
    pos += 4;
    let comp_tex_len = u32::from_le_bytes(data[pos..pos+4].try_into().unwrap()) as usize;
    pos += 4;
    let decomp_tex_len = u32::from_le_bytes(data[pos..pos+4].try_into().unwrap()) as usize;
    pos += 4;
    println!("  pool_type={} comp_tex={} decomp_tex={}", pool_type, comp_tex_len, decomp_tex_len);
    
    // Skip compressed texture data
    pos += comp_tex_len;
    
    if pos + 12 > data.len() { println!("  POOL truncated at mesh section"); return; }
    let comp_mesh = u32::from_le_bytes(data[pos..pos+4].try_into().unwrap()) as usize;
    pos += 4;
    let decomp_mesh = u32::from_le_bytes(data[pos..pos+4].try_into().unwrap()) as usize;
    pos += 4;
    println!("  comp_mesh={} decomp_mesh={}", comp_mesh, decomp_mesh);
    
    pos += comp_mesh;
    if pos + 8 > data.len() { println!("  POOL truncated at face section"); return; }
    let comp_face = u32::from_le_bytes(data[pos..pos+4].try_into().unwrap()) as usize;
    pos += 4;
    let decomp_face = u32::from_le_bytes(data[pos..pos+4].try_into().unwrap()) as usize;
    println!("  comp_face={} decomp_face={}", comp_face, decomp_face);
}
