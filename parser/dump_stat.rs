use hwr_hod_parser::hod::{HODModel};
fn main() {
    for filename in &["../testing/ter_centaur/ter_centaur_hodor.hod", "../testing/ter_centaur/ter_centaur_from_dae.hod"] {
        let bytes = std::fs::read(filename).unwrap();
        let model = HODModel::parse(&bytes).unwrap();
        println!("{} Materials: {}", filename, model.materials.len());
        for (i, m) in model.materials.iter().enumerate() {
            println!("  Mat {}: name='{}' shader='{}' textures={:?}", i, m.name, m.shader_name, m.texture_maps);
        }
    }
}
