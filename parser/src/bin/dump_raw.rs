use hwr_hod_parser::iff::IffChunk;
use std::fs::File;
use std::io::{Cursor, Read};

fn main() {
    let path = std::env::args().nth(1).expect("usage: dump_raw <file.hod> [chunk_id]");
    let filter = std::env::args().nth(2);
    let mut file = File::open(&path).expect("Failed to open HOD");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("Failed to read");

    let mut cursor = Cursor::new(&buffer);
    let mut chunks = Vec::new();
    while (cursor.position() as usize) < buffer.len() {
        match IffChunk::read_chunk(&mut cursor) {
            Ok(c) => chunks.push(c),
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
        }
    }

    fn walk<'a>(c: &'a IffChunk, out: &mut Vec<&'a IffChunk>) {
        out.push(c);
        for ch in &c.children {
            walk(ch, out);
        }
    }
    let mut all = Vec::new();
    for c in &chunks {
        walk(c, &mut all);
    }

    for c in &all {
        if let Some(ref f) = filter {
            if c.id.trim() != f.as_str() {
                continue;
            }
        }
        println!("\n===== '{}' type={:?} data_len={} children={} =====", c.id, c.chunk_type, c.data.len(), c.children.len());
        if c.data.is_empty() {
            continue;
        }
        let n = c.data.len().min(256);
        for (row, slice) in c.data[..n].chunks(16).enumerate() {
            let hex: Vec<String> = slice.iter().map(|b| format!("{:02x}", b)).collect();
            let ascii: String = slice
                .iter()
                .map(|&b| if b >= 32 && b <= 126 { b as char } else { '.' })
                .collect();
            println!("  {:04x}: {:<47} {}", row * 16, hex.join(" "), ascii);
        }
        if c.data.len() > 256 {
            println!("  ... ({} more bytes)", c.data.len() - 256);
        }
    }
}
