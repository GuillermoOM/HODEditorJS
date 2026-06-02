with open("parser/src/hod.rs", "r") as f:
    code = f.read()

# 1. Remove geometry scaling from upgrade_v1_to_v2
new_upgrade = """
    pub fn upgrade_v1_to_v2(&mut self) {
        if self.is_v2 {
            return;
        }
        
        // HOD 1.0 (VERS 1000) and HOD 2.0 (VERS 1001) both use meters.
        // We do NOT need to scale positions!
        // The only incompatibility is that HOD 2.0 uses joint scale vectors as gimbal limits.
        // HW2 Classic exported (1.0, 1.0, 1.0) for scale, which HWRM interprets as a 1.0 radian gimbal limit,
        // causing severe coordinate space distortion and rotation.
        
        for joint in &mut self.joints {
            // In HOD 2.0, the scale field acts as gimbal limits, not geometry scale.
            // Using (1,1,1) in HOD 2.0 causes the engine to rotate the ship sideways.
            joint.scale = Some(Vector3 { x: 0.0, y: 0.0, z: 0.0 });
            
            // Recompose matrix with the zeroed scale
            if let (Some(ref pos), Some(ref rot), Some(ref scale)) = (&joint.position, &joint.rotation, &joint.scale) {
                joint.local_transform = compose_transform_matrix(pos.clone(), rot.clone(), scale.clone());
            }
        }
        
        self.is_v2 = true;
    }
"""
import re
code = re.sub(r'    pub fn upgrade_v1_to_v2\(&mut self\) \{.*?\n    pub fn clean_hierarchy', new_upgrade + '\n    pub fn clean_hierarchy', code, flags=re.DOTALL)

# 2. Remove STAT/MATT extraction
code = code.replace(
"""                        } else if child.id == "STAT" || child.id == "MATT" {
                            let mut child_clone = child.clone();
                            child_clone.id = "STAT".to_string();
                            original_stat_chunks.push(child_clone);
                        }""",
"""                        }"""
)

# 3. Remove original_stat_chunks usage
code = code.replace(
"""    // Preserve original STAT chunks when available, otherwise generate from materials
    if !original_stat_chunks.is_empty() {
        hvmd_children.extend(original_stat_chunks);
    } else if !model.materials.is_empty() {""",
"""    // Generate STAT chunks directly from materials so UI edits apply
    if !model.materials.is_empty() {"""
)

with open("parser/src/hod.rs", "w") as f:
    f.write(code)
