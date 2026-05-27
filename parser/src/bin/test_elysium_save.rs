use hwr_hod_parser::hod::{HODModel, save_edits};
use std::fs;

fn main() {
    let path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/ship/ter_elysium/ter_elysium.hod";
    let bytes = fs::read(path).unwrap();
    let mut model = HODModel::parse(&bytes).unwrap();
    
    let out_bytes = save_edits(&bytes, &mut model).unwrap();
    fs::write("ter_elysium_generated.hod", out_bytes).unwrap();
}
