use hwr_hod_parser::{hod::HODModel, iff::IffChunk};
use std::fs;
use std::io::Cursor;

fn main() {
    let orig1 = fs::read("../testing/ter_zephyrus/ter_zephyrus_1.0_original.hod").unwrap();
    let mut cursor = Cursor::new(&orig1);
    while cursor.position() < orig1.len() as u64 {
        if let Ok(chunk) = IffChunk::read_chunk(&mut cursor) {
            find_matt(&chunk);
        } else {
            break;
        }
    }
}

fn find_matt(chunk: &IffChunk) {
    if chunk.id == "MATT" || chunk.id == "STAT" {
        println!("Chunk {}: len={}", chunk.id, chunk.data.len());
        println!("{:02x?}", &chunk.data[..std::cmp::min(128, chunk.data.len())]);
    }
    for c in &chunk.children {
        find_matt(c);
    }
}
