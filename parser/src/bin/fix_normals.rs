use std::fs;
use hwr_hod_parser::hod::{HODModel, generate_v2_from_model};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let in_path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/pebble/pebble_0/pebble_0_from_scratch.hod";
    let out_path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/pebble/pebble_0/pebble_0.hod";
    
    let bytes = fs::read(in_path)?;
    let mut model = HODModel::parse(&bytes)?;
    
    // model will now save with Normal.w = 1.0 automatically thanks to our hod.rs fix!
    let out_bytes = generate_v2_from_model(&bytes, &model)?;
    fs::write(out_path, &out_bytes)?;
    println!("Saved fixed normals to {}", out_path);
    Ok(())
}
