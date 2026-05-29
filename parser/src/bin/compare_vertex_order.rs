use byteorder::{LittleEndian, ReadBytesExt};
use hwr_hod_parser::xpress;
use std::fs;
use std::io::{Cursor, Read};

fn extract_pool(hod_path: &str) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let data = fs::read(hod_path).unwrap();
    let mut cursor = Cursor::new(&data);
    loop {
        let mut id_bytes = [0u8; 4];
        cursor.read_exact(&mut id_bytes).unwrap();
        let id = String::from_utf8_lossy(&id_bytes).to_string();
        let size = cursor.read_u32::<byteorder::BigEndian>().unwrap();
        if id == "POOL" {
            let _pool_type = cursor.read_u32::<LittleEndian>().unwrap();
            let comp_tex_len = cursor.read_u32::<LittleEndian>().unwrap() as usize;
            let decomp_tex_len = cursor.read_u32::<LittleEndian>().unwrap() as usize;
            let mut comp_tex = vec![0u8; comp_tex_len];
            cursor.read_exact(&mut comp_tex).unwrap();
            let _decomp_tex = xpress::decompress(&comp_tex, decomp_tex_len).unwrap();
            let comp_mesh_len = cursor.read_u32::<LittleEndian>().unwrap() as usize;
            let decomp_mesh_len = cursor.read_u32::<LittleEndian>().unwrap() as usize;
            let mut comp_mesh = vec![0u8; comp_mesh_len];
            cursor.read_exact(&mut comp_mesh).unwrap();
            let decomp_mesh = xpress::decompress(&comp_mesh, decomp_mesh_len).unwrap();
            let comp_face_len = cursor.read_u32::<LittleEndian>().unwrap() as usize;
            let decomp_face_len = cursor.read_u32::<LittleEndian>().unwrap() as usize;
            let mut comp_face = vec![0u8; comp_face_len];
            cursor.read_exact(&mut comp_face).unwrap();
            let decomp_face = xpress::decompress(&comp_face, decomp_face_len).unwrap();
            return (Vec::new(), decomp_mesh, decomp_face);
        }
        let mut skip = vec![0u8; size as usize];
        cursor.read_exact(&mut skip).unwrap();
    }
}

fn read_vertex_positions(mesh: &[u8], vert_count: usize, stride: usize) -> Vec<[f32; 3]> {
    let mut positions = Vec::new();
    for i in 0..vert_count {
        let offset = i * stride;
        let x = f32::from_le_bytes([mesh[offset], mesh[offset+1], mesh[offset+2], mesh[offset+3]]);
        let y = f32::from_le_bytes([mesh[offset+4], mesh[offset+5], mesh[offset+6], mesh[offset+7]]);
        let z = f32::from_le_bytes([mesh[offset+8], mesh[offset+9], mesh[offset+10], mesh[offset+11]]);
        positions.push([x, y, z]);
    }
    positions
}

fn read_indices(face: &[u8], idx_count: usize) -> Vec<u16> {
    let mut indices = Vec::new();
    for i in 0..idx_count {
        let offset = i * 2;
        indices.push(u16::from_le_bytes([face[offset], face[offset + 1]]));
    }
    indices
}

fn main() {
    let hodor_path = "../testing/ter_centaur/ter_centaur_hodor.hod";
    let gen_path = "../testing/ter_centaur/ter_centaur_generated.hod";

    let (_, hodor_mesh, hodor_face) = extract_pool(hodor_path);
    let (_, gen_mesh, gen_face) = extract_pool(gen_path);

    let stride = 64usize;
    let centaur_vert_count = 3845usize;
    let centaur_idx_count = 3915usize;

    // Read vertex positions
    let hodor_pos = read_vertex_positions(&hodor_mesh, centaur_vert_count, stride);
    let gen_pos = read_vertex_positions(&gen_mesh, centaur_vert_count, stride);

    // Read indices
    let hodor_idx = read_indices(&hodor_face, centaur_idx_count);
    let gen_idx = read_indices(&gen_face, centaur_idx_count);

    // For each triangle, print the vertex positions referenced by the first few triangles
    println!("=== First 10 triangles comparison ===");
    for tri in 0..10 {
        let i0 = hodor_idx[tri * 3] as usize;
        let i1 = hodor_idx[tri * 3 + 1] as usize;
        let i2 = hodor_idx[tri * 3 + 2] as usize;

        let gi0 = gen_idx[tri * 3] as usize;
        let gi1 = gen_idx[tri * 3 + 1] as usize;
        let gi2 = gen_idx[tri * 3 + 2] as usize;

        println!("Triangle {}:", tri);
        println!("  HODOR indices: [{}, {}, {}]", i0, i1, i2);
        println!("  GEN   indices: [{}, {}, {}]", gi0, gi1, gi2);

        let h0 = hodor_pos[i0];
        let h1 = hodor_pos[i1];
        let h2 = hodor_pos[i2];
        let g0 = gen_pos[gi0];
        let g1 = gen_pos[gi1];
        let g2 = gen_pos[gi2];

        println!("  HODOR v0: ({:.6}, {:.6}, {:.6})", h0[0], h0[1], h0[2]);
        println!("  GEN   v0: ({:.6}, {:.6}, {:.6})", g0[0], g0[1], g0[2]);
        println!("  HODOR v1: ({:.6}, {:.6}, {:.6})", h1[0], h1[1], h1[2]);
        println!("  GEN   v1: ({:.6}, {:.6}, {:.6})", g1[0], g1[1], g1[2]);
        println!("  HODOR v2: ({:.6}, {:.6}, {:.6})", h2[0], h2[1], h2[2]);
        println!("  GEN   v2: ({:.6}, {:.6}, {:.6})", g2[0], g2[1], g2[2]);
    }

    // Check if the vertex positions are the same but in different order
    // Build a set of HODOR positions and check how many GEN positions match
    let mut hodor_set: std::collections::HashSet<[u32; 3]> = std::collections::HashSet::new();
    for pos in &hodor_pos {
        hodor_set.insert([pos[0].to_bits(), pos[1].to_bits(), pos[2].to_bits()]);
    }

    let mut gen_set: std::collections::HashSet<[u32; 3]> = std::collections::HashSet::new();
    for pos in &gen_pos {
        gen_set.insert([pos[0].to_bits(), pos[1].to_bits(), pos[2].to_bits()]);
    }

    let only_in_hodor = hodor_set.difference(&gen_set).count();
    let only_in_gen = gen_set.difference(&hodor_set).count();
    let in_both = hodor_set.intersection(&gen_set).count();

    println!("\n=== Position set comparison ===");
    println!("HODOR unique positions: {}", hodor_set.len());
    println!("GEN unique positions: {}", gen_set.len());
    println!("In both: {}", in_both);
    println!("Only in HODOR: {}", only_in_hodor);
    println!("Only in GEN: {}", only_in_gen);
}
