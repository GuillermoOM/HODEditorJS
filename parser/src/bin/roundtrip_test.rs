use hwr_hod_parser::hod::{save_edits, HODModel};
use std::fs;

fn summarize(tag: &str, m: &HODModel) {
    println!("[{}] tex={} mats={}", tag, m.textures.len(), m.materials.len());
    for t in &m.textures {
        println!("   tex '{}' {}x{} {}", t.name, t.width, t.height, t.format);
    }
    for mat in &m.materials {
        println!("   mat '{}' [{}] maps={:?}", mat.name, mat.shader_name, mat.texture_maps);
        for prm in &mat.params {
            println!("      param type={} name='{}' value_len={}", prm.param_type, prm.name, prm.value.len());
        }
    }
}

fn main() {
    let path = std::env::args().nth(1).expect("usage: roundtrip_test <file.hod>");
    let bytes = fs::read(&path).unwrap();
    let model = HODModel::parse(&bytes).unwrap();
    summarize("loaded", &model);
    let out = save_edits(&bytes, &model).unwrap();
    println!("saved {} -> {} bytes", bytes.len(), out.len());
    let m2 = HODModel::parse(&out).unwrap();
    summarize("reloaded", &m2);
}
