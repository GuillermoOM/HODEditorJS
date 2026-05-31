use hwr_hod_parser::hod::HODModel;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: dump_mesh_info <file1.hod> [file2.hod] ...");
        return;
    }

    for arg in args.iter().skip(1) {
        println!("=====================================");
        println!("File: {}", arg);
        let bytes = fs::read(arg).unwrap();
        let model = HODModel::parse(&bytes).unwrap();
        
        for (i, mesh) in model.meshes.iter().enumerate() {
            println!("Mesh {} ({}) - parent: {:?}", i, mesh.name, mesh.parent_name);
            println!("  has_mult_tags: {}", mesh.has_mult_tags);
            if !mesh.parts.is_empty() && !mesh.parts[0].vertices.is_empty() {
                let v = &mesh.parts[0].vertices[0].position;
                println!("  Vertex 0: ({}, {}, {})", v.x, v.y, v.z);
            }
        }
    }
}
