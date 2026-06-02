use hwr_hod_parser::hod::HODModel;
use std::fs;

fn main() {
    let path = "../testing/ter_zephyrus/ter_zephyrus_1.0_original.hod";
    let bytes = fs::read(path).unwrap();
    let model = HODModel::parse(&bytes).unwrap();
    println!("Textures length: {}", model.textures.len());
    for (i, t) in model.textures.iter().enumerate() {
        println!("Texture {}: {}", i, t.name);
    }
}
