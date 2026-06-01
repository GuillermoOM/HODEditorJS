use hwr_hod_parser::hod::{HODModel, save_edits, generate_v2_from_model};
use std::fs;

fn main() {
    let hod_path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/mod-tools/HODEditorJS/testing/ter_zephyrus/ter_zephyrus_1.0_original.hod";
    
    println!("=== Loading ter_zephyrus_1.0_original.hod ===\n");
    
    let hod_bytes = fs::read(hod_path).expect("Failed to read HOD file");
    let model = HODModel::parse(&hod_bytes).expect("Failed to parse HOD");
    
    println!("\n=== Parse Complete ===");
    println!("Textures: {}", model.textures.len());
    for (i, tex) in model.textures.iter().enumerate() {
        println!("  Texture {}: {} ({}x{}, {})", 
                i, tex.name, tex.width, tex.height, tex.format);
    }
    println!("Materials: {}", model.materials.len());
    for (i, mat) in model.materials.iter().enumerate() {
        println!("  Material {}: {} ({}) -> {:?}", i, mat.name, mat.shader_name, mat.texture_maps);
    }
    
    println!("\n=== Generating HOD 2.0 bytes ===");
    let out_bytes = generate_v2_from_model(&[], &model).expect("Failed");
    let reparsed = HODModel::parse(&out_bytes).expect("Failed to reparse");
    println!("Reparsed Textures: {}", reparsed.textures.len());
    for (i, tex) in reparsed.textures.iter().enumerate() {
        println!("  Texture {}: {} ({}x{}, {})", 
                i, tex.name, tex.width, tex.height, tex.format);
    }
    println!("Reparsed Materials: {}", reparsed.materials.len());
    for (i, mat) in reparsed.materials.iter().enumerate() {
        println!("  Material {}: {} ({}) -> {:?}", i, mat.name, mat.shader_name, mat.texture_maps);
    }
}
