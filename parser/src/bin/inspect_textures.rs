use hwr_hod_parser::{hod::HODModel, iff::IffChunk};
use std::fs;

fn main() {
    let orig = fs::read("../testing/ter_zephyrus/ter_zephyrus_2.0_original.hod").unwrap();
    let model = HODModel::parse(&orig).unwrap();
    for tex in &model.textures {
        println!("Tex: name='{}', format='{}'", tex.name, tex.format);
    }
}
