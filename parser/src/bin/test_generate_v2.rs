use hwr_hod_parser::hod::{HODModel, generate_v2_from_model};
use std::fs;

fn main() {
    let path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/uncompressed_bigs/HWRM/pebble/pebble_0/pebble_0.hod";
    let bytes = fs::read(path).unwrap();
    let mut model = HODModel::parse(&bytes).unwrap();
    
    let out_bytes = generate_v2_from_model(&bytes, &model).unwrap();
    fs::write("pebble_0_generate_v2.hod", out_bytes).unwrap();
}
