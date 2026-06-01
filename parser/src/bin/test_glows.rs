use hwr_hod_parser::hod::HODModel;
use std::fs;

fn main() {
    let bytes = fs::read("/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/uncompressed_bigs/HWRM/ship/hgn_ioncannonfrigate/hgn_ioncannonfrigate.hod").unwrap();
    let model = HODModel::parse(&bytes).unwrap();
    for glow in model.engine_glows {
        println!("Glow name: {}, LOD: {}, parent: {}", glow.name, glow.lod, glow.parent_name);
    }
}
