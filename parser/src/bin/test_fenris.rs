use std::fs;
fn main() {
    let hod_path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/ship/ter_fenris/ter_fenris.hod";
    let bytes = fs::read(hod_path).unwrap();
    match hwr_hod_parser::hod::HODModel::parse_with_external(&bytes, Some(hod_path), None) {
        Ok(model) => {
            for anim in model.animations {
                for track in anim.tracks {
                    if track.joint_name == "RadarDish" {
                        for kf in track.keyframes {
                            println!("  kf time: {:.2}, euler: {:?}, scale: {:?}", kf.time, kf.rotation_euler, kf.scale);
                        }
                    }
                }
            }
        },
        Err(e) => println!("Error: {}", e),
    }
}
