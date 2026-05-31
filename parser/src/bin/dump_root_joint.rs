use hwr_hod_parser::hod::HODModel;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let bytes = std::fs::read(&args[1]).unwrap();
    let model = HODModel::parse(&bytes).unwrap();
    if let Some(root) = model.joints.first() {
        println!("Root Joint: {}", root.name);
        println!("Pos: {:?}", root.position);
        println!("Rot: {:?}", root.rotation);
        println!("Scl: {:?}", root.scale);
        println!("Transform:");
        for row in &root.local_transform.m {
            println!("  {:?}", row);
        }
    }
}
