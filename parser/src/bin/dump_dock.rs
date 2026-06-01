use byteorder::{LittleEndian, ReadBytesExt};
use hwr_hod_parser::iff::IffChunk;
use std::fs::File;
use std::io::Cursor;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let path = if args.len() > 1 {
        &args[1]
    } else {
        "asteroid_3.hod"
    };
    let mut f = File::open(path)?;
    while let Ok(root) = IffChunk::read_chunk(&mut f) {
        extract_dock(&root);
    }
    Ok(())
}

fn extract_dock(chunk: &IffChunk) {
    if chunk.id.trim() == "DOCK" {
        let mut r = Cursor::new(&chunk.data);
        if let Ok(first) = r.read_u32::<LittleEndian>() {
            let mut count = first;
            if first >= 10 && chunk.data.len() > 8 {
                count = r.read_u32::<LittleEndian>().unwrap_or(0);
            }
            println!("Found DOCK with {} paths", count);
            for i in 0..count {
                if let Ok(name) = hwr_hod_parser::hod::read_len_string(&mut r) {
                    if let Ok(parent) = hwr_hod_parser::hod::read_len_string(&mut r) {
                        println!("  Dockpath name: {}", name);
                        println!("  Dockpath parent: {}", parent);
                    }
                } else if let Ok(name2) = hwr_hod_parser::hod::read_len_string(&mut r) {
                    r.set_position(r.position() + 8); // padding
                    if let Ok(pts) = r.read_i32::<LittleEndian>() {
                        r.set_position(r.position() + pts as u64 * 50);
                    }
                }
            }
        }
    }
    for child in &chunk.children {
        extract_dock(child);
    }
}
