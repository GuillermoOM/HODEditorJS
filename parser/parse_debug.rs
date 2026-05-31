use hwr_hod_parser::dae::parse_dae;
fn main() {
    let xml = std::fs::read_to_string("../testing/ter_centaur/ter_centaur.DAE").unwrap();
    let model = parse_dae(&xml).unwrap();
    for j in &model.joints {
        if j.name == "EngineNozzle1" {
            println!("EngineNozzle1 Transform: {:?}", j.local_transform.m);
        }
    }
}
