use hwr_hod_parser::hod::{HODModel, generate_v2_from_model};
use std::fs;

fn main() {
    let original_bytes = fs::read("../testing/ter_zephyrus/ter_zephyrus_1.0_original.hod").unwrap();
    let mut model = HODModel::parse(&original_bytes).unwrap();
    model.auto_assign_and_resize_textures();
    let new_bytes = generate_v2_from_model(&original_bytes, &model).unwrap();
    fs::write("../testing/ter_zephyrus/ter_zephyrus_from_1.0_to_2.0_FIXED.hod", new_bytes).unwrap();
    
    let m2 = HODModel::parse(&fs::read("../testing/ter_zephyrus/ter_zephyrus_from_1.0_to_2.0_FIXED.hod").unwrap()).unwrap();
    println!("--- FIXED EDITED ---");
    for (i, mat) in m2.materials.iter().enumerate() {
        println!("Material {}: {} ({}) -> {:?}", i, mat.name, mat.shader_name, mat.texture_maps);
    }
}
