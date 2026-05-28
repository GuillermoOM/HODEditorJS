use std::fs;

fn main() {
    let orig_path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/pebble/pebble_0/pebble_0_original.hod";
    let new_path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/pebble/pebble_0/pebble_0.hod";
    
    let orig_bytes = fs::read(orig_path).unwrap();
    let new_bytes = fs::read(new_path).unwrap();
    
    // Find POOL sizes
    let mut orig_pool = 0;
    for i in 0..orig_bytes.len()-4 {
        if &orig_bytes[i..i+4] == b"POOL" {
            orig_pool = i; break;
        }
    }
    let mut new_pool = 0;
    for i in 0..new_bytes.len()-4 {
        if &new_bytes[i..i+4] == b"POOL" {
            new_pool = i; break;
        }
    }
    
    println!("Original total size: {}, POOL at {}", orig_bytes.len(), orig_pool);
    println!("New total size: {}, POOL at {}", new_bytes.len(), new_pool);
    
    // Decompress and compare using HODModel
    use hwr_hod_parser::hod::HODModel;
    
    let orig_model = HODModel::parse(&orig_bytes).unwrap();
    let new_model = HODModel::parse(&new_bytes).unwrap();
    
    println!("\n=== HODModel Comparison ===");
    println!("Meshes: orig={}, new={}", orig_model.meshes.len(), new_model.meshes.len());
    if orig_model.meshes.len() == new_model.meshes.len() {
        for (i, (om, nm)) in orig_model.meshes.iter().zip(new_model.meshes.iter()).enumerate() {
            println!("Mesh {}", i);
            println!("  Name: orig={}, new={}", om.name, nm.name);
            println!("  LOD: orig={}, new={}", om.lod, nm.lod);
            println!("  Parts: orig={}, new={}", om.parts.len(), nm.parts.len());
            for (k, (op, np)) in om.parts.iter().zip(nm.parts.iter()).enumerate() {
                println!("    Part {}:", k);
                println!("      Material: orig={}, new={}", op.material_index, np.material_index);
                println!("      Vertices: orig={}, new={}", op.vertices.len(), np.vertices.len());
                println!("      Indices: orig={}, new={}", op.indices.len(), np.indices.len());
                
                let mut diff_v = 0;
                for (v1, v2) in op.vertices.iter().zip(np.vertices.iter()) {
                    let eps = 0.01;
                    if (v1.position.x - v2.position.x).abs() > eps || (v1.position.y - v2.position.y).abs() > eps || (v1.position.z - v2.position.z).abs() > eps {
                        diff_v += 1;
                    }
                }
                println!("      Mismatched pos verts: {}", diff_v);
                
                let mut diff_n = 0;
                for (v1, v2) in op.vertices.iter().zip(np.vertices.iter()) {
                    let eps = 0.01;
                    if let (Some(n1), Some(n2)) = (&v1.normal, &v2.normal) {
                        if (n1.x - n2.x).abs() > eps || (n1.y - n2.y).abs() > eps || (n1.z - n2.z).abs() > eps {
                            diff_n += 1;
                        }
                    }
                }
                println!("      Mismatched normal verts: {}", diff_n);

                let mut diff_tan = 0;
                for (v1, v2) in op.vertices.iter().zip(np.vertices.iter()) {
                    let eps = 0.01;
                    if let (Some(t1), Some(t2)) = (&v1.tangent, &v2.tangent) {
                        if (t1.x - t2.x).abs() > eps || (t1.y - t2.y).abs() > eps || (t1.z - t2.z).abs() > eps {
                            diff_tan += 1;
                        }
                    }
                }
                println!("      Mismatched tangent verts: {}", diff_tan);
            }
        }
    }
}
