use std::env;
use std::fs;
use hwr_hod_parser::hod::HODModel;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/uncompressed_bigs/HWRM/pebble/pebble_0/pebble_0.hod";
    let bytes = fs::read(path)?;
    let model = HODModel::parse(&bytes)?;
    
    println!("Parsed HOD: {}", path);
    println!("Meshes: {}", model.meshes.len());
    for (i, mesh) in model.meshes.iter().enumerate() {
        println!("Mesh {}: {}", i, mesh.name);
        for (j, part) in mesh.parts.iter().enumerate() {
            println!("  Part {}: Vertices: {}, Faces: {}", j, part.vertices.len(), part.faces.len());
        }
    }
    
    Ok(())
}
