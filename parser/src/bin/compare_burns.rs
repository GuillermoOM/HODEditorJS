use hwr_hod_parser::hod::HODModel;
use std::fs;

fn main() {
    let original_path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/mod-tools/HODEditorJS/testing/ter_myrmidon/ter_myrmidon_2.0_original.hod";
    let saved_path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/mod-tools/HODEditorJS/testing/ter_myrmidon/ter_myrmidon_from_2.0_to_2.0.hod";

    let orig_bytes = fs::read(original_path).unwrap();
    let saved_bytes = fs::read(saved_path).unwrap();

    let orig_model = HODModel::parse(&orig_bytes).unwrap();
    let saved_model = HODModel::parse(&saved_bytes).unwrap();

    println!("=== BURN COMPARISON ===\n");
    println!("Original BURN count: {}", orig_model.engine_burns.len());
    println!("Saved BURN count: {}\n", saved_model.engine_burns.len());

    for (i, (orig_burn, saved_burn)) in orig_model.engine_burns.iter().zip(saved_model.engine_burns.iter()).enumerate() {
        println!("--- BURN[{}] ---", i);
        println!("  Name: {} -> {}", orig_burn.name, saved_burn.name);
        println!("  Parent: {} -> {}", orig_burn.parent_name, saved_burn.parent_name);
        println!("  Divisions: {} -> {}", orig_burn.num_divisions, saved_burn.num_divisions);
        println!("  Flames: {} -> {}", orig_burn.num_flames, saved_burn.num_flames);
        println!("  Vertices: {} -> {}", orig_burn.vertices.len(), saved_burn.vertices.len());
        
        let mut diffs = 0;
        for (j, (ov, sv)) in orig_burn.vertices.iter().zip(saved_burn.vertices.iter()).enumerate() {
            if (ov.x - sv.x).abs() > 0.0001 || (ov.y - sv.y).abs() > 0.0001 || (ov.z - sv.z).abs() > 0.0001 {
                println!("    Vertex[{}]: ({:.6}, {:.6}, {:.6}) -> ({:.6}, {:.6}, {:.6})", 
                    j, ov.x, ov.y, ov.z, sv.x, sv.y, sv.z);
                diffs += 1;
            }
        }
        if diffs == 0 {
            println!("    All vertices identical");
        } else {
            println!("    {} vertices differ", diffs);
        }
    }

    println!("\n=== JOINT COMPARISON (BURN parents) ===\n");
    
    let burn_parent_names: Vec<String> = orig_model.engine_burns.iter()
        .map(|b| b.parent_name.clone())
        .collect();
    
    for parent_name in &burn_parent_names {
        let orig_joint = orig_model.joints.iter().find(|j| j.name == *parent_name);
        let saved_joint = saved_model.joints.iter().find(|j| j.name == *parent_name);
        
        match (orig_joint, saved_joint) {
            (Some(oj), Some(sj)) => {
                println!("--- Joint: {} ---", parent_name);
                println!("  Parent: {:?} -> {:?}", oj.parent_name, sj.parent_name);
                
                let op = oj.position.as_ref().unwrap();
                let sp = sj.position.as_ref().unwrap();
                let or = oj.rotation.as_ref().unwrap();
                let sr = sj.rotation.as_ref().unwrap();
                let os = oj.scale.as_ref().unwrap();
                let ss = sj.scale.as_ref().unwrap();
                
                let pos_diff = (op.x - sp.x).abs() > 0.0001 || (op.y - sp.y).abs() > 0.0001 || (op.z - sp.z).abs() > 0.0001;
                let rot_diff = (or.x - sr.x).abs() > 0.0001 || (or.y - sr.y).abs() > 0.0001 || (or.z - sr.z).abs() > 0.0001;
                let scale_diff = (os.x - ss.x).abs() > 0.0001 || (os.y - ss.y).abs() > 0.0001 || (os.z - ss.z).abs() > 0.0001;
                
                if pos_diff {
                    println!("  Position: ({:.6}, {:.6}, {:.6}) -> ({:.6}, {:.6}, {:.6}) DIFFERS", 
                        op.x, op.y, op.z, sp.x, sp.y, sp.z);
                } else {
                    println!("  Position: identical ({:.6}, {:.6}, {:.6})", op.x, op.y, op.z);
                }
                
                if rot_diff {
                    println!("  Rotation: ({:.6}, {:.6}, {:.6}) -> ({:.6}, {:.6}, {:.6}) DIFFERS", 
                        or.x, or.y, or.z, sr.x, sr.y, sr.z);
                } else {
                    println!("  Rotation: identical ({:.6}, {:.6}, {:.6})", or.x, or.y, or.z);
                }
                
                if scale_diff {
                    println!("  Scale: ({:.6}, {:.6}, {:.6}) -> ({:.6}, {:.6}, {:.6}) DIFFERS", 
                        os.x, os.y, os.z, ss.x, ss.y, ss.z);
                } else {
                    println!("  Scale: identical ({:.6}, {:.6}, {:.6})", os.x, os.y, os.z);
                }
                
                // Compare local_transform matrices
                let mut mat_diff = false;
                for r in 0..4 {
                    for c in 0..4 {
                        if (oj.local_transform.m[r][c] - sj.local_transform.m[r][c]).abs() > 0.0001 {
                            mat_diff = true;
                        }
                    }
                }
                if mat_diff {
                    println!("  Matrix DIFFERS:");
                    for r in 0..4 {
                        println!("    [{:.6}, {:.6}, {:.6}, {:.6}] -> [{:.6}, {:.6}, {:.6}, {:.6}]",
                            oj.local_transform.m[r][0], oj.local_transform.m[r][1], 
                            oj.local_transform.m[r][2], oj.local_transform.m[r][3],
                            sj.local_transform.m[r][0], sj.local_transform.m[r][1], 
                            sj.local_transform.m[r][2], sj.local_transform.m[r][3]);
                    }
                } else {
                    println!("  Matrix: identical");
                }
            }
            _ => {
                println!("--- Joint: {} --- NOT FOUND in one or both files", parent_name);
            }
        }
    }

    // Also check ALL joints for differences
    println!("\n=== ALL JOINT DIFFERENCES ===\n");
    let mut joint_diffs = 0;
    for oj in &orig_model.joints {
        if let Some(sj) = saved_model.joints.iter().find(|j| j.name == oj.name) {
            let op = oj.position.as_ref().unwrap();
            let sp = sj.position.as_ref().unwrap();
            let or = oj.rotation.as_ref().unwrap();
            let sr = sj.rotation.as_ref().unwrap();
            
            let pos_diff = (op.x - sp.x).abs() > 0.0001 || (op.y - sp.y).abs() > 0.0001 || (op.z - sp.z).abs() > 0.0001;
            let rot_diff = (or.x - sr.x).abs() > 0.0001 || (or.y - sr.y).abs() > 0.0001 || (or.z - sr.z).abs() > 0.0001;
            
            if pos_diff || rot_diff {
                println!("Joint '{}' DIFFERS:", oj.name);
                if pos_diff {
                    println!("  Pos: ({:.6}, {:.6}, {:.6}) -> ({:.6}, {:.6}, {:.6})", 
                        op.x, op.y, op.z, sp.x, sp.y, sp.z);
                }
                if rot_diff {
                    println!("  Rot: ({:.6}, {:.6}, {:.6}) -> ({:.6}, {:.6}, {:.6})", 
                        or.x, or.y, or.z, sr.x, sr.y, sr.z);
                }
                joint_diffs += 1;
            }
        }
    }
    if joint_diffs == 0 {
        println!("All joints identical");
    } else {
        println!("\n{} joints differ", joint_diffs);
    }

    println!("\n=== NAVLIGHT NAMES ===\n");
    println!("Original navlights ({}):", orig_model.nav_lights.len());
    for n in &orig_model.nav_lights {
        println!("  {}", n.name);
    }

    println!("\n=== ALL JOINT NAMES ===\n");
    println!("Original joints ({}):", orig_model.joints.len());
    for (i, j) in orig_model.joints.iter().enumerate() {
        println!("  [{}] {} (parent: {:?})", i, j.name, j.parent_name);
    }
    println!("\nSaved joints ({}):", saved_model.joints.len());
    for (i, j) in saved_model.joints.iter().enumerate() {
        println!("  [{}] {} (parent: {:?})", i, j.name, j.parent_name);
    }
}