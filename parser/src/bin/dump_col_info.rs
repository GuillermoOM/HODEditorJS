use hwr_hod_parser::hod::HODModel;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return;
    }

    for arg in args.iter().skip(1) {
        println!("File: {}", arg);
        let bytes = fs::read(arg).unwrap();
        let model = HODModel::parse(&bytes).unwrap();
        
        for (i, col) in model.collision_meshes.iter().enumerate() {
            println!("Collision Mesh {} ({}) - parent: {:?}", i, col.name, col.mesh.parent_name);
            println!("  has_mult_tags: {}", col.mesh.has_mult_tags);
            if !col.mesh.parts.is_empty() && !col.mesh.parts[0].vertices.is_empty() {
                let v = &col.mesh.parts[0].vertices[0].position;
                println!("  Vertex 0: ({}, {}, {})", v.x, v.y, v.z);
            }
        }
    }
}
