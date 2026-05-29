use byteorder::{LittleEndian, ReadBytesExt};
use hwr_hod_parser::xpress;
use roxmltree::Document;
use std::collections::HashMap;
use std::fs;
use std::io::{Cursor, Read};

fn main() -> Result<(), String> {
    let hodor_path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/mod-tools/HODEditorJS/testing/ter_centaur/ter_centaur_hodor.hod";
    let data = fs::read(hodor_path).unwrap();
    let mut cursor = Cursor::new(&data);

    let mut decomp_face = Vec::new();
    loop {
        if cursor.position() >= data.len() as u64 {
            break;
        }
        let mut id_bytes = [0u8; 4];
        cursor.read_exact(&mut id_bytes).unwrap();
        let id = String::from_utf8_lossy(&id_bytes).to_string();
        let size = cursor.read_u32::<byteorder::BigEndian>().unwrap();

        if id == "POOL" {
            let _ = cursor.read_u32::<LittleEndian>().unwrap();
            let comp_tex_len = cursor.read_u32::<LittleEndian>().unwrap() as usize;
            let decomp_tex_len = cursor.read_u32::<LittleEndian>().unwrap() as usize;
            let mut comp_tex = vec![0u8; comp_tex_len];
            cursor.read_exact(&mut comp_tex).unwrap();

            let comp_mesh_len = cursor.read_u32::<LittleEndian>().unwrap() as usize;
            let decomp_mesh_len = cursor.read_u32::<LittleEndian>().unwrap() as usize;
            let mut comp_mesh = vec![0u8; comp_mesh_len];
            cursor.read_exact(&mut comp_mesh).unwrap();

            let comp_face_len = cursor.read_u32::<LittleEndian>().unwrap() as usize;
            let decomp_face_len = cursor.read_u32::<LittleEndian>().unwrap() as usize;
            let mut comp_face = vec![0u8; comp_face_len];
            cursor.read_exact(&mut comp_face).unwrap();
            decomp_face = xpress::decompress(&comp_face, decomp_face_len).unwrap();
            break;
        } else {
            let mut skip = vec![0u8; size as usize];
            cursor.read_exact(&mut skip).unwrap();
        }
    }

    let mut f_cursor = Cursor::new(&decomp_face);
    let mut part0_indices = Vec::new();
    for _ in 0..3915 {
        part0_indices.push(f_cursor.read_u16::<LittleEndian>().unwrap());
    }

    // Now parse DAE tuples
    let dae_path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/mod-tools/HODEditorJS/testing/ter_centaur/ter_centaur.DAE";
    let xml = fs::read_to_string(dae_path).map_err(|e| e.to_string())?;
    let doc = Document::parse(&xml).map_err(|e| e.to_string())?;

    let mut dae_tuples = Vec::new();
    for geometry in doc
        .descendants()
        .filter(|node| node.has_tag_name("geometry"))
    {
        let geom_id = geometry.attribute("id").unwrap_or_default();
        if geom_id != "MULT[Root_mesh]_LOD[0]-lib" {
            continue;
        }
        let Some(mesh_node) = geometry.children().find(|node| node.has_tag_name("mesh")) else {
            continue;
        };
        for triangles in mesh_node
            .children()
            .filter(|node| node.has_tag_name("triangles"))
        {
            let material = triangles.attribute("material").unwrap_or("none");
            if !material.starts_with("MAT[centaur]") {
                continue;
            }
            let mut position_offset = None;
            let mut normal_offset = None;
            let mut uv_offset = None;
            let mut max_offset = 0usize;
            for input in triangles
                .children()
                .filter(|node| node.has_tag_name("input"))
            {
                let offset = input
                    .attribute("offset")
                    .unwrap_or("0")
                    .parse::<usize>()
                    .unwrap_or(0);
                max_offset = max_offset.max(offset);
                match input.attribute("semantic") {
                    Some("VERTEX") => position_offset = Some(offset),
                    Some("NORMAL") => normal_offset = Some(offset),
                    Some("TEXCOORD") => uv_offset = Some(offset),
                    _ => {}
                }
            }
            let stride = max_offset + 1;
            for p in triangles.children().filter(|node| node.has_tag_name("p")) {
                let Some(text) = p.text() else {
                    continue;
                };
                let indices: Vec<usize> = text
                    .split_whitespace()
                    .map(|value| value.parse::<usize>().unwrap_or(0))
                    .collect();
                for vertex in indices.chunks(stride) {
                    if vertex.len() < stride {
                        continue;
                    }
                    let p_idx = position_offset.map(|offset| vertex[offset]).unwrap_or(0);
                    let uv_idx = uv_offset.map(|offset| vertex[offset]).unwrap_or(0);
                    let n_idx = normal_offset.map(|offset| vertex[offset]).unwrap_or(0);
                    dae_tuples.push((p_idx, uv_idx, n_idx));
                }
            }
        }
    }

    // Map HODOR index to its first occurrence position in part0_indices
    let mut hodor_idx_to_first_pos = HashMap::new();
    let mut dup_pairs = Vec::new();
    for (pos, &idx) in part0_indices.iter().enumerate() {
        if let Some(&first_pos) = hodor_idx_to_first_pos.get(&idx) {
            dup_pairs.push((pos, first_pos));
        } else {
            hodor_idx_to_first_pos.insert(idx, pos);
        }
    }

    println!("Total duplicates found: {}", dup_pairs.len());
    println!("Duplicate pairs (pos, first_pos):");
    for (idx, (pos, first_pos)) in dup_pairs.iter().enumerate() {
        println!(
            "  {:2}: pos {} is duplicate of pos {} -> DAE tuple is {:?}",
            idx, pos, first_pos, dae_tuples[*pos]
        );
    }

    Ok(())
}
