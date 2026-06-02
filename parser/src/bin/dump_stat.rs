use byteorder::{LittleEndian, ReadBytesExt};
use hwr_hod_parser::iff::IffChunk;
use std::fs;
use std::io::Cursor;

fn read_len_string(r: &mut Cursor<&[u8]>) -> String {
    let len = r.read_u32::<LittleEndian>().unwrap() as usize;
    let mut buf = vec![0u8; len];
    std::io::Read::read_exact(r, &mut buf).unwrap();
    String::from_utf8_lossy(&buf).to_string()
}

fn process(chunk: &IffChunk) {
    if chunk.id == "STAT" || chunk.id == "MATT" {
        println!("{} Chunk Data: {:?}", chunk.id, chunk.data);
        let mut r = Cursor::new(chunk.data.as_slice());
        println!("Material name: {}", read_len_string(&mut r));
        println!("Shader name: {}", read_len_string(&mut r));
        let param_count = r.read_u32::<LittleEndian>().unwrap();
        println!("Param count: {}", param_count);
        if param_count > 0 {
            println!("Extra1: {}", r.read_u32::<LittleEndian>().unwrap());
            println!("Extra2: {}", r.read_u32::<LittleEndian>().unwrap());
            for i in 0..param_count {
                if i == 0 {
                    println!("Index 0: {}", r.read_u32::<LittleEndian>().unwrap());
                } else {
                    println!("Extra3?: {}", r.read_u32::<LittleEndian>().unwrap());
                    println!("Extra4?: {}", r.read_u32::<LittleEndian>().unwrap());
                    println!("Index {}?: {}", i, r.read_u32::<LittleEndian>().unwrap());
                }
                println!("Param name?: {}", read_len_string(&mut r));
            }
        }
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
