use hwr_hod_parser::hod::HODModel;
use std::fs;

fn main() {
    let bytes1 = fs::read("../testing/ter_zephyrus/ter_zephyrus_1.0_original.hod").unwrap();
    let m1 = HODModel::parse(&bytes1).unwrap();
    println!("1.0 Materials count: {}", m1.materials.len());
    for mat in &m1.materials {
        println!("  - {}", mat.name);
    }
}
