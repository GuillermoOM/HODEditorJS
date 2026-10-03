use byteorder::{LittleEndian, ReadBytesExt};
use hwr_hod_parser::iff::IffChunk;
use std::fs::File;
use std::io::{Cursor, Read};

fn main() {
    let path = std::env::args().nth(1).expect("usage: dump_texpool <file.hod>");
    let mut file = File::open(&path).expect("open");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("read");
    let mut cursor = Cursor::new(&buffer[..]);
    let mut pool_data: Vec<u8> = Vec::new();
    while (cursor.position() as usize) < buffer.len() {
        let chunk = IffChunk::read_chunk(&mut cursor).expect("chunk");
        if chunk.id == "POOL" {
            pool_data = chunk.data;
        }
    }
    let mut pc = Cursor::new(&pool_data[..]);
    let _pool_type = pc.read_u32::<LittleEndian>().unwrap();
    let comp_tex_len = pc.read_u32::<LittleEndian>().unwrap() as usize;
    let decomp_tex_len = pc.read_u32::<LittleEndian>().unwrap() as usize;
    println!("pool_type={} comp_tex={} decomp_tex={}", _pool_type, comp_tex_len, decomp_tex_len);
    let mut comp_tex = vec![0u8; comp_tex_len];
    pc.read_exact(&mut comp_tex).unwrap();
    let decomp_tex: Vec<u8> = if comp_tex_len == decomp_tex_len {
        comp_tex
    } else {
        hwr_hod_parser::xpress::decompress(&comp_tex, decomp_tex_len).expect("xpress")
    };
    println!("decomp texture pool len={}", decomp_tex.len());
    std::fs::write("/tmp/opencode/texpool.bin", &decomp_tex).unwrap();
    println!("wrote /tmp/opencode/texpool.bin");
    for i in 0..16.min((decomp_tex.len() + 15) / 16) {
        let off = i * 16;
        let end = (off + 16).min(decomp_tex.len());
        let hex: String = decomp_tex[off..end].iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ");
        println!("{:06x}: {}", off, hex);
    }
}

#[allow(dead_code)]
fn _unused() {}
