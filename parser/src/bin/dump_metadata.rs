use hwr_hod_parser::hod;
use std::fs;
use std::path::Path;

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: dump_metadata <hod_file> <output_dir>");
        return Ok(());
    }

    let hod_path = &args[1];
    let out_dir = Path::new(&args[2]);

    let bytes = fs::read(hod_path).map_err(|e| e.to_string())?;
    let model = hod::HODModel::parse(&bytes).map_err(|e| e.to_string())?;

    fs::write(
        out_dir.join("joints.json"),
        serde_json::to_string_pretty(&model.joints).unwrap()
    ).unwrap();

    fs::write(
        out_dir.join("markers.json"),
        serde_json::to_string_pretty(&model.markers).unwrap()
    ).unwrap();

    fs::write(
        out_dir.join("navlights.json"),
        serde_json::to_string_pretty(&model.nav_lights).unwrap()
    ).unwrap();

    fs::write(
        out_dir.join("engine_burns.json"),
        serde_json::to_string_pretty(&model.engine_burns).unwrap()
    ).unwrap();

    fs::write(
        out_dir.join("collision_meshes.json"),
        serde_json::to_string_pretty(&model.collision_meshes).unwrap()
    ).unwrap();

    println!("Dumped metadata to {}", out_dir.display());

    Ok(())
}
