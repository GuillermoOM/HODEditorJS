use std::env;
use std::fs;
use byteorder::{ReadBytesExt, LittleEndian, BigEndian};
use std::io::{Cursor, Read, Seek, SeekFrom};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 { return; }
    
    for arg in args.iter().skip(1) {
        println!("File: {}", arg);
        let bytes = fs::read(arg).unwrap();
        let mut pos = 0;
        while pos < bytes.len() - 8 {
            if &bytes[pos..pos+4] == b"HIER" {
                let mut cursor = Cursor::new(&bytes[pos+4..]);
                let size = cursor.read_u32::<BigEndian>().unwrap_or(0);
                println!("  Found HIER size {}", size);
                let dump_len = std::cmp::min(size as usize, 128);
                for chunk in bytes[pos+8..pos+8+dump_len].chunks(16) {
                    print!("    ");
                    for b in chunk { print!("{:02x} ", b); }
                    println!();
                }
            }
            pos += 1;
        }
    }
}
