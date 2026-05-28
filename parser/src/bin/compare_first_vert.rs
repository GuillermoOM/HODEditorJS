use std::fs;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;
use hwr_hod_parser::iff::IffParser;
use hwr_hod_parser::xpress;

fn decompress_mesh_pool(bytes: &[u8]) -> Vec<u8> {
    let mut parser = IffParser::new(bytes);
    let chunks = parser.parse_all().unwrap();
    
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let orig_path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/pebble/pebble_0/pebble_0_original.hod";
    let gen_path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/pebble/pebble_0/pebble_0.hod";
    
    let orig_bytes = fs::read(orig_path)?;
    let gen_bytes = fs::read(gen_path)?;
    
    let orig_mesh = decompress_mesh_pool(&orig_bytes);
    let gen_mesh = decompress_mesh_pool(&gen_bytes);
    
    println!("Orig mesh pool decompressed size: {}", orig_mesh.len());
    println!("Gen mesh pool decompressed size: {}", gen_mesh.len());
    
    println!("\nFirst Vertex (64 bytes):");
    println!("OFFSET | ORIGINAL                           | GENERATED                          | DIFF?");
    for i in 0..64 {
        let o = orig_mesh[i];
        let g = gen_mesh[i];
        let diff = if o != g { "<-- DIFF" } else { "" };
        println!("{:04X}   | {:02X}                                 | {:02X}                                 | {}", i, o, g, diff);
    }
    
    Ok(())
}
