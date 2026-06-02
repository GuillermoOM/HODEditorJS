use hwr_hod_parser::hod::{generate_v2_from_model, HODModel};
use std::fs;

fn main() {
    let path = "../testing/ter_zephyrus/ter_zephyrus_1.0_original.hod";
    let bytes = fs::read(path).unwrap();
    let model = HODModel::parse(&bytes).unwrap();
    let out_bytes = generate_v2_from_model(&bytes, &model).unwrap();
    fs::write("test_out.hod", out_bytes).unwrap();
}
