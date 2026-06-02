use hwr_hod_parser::iff::IffChunk;
use std::fs;
use std::io::{Cursor, Read};
use byteorder::{LittleEndian, ReadBytesExt};

fn process(chunk: &IffChunk) {
    if chunk.id == "LMIP" || chunk.id == "TEXM" {
        let mut r = Cursor::new(&chunk.data);
        let mut name = String::new();
        let len = r.read_u32::<LittleEndian>().unwrap() as usize;
        let mut bytes = vec![0; len];
        r.read_exact(&mut bytes).unwrap();
        name = String::from_utf8_lossy(&bytes).into_owned();
        
        let pool_offset = r.read_u32::<LittleEndian>().unwrap();
        let pool_size = r.read_u32::<LittleEndian>().unwrap();
        println!("{} Chunk: {} offset={}, size={}", chunk.id, name, pool_offset, pool_size);
    }
    for c in &chunk.children {
        process(c);
    }
}

fn main() {
    let bytes = fs::read("test_out.hod").unwrap();
    let mut cursor = Cursor::new(bytes.as_slice());
    while cursor.position() < bytes.len() as u64 {
        if let Ok(chunk) = IffChunk::read_chunk(&mut cursor) {
            process(&chunk);
        }
    }
}
