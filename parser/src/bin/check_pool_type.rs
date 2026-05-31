use std::fs;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;
use hwr_hod_parser::iff::IffChunk;

fn check_pool(path: &str) {
    let bytes = fs::read(path).unwrap();
    let mut cursor = Cursor::new(bytes);
    while cursor.position() < cursor.get_ref().len() as u64 {
        if let Ok(chunk) = IffChunk::read_chunk(&mut cursor) {
            if chunk.id == "POOL" {
                let mut p = Cursor::new(&chunk.data);
                let pool_type = p.read_u32::<LittleEndian>().unwrap();
                println!("{}: POOL type = {} (0x{:X})", path, pool_type, pool_type);
            }
        }
    }
}

fn main() {
    check_pool("../testing/ter_centaur/ter_centaur_hodor.hod");
    check_pool("../../uncompressed_bigs/HWRM/pebble/pebble_0/pebble_0.hod");
}
