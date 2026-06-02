use hwr_hod_parser::hod::HODModel;
use std::fs;

fn main() {
    let bytes = fs::read("../testing/ter_zephyrus/ter_zephyrus_1.0_original.hod").unwrap();
    let model = HODModel::parse(&bytes).unwrap();
    for mat in &model.materials {
        println!("Material: {} ({})", mat.name, mat.shader_name);
        for tex in &mat.texture_maps {
            println!("  Texture map: {}", tex);
        }
    }
    for tex in &model.textures {
        println!("Texture: {}", tex.name);
    }
}
