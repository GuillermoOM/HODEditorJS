use hwr_hod_parser::hod::HODModel;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 { return; }
    let bytes = fs::read(&args[1]).unwrap();
    let model = HODModel::parse(&bytes).unwrap();
    for joint in model.joints {
        if joint.name == "Root" {
            println!("Root joint transform: {:?}", joint.local_transform.m);
            println!("Root joint parent: {:?}", joint.parent_name);
            println!("Root joint rotation: {:?}", joint.rotation);
        }
    }
}
