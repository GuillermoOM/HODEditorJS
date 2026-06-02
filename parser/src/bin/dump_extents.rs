use hwr_hod_parser::hod::HODModel;
use std::fs;

fn get_extents(m: &HODModel) -> (f32, f32) {
    let mut min_z = f32::MAX;
    let mut max_z = f32::MIN;
    for mesh in &m.meshes {
        for p in &mesh.parts {
            for v in &p.vertices {
                if v.position.z < min_z { min_z = v.position.z; }
                if v.position.z > max_z { max_z = v.position.z; }
            }
        }
    }
    (min_z, max_z)
}

fn main() {
    let bytes1 = fs::read("../testing/ter_zephyrus/ter_zephyrus_1.0_original.hod").unwrap();
    let m1 = HODModel::parse(&bytes1).unwrap();
    let (min1, max1) = get_extents(&m1);
    println!("1.0 Upgraded Extents Z: {} to {}", min1, max1);
    println!("1.0 Original Extents Z: {} to {}", min1*100.0, max1*100.0);
    
    let bytes2 = fs::read("../testing/ter_zephyrus/ter_zephyrus_2.0_original.hod").unwrap();
    let m2 = HODModel::parse(&bytes2).unwrap();
    let (min2, max2) = get_extents(&m2);
    println!("2.0 Extents Z: {} to {}", min2, max2);
}
