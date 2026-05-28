use std::fs;
use hwr_hod_parser::iff::IffParser;
use hwr_hod_parser::xpress;
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::HashSet;

fn decompress_mesh_pool(bytes: &[u8]) -> Vec<u8> {
    let chunks = hwr_hod_parser::iff::parse_iff(bytes).unwrap();
    let pool_chunk = chunks.iter().find(|c| c.id == "POOL").unwrap();
    let mut pool_cursor = Cursor::new(&pool_chunk.data);
    let _pool_type = pool_cursor.read_u32::<LittleEndian>().unwrap();
    let comp_tex_len = pool_cursor.read_u32::<LittleEndian>().unwrap();
    let _decomp_tex_len = pool_cursor.read_u32::<LittleEndian>().unwrap();
    pool_cursor.set_position(pool_cursor.position() + comp_tex_len as u64);
    let comp_mesh_len = pool_cursor.read_u32::<LittleEndian>().unwrap();
    let decomp_mesh_len = pool_cursor.read_u32::<LittleEndian>().unwrap();
    let mut comp_mesh = vec![0u8; comp_mesh_len as usize];
    std::io::Read::read_exact(&mut pool_cursor, &mut comp_mesh).unwrap();
    xpress::decompress(&comp_mesh, decomp_mesh_len as usize).unwrap()
}

fn main() {
    let orig_path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/pebble/pebble_0/pebble_0_original.hod";
    let gen_path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/pebble/pebble_0/pebble_0.hod";
    
    let orig_mesh = decompress_mesh_pool(&fs::read(orig_path).unwrap());
    let gen_mesh = decompress_mesh_pool(&fs::read(gen_path).unwrap());
    
    let mut orig_unique = HashSet::new();
    for i in (0..orig_mesh.len()).step_by(64) {
        if i + 64 <= orig_mesh.len() {
            orig_unique.insert(&orig_mesh[i..i+64]);
        }
    }
    
    let mut gen_unique = HashSet::new();
    for i in (0..gen_mesh.len()).step_by(64) {
        if i + 64 <= gen_mesh.len() {
            gen_unique.insert(&gen_mesh[i..i+64]);
        }
    }
    
    println!("Orig mesh vertices: {}, unique: {}", orig_mesh.len() / 64, orig_unique.len());
    println!("Gen mesh vertices: {}, unique: {}", gen_mesh.len() / 64, gen_unique.len());
}
