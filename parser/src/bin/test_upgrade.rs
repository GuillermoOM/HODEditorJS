use hwr_hod_parser::hod::HODModel;
use std::fs;

fn main() {
    let bytes = fs::read("../testing/ter_zephyrus/ter_zephyrus_1.0_original.hod").unwrap();
    let m = HODModel::parse(&bytes).unwrap();
    println!("File parsed and upgraded!");
    println!("is_v2: {}", m.is_v2);
    println!("Root position: {:?}", m.joints.iter().find(|j| j.name == "Root").unwrap().position);
    println!("Root scale: {:?}", m.joints.iter().find(|j| j.name == "Root").unwrap().scale);
    println!("Weapon_01_Position: {:?}", m.joints.iter().find(|j| j.name == "Weapon_01_Position").unwrap().position);
    println!("First vertex x: {:?}", m.meshes[0].parts[0].vertices[0].position.x);
}
