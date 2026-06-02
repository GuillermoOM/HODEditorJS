use hwr_hod_parser::iff::{parse_iff, ChunkType, IffChunk};
use std::fs;

fn process(chunk: &IffChunk) {
    if chunk.id == "BMSH" {
        if chunk.data.len() > 12 {
            let num_parts = u32::from_le_bytes([chunk.data[8], chunk.data[9], chunk.data[10], chunk.data[11]]);
            println!("BMSH parts: {}", num_parts);
        }
    }
    for child in &chunk.children {
        process(child);
    }
}

fn main() {
    let bytes = fs::read("../testing/ter_zephyrus/ter_zephyrus_1.0_original.hod").unwrap();
    let iff = parse_iff(&bytes).unwrap();
    for c in iff.chunks {
        process(&c);
    }
}
