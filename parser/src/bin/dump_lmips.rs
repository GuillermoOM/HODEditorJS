use byteorder::{LittleEndian, ReadBytesExt};
use hwr_hod_parser::iff::IffChunk;
use std::fs;
use std::io::Cursor;

fn process(chunk: &IffChunk) {
    if chunk.id == "LMIP" {
        println!("LMIP Chunk! Data len: {}", chunk.data.len());
    }
    for child in &chunk.children {
        process(child);
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
