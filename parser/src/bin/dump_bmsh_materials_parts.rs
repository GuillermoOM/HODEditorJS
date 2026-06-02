use hwr_hod_parser::{hod::HODModel, iff::IffChunk};
use std::fs;

fn main() {
    let orig1 = fs::read("../testing/ter_zephyrus/ter_zephyrus_1.0_original.hod").unwrap();
    let model1 = HODModel::parse(&orig1).unwrap();
    println!("Meshes in 1.0:");
    for m in &model1.meshes {
        println!("Mesh: {}", m.name);
        for (i, part) in m.parts.iter().enumerate() {
            println!("  part {}: mat_idx={}", i, part.material_index);
        }
    }
    
    let orig2 = fs::read("../testing/ter_zephyrus/ter_zephyrus_2.0_original.hod").unwrap();
    let model2 = HODModel::parse(&orig2).unwrap();
    println!("Meshes in 2.0:");
    for m in &model2.meshes {
        println!("Mesh: {}", m.name);
        for (i, part) in m.parts.iter().enumerate() {
            println!("  part {}: mat_idx={}", i, part.material_index);
        }
    }
}
