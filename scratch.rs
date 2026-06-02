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

                    if s_name.contains("GLOW") && tn.ends_with("_glow") {
                        return true;
                    }
                    if s_name.contains("TEAM") && tn.ends_with("_team") {
                        return true;
                    }
                    if s_name.contains("NORM") && tn.ends_with("_norm") {
                        return true;
                    }
                    if s_name.contains("SPEC") && tn.ends_with("_spec") {
                        return true;
                    }
                    if s_name.contains("DIFF")
                        && !tn.ends_with("_glow")
                        && !tn.ends_with("_team")
                        && !tn.ends_with("_norm")
                        && !tn.ends_with("_spec")
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

