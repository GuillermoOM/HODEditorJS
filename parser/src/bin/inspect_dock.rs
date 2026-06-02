use hwr_hod_parser::{hod::HODModel, iff::IffChunk};
use std::fs;

fn get_dock(bytes: &[u8]) -> Option<IffChunk> {
    let model = HODModel::parse(bytes).unwrap();
    for chunk in &model.preserved_chunks {
        if chunk.id == "DTRM" {
            for child in &chunk.children {
                if child.id == "DOCK" {
                    return Some(child.clone());
                }
            }
        }
    }
    None
}

fn main() {
    let orig = fs::read("../testing/ter_zephyrus/ter_zephyrus_2.0_original.hod").unwrap();
    let orig_dock = get_dock(&orig).expect("No DOCK in orig");
    let orig_data = &orig_dock.data;
    println!("Orig DOCK len={}", orig_data.len());
    println!("Orig bytes 0-16: {:?}", &orig_data[0..std::cmp::min(16, orig_data.len())]);
    
    // Parse it manually to see
    use byteorder::{LittleEndian, ReadBytesExt};
    use std::io::Cursor;
    let mut r = Cursor::new(orig_data);
    let v1 = r.read_u32::<LittleEndian>().unwrap();
    println!("v1 = {}", v1);
    if v1 >= 10 && orig_data.len() > 8 {
        let v2 = r.read_u32::<LittleEndian>().unwrap();
        println!("v2 = {}", v2);
    }
}
