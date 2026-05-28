use std::fs;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;
use hwr_hod_parser::iff::IffParser;

fn main() {
    let orig_path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/pebble/pebble_0/pebble_0_original.hod";
    let bytes = fs::read(orig_path).unwrap();
    let mut parser = IffParser::new(&bytes);
    let chunks = parser.parse_all().unwrap();
    let pool = chunks.iter().find(|c| c.id == "POOL").unwrap();
    
    let mut cursor = Cursor::new(&pool.data);
    let _pool_type = cursor.read_u32::<LittleEndian>().unwrap();
    let comp_tex_len = cursor.read_u32::<LittleEndian>().unwrap();
    let _decomp_tex_len = cursor.read_u32::<LittleEndian>().unwrap();
    
    let mut comp_tex = vec![0u8; comp_tex_len as usize];
    std::io::Read::read_exact(&mut cursor, &mut comp_tex).unwrap();
    
    // MS Xpress LZ77 has no standard chunk header unless it's framed. 
    // Is it framed?
    println!("First 16 bytes of comp_tex: {:?}", &comp_tex[0..16]);
}
