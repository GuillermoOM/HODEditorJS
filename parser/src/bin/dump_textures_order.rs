use hwr_hod_parser::hod::HODModel;
use std::fs;

fn main() {
    let bytes1 = fs::read("../testing/ter_zephyrus/ter_zephyrus_1.0_original.hod").unwrap();
    let m1 = HODModel::parse(&bytes1).unwrap();
    
    let bytes2 = fs::read("../testing/ter_zephyrus/ter_zephyrus_from_1.0_to_2.0.hod").unwrap();
    let m2 = HODModel::parse(&bytes2).unwrap();

    println!("--- ORIGINAL TEXTURES ---");
    for (i, t) in m1.textures.iter().enumerate() {
        println!("{}: {}", i, t.name);
    }

    println!("\n--- EDITED TEXTURES ---");
    for (i, t) in m2.textures.iter().enumerate() {
        println!("{}: {}", i, t.name);
    }
}
