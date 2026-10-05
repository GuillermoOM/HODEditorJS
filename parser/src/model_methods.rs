use crate::math::*;
use crate::types::*;

pub fn normalize_collision_meshes(model: &mut HODModel) {
    if model.collision_meshes.is_empty() {
        return;
    }

    model.collision_meshes.truncate(1);
    let collision = &mut model.collision_meshes[0];
    collision.name = "Root".to_string();
    collision.mesh.name = "CollisionMesh".to_string();
    collision.mesh.parent_name = "Root".to_string();
}

impl HODModel {
    pub fn auto_repair_assembly_names(&mut self) {
        let mut renames: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        let suffixes = ["Heading", "Position", "Direction", "Rest", "Left", "Up"];

        for joint in &self.joints {
            let name = &joint.name;
            for suffix in &suffixes {
                if name.ends_with(suffix) && !name.ends_with(&format!("_{}", suffix)) {
                    let base = &name[..name.len() - suffix.len()];
                    if !base.is_empty() && !base.ends_with('_') {
                        let new_name = format!("{}_{}", base, suffix);
                        renames.insert(name.clone(), new_name);
                    }
                }
            }
            if let Some(idx) = name.find("Muzzle") {
                if idx > 0 && name.as_bytes()[idx - 1] != b'_' {
                    let base = &name[..idx];
                    let suffix = &name[idx..];
                    if !base.is_empty() && !base.ends_with('_') {
                        let new_name = format!("{}_{}", base, suffix);
                        renames.insert(name.clone(), new_name);
                    }
                }
            }
        }

        if renames.is_empty() {
            return;
        }

        for joint in &mut self.joints {
            if let Some(new_name) = renames.get(&joint.name) {
                joint.name = new_name.clone();
            }
            if let Some(parent_name) = &joint.parent_name {
                if let Some(new_parent) = renames.get(parent_name) {
                    joint.parent_name = Some(new_parent.clone());
                }
            }
        }

        for mesh in &mut self.meshes {
            if let Some(new_name) = renames.get(&mesh.parent_name) {
                mesh.parent_name = new_name.clone();
            }
        }

        for marker in &mut self.markers {
            if let Some(new_parent) = renames.get(&marker.parent_joint) {
                marker.parent_joint = new_parent.clone();
            }
        }

        for dockpath in &mut self.dockpaths {
            if let Some(new_parent) = renames.get(&dockpath.parent_name) {
                dockpath.parent_name = new_parent.clone();
            }
        }

        for anim in &mut self.animations {
            for track in &mut anim.tracks {
                if let Some(new_name) = renames.get(&track.joint_name) {
                    track.joint_name = new_name.clone();
                }
            }
        }
    }

