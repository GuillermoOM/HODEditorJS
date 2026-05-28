use std::fs;
use hwr_hod_parser::hod::{HODModel, generate_v2_from_model};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let vanilla_path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/pebble/pebble_0/pebble_0_original.hod";
    let from_scratch_path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/pebble/pebble_0/pebble_0.hod";
    
    println!("Reading vanilla HOD: {}", vanilla_path);
    let vanilla_bytes = fs::read(vanilla_path)?;
    let vanilla_model = HODModel::parse(&vanilla_bytes)?;
    
    // Extract KDOP and COLD from vanilla
    let kdop_chunk = vanilla_model.preserved_chunks.iter().find(|c| c.id == "KDOP").cloned();
    let cold_chunk = vanilla_model.preserved_chunks.iter().find(|c| c.id == "COLD").cloned();
    
    println!("Reading from-scratch HOD: {}", from_scratch_path);
    let from_scratch_bytes = fs::read(from_scratch_path)?;
    let mut scratch_model = HODModel::parse(&from_scratch_bytes)?;
    
    // Rename current from-scratch to keep a backup
    let backup_path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/pebble/pebble_0/pebble_0_from_scratch.hod";
    fs::rename(from_scratch_path, backup_path)?;
    println!("Renamed current from-scratch file to: {}", backup_path);

    // Inject KDOP and COLD
    if let Some(kdop) = kdop_chunk {
        println!("Injecting original KDOP chunk (Size: {})", kdop.data.len());
        scratch_model.preserved_chunks.push(kdop);
    }
    if let Some(cold) = cold_chunk {
        println!("Injecting original COLD chunk (Size: {})", cold.data.len());
        scratch_model.preserved_chunks.push(cold);
    }
    
    // Save the KDOP-injected equivalent
    let with_kdop_bytes = generate_v2_from_model(&from_scratch_bytes, &scratch_model)?;
    fs::write(from_scratch_path, &with_kdop_bytes)?;
    println!("Saved WITH-KDOP (injected) back to: {}", from_scratch_path);

    Ok(())
}
