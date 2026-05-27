use hwr_hod_parser::hod::{HODModel, save_edits};
use std::fs;

fn main() {
    let path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/uncompressed_bigs/HWRM/pebble/pebble_0/pebble_0.hod";
    let bytes = fs::read(path).unwrap();
    let mut model = HODModel::parse(&bytes).unwrap();
    
    // Dump to JSON
    let json_str = serde_json::to_string(&model).unwrap();
    fs::write("pebble_0_rust.json", json_str).unwrap();
}
