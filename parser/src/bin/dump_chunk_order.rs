use hwr_hod_parser::iff::IffChunk;
use std::fs;
use std::io::Cursor;

fn process(chunk: &IffChunk, depth: usize) {
    if chunk.id == "HVMD" {
        for c in &chunk.children {
            println!("{}HVMD Child: {}", " ".repeat(depth), c.id);
        }
    }
    for c in &chunk.children {
        process(c, depth + 2);
    }
}

fn main() {
    let bytes = fs::read("../testing/ter_zephyrus/ter_zephyrus_1.0_original.hod").unwrap();
    let mut cursor = Cursor::new(bytes.as_slice());
    while cursor.position() < bytes.len() as u64 {
        if let Ok(chunk) = IffChunk::read_chunk(&mut cursor) {
            process(&chunk, 0);
        } else { break; }
    }
}