    pub fn convert_weapon_to_turret(&mut self, base_name: &str) -> Result<(), String> {
        let pos_name = format!("{}_Position", base_name);

        let pos_exists = self
            .joints
            .iter()
            .any(|j| j.name.eq_ignore_ascii_case(&pos_name));
        if !pos_exists {
            return Err("Weapon assembly missing _Position joint, cannot convert.".to_string());
        }

        let dir_name = format!("{}_Direction", base_name);
        let head_name = format!("{}_Heading", base_name);

        let mut has_heading = self
            .joints
            .iter()
            .any(|j| j.name.eq_ignore_ascii_case(&head_name));
        if !has_heading {
            // Rename _Direction to _Heading
            if let Some(dir_joint) = self
                .joints
                .iter_mut()
                .find(|j| j.name.eq_ignore_ascii_case(&dir_name))
            {
                dir_joint.name = head_name.clone();
                has_heading = true;
            }
        }

        if !has_heading {
        let new_heading = HODJoint {
                name: head_name.clone(),
                parent_name: Some(pos_name.clone()),
                local_transform: compose_transform_matrix(
            Vector3 {
                        x: 0.0,
                        y: 0.0,
                        z: 5.0,
                    },
            Vector3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
            Vector3 {
                        x: 1.0,
                        y: 1.0,
                        z: 1.0,
                    },
                ),
                position: None,
                rotation: None,
                scale: None,
            };
            self.joints.push(new_heading);
        }

        // Make sure any children that used Direction as parent now use Heading
        for joint in &mut self.joints {
            if let Some(parent) = &joint.parent_name {
                if parent.eq_ignore_ascii_case(&dir_name) {
                    joint.parent_name = Some(head_name.clone());
                }
            }
        }
        for mesh in &mut self.meshes {
            if mesh.parent_name.eq_ignore_ascii_case(&dir_name) {
                mesh.parent_name = head_name.clone();
            }
        }

        let muzzle_name = format!("{}_Muzzle", base_name);
        let has_muzzle = self
            .joints
            .iter()
            .any(|j| j.name.eq_ignore_ascii_case(&muzzle_name));
        if !has_muzzle {
        let new_muzzle = HODJoint {
                name: muzzle_name,
                parent_name: Some(head_name), // Parented to Heading
                local_transform: compose_transform_matrix(
            Vector3 {
                        x: 0.0,
                        y: 0.0,
                        z: 5.0,
                    },
            Vector3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
            Vector3 {
                        x: 1.0,
                        y: 1.0,
                        z: 1.0,
                    },
                ),
                position: None,
                rotation: None,
                scale: None,
            };
            self.joints.push(new_muzzle);
        }

        Ok(())
    }

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
            joint.scale = Some(Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            });

            // Recompose matrix with the zeroed scale
            if let (Some(ref pos), Some(ref rot), Some(ref scale)) =
                (&joint.position, &joint.rotation, &joint.scale)
            {
                joint.local_transform =
                    compose_transform_matrix(pos.clone(), rot.clone(), scale.clone());
            }
        }

        self.is_v2 = true;
    }

    pub fn clean_hierarchy(&mut self) {
        // Special prefixes that shouldn't have arbitrary children
        let special_prefixes = ["NAVL[", "BURN[", "MARK[", "MULT[", "COL[", "SHAP[", "GLOW["];

        // Loop multiple times in case of deep invalid nesting
        let mut changed = true;
        while changed {
            changed = false;
            let mut updates = Vec::new(); // (child_idx, new_parent_name, new_transform)

            for (i, joint) in self.joints.iter().enumerate() {
                if let Some(parent_name) = &joint.parent_name {
                    // Check if parent name indicates a special node
                    let is_invalid_parent = special_prefixes
                        .iter()
                        .any(|&p| parent_name.starts_with(p) || parent_name == &p[..p.len() - 1]);

                    // Flame joints are allowed under BURN
                    let is_allowed_flame =
                        parent_name.starts_with("BURN[") && joint.name.starts_with("Flame[");

                    if is_invalid_parent && !is_allowed_flame {
                        // Find the parent joint to extract its transform and its parent
                        if let Some(parent_joint) =
                            self.joints.iter().find(|j| j.name == *parent_name)
                        {
                            let new_parent_name = parent_joint.parent_name.clone();
                            // Multiply parent's local transform by child's local transform
                            // In Homeworld matrices, transform = parent * child
                            let p_mat = &parent_joint.local_transform.m;
                            let c_mat = &joint.local_transform.m;

        let mut new_transform = Matrix4 { m: [[0.0; 4]; 4] };
                            for r in 0..4 {
                                for c in 0..4 {
                                    new_transform.m[r][c] = p_mat[r][0] * c_mat[0][c]
                                        + p_mat[r][1] * c_mat[1][c]
                                        + p_mat[r][2] * c_mat[2][c]
                                        + p_mat[r][3] * c_mat[3][c];
                                }
                            }

                            updates.push((i, new_parent_name, new_transform));
                            changed = true;
                        }
                    }
                }
            }

            // Apply updates
            for (idx, new_parent, new_transform) in updates {
                self.joints[idx].parent_name = new_parent;
                self.joints[idx].local_transform = new_transform.clone();
                // Update stored position/rotation/scale to match the cleaned transform
                let (pos, rot, scale) = decompose_matrix(new_transform);
                self.joints[idx].position = Some(pos);
                self.joints[idx].rotation = Some(rot);
                self.joints[idx].scale = Some(scale);
            }

            // --- FUNCTIONAL NODES CLEANUP ---
            // If any functional node has its parent_name pointing to a special endpoint, we un-nest it.
            // Functional nodes do not carry a full hierarchical local_transform matrix to adapt in the same way,
            // they simply inherit their parent's absolute space.

            let mut check_and_update = |parent_name_ref: &mut String| {
                if special_prefixes.iter().any(|&p| {
                    parent_name_ref.starts_with(p) || parent_name_ref == &p[..p.len() - 1]
                }) {
                    if let Some(parent_joint) =
                        self.joints.iter().find(|j| &j.name == parent_name_ref)
                    {
                        if let Some(new_p) = &parent_joint.parent_name {
                            *parent_name_ref = new_p.clone();
                            changed = true;
                        }
                    }
                }
            };

            for burn in &mut self.engine_burns {
                check_and_update(&mut burn.parent_name);
            }
            for glow in &mut self.engine_glows {
                check_and_update(&mut glow.parent_name);
            }
            for shape in &mut self.engine_shapes {
                check_and_update(&mut shape.parent_name);
            }
            for path in &mut self.dockpaths {
                check_and_update(&mut path.parent_name);
            }
            for mesh in &mut self.meshes {
                check_and_update(&mut mesh.parent_name);
            }
            for marker in &mut self.markers {
                check_and_update(&mut marker.parent_joint);
            }
        }

        // Remove self-parented identity joints that duplicate a real joint name.
        // These are NavLight endpoint artifacts in some HOD 2.0 files (e.g., EngineNozzle1-4).
        let mut names_with_real_joint = std::collections::HashSet::new();
        for joint in &self.joints {
            let is_self_parented = joint.parent_name.as_deref() == Some(&joint.name);
            if !is_self_parented {
                names_with_real_joint.insert(joint.name.clone());
            }
        }
        self.joints.retain(|j| {
            let is_self_parented = j.parent_name.as_deref() == Some(&j.name);
            if is_self_parented && names_with_real_joint.contains(&j.name) {
                println!(
                    "[RUST] clean_hierarchy: Removing duplicate self-parented joint '{}'",
                    j.name
                );
                return false;
            }
            true
        });
    }

    pub fn deduplicate_names(&mut self) {
        use std::collections::HashMap;

        let mut joint_names = HashMap::new();
        let mut renames = HashMap::new(); // Old Name -> New Name

        // First pass: Rename duplicate joints
        for joint in &mut self.joints {
            let count = joint_names.entry(joint.name.clone()).or_insert(0);
            *count += 1;
            if *count > 1 {
                let new_name = format!("{}_{}", joint.name, count);
                renames.insert(joint.name.clone(), new_name.clone());
                println!(
                    "[RUST] WARNING: Duplicate joint name '{}' found. Renamed to '{}'.",
                    joint.name, new_name
                );
                joint.name = new_name;
            }
        }

        let mut mesh_names = HashMap::new();
        for mesh in &mut self.meshes {
            // Use (name, lod) as key — LOD variants with the same name
            // are not duplicates, they're separate LODs of the same mesh.
            let key = format!("{}_{}", mesh.name, mesh.lod);
            let count = mesh_names.entry(key).or_insert(0);
            *count += 1;
            if *count > 1 {
                let new_name = format!("{}_{}", mesh.name, count);
                println!(
                    "[RUST] WARNING: Duplicate mesh name '{}' lod {} found. Renamed to '{}'.",
                    mesh.name, mesh.lod, new_name
                );
                mesh.name = new_name;
            }
        }
    }

    pub fn auto_assign_and_resize_textures(&mut self) {
        use base64::{engine::general_purpose, Engine as _};
        use image::imageops::FilterType;
        use image::io::Reader as ImageReader;
        use std::collections::HashMap;
        use std::io::Cursor;

        let mut textures_by_name = HashMap::new();
        for t in &self.textures {
            let lower = t
                .name
                .to_lowercase()
                .replace(".tga", "")
                .replace(".png", "")
                .replace(".dds", "");
            textures_by_name.insert(lower, t.clone());
        }

        let mut updated_textures = Vec::new();
        let mut texture_names_used = Vec::new();

        for mat in &mut self.materials {
            let shader = mat.shader_name.to_lowercase();
            let mut expected_slots = vec![
                "Diffuse Map (DIFF)",
                "Glow Map (GLOW)",
                "Team Paint Map (TEAM)",
                "Normal Map (NORM)",
            ];

            if shader.contains("badge") && !shader.contains("glow") {
                expected_slots = vec!["Diffuse Map (DIFF)"];
            } else if shader.contains("badgeglow") {
                expected_slots = vec!["Badge Diffuse Map (DIFF)", "Glow Map (GLOW)"];
            } else if shader.contains("thruster") {
                expected_slots = vec![
                    "Diffuse On (DIFF)",
                    "Glow On (GLOW)",
                    "Team Paint Map (TEAM)",
                    "Normal Map (NORM)",
                    "Diffuse Off (DIFF_OFF)",
                    "Glow Off (GLOW_OFF)",
                ];
            } else if shader.contains("ship") {
                expected_slots = vec![
                    "Diffuse Map (DIFF)",
                    "Glow Map (GLOW)",
                    "Team Paint Map (TEAM)",
                    "Normal Map (NORM)",
                    "Specular Map (SPEC)",
                ];
            } else if shader.contains("asteroid") {
                expected_slots = vec![
                    "Diffuse Map (DIFF)",
                    "Normal Map (NORM)",
                    "Specular Map (SPEC)",
                ];
            } else if shader.contains("cloud") || shader.contains("background") {
                expected_slots = vec!["Diffuse Map (DIFF)"];
            } else if shader.contains("resource") {
                expected_slots = vec![
                    "Diffuse Map (DIFF)",
                    "Glow Map (GLOW)",
                    "Normal Map (NORM)",
                    "Specular Map (SPEC)",
                ];
            }

            let mut mapped = Vec::new();
            let mut base_width = 0;
            let mut base_height = 0;

            for (idx, slot) in expected_slots.iter().enumerate() {
                let mut best_match = String::new();

                let is_match = |t_name: &str, s_name: &str| -> bool {
                    let mut tn = t_name.to_lowercase();
                    if tn.ends_with(".tga") {
                        tn.truncate(tn.len() - 4);
                    }
                    if tn.ends_with(".png") {
                        tn.truncate(tn.len() - 4);
                    }
                    if tn.ends_with(".dds") {
                        tn.truncate(tn.len() - 4);
                    }

                    if s_name.contains("GLOW") && (tn.contains("_glow") || tn.contains("glow")) {
                        return true;
                    }
                    if s_name.contains("TEAM") && (tn.contains("_team") || tn.contains("team")) {
                        return true;
                    }
                    if s_name.contains("NORM") && (tn.contains("_norm") || tn.contains("norm")) {
                        return true;
                    }
                    if s_name.contains("SPEC") && (tn.contains("_spec") || tn.contains("spec")) {
                        return true;
                    }
                    if s_name.contains("DIFF")
                        && !tn.contains("glow")
                        && !tn.contains("team")
                        && !tn.contains("norm")
                        && !tn.contains("spec")
                    {
                        return true;
                    }
                    false
                };

                for t_name in &mat.texture_maps {
                    if is_match(t_name, slot) {
                        best_match = t_name.clone();
                        break;
                    }
                }

                if best_match.is_empty() {
                    let mut expected_suffix = "";
                    if slot.contains("GLOW") {
                        expected_suffix = "_glow";
                    }
                    if slot.contains("TEAM") {
                        expected_suffix = "_team";
                    }
                    if slot.contains("NORM") {
                        expected_suffix = "_norm";
                    }
                    if slot.contains("SPEC") {
                        expected_suffix = "_spec";
                    }
                    if slot.contains("DIFF") {
                        expected_suffix = "_diff";
                    }

                    if !expected_suffix.is_empty() {
                        let potential_name = format!("{}{}", mat.name, expected_suffix);
                        if textures_by_name.contains_key(&potential_name.to_lowercase()) {
                            best_match = textures_by_name[&potential_name.to_lowercase()]
                                .name
                                .clone();
                        }
                    }
                }

                mapped.push(best_match.clone());

                if !best_match.is_empty() {
                    let tn_lower = best_match
                        .to_lowercase()
                        .replace(".tga", "")
                        .replace(".png", "");
                    if let Some(tex) = textures_by_name.get(&tn_lower) {
                        if idx == 0 || (base_width == 0 && base_height == 0) {
                            base_width = tex.width;
                            base_height = tex.height;
                        }

                        let mut new_tex = tex.clone();
                        if (new_tex.width != base_width || new_tex.height != base_height)
                            && base_width > 0
                            && base_height > 0
                        {
                            println!(
                                "[RUST] Autofixing texture '{}' dimensions from {}x{} to {}x{}",
                                new_tex.name,
                                new_tex.width,
                                new_tex.height,
                                base_width,
                                base_height
                            );
                            if let Some(png_data) = &new_tex.png_data {
                                if let Ok(decoded) = general_purpose::STANDARD.decode(png_data) {
                                    if let Ok(img) = ImageReader::new(Cursor::new(decoded))
                                        .with_guessed_format()
                                        .unwrap()
                                        .decode()
                                    {
                                        let resized = img.resize_exact(
                                            base_width,
                                            base_height,
                                            FilterType::Lanczos3,
                                        );
                                        let mut out_bytes = Vec::new();
                                        if resized
                                            .write_to(
                                                &mut Cursor::new(&mut out_bytes),
                                                image::ImageFormat::Png,
                                            )
                                            .is_ok()
                                        {
                                            new_tex.png_data =
                                                Some(general_purpose::STANDARD.encode(&out_bytes));

                                            let prev_resized = img.resize_exact(
                                                128.min(base_width),
                                                128.min(base_height),
                                                FilterType::Lanczos3,
                                            );
                                            let mut prev_bytes = Vec::new();
                                            if prev_resized
                                                .write_to(
                                                    &mut Cursor::new(&mut prev_bytes),
                                                    image::ImageFormat::Png,
                                                )
                                                .is_ok()
                                            {
                                                new_tex.png_preview = Some(
                                                    general_purpose::STANDARD.encode(&prev_bytes),
                                                );
                                            }

                                            new_tex.width = base_width;
                                            new_tex.height = base_height;
                                        }
                                    }
                                }
                            }
                        }

                        if !texture_names_used.contains(&new_tex.name) {
                            texture_names_used.push(new_tex.name.clone());
                            updated_textures.push(new_tex);
                        }
                    }
                }
            }

            mat.texture_maps = mapped;
        }

        for t in &self.textures {
            if !texture_names_used.contains(&t.name) {
                updated_textures.push(t.clone());
            }
        }

        self.textures = updated_textures;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity_matrix() -> Matrix4 {
        Matrix4 {
            m: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    #[test]
    fn model_methods_deduplicate_names_makes_unique() {
        let mut model = HODModel::new();
        model.joints = vec![
            HODJoint {
                name: "A".to_string(),
                parent_name: None,
                local_transform: identity_matrix(),
                position: None,
                rotation: None,
                scale: None,
            },
            HODJoint {
                name: "A".to_string(),
                parent_name: None,
                local_transform: identity_matrix(),
                position: None,
                rotation: None,
                scale: None,
            },
        ];

        model.deduplicate_names();

        assert_eq!(model.joints[0].name, "A");
        assert_eq!(model.joints[1].name, "A_2");
    }

    #[test]
    fn model_methods_clean_hierarchy_unnests_special_prefixes() {
        let mut model = HODModel::new();
        model.joints = vec![
            HODJoint {
                name: "Root".to_string(),
                parent_name: None,
                local_transform: identity_matrix(),
                position: None,
                rotation: None,
                scale: None,
            },
            HODJoint {
                name: "COL[0]".to_string(),
                parent_name: Some("Root".to_string()),
                local_transform: identity_matrix(),
                position: None,
                rotation: None,
                scale: None,
            },
            HODJoint {
                name: "NAVL[0]".to_string(),
                parent_name: Some("COL[0]".to_string()),
                local_transform: identity_matrix(),
                position: None,
                rotation: None,
                scale: None,
            },
        ];

        model.clean_hierarchy();

        assert_eq!(model.joints[2].parent_name.as_deref(), Some("Root"));
    }
}
