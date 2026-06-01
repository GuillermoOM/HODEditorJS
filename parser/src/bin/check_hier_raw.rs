use hwr_hod_parser::hod::HODModel;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 { return; }
    let bytes = std::fs::read(&args[1]).unwrap();
    let model = HODModel::parse(&bytes).unwrap();

    for j in &model.joints {
        if j.name.starts_with("EngineNozzle") {
            println!("Joint: {}", j.name);
            println!("  Pos: {:?}", j.position.as_ref().unwrap());
            println!("  Rot: {:?}", j.rotation.as_ref().unwrap());
            println!("  Scl: {:?}", j.scale.as_ref().unwrap());
        }
    }
}
