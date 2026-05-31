use hwr_hod_parser::dae;
use std::fs;

fn main() {
    let dae_str = fs::read_to_string("../testing/ter_centaur/ter_centaur.DAE").unwrap();
    let model = dae::parse_dae(&dae_str).unwrap();
    
    for mesh in model.meshes {
        println!("Mesh: {}", mesh.name);
        for (i, part) in mesh.parts.iter().enumerate() {
            println!("  Part {}: verts={}, indices={}", i, part.vertices.len(), part.indices.len());
        }
    }
}
