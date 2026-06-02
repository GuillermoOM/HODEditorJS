use hwr_hod_parser::iff::parse_iff;
use std::fs;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;
use hwr_hod_parser::hod::read_len_string;

fn main() {
    let bytes1 = fs::read("../testing/ter_zephyrus/ter_zephyrus_1.0_original.hod").unwrap();
    let iff1 = parse_iff(&bytes1).unwrap();
    
    println!("1.0 TEXM OFFSETS:");
    for c in &iff1.chunks[0].children {
        if c.id == "TEXM" {
            let mut r = Cursor::new(&c.data);
            let name = read_len_string(&mut r).unwrap();
            r.set_position(r.position() + 16);
            let pool_offset = r.read_u32::<LittleEndian>().unwrap();
            let pool_size = r.read_u32::<LittleEndian>().unwrap();
            println!("  {}: offset={}, size={}", name, pool_offset, pool_size);
        }
    }

    let bytes2 = fs::read("test_out.hod").unwrap();
    let iff2 = parse_iff(&bytes2).unwrap();
    
    println!("\n2.0 LMIP OFFSETS:");
    for c in &iff2.chunks[0].children {
        if c.id == "LMIP" {
            let mut r = Cursor::new(&c.data);
            let name = read_len_string(&mut r).unwrap();
            r.set_position(r.position() + 16);
            let pool_offset = r.read_u32::<LittleEndian>().unwrap();
            let pool_size = r.read_u32::<LittleEndian>().unwrap();
            println!("  {}: offset={}, size={}", name, pool_offset, pool_size);
        }
    }
}
