use hwr_hod_parser::hod::HODModel;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 { return; }
    let bytes = std::fs::read(&args[1]).unwrap();
    let model = HODModel::parse(&bytes).unwrap();

    for burn in &model.engine_burns {
        println!("Burn: {}", burn.name);
        println!("  Parent: {}", burn.parent_name);
        println!("  Vertices: {:?}", &burn.vertices.get(..5.min(burn.vertices.len())));
        if let Some(joint) = model.joints.iter().find(|j| j.name == burn.parent_name) {
            println!("  Parent Transform (Nozzle):");
            for row in &joint.local_transform.m {
                println!("    {:?}", row);
            }
        }
        println!();
    }
}
