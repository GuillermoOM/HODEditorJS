use hwr_hod_parser::dae::parse_dae;
fn main() {
    let bytes = std::fs::read_to_string("../testing/ter_centaur/ter_centaur.DAE").unwrap();
    let model = parse_dae(&bytes).unwrap();
    for j in &model.joints {
        println!("Joint: {}", j.name);
        println!("  Pos: {:?}", j.position);
        println!("  Transform: {:?}", j.local_transform.m);
    }
}
