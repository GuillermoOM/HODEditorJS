use std::fs;
use hwr_hod_parser::hod::HODModel;

fn main() {
    let orig_path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/pebble/pebble_0/pebble_0_original.hod";
    let bytes = fs::read(orig_path).unwrap();
    let model = HODModel::parse(&bytes).unwrap();
    for t in model.textures {
        println!("Tex: {}, Format: {}", t.name, t.format);
    }
}
