use hwr_hod_parser::hod::{HODModel, save_edits};
use std::fs;

fn main() {
    let path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/uncompressed_bigs/HWRM/pebble/pebble_0/pebble_0.hod";
    let bytes = fs::read(path).unwrap();
    let mut model = HODModel::parse(&bytes).unwrap();
    
    // Simulate Tauri JSON roundtrip
    let json_str = serde_json::to_string(&model).unwrap();
    let mut roundtrip_model: HODModel = serde_json::from_str(&json_str).unwrap();

    let out_bytes = save_edits(&bytes, &mut roundtrip_model).unwrap();
    fs::write("pebble_0_json_roundtrip.hod", out_bytes).unwrap();
}
