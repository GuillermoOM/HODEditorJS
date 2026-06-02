use hwr_hod_parser::hod::HODModel;
use std::fs;

fn dump_pos(path: &str) {
    let bytes = fs::read(path).unwrap();
    let m = HODModel::parse(&bytes).unwrap();
    println!("File: {}", path);
    for j in &m.joints {
        if let Some(ref p) = j.position {
            println!("  {}: pos: {:.3}, {:.3}, {:.3}", j.name, p.x, p.y, p.z);
        }
    }
}

fn main() {
    dump_pos("../testing/ter_zephyrus/ter_zephyrus_1.0_original.hod");
}
