use std::fs;
use hwr_hod_parser::hod;

fn main() {
    for file in ["ter_zephyrus_hodor.hod", "ter_zephyrus_1.0.hod", "ter_zephyrus_edited.hod"].iter() {
        let path = format!("/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/mod-tools/HODEditorJS/testing/ter_zephyrus/{}", file);
        let bytes = fs::read(&path).unwrap();
        let model = hod::HODModel::parse_with_external(&bytes, None, None).unwrap();
        println!("File: {}", file);
        if let Some(mesh) = model.meshes.first() {
            println!("  Mesh {} Vertices:", mesh.name);
            if let Some(part) = mesh.parts.first() {
                for i in 0..std::cmp::min(3, part.vertices.len()) {
                    let v = &part.vertices[i];
                    println!("    v{}: ({:?}, {:?}, {:?})", i, v.position.x, v.position.y, v.position.z);
                }
            }
        }
    }
}
