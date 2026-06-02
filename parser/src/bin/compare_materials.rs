use hwr_hod_parser::hod::HODModel;
use std::fs;

fn main() {
    let bytes1 = fs::read("../testing/ter_zephyrus/ter_zephyrus_1.0_original.hod").unwrap();
    let model1 = HODModel::parse(&bytes1).unwrap();

    let bytes2 = fs::read("test_out.hod").unwrap();
    let model2 = HODModel::parse(&bytes2).unwrap();

    println!("ORIGINAL MATERIALS:");
    for (i, m) in model1.materials.iter().enumerate() {
        println!("  {}: {} -> {:?}", i, m.name, m.texture_maps);
    }
    println!("GENERATED MATERIALS:");
    for (i, m) in model2.materials.iter().enumerate() {
        println!("  {}: {} -> {:?}", i, m.name, m.texture_maps);
    }

    println!("\nORIGINAL MESHES:");
    for mesh in &model1.meshes {
        print!("  {}: ", mesh.name);
        for part in &mesh.parts {
            print!("{} ", part.material_index);
        }
        println!();
    }
    println!("GENERATED MESHES:");
    for mesh in &model2.meshes {
        print!("  {}: ", mesh.name);
        for part in &mesh.parts {
            print!("{} ", part.material_index);
        }
        println!();
    }
}
