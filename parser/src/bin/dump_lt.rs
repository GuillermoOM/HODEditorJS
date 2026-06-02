use hwr_hod_parser::hod::HODModel;
use std::fs;

fn dump_lt(path: &str) {
    let bytes = fs::read(path).unwrap();
    let m = HODModel::parse(&bytes).unwrap();
    println!("File: {}", path);
    for j in m.joints.iter().take(2) {
        println!("  {}: lt: {:?}", j.name, j.local_transform.m);
    }
}

fn main() {
    dump_lt("../testing/ter_zephyrus/ter_zephyrus_1.0_original.hod");
    println!("-------------------");
    dump_lt("../testing/ter_zephyrus/ter_zephyrus_2.0_original.hod");
}
