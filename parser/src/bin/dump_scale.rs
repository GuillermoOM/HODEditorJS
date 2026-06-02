use hwr_hod_parser::hod::HODModel;
use std::fs;

fn dump_joints(path: &str) {
    let bytes = fs::read(path).unwrap();
    let m = HODModel::parse(&bytes).unwrap();
    println!("File: {}", path);
    for j in &m.joints {
        if let Some(ref scale) = j.scale {
            println!("  {}: scale: {:.3}, {:.3}, {:.3}", j.name, scale.x, scale.y, scale.z);
        } else {
            println!("  {}: scale: None", j.name);
        }
    }
}

fn main() {
    dump_joints("../testing/ter_zephyrus/ter_zephyrus_1.0_original.hod");
}
