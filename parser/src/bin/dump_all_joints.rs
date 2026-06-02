use hwr_hod_parser::hod::HODModel;
use std::fs;

fn dump_joints(path: &str) {
    let bytes = fs::read(path).unwrap();
    let m = HODModel::parse(&bytes).unwrap();
    println!("File: {}", path);
    for j in &m.joints {
        println!("  {}: lt: {:?}", j.name, j.local_transform);
    }
}

fn main() {
    dump_joints("../testing/ter_zephyrus/ter_zephyrus_1.0_original.hod");
    println!("-------------------");
    dump_joints("../testing/ter_zephyrus/ter_zephyrus_2.0_original.hod");
}
