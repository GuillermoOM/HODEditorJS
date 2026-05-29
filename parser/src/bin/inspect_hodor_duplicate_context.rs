use byteorder::{LittleEndian, ReadBytesExt};
use hwr_hod_parser::xpress;
use roxmltree::Document;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{Cursor, Read};

fn extract_hodor_lod0_centaur_indices() -> Vec<u16> {
    let hodor_path = "../testing/ter_centaur/ter_centaur_hodor.hod";
    let data = fs::read(hodor_path).unwrap();
    let mut cursor = Cursor::new(&data);
    loop {
        let mut id_bytes = [0u8; 4];
        cursor.read_exact(&mut id_bytes).unwrap();
        let id = String::from_utf8_lossy(&id_bytes).to_string();
        let size = cursor.read_u32::<byteorder::BigEndian>().unwrap();
        if id == "POOL" {
            let _pool_type = cursor.read_u32::<LittleEndian>().unwrap();
            let comp_tex_len = cursor.read_u32::<LittleEndian>().unwrap() as usize;
            let _decomp_tex_len = cursor.read_u32::<LittleEndian>().unwrap() as usize;
            let mut comp_tex = vec![0u8; comp_tex_len];
            cursor.read_exact(&mut comp_tex).unwrap();
            let comp_mesh_len = cursor.read_u32::<LittleEndian>().unwrap() as usize;
            let _decomp_mesh_len = cursor.read_u32::<LittleEndian>().unwrap() as usize;
            let mut comp_mesh = vec![0u8; comp_mesh_len];
            cursor.read_exact(&mut comp_mesh).unwrap();
            let comp_face_len = cursor.read_u32::<LittleEndian>().unwrap() as usize;
            let decomp_face_len = cursor.read_u32::<LittleEndian>().unwrap() as usize;
            let mut comp_face = vec![0u8; comp_face_len];
            cursor.read_exact(&mut comp_face).unwrap();
            let decomp_face = xpress::decompress(&comp_face, decomp_face_len).unwrap();
            let mut f_cursor = Cursor::new(&decomp_face);
            return (0..3915)
                .map(|_| f_cursor.read_u16::<LittleEndian>().unwrap())
                .collect();
        }
        let mut skip = vec![0u8; size as usize];
        cursor.read_exact(&mut skip).unwrap();
    }
}

fn dae_tuples_for_centaur_lod0() -> Vec<(usize, usize, usize)> {
    let xml = fs::read_to_string("../testing/ter_centaur/ter_centaur.DAE").unwrap();
    let doc = Document::parse(&xml).unwrap();
    let mut tuples = Vec::new();
    for geometry in doc
        .descendants()
        .filter(|node| node.has_tag_name("geometry"))
    {
        if geometry.attribute("id") != Some("MULT[Root_mesh]_LOD[0]-lib") {
            continue;
        }
        let mesh = geometry
            .children()
            .find(|node| node.has_tag_name("mesh"))
            .unwrap();
        for triangles in mesh
            .children()
            .filter(|node| node.has_tag_name("triangles"))
        {
            if !triangles
                .attribute("material")
                .unwrap_or_default()
                .starts_with("MAT[centaur]")
            {
                continue;
            }
            let mut pos_offset = 0usize;
            let mut norm_offset = 0usize;
            let mut uv_offset = 0usize;
            let mut max_offset = 0usize;
            for input in triangles
                .children()
                .filter(|node| node.has_tag_name("input"))
            {
                let offset = input
                    .attribute("offset")
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0);
                max_offset = max_offset.max(offset);
                match input.attribute("semantic") {
                    Some("VERTEX") => pos_offset = offset,
                    Some("NORMAL") => norm_offset = offset,
                    Some("TEXCOORD") => uv_offset = offset,
                    _ => {}
                }
            }
            let stride = max_offset + 1;
            let values: Vec<usize> = triangles
                .children()
                .find(|node| node.has_tag_name("p"))
                .unwrap()
                .text()
                .unwrap()
                .split_whitespace()
                .map(|value| value.parse().unwrap())
                .collect();
            for vertex in values.chunks(stride) {
                tuples.push((vertex[pos_offset], vertex[uv_offset], vertex[norm_offset]));
            }
        }
    }
    tuples
}

fn main() {
    let indices = extract_hodor_lod0_centaur_indices();
    let tuples = dae_tuples_for_centaur_lod0();
    let mut first_by_hodor_index = HashMap::new();
    let mut duplicate_positions = HashSet::new();
    for (pos, &idx) in indices.iter().enumerate() {
        if first_by_hodor_index.insert(idx, pos).is_some() {
            duplicate_positions.insert(pos);
        }
    }

    let mut tuple_groups: HashMap<(usize, usize, usize), Vec<usize>> = HashMap::new();
    for (pos, &tuple) in tuples.iter().enumerate() {
        tuple_groups.entry(tuple).or_default().push(pos);
    }

    println!("hodor duplicate positions: {}", duplicate_positions.len());
    let mut repeat_tuple_positions = 0usize;
    let mut hodor_duplicates_from_repeat_tuples = 0usize;
    for positions in tuple_groups.values() {
        if positions.len() > 1 {
            repeat_tuple_positions += positions.len() - 1;
            hodor_duplicates_from_repeat_tuples += positions
                .iter()
                .filter(|pos| duplicate_positions.contains(pos))
                .count();
        }
    }
    println!(
        "repeat tuple duplicate candidates: {}",
        repeat_tuple_positions
    );
    println!(
        "hodor duplicate positions among repeated tuples: {}",
        hodor_duplicates_from_repeat_tuples
    );

    for (pos, &idx) in indices.iter().enumerate() {
        let Some(&first_pos) = first_by_hodor_index.get(&idx) else {
            continue;
        };
        if first_pos == pos {
            continue;
        }
        println!(
            "pos {:4} -> {:4}: dup tuple {:?}, first tuple {:?}",
            pos, first_pos, tuples[pos], tuples[first_pos]
        );
    }
}
