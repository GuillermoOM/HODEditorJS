use std::fs;
use hwr_hod_parser::hod::{HODModel, generate_v2_from_model};

fn main() {
    let orig_path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/pebble/pebble_0/pebble_0_original.hod";
    let out_path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/pebble/pebble_0/pebble_0.hod";
    
    let bytes = fs::read(orig_path).unwrap();
    let model = HODModel::parse(&bytes).unwrap();
    
    // Pass empty original bytes to FORCE from-scratch generation
    let new_bytes = generate_v2_from_model(&[], &model).unwrap();
    fs::write(out_path, new_bytes).unwrap();
    println!("Generated from-scratch successfully to {}", out_path);
}
