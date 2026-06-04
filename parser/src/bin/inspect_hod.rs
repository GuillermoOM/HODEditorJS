use std::fs::File;
use std::io::Read;
use hwr_hod_parser::hod::HODModel;

fn main() {
    let path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/uncompressed_bigs/HWRM/ship/vgr_planetkiller/vgr_planetkiller.hod";
    let mut file = File::open(path).expect("Failed to open HOD");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("Failed to read");
    let hod = HODModel::parse_with_external(&buffer, Some(path), None).expect("Failed to parse");
    
    println!("Animations found: {}", hod.animations.len());
    for (i, anim) in hod.animations.iter().enumerate() {
        println!(" - Animation {}: name={}, duration={}, tracks={}", i, anim.name, anim.duration, anim.tracks.len());
    }

    println!("\nJoints:");
    for joint in &hod.joints {
        println!(" - Joint: {}, Pos: {:?}, Rot: {:?}, Scale: {:?}", joint.name, joint.position, joint.rotation, joint.scale);
    }
}
