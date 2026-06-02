use hwr_hod_parser::hod::HODModel;
use std::fs;

fn dump_root(path: &str) {
    let bytes = fs::read(path).unwrap();
    let m = HODModel::parse(&bytes).unwrap();
    let root = &m.joints[0];
    println!("File: {}", path);
    println!("  Root: {}", root.name);
    println!("  local_transform: {:?}", root.local_transform);
}

fn main() {
    dump_root("../testing/ter_zephyrus/ter_zephyrus_1.0_original.hod");
    dump_root("../testing/ter_zephyrus/ter_zephyrus_2.0_original.hod");
}
