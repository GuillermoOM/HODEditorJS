use hwr_hod_parser::dae::parse_dae;
use hwr_hod_parser::hod::{generate_v2_from_model, HODModel};
use std::fs;

fn main() {
    let dae_path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/mod-tools/HODEditorJS/testing/ter_centaur/ter_centaur.DAE";
    let out_path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/mod-tools/HODEditorJS/testing/ter_centaur/ter_centaur_from_dae_fixed.hod";

    println!("Loading DAE from: {}", dae_path);
    let dae_bytes = fs::read(dae_path).unwrap();
    let dae_str = String::from_utf8(dae_bytes).unwrap();
    let mut model = parse_dae(&dae_str).unwrap();
    model.is_v2 = true;

    println!("Model: meshes={}, materials={}, collision_meshes={}", 
        model.meshes.len(), model.materials.len(), model.collision_meshes.len());
    
    for (i, mesh) in model.meshes.iter().enumerate() {
        println!("  Mesh {}: name='{}', lod={}, parts={}", i, mesh.name, mesh.lod, mesh.parts.len());
        for (j, part) in mesh.parts.iter().enumerate() {
            println!("    Part {}: verts={}, indices={}", j, part.vertices.len(), part.indices.len());
        }
    }
    
    for (i, cm) in model.collision_meshes.iter().enumerate() {
        println!("  Collision {}: name='{}', parts={}", i, cm.name, cm.mesh.parts.len());
        for (j, part) in cm.mesh.parts.iter().enumerate() {
            println!("    Part {}: verts={}, indices={}", j, part.vertices.len(), part.indices.len());
        }
    }

    println!("\nGenerating HOD 2.0...");
    let new_bytes = generate_v2_from_model(&[], &model).unwrap();
    println!("Generated {} bytes", new_bytes.len());
    
    fs::write(out_path, &new_bytes).unwrap();
    println!("Saved to: {}", out_path);

    // Verify by re-parsing
    println!("\nVerifying by re-parsing...");
    match HODModel::parse(&new_bytes) {
        Ok(reparsed) => {
            println!("SUCCESS: Re-parsed generated file!");
            println!("  Meshes={}, Joints={}, Materials={}, CollisionMeshes={}", 
                reparsed.meshes.len(), reparsed.joints.len(), reparsed.materials.len(), reparsed.collision_meshes.len());
            
            for (i, cm) in reparsed.collision_meshes.iter().enumerate() {
                println!("  Collision {}: name='{}', parts={}", i, cm.name, cm.mesh.parts.len());
                for (j, part) in cm.mesh.parts.iter().enumerate() {
                    println!("    Part {}: verts={}, indices={}", j, part.vertices.len(), part.indices.len());
                }
            }
            
            // Check chunk structure
            for chunk in &reparsed.preserved_chunks {
                println!("  Preserved chunk: id='{}', size={}", chunk.id, chunk.data.len());
            }
        }
        Err(e) => {
            println!("FAILED: Could not re-parse generated file! {}", e);
        }
    }
}
