use std::fs;
use hwr_hod_parser::hod::HODModel;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let orig_path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/pebble/pebble_0/pebble_0_original.hod";
    let gen_path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/pebble/pebble_0/pebble_0.hod";
    
    let orig_bytes = fs::read(orig_path)?;
    println!("Vanilla POOL size: {}", find_pool_size(&orig_bytes));
    
    let gen_bytes = fs::read(gen_path)?;
    println!("Generated POOL size: {}", find_pool_size(&gen_bytes));
    Ok(())
}

fn find_pool_size(bytes: &[u8]) -> usize {
    let mut pos = 12; // skip HOD header
    while pos + 8 <= bytes.len() {
        let mut id = [0u8; 4];
        id.copy_from_slice(&bytes[pos..pos+4]);
        let size = u32::from_le_bytes(bytes[pos+4..pos+8].try_into().unwrap()) as usize;
        let id_str = String::from_utf8_lossy(&id);
        
        if id_str == "POOL" {
            let pool_data = &bytes[pos+8..pos+8+size];
            let comp_tex = u32::from_le_bytes(pool_data[4..8].try_into().unwrap());
            let _decomp_tex = u32::from_le_bytes(pool_data[8..12].try_into().unwrap());
            
            let mesh_offset = 12 + comp_tex as usize;
            let comp_mesh = u32::from_le_bytes(pool_data[mesh_offset..mesh_offset+4].try_into().unwrap());
            let decomp_mesh = u32::from_le_bytes(pool_data[mesh_offset+4..mesh_offset+8].try_into().unwrap());
            println!("  Mesh pool: comp={}, decomp={}", comp_mesh, decomp_mesh);
            return size;
        }
        pos += 8 + size;
    }
    0
}
