use std::fs::File;
use std::io::Read;
use hwr_hod_parser::hod::HODModel;

fn main() {
    let path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/mod-tools/HODEditorJS/testing/ter_fenris/ter_fenris_1.3_original.hod";
    let mut file = File::open(path).expect("Failed to open HOD");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("Failed to read");
    
    let hod = HODModel::parse(&buffer).expect("Failed to parse");
    
    println!("Textures:");
    for tex in hod.textures {
        println!(" - {}", tex.name);
    }
    
    println!("\nMaterials:");
    for mat in hod.materials {
        println!(" - {}: shader={}, maps={:?}", mat.name, mat.shader_name, mat.texture_maps);
    }
}
