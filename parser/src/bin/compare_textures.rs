use hwr_hod_parser::hod::HODModel;
use std::fs;

fn main() {
    let bytes1 = fs::read("../testing/ter_zephyrus/ter_zephyrus_1.0_original.hod").unwrap();
    let model1 = HODModel::parse(&bytes1).unwrap();

    let bytes2 = fs::read("../testing/ter_zephyrus/ter_zephyrus_2.0_original.hod").unwrap();
    let model2 = HODModel::parse(&bytes2).unwrap();

    println!("1.0 TEXTURES:");
    for (i, t) in model1.textures.iter().enumerate() {
        println!("  {}: {}", i, t.name);
    }
    println!("\n2.0 TEXTURES:");
    for (i, t) in model2.textures.iter().enumerate() {
        println!("  {}: {}", i, t.name);
    }
}
