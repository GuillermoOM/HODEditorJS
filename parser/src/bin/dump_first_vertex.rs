use hwr_hod_parser::hod::HODModel;
use std::fs;

fn dump_v(path: &str) {
    let bytes = fs::read(path).unwrap();
    let m = HODModel::parse(&bytes).unwrap();
    if let Some(mesh) = m.meshes.first() {
        if let Some(part) = mesh.parts.first() {
            println!("{}: v0 = {:?}", path, part.vertices.first().map(|v| (v.position.x, v.position.y, v.position.z)));
        }
    }
}

fn main() {
    dump_v("../testing/ter_zephyrus/ter_zephyrus_1.0_original.hod");
    dump_v("../testing/ter_zephyrus/ter_zephyrus_2.0_original.hod");
}
