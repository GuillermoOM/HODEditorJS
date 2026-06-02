use hwr_hod_parser::{hod::HODModel, iff::IffChunk};
use std::fs;

fn main() {
    let orig2 = fs::read("../testing/ter_zephyrus/ter_zephyrus_2.0_original.hod").unwrap();
    let model2 = HODModel::parse(&orig2).unwrap();
    println!("Materials in 2.0:");
    for (i, m) in model2.materials.iter().enumerate() {
        println!("  {}: {}", i, m.name);
    }
}
