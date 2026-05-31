use hwr_hod_parser::hod::HODModel;
use std::fs;

fn main() {
    let from_dae_path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/mod-tools/HODEditorJS/testing/ter_centaur/ter_centaur_from_dae.hod";
    let hodor_path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/mod-tools/HODEditorJS/testing/ter_centaur/ter_centaur_hodor.hod";

    let from_dae_bytes = fs::read(from_dae_path).unwrap();
    let hodor_bytes = fs::read(hodor_path).unwrap();

    println!("=== FROM_DAE ===");
    let from_dae = HODModel::parse(&from_dae_bytes).unwrap();
    print_model_info(&from_dae, "from_dae");

    println!("\n=== HODOR ===");
    let hodor = HODModel::parse(&hodor_bytes).unwrap();
    print_model_info(&hodor, "hodor");

    // Compare STAT chunks via preserved_chunks
    println!("\n=== PRESERVED CHUNKS COMPARISON ===");
    println!("from_dae preserved_chunks: {}", from_dae.preserved_chunks.len());
    for (i, chunk) in from_dae.preserved_chunks.iter().enumerate() {
        println!("  Chunk {}: id='{}', size={}, children={}", i, chunk.id, chunk.data.len(), chunk.children.len());
        for (j, child) in chunk.children.iter().enumerate() {
            println!("    Child {}: id='{}', size={}", j, child.id, child.data.len());
        }
    }
    println!("hodor preserved_chunks: {}", hodor.preserved_chunks.len());
    for (i, chunk) in hodor.preserved_chunks.iter().enumerate() {
        println!("  Chunk {}: id='{}', size={}, children={}", i, chunk.id, chunk.data.len(), chunk.children.len());
        for (j, child) in chunk.children.iter().enumerate() {
            println!("    Child {}: id='{}', size={}", j, child.id, child.data.len());
        }
    }

    // Compare materials
    println!("\n=== MATERIAL COMPARISON ===");
    println!("from_dae materials: {}", from_dae.materials.len());
    for (i, mat) in from_dae.materials.iter().enumerate() {
        println!("  Material {}: name='{}', shader='{}', textures={:?}", i, mat.name, mat.shader_name, mat.texture_maps);
    }
    println!("hodor materials: {}", hodor.materials.len());
    for (i, mat) in hodor.materials.iter().enumerate() {
        println!("  Material {}: name='{}', shader='{}', textures={:?}", i, mat.name, mat.shader_name, mat.texture_maps);
    }

    // Compare mesh parts
    println!("\n=== MESH PART COMPARISON ===");
    println!("from_dae meshes: {}", from_dae.meshes.len());
    for (i, mesh) in from_dae.meshes.iter().enumerate() {
        println!("  Mesh {}: name='{}', lod={}, parts={}", i, mesh.name, mesh.lod, mesh.parts.len());
        for (j, part) in mesh.parts.iter().enumerate() {
            println!("    Part {}: verts={}, indices={}, material_index={}, mask=0x{:x}", j, part.vertices.len(), part.indices.len(), part.material_index, part.vertex_mask);
        }
    }
    println!("hodor meshes: {}", hodor.meshes.len());
    for (i, mesh) in hodor.meshes.iter().enumerate() {
        println!("  Mesh {}: name='{}', lod={}, parts={}", i, mesh.name, mesh.lod, mesh.parts.len());
        for (j, part) in mesh.parts.iter().enumerate() {
            println!("    Part {}: verts={}, indices={}, material_index={}, mask=0x{:x}", j, part.vertices.len(), part.indices.len(), part.material_index, part.vertex_mask);
        }
    }

    // Compare collision meshes
    println!("\n=== COLLISION MESH COMPARISON ===");
    println!("from_dae collision_meshes: {}", from_dae.collision_meshes.len());
    for (i, cm) in from_dae.collision_meshes.iter().enumerate() {
        println!("  Collision {}: name='{}', parts={}", i, cm.name, cm.mesh.parts.len());
        println!("    min_extents: ({}, {}, {})", cm.min_extents.x, cm.min_extents.y, cm.min_extents.z);
        println!("    max_extents: ({}, {}, {})", cm.max_extents.x, cm.max_extents.y, cm.max_extents.z);
        println!("    center: ({}, {}, {})", cm.center.x, cm.center.y, cm.center.z);
        println!("    radius: {}", cm.radius);
        for (j, part) in cm.mesh.parts.iter().enumerate() {
            println!("    Part {}: verts={}, indices={}", j, part.vertices.len(), part.indices.len());
            if !part.vertices.is_empty() {
                println!("      First vert: ({}, {}, {})", part.vertices[0].position.x, part.vertices[0].position.y, part.vertices[0].position.z);
            }
        }
    }
    println!("hodor collision_meshes: {}", hodor.collision_meshes.len());
    for (i, cm) in hodor.collision_meshes.iter().enumerate() {
        println!("  Collision {}: name='{}', parts={}", i, cm.name, cm.mesh.parts.len());
        println!("    min_extents: ({}, {}, {})", cm.min_extents.x, cm.min_extents.y, cm.min_extents.z);
        println!("    max_extents: ({}, {}, {})", cm.max_extents.x, cm.max_extents.y, cm.max_extents.z);
        println!("    center: ({}, {}, {})", cm.center.x, cm.center.y, cm.center.z);
        println!("    radius: {}", cm.radius);
        for (j, part) in cm.mesh.parts.iter().enumerate() {
            println!("    Part {}: verts={}, indices={}", j, part.vertices.len(), part.indices.len());
            if !part.vertices.is_empty() {
                println!("      First vert: ({}, {}, {})", part.vertices[0].position.x, part.vertices[0].position.y, part.vertices[0].position.z);
            }
        }
    }

    // Check pool sizes
    println!("\n=== POOL SIZE COMPARISON ===");
    // Note: texture_pool, mesh_pool, face_pool are not fields on HODModel - they are internal to parsing
    // We need to check the POOL chunk in preserved_chunks
}

fn print_model_info(model: &HODModel, label: &str) {
    println!("{}: name='{}', meshes={}, joints={}, materials={}, collision_meshes={}, preserved_chunks={}", 
        label, model.name, model.meshes.len(), model.joints.len(), model.materials.len(), model.collision_meshes.len(), model.preserved_chunks.len());
}
