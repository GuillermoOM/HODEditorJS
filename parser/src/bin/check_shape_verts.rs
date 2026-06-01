use hwr_hod_parser::hod::HODModel;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 { return; }
    let bytes = std::fs::read(&args[1]).unwrap();
    let model = HODModel::parse(&bytes).unwrap();

    for shape in &model.engine_shapes {
        println!("Shape: {}", shape.name);
        println!("  Parent: {}", shape.parent_name);
        if let Some(part) = shape.mesh.parts.first() {
            println!("  Vertices: {:?}", &part.vertices.get(..5.min(part.vertices.len())));
        }
        println!();
    }
}
