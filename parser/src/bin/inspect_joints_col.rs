use hwr_hod_parser::hod::HODModel;
use std::fs;

fn main() {
    let orig1 = fs::read("../testing/ter_zephyrus/ter_zephyrus_1.0_original.hod").unwrap();
    let model1 = HODModel::parse(&orig1).unwrap();
    println!("1.0 Joints:");
    for j in &model1.joints {
        if j.name.contains("COL") {
            println!("  {}", j.name);
        }
    }
    
    let orig2 = fs::read("../testing/ter_zephyrus/ter_zephyrus_2.0_original.hod").unwrap();
    let model2 = HODModel::parse(&orig2).unwrap();
    println!("2.0 Joints:");
    for j in &model2.joints {
        if j.name.contains("COL") {
            println!("  {}", j.name);
        }
    }
}
