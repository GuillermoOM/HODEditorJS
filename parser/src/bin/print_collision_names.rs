use hwr_hod_parser::hod;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    let hod_path = &args[1];
    let bytes = fs::read(hod_path).unwrap();
    let model = hod::HODModel::parse(&bytes).unwrap();
    println!("File: {}", hod_path);
    for cm in &model.collision_meshes {
        println!("Collision mesh name: '{}'", cm.name);
    }
}
