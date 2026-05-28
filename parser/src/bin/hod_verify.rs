use std::fs;
use hwr_hod_parser::hod::HODModel;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: hod_verify <path.hod>");
        std::process::exit(1);
    }
    let path = &args[1];
    let bytes = fs::read(path).expect("Cannot read file");
    println!("File: {}", path);
    println!("Size: {} bytes", bytes.len());
    println!();
    
    match HODModel::parse(&bytes) {
        Ok(model) => {
            println!("PARSE SUCCESS!");
            println!("  Version: 0x{:04X} ({})", model.version, if model.is_v2 { "HOD 2.0" } else { "HOD 1.0" });
            println!("  Name: '{}'", model.name);
            println!("  Meshes: {}", model.meshes.len());
            for (i, mesh) in model.meshes.iter().enumerate() {
                let total_verts: usize = mesh.parts.iter().map(|p| p.vertices.len()).sum();
                let total_indices: usize = mesh.parts.iter().map(|p| p.indices.len()).sum();
                println!("    [{}] '{}' LOD={} parts={} verts={} indices={}", i, mesh.name, mesh.lod, mesh.parts.len(), total_verts, total_indices);
            }
            println!("  Joints: {}", model.joints.len());
            for (i, joint) in model.joints.iter().enumerate() {
                println!("    [{}] '{}' parent={:?}", i, joint.name, joint.parent_name);
            }
            println!("  Textures: {}", model.textures.len());
            for (i, tex) in model.textures.iter().enumerate() {
                println!("    [{}] '{}' {}x{} fmt={}", i, tex.name, tex.width, tex.height, tex.format);
            }
            println!("  Materials: {}", model.materials.len());
            for (i, mat) in model.materials.iter().enumerate() {
                println!("    [{}] '{}' shader='{}' maps={:?}", i, mat.name, mat.shader_name, mat.texture_maps);
            }
            println!("  NavLights: {}", model.nav_lights.len());
            println!("  Markers: {}", model.markers.len());
            println!("  EngineBurns: {}", model.engine_burns.len());
            println!("  CollisionMeshes: {}", model.collision_meshes.len());
            println!("  Preserved chunks: {}", model.preserved_chunks.len());
            for (i, chunk) in model.preserved_chunks.iter().enumerate() {
                println!("    [{}] '{}' type={:?}", i, chunk.id, chunk.chunk_type);
            }
            println!();
            
            // Try to re-serialize
            println!("Attempting re-serialize...");
            match hwr_hod_parser::hod::generate_v2_from_model(&bytes, &model) {
                Ok(new_bytes) => {
                    println!("  Re-serialize SUCCESS! Output: {} bytes (delta: {})", new_bytes.len(), new_bytes.len() as isize - bytes.len() as isize);
                    
                    // Re-parse to verify
                    match HODModel::parse(&new_bytes) {
                        Ok(reparsed) => {
                            println!("  Re-parse SUCCESS! Meshes={}, Joints={}", reparsed.meshes.len(), reparsed.joints.len());
                        }
                        Err(e) => {
                            println!("  Re-parse FAILED: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("  Re-serialize FAILED: {}", e);
                }
            }
        }
        Err(e) => {
            println!("PARSE FAILED: {}", e);
        }
    }
}
