use std::fs;
use hwr_hod_parser::hod;

fn main() {
    let path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/uncompressed_bigs/HWRM/ship/hgn_ioncannonfrigate/hgn_ioncannonfrigate.hod";
    let bytes = fs::read(&path).unwrap();
    let model = hod::HODModel::parse_with_external(&bytes, None, None).unwrap();
    
    if let Some(tex) = model.textures.first() {
        // We can't directly see the raw bytes, but we can look at the sizes of the PNGs
        println!("Texture {}", tex.name);
    }
}
