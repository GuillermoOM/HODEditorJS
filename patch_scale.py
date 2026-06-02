with open("parser/src/hod.rs", "r") as f:
    code = f.read()

func = """
    pub fn upgrade_v1_to_v2(&mut self) {
        if self.is_v2 {
            return;
        }
        
        // HOD 1.0 (VERS 1000) used centimeters. HOD 2.0 uses meters.
        // Scale down all geometry by 0.01
        for mesh in &mut self.meshes {
            for part in &mut mesh.parts {
                for v in &mut part.vertices {
                    v.position.x *= 0.01;
                    v.position.y *= 0.01;
                    v.position.z *= 0.01;
                }
            }
        }
        
        for joint in &mut self.joints {
            if let Some(ref mut pos) = joint.position {
                pos.x *= 0.01;
                pos.y *= 0.01;
                pos.z *= 0.01;
            }
            // In HOD 2.0, the scale field acts as gimbal limits, not geometry scale.
            // Using (1,1,1) in HOD 2.0 causes the engine to rotate the ship sideways.
            joint.scale = Some(Vector3 { x: 0.0, y: 0.0, z: 0.0 });
            
            // Recompose matrix
            if let (Some(ref pos), Some(ref rot), Some(ref scale)) = (&joint.position, &joint.rotation, &joint.scale) {
                joint.local_transform = compose_transform_matrix(pos.clone(), rot.clone(), scale.clone());
            }
        }
        
        for col in &mut self.collision_meshes {
            col.center.x *= 0.01;
            col.center.y *= 0.01;
            col.center.z *= 0.01;
            col.radius *= 0.01;
            col.min_extents.x *= 0.01;
            col.min_extents.y *= 0.01;
            col.min_extents.z *= 0.01;
            col.max_extents.x *= 0.01;
            col.max_extents.y *= 0.01;
            col.max_extents.z *= 0.01;
            for part in &mut col.mesh.parts {
                for v in &mut part.vertices {
                    v.position.x *= 0.01;
                    v.position.y *= 0.01;
                    v.position.z *= 0.01;
                }
            }
        }
        
        for marker in &mut self.markers {
            marker.position.x *= 0.01;
            marker.position.y *= 0.01;
            marker.position.z *= 0.01;
        }
        
        for nav in &mut self.nav_lights {
            nav.position.x *= 0.01;
            nav.position.y *= 0.01;
            nav.position.z *= 0.01;
        }
        
        self.is_v2 = true;
    }
"""

# Insert inside impl HODModel. Let's find `pub fn clean_hierarchy`
code = code.replace("    pub fn clean_hierarchy(&mut self) {", func + "\n    pub fn clean_hierarchy(&mut self) {")

call = """        // Upgrade V1 to V2 in memory
        if !model.is_v2 {
            model.upgrade_v1_to_v2();
        }

        Ok(model)"""
code = code.replace("        Ok(model)\n    }\n", call + "\n    }\n")

with open("parser/src/hod.rs", "w") as f:
    f.write(code)
