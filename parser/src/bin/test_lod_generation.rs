use hwr_hod_parser::collision::generate_all_lod_meshes;
use hwr_hod_parser::hod::HODModel;
use std::fs;

fn main() {
    let hod_path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/mod-tools/HODEditorJS/testing/ter_centaur/ter_centaur_hodor.hod";

    println!("Loading HOD from: {}", hod_path);
    let bytes = fs::read(hod_path).unwrap();
    let mut model = HODModel::parse(&bytes).unwrap();

    println!("Original model:");
    println!("  Meshes: {}", model.meshes.len());
    for mesh in &model.meshes {
        let total_verts: usize = mesh.parts.iter().map(|p| p.vertices.len()).sum();
        let total_tris: usize = mesh.parts.iter().map(|p| p.indices.len() / 3).sum();
        println!("    {} (LOD {}): {} parts, {} verts, {} tris", 
            mesh.name, mesh.lod, mesh.parts.len(), total_verts, total_tris);
    }

    println!("\nGenerating LOD meshes...");
    generate_all_lod_meshes(&mut model);

    println!("\nModel after LOD generation:");
    println!("  Meshes: {}", model.meshes.len());
    for mesh in &model.meshes {
        let total_verts: usize = mesh.parts.iter().map(|p| p.vertices.len()).sum();
        let total_tris: usize = mesh.parts.iter().map(|p| p.indices.len() / 3).sum();
        println!("    {} (LOD {}): {} parts, {} verts, {} tris", 
            mesh.name, mesh.lod, mesh.parts.len(), total_verts, total_tris);
    }

    // Verify LOD structure
    let lod0 = model.meshes.iter().find(|m| m.lod == 0);
    let lod1 = model.meshes.iter().find(|m| m.lod == 1);
    let lod2 = model.meshes.iter().find(|m| m.lod == 2);
    let lod3 = model.meshes.iter().find(|m| m.lod == 3);

    println!("\nLOD Chain Verification:");
    if let Some(lod0) = lod0 {
        let v0: usize = lod0.parts.iter().map(|p| p.vertices.len()).sum();
        println!("  LOD 0: {} vertices (100%)", v0);
        
        if let Some(lod1) = lod1 {
            let v1: usize = lod1.parts.iter().map(|p| p.vertices.len()).sum();
            println!("  LOD 1: {} vertices ({:.1}%)", v1, (v1 as f32 / v0 as f32) * 100.0);
        }
        if let Some(lod2) = lod2 {
            let v2: usize = lod2.parts.iter().map(|p| p.vertices.len()).sum();
            println!("  LOD 2: {} vertices ({:.1}%)", v2, (v2 as f32 / v0 as f32) * 100.0);
        }
        if let Some(lod3) = lod3 {
            let v3: usize = lod3.parts.iter().map(|p| p.vertices.len()).sum();
            println!("  LOD 3: {} vertices ({:.1}%)", v3, (v3 as f32 / v0 as f32) * 100.0);
        }
    }

    println!("\nSUCCESS: LOD generation completed!");
}
