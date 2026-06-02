use hwr_hod_parser::{hod::HODModel, iff::IffChunk};
use std::fs;
use std::io::Cursor;

fn main() {
    let orig1 = fs::read("../testing/ter_zephyrus/ter_zephyrus_1.0_original.hod").unwrap();
    let model1 = HODModel::parse(&orig1).unwrap();
    println!("Materials:");
    for (i, m) in model1.materials.iter().enumerate() {
        println!("  {}: {}", i, m.name);
    }
    println!("Meshes:");
    for m in &model1.meshes {
        if let Some(part) = m.parts.first() {
            println!("  {}: mat_idx={}", m.name, part.material_index);
        }
    }
}
