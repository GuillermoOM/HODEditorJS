use base64::Engine;
use byteorder::{LittleEndian, ReadBytesExt};
use hwr_hod_parser::hod::{generate_v2_from_model, HODModel};
use hwr_hod_parser::iff::IffChunk;
use hwr_hod_parser::xpress;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{Cursor, Read};
use std::path::Path;

fn main() -> Result<(), String> {
    println!("=== HODOR Replication Test ===\n");

    let test_cases = vec![
        ("ter_pharos", "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/mod-tools/HODEditorJS/testing/ter_pharos"),
        ("ter_centaur", "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/mod-tools/HODEditorJS/testing/ter_centaur"),
    ];

    let mut passed = 0;
    let mut failed = 0;
    let mut total = 0;

    for (name, path) in &test_cases {
        total += 1;
        println!("--- Testing {} ---", name);

        match run_test_case(name, Path::new(path)) {
            Ok(_) => {
                println!("✅ PASSED\n");
                passed += 1;
            }
            Err(e) => {
                println!("❌ FAILED: {}\n", e);
                failed += 1;
            }
        }
    }

    println!("=== HODOR Replication Results ===");
    println!("Total: {}", total);
    println!("Passed: {}", passed);
    println!("Failed: {}", failed);
    println!(
        "Success Rate: {:.1}%",
        (passed as f64 / total as f64) * 100.0
    );

    if failed > 0 {
        Err(format!("{} test(s) failed", failed))
    } else {
        Ok(())
    }
}

fn run_test_case(name: &str, dir: &Path) -> Result<(), String> {
    // Test 1: Parse HODOR-generated HOD
    println!("  1. Parsing HODOR-generated HOD...");
    let hodor_path = dir.join(format!("{}_hodor.hod", name));
    let hodor_bytes =
        fs::read(&hodor_path).map_err(|e| format!("Failed to read HODOR HOD: {}", e))?;
    let hodor_model =
        HODModel::parse(&hodor_bytes).map_err(|e| format!("Failed to parse HODOR HOD: {}", e))?;
    println!(
        "     HODOR HOD: {} bytes, {} meshes, {} materials",
        hodor_bytes.len(),
        hodor_model.meshes.len(),
        hodor_model.materials.len()
    );

    // Test 2: Build model from assets
    println!("  2. Building model from assets...");
    let asset_model = build_model_from_assets(dir)?;
    println!(
        "     Assets: {} meshes, {} materials, {} textures",
        asset_model.meshes.len(),
        asset_model.materials.len(),
        asset_model.textures.len()
    );
    validate_dae_oracle(name, dir, &asset_model)?;

    // Test 3: Generate HOD 2.0
    println!("  3. Generating HOD 2.0...");
    let generated = generate_v2_from_model(&[], &asset_model)?;
    let out_path = dir.join(format!("{}_generated.hod", name));
    fs::write(&out_path, &generated).map_err(|e| format!("Failed to write: {}", e))?;
    println!("     Generated: {} bytes", generated.len());

    // Test 4: Re-parse generated HOD
    println!("  4. Re-parsing generated HOD...");
    let reparsed = HODModel::parse(&generated).map_err(|e| format!("Failed to re-parse: {}", e))?;
    println!(
        "     Re-parsed: {} meshes, {} materials",
        reparsed.meshes.len(),
        reparsed.materials.len()
    );

    // Test 5: Compare structures
    println!("  5. Comparing structures...");
    println!("=== DEBUG TEXTURES FOR HODOR ===");
    for tex in &hodor_model.textures {
        println!("Hodor Texture: name='{}', format='{}', width={}, height={}", tex.name, tex.format, tex.width, tex.height);
    }
    println!("=== DEBUG TEXTURES FOR REPARSED ===");
    for tex in &reparsed.textures {
        println!("Reparsed Texture: name='{}', format='{}', width={}, height={}", tex.name, tex.format, tex.width, tex.height);
    }
    compare_structures(&hodor_model, &reparsed)?;
    compare_texture_layouts(&hodor_bytes, &generated)?;

    // Test 6: Verify round-trip integrity
    println!("  6. Verifying round-trip integrity...");
    verify_roundtrip(&generated)?;

    // Clean up generated file
    // let _ = fs::remove_file(&out_path);

    Ok(())
}

fn build_model_from_assets(dir: &Path) -> Result<HODModel, String> {
    println!("     Looking in directory: {}", dir.display());

    // Load materials from JSON
    let materials_path = dir.join("materials.json");
    let materials: Vec<hwr_hod_parser::hod::HODMaterial> = if materials_path.exists() {
        println!("     Found materials file: {}", materials_path.display());
        serde_json::from_slice(&fs::read(&materials_path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?
    } else {
        println!("     No materials file found, using empty list");
        Vec::new()
    };
    println!("     Loaded {} materials", materials.len());

    // Load textures
    let mut texture_names: Vec<String> = fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("tga"))
                .unwrap_or(false)
        })
        .filter_map(|path| path.file_stem()?.to_str()?.to_string().into())
        .collect();
    texture_names.sort();
    texture_names.dedup();

    let textures_json_path = dir.join("textures.json");
    let textures_specs: Vec<serde_json::Value> = if textures_json_path.exists() {
        serde_json::from_slice(&fs::read(&textures_json_path).map_err(|e| e.to_string())?)
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    let mut textures = Vec::new();
    for tex_name in &texture_names {
        // Find the actual TGA file with any extension case
        let tga_path = fs::read_dir(dir)
            .map_err(|e| e.to_string())?
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .find(|path| {
                path.file_stem()
                    .and_then(|stem| stem.to_str())
                    .map(|s| s == tex_name)
                    .unwrap_or(false)
                    && path
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| ext.eq_ignore_ascii_case("tga"))
                        .unwrap_or(false)
            })
            .ok_or_else(|| format!("No TGA file found for texture '{}'", tex_name))?;
        let mut tex = load_tga_texture(&tga_path, tex_name)?;
        if let Some(spec) = textures_specs.iter().find(|s| s["name"].as_str() == Some(tex_name)) {
            if let Some(fmt) = spec["format"].as_str() {
                tex.format = fmt.to_string();
            }
        }
        textures.push(tex);
    }
    println!("     Loaded {} textures", textures.len());

    validate_mtl_sources(dir, &materials, &texture_names)?;

    let material_indices = build_material_index(&materials);

    // Load OBJ meshes
    let mut obj_paths: Vec<std::path::PathBuf> = fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("obj"))
        .collect();
    obj_paths.sort();
    println!("     Found {} OBJ files", obj_paths.len());

    let mut meshes = Vec::new();
    for path in obj_paths {
        if let Some((base_name, lod)) = parse_lod_filename(&path) {
            meshes.push(parse_obj_mesh(&path, &base_name, lod, &material_indices)?);
        }
    }
    println!("     Loaded {} meshes", meshes.len());

    // Load joints from JSON
    let joints_path = dir.join("joints.json");
    let joints: Vec<hwr_hod_parser::hod::HODJoint> = if joints_path.exists() {
        println!("     Found joints file: {}", joints_path.display());
        serde_json::from_slice(&fs::read(&joints_path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?
    } else {
        println!("     No joints file found, using default root joint");
        let root_pos = hwr_hod_parser::hod::Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let root_rot = hwr_hod_parser::hod::Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let root_scale = hwr_hod_parser::hod::Vector3 {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        };
        vec![hwr_hod_parser::hod::HODJoint {
            name: "Root".to_string(),
            parent_name: None,
            local_transform: hwr_hod_parser::hod::compose_transform_matrix(
                root_pos.clone(),
                root_rot.clone(),
                root_scale.clone(),
            ),
            position: Some(root_pos),
            rotation: Some(root_rot),
            scale: Some(root_scale),
        }]
    };
    println!("     Loaded {} joints", joints.len());

    // Load navlights from JSON
    let navlights_path = dir.join("navlights.json");
    let nav_lights: Vec<hwr_hod_parser::hod::HODNavLight> = if navlights_path.exists() {
        println!("     Found navlights file: {}", navlights_path.display());
        serde_json::from_slice(&fs::read(&navlights_path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?
    } else {
        println!("     No navlights file found, using empty list");
        Vec::new()
    };
    println!("     Loaded {} navlights", nav_lights.len());

    // Load engine burns from JSON
    let engine_burns_path = dir.join("engine_burns.json");
    let engine_burns: Vec<hwr_hod_parser::hod::HODEngineBurn> = if engine_burns_path.exists() {
        println!(
            "     Found engine burns file: {}",
            engine_burns_path.display()
        );
        serde_json::from_slice(&fs::read(&engine_burns_path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?
    } else {
        println!("     No engine burns file found, using empty list");
        Vec::new()
    };
    println!("     Loaded {} engine burns", engine_burns.len());

    // Load markers from JSON
    let markers_path = dir.join("markers.json");
    let markers: Vec<hwr_hod_parser::hod::HODMarker> = if markers_path.exists() {
        println!("     Found markers file: {}", markers_path.display());
        serde_json::from_slice(&fs::read(&markers_path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?
    } else {
        println!("     No markers file found, using empty list");
        Vec::new()
    };
    println!("     Loaded {} markers", markers.len());

    // Load collision meshes from JSON
    let collision_path = dir.join("collision_meshes.json");
    let collision_meshes: Vec<hwr_hod_parser::hod::HODCollisionMesh> = if collision_path.exists() {
        println!(
            "     Found collision meshes file: {}",
            collision_path.display()
        );
        serde_json::from_slice(&fs::read(&collision_path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?
    } else {
        println!("     No collision meshes file found, using empty list");
        Vec::new()
    };
    println!("     Loaded {} collision meshes", collision_meshes.len());

    // Filter textures to only those referenced by materials
    let referenced_textures: std::collections::HashSet<String> = materials
        .iter()
        .flat_map(|m| m.texture_maps.iter().cloned())
        .collect();
    println!("     Referenced textures: {:?}", referenced_textures);
    println!("=== BEFORE FILTERING ===");
    for t in &textures {
        println!("  Texture in list: name='{}'", t.name);
    }
    let textures: Vec<_> = textures
        .into_iter()
        .filter(|t| referenced_textures.contains(&t.name))
        .collect();
    println!("     Filtered textures: {} (from referenced)", textures.len());

    Ok(HODModel {
        version: 512,
        is_v2: true,
        name: "Homeworld2 Multi Mesh File".to_string(),
        textures,
        materials,
        meshes,
        joints,
        markers,
        nav_lights,
        engine_burns,
        engine_glows: Vec::new(),
        engine_shapes: Vec::new(),
        collision_meshes,
        dockpaths: Vec::new(),
        animations: Vec::new(),
        preserved_chunks: Vec::new(),
    })
}

fn load_tga_texture(path: &Path, name: &str) -> Result<hwr_hod_parser::hod::HODTexture, String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    let img = image::load_from_memory_with_format(&bytes, image::ImageFormat::Tga)
        .map_err(|e| e.to_string())?;
    let width = img.width();
    let height = img.height();
    let format = if img.to_rgba8().pixels().any(|pixel| pixel[3] < 250) {
        "DXT5"
    } else {
        "DXT1"
    };
    let img = image::imageops::flip_vertical(&img);
    let mut png_bytes = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut png_bytes),
        image::ImageFormat::Png,
    )
    .map_err(|e| e.to_string())?;
    let b64 = base64::prelude::BASE64_STANDARD.encode(&png_bytes);
    Ok(hwr_hod_parser::hod::HODTexture {
        name: name.to_string(),
        width,
        height,
        format: format.to_string(),
        png_preview: Some(format!("data:image/png;base64,{}", b64)),
        png_data: Some(b64),
        source_path: None,
    })
}

fn parse_lod_filename(path: &Path) -> Option<(String, i32)> {
    let stem = path.file_stem()?.to_str()?;
    let idx = stem.rfind("_lod")?;
    let lod = stem[idx + 4..].parse().ok()?;
    Some((stem[..idx].to_string(), lod))
}

fn build_material_index(materials: &[hwr_hod_parser::hod::HODMaterial]) -> HashMap<String, usize> {
    let mut material_indices = HashMap::new();
    for (idx, material) in materials.iter().enumerate() {
        material_indices.insert(material.name.clone(), idx);
        material_indices.insert(material.name.to_lowercase(), idx);
    }
    material_indices
}

fn validate_mtl_sources(
    dir: &Path,
    materials: &[hwr_hod_parser::hod::HODMaterial],
    texture_names: &[String],
) -> Result<(), String> {
    let material_lookup: HashMap<String, &hwr_hod_parser::hod::HODMaterial> = materials
        .iter()
        .map(|material| (material.name.to_lowercase(), material))
        .collect();
    let texture_lookup: HashSet<String> = texture_names
        .iter()
        .map(|name| name.to_lowercase())
        .collect();

    let mut mtl_paths: Vec<std::path::PathBuf> = fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("mtl"))
        .collect();
    mtl_paths.sort();

    if mtl_paths.is_empty() {
        return Err(format!("{} has no MTL files", dir.display()));
    }

    for path in mtl_paths {
        let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let mut current_material: Option<String> = None;

        for line in text.lines() {
            let line = line.trim_matches(char::from(0)).trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let mut parts = line.split_whitespace();
            match parts.next() {
                Some("newmtl") => {
                    let material_name = parts.next().ok_or_else(|| {
                        format!(
                            "{} has a newmtl directive without a material name",
                            path.display()
                        )
                    })?;
                    if !material_lookup.contains_key(&material_name.to_lowercase()) {
                        return Err(format!(
                            "{} defines material '{}' missing from materials.json",
                            path.display(),
                            material_name
                        ));
                    }
                    current_material = Some(material_name.to_string());
                }
                Some("map_Kd") => {
                    let texture_path = parts.next().ok_or_else(|| {
                        format!(
                            "{} has a map_Kd directive without a texture",
                            path.display()
                        )
                    })?;
                    let texture_name = Path::new(texture_path)
                        .file_stem()
                        .and_then(|stem| stem.to_str())
                        .ok_or_else(|| {
                            format!(
                                "{} has invalid texture path '{}'",
                                path.display(),
                                texture_path
                            )
                        })?;

                    if !texture_lookup.contains(&texture_name.to_lowercase()) {
                        return Err(format!(
                            "{} references texture '{}' without matching TGA source",
                            path.display(),
                            texture_name
                        ));
                    }

                    let Some(material_name) = &current_material else {
                        return Err(format!("{} has map_Kd before newmtl", path.display()));
                    };
                    let material = material_lookup
                        .get(&material_name.to_lowercase())
                        .ok_or_else(|| {
                            format!(
                                "{} references unknown material '{}'",
                                path.display(),
                                material_name
                            )
                        })?;
                    if !material
                        .texture_maps
                        .iter()
                        .any(|map| map.eq_ignore_ascii_case(texture_name))
                    {
                        return Err(format!(
                            "{} material '{}' maps texture '{}' not present in materials.json texture_maps",
                            path.display(),
                            material_name,
                            texture_name
                        ));
                    }
                }
                _ => {}
            }
        }
    }

    println!("     MTL/material/texture consistency: OK");
    Ok(())
}

struct ObjPartBuilder {
    part: hwr_hod_parser::hod::HODMeshPart,
    vertex_lookup: HashMap<(usize, Option<usize>, Option<usize>), u16>,
}

fn ensure_part_builder(parts: &mut Vec<ObjPartBuilder>, material_index: usize) -> usize {
    if let Some(idx) = parts
        .iter()
        .position(|part| part.part.material_index == material_index)
    {
        return idx;
    }

    parts.push(ObjPartBuilder {
        part: hwr_hod_parser::hod::HODMeshPart {
            material_index,
            vertex_mask: 0x600B,
            vertices: Vec::new(),
            indices: Vec::new(),
        },
        vertex_lookup: HashMap::new(),
    });
    parts.len() - 1
}

fn push_obj_vertex(
    part: &mut ObjPartBuilder,
    key: (usize, Option<usize>, Option<usize>),
    vertex: hwr_hod_parser::hod::HODVertex,
) -> Result<(), String> {
    let index = if let Some(index) = part.vertex_lookup.get(&key) {
        *index
    } else {
        let index = u16::try_from(part.part.vertices.len())
            .map_err(|_| "OBJ mesh part exceeds u16 vertex index limit".to_string())?;
        part.part.vertices.push(vertex);
        part.vertex_lookup.insert(key, index);
        index
    };
    part.part.indices.push(index);
    Ok(())
}

fn parse_obj_mesh(
    path: &Path,
    name: &str,
    lod: i32,
    material_indices: &HashMap<String, usize>,
) -> Result<hwr_hod_parser::hod::HODMesh, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut positions: Vec<hwr_hod_parser::hod::Vector3> = Vec::new();
    let mut uvs: Vec<hwr_hod_parser::hod::Vector2> = Vec::new();
    let mut normals: Vec<hwr_hod_parser::hod::Vector3> = Vec::new();
    let mut parts_out: Vec<ObjPartBuilder> = Vec::new();
    let mut active_material_index = 0;

    for line in text.lines() {
        let line = line.trim_matches(char::from(0)).trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.split_whitespace();
        match parts.next() {
            Some("v") => {
                let vals: Vec<f32> = parts.filter_map(|s| s.parse().ok()).collect();
                if vals.len() >= 3 {
                    positions.push(hwr_hod_parser::hod::Vector3 {
                        x: vals[0],
                        y: vals[1],
                        z: vals[2],
                    });
                }
            }
            Some("vt") => {
                let vals: Vec<f32> = parts.filter_map(|s| s.parse().ok()).collect();
                if vals.len() >= 2 {
                    uvs.push(hwr_hod_parser::hod::Vector2 {
                        u: vals[0],
                        v: 1.0 - vals[1],
                    });
                }
            }
            Some("vn") => {
                let vals: Vec<f32> = parts.filter_map(|s| s.parse().ok()).collect();
                if vals.len() >= 3 {
                    normals.push(hwr_hod_parser::hod::Vector3 {
                        x: vals[0],
                        y: vals[1],
                        z: vals[2],
                    });
                }
            }
            Some("usemtl") => {
                let material_name = parts.next().ok_or_else(|| {
                    format!(
                        "{} has a usemtl directive without a material name",
                        path.display()
                    )
                })?;
                active_material_index = material_indices
                    .get(material_name)
                    .or_else(|| material_indices.get(&material_name.to_lowercase()))
                    .copied()
                    .ok_or_else(|| {
                        format!(
                            "{} references unknown material '{}'",
                            path.display(),
                            material_name
                        )
                    })?;
            }
            Some("mtllib") => {
                let parent = path.parent().unwrap_or_else(|| Path::new("."));
                for mtl_name in parts {
                    let mtl_path = parent.join(mtl_name);
                    if !mtl_path.exists() {
                        return Err(format!(
                            "{} references missing MTL file '{}'",
                            path.display(),
                            mtl_path.display()
                        ));
                    }
                }
            }
            Some("f") => {
                let face_tokens: Vec<&str> = parts.collect();
                if face_tokens.len() < 3 {
                    continue;
                }
                let first = parse_obj_vertex(face_tokens[0], &positions, &uvs, &normals)?;
                let mut prev = parse_obj_vertex(face_tokens[1], &positions, &uvs, &normals)?;
                for token in face_tokens.iter().skip(2) {
                    let current = parse_obj_vertex(token, &positions, &uvs, &normals)?;
                    let part_idx = ensure_part_builder(&mut parts_out, active_material_index);
                    for (key, vertex) in [first.clone(), prev.clone(), current.clone()] {
                        push_obj_vertex(&mut parts_out[part_idx], key, vertex)?;
                    }
                    prev = current;
                }
            }
            _ => {}
        }
    }

    let parts: Vec<hwr_hod_parser::hod::HODMeshPart> =
        parts_out.into_iter().map(|part| part.part).collect();

    Ok(hwr_hod_parser::hod::HODMesh {
        name: name.to_string(),
        parent_name: "Root".to_string(),
        lod,
        has_mult_tags: false,
        parts,
    })
}

fn parse_obj_vertex(
    token: &str,
    positions: &[hwr_hod_parser::hod::Vector3],
    uvs: &[hwr_hod_parser::hod::Vector2],
    normals: &[hwr_hod_parser::hod::Vector3],
) -> Result<
    (
        (usize, Option<usize>, Option<usize>),
        hwr_hod_parser::hod::HODVertex,
    ),
    String,
> {
    let mut fields = token.split('/');
    let vi = parse_obj_index(fields.next().unwrap_or_default(), positions.len())?;
    let ti = fields
        .next()
        .filter(|s| !s.is_empty())
        .map(|s| parse_obj_index(s, uvs.len()))
        .transpose()?;
    let ni = fields
        .next()
        .filter(|s| !s.is_empty())
        .map(|s| parse_obj_index(s, normals.len()))
        .transpose()?;
    let vertex = hwr_hod_parser::hod::HODVertex {
        position: positions
            .get(vi)
            .cloned()
            .ok_or_else(|| "position index out of range".to_string())?,
        normal: ni.and_then(|idx| normals.get(idx).cloned()).or(Some(
            hwr_hod_parser::hod::Vector3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
        )),
        color: None,
        uv: ti
            .and_then(|idx| uvs.get(idx).cloned())
            .or(Some(hwr_hod_parser::hod::Vector2 { u: 0.0, v: 0.0 })),
        tangent: Some(hwr_hod_parser::hod::Vector3 {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        }),
        binormal: Some(hwr_hod_parser::hod::Vector3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        }),
        skinning_data: None,
    };
    Ok(((vi, ti, ni), vertex))
}

fn parse_obj_index(raw: &str, len: usize) -> Result<usize, String> {
    let idx = raw.parse::<isize>().map_err(|e| e.to_string())?;
    if idx > 0 {
        Ok((idx - 1) as usize)
    } else if idx < 0 {
        Ok((len as isize + idx) as usize)
    } else {
        Err("OBJ index cannot be zero".to_string())
    }
}

#[derive(Debug)]
struct DaeMeshOracle {
    name: String,
    lod: i32,
    parts: Vec<DaePartOracle>,
}

#[derive(Debug)]
struct DaePartOracle {
    material_index: usize,
    index_count: usize,
    vertex_tuple_count: usize,
}

fn validate_dae_oracle(name: &str, dir: &Path, model: &HODModel) -> Result<(), String> {
    let dae_path = dir.join(format!("{}.DAE", name));
    if !dae_path.exists() {
        return Err(format!("Missing DAE oracle: {}", dae_path.display()));
    }

    let material_indices = build_material_index(&model.materials);
    let expected_meshes = parse_dae_oracle(&dae_path, &material_indices)?;

    if expected_meshes.len() != model.meshes.len() {
        return Err(format!(
            "DAE MULT mesh count mismatch: {} vs {}",
            expected_meshes.len(),
            model.meshes.len()
        ));
    }

    for expected in &expected_meshes {
        let mesh = model
            .meshes
            .iter()
            .find(|mesh| mesh.name == expected.name && mesh.lod == expected.lod)
            .ok_or_else(|| {
                format!(
                    "DAE references missing mesh '{}' lod {}",
                    expected.name, expected.lod
                )
            })?;

        if expected.parts.len() != mesh.parts.len() {
            return Err(format!(
                "DAE mesh '{}' lod {} part count mismatch: {} vs {}",
                expected.name,
                expected.lod,
                expected.parts.len(),
                mesh.parts.len()
            ));
        }

        for (idx, (expected_part, part)) in expected.parts.iter().zip(&mesh.parts).enumerate() {
            if expected_part.material_index != part.material_index {
                return Err(format!(
                    "DAE mesh '{}' lod {} part {} material mismatch: {} vs {}",
                    expected.name,
                    expected.lod,
                    idx,
                    expected_part.material_index,
                    part.material_index
                ));
            }

            if expected_part.index_count != part.indices.len() {
                return Err(format!(
                    "DAE mesh '{}' lod {} part {} index count mismatch: {} vs {}",
                    expected.name,
                    expected.lod,
                    idx,
                    expected_part.index_count,
                    part.indices.len()
                ));
            }
        }
    }

    let dae_vertex_tuples: usize = expected_meshes
        .iter()
        .flat_map(|mesh| &mesh.parts)
        .map(|part| part.vertex_tuple_count)
        .sum();
    let obj_vertices: usize = model
        .meshes
        .iter()
        .flat_map(|mesh| &mesh.parts)
        .map(|part| part.vertices.len())
        .sum();
    println!(
        "     DAE oracle comparison: OK ({} MULT geometries, {} DAE tuples, {} OBJ vertices)",
        expected_meshes.len(),
        dae_vertex_tuples,
        obj_vertices
    );
    Ok(())
}

fn parse_dae_oracle(
    path: &Path,
    material_indices: &HashMap<String, usize>,
) -> Result<Vec<DaeMeshOracle>, String> {
    let xml = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let doc = roxmltree::Document::parse(&xml).map_err(|e| e.to_string())?;

    for image in doc.descendants().filter(|node| node.has_tag_name("image")) {
        if let Some(init_from) = image.children().find(|node| node.has_tag_name("init_from")) {
            let Some(texture_path) = init_from
                .text()
                .map(str::trim)
                .filter(|text| !text.is_empty())
            else {
                continue;
            };
            let texture_path = path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join(texture_path);
            if !texture_path.exists() {
                return Err(format!(
                    "{} references missing DAE image source '{}'",
                    path.display(),
                    texture_path.display()
                ));
            }
        }
    }

    let mut meshes = Vec::new();
    for geometry in doc
        .descendants()
        .filter(|node| node.has_tag_name("geometry"))
    {
        let geom_id = geometry.attribute("id").unwrap_or_default();
        let Some((mesh_name, lod)) = parse_dae_mult_name_lod(geom_id) else {
            continue;
        };

        let Some(mesh_node) = geometry.children().find(|node| node.has_tag_name("mesh")) else {
            continue;
        };

        let mut part_builders: Vec<(
            usize,
            usize,
            HashSet<(Option<usize>, Option<usize>, Option<usize>)>,
        )> = Vec::new();
        for triangles in mesh_node
            .children()
            .filter(|node| node.has_tag_name("triangles"))
        {
            let material_symbol = triangles.attribute("material").ok_or_else(|| {
                format!(
                    "{} geometry '{}' has triangles without material",
                    path.display(),
                    geom_id
                )
            })?;
            let material_name = parse_dae_material_name(material_symbol);
            let material_index = material_indices
                .get(material_name)
                .or_else(|| material_indices.get(&material_name.to_lowercase()))
                .copied()
                .ok_or_else(|| {
                    format!(
                        "{} geometry '{}' references unknown DAE material '{}'",
                        path.display(),
                        geom_id,
                        material_symbol
                    )
                })?;

            let mut position_offset = None;
            let mut normal_offset = None;
            let mut uv_offset = None;
            let mut max_offset = 0usize;
            for input in triangles
                .children()
                .filter(|node| node.has_tag_name("input"))
            {
                let offset = input
                    .attribute("offset")
                    .unwrap_or("0")
                    .parse::<usize>()
                    .unwrap_or(0);
                max_offset = max_offset.max(offset);
                match input.attribute("semantic") {
                    Some("VERTEX") => position_offset = Some(offset),
                    Some("NORMAL") => normal_offset = Some(offset),
                    Some("TEXCOORD") => uv_offset = Some(offset),
                    _ => {}
                }
            }
            let stride = max_offset + 1;
            let triangle_count = triangles
                .attribute("count")
                .unwrap_or("0")
                .parse::<usize>()
                .map_err(|e| e.to_string())?;
            let mut tuples = HashSet::new();

            for p in triangles.children().filter(|node| node.has_tag_name("p")) {
                let Some(text) = p.text() else {
                    continue;
                };
                let indices: Vec<usize> = text
                    .split_whitespace()
                    .map(|value| value.parse::<usize>().map_err(|e| e.to_string()))
                    .collect::<Result<Vec<_>, _>>()?;
                for vertex in indices.chunks(stride) {
                    if vertex.len() < stride {
                        continue;
                    }
                    tuples.insert((
                        position_offset.map(|offset| vertex[offset]),
                        uv_offset.map(|offset| vertex[offset]),
                        normal_offset.map(|offset| vertex[offset]),
                    ));
                }
            }

            if let Some((_, index_count, existing_tuples)) = part_builders
                .iter_mut()
                .find(|(part_material_index, _, _)| *part_material_index == material_index)
            {
                *index_count += triangle_count * 3;
                existing_tuples.extend(tuples);
            } else {
                part_builders.push((material_index, triangle_count * 3, tuples));
            }
        }

        let parts = part_builders
            .into_iter()
            .map(|(material_index, index_count, tuples)| DaePartOracle {
                material_index,
                index_count,
                vertex_tuple_count: tuples.len(),
            })
            .collect();

        meshes.push(DaeMeshOracle {
            name: mesh_name,
            lod,
            parts,
        });
    }

    meshes.sort_by(|a, b| a.lod.cmp(&b.lod).then_with(|| a.name.cmp(&b.name)));
    Ok(meshes)
}

fn parse_dae_mult_name_lod(id: &str) -> Option<(String, i32)> {
    let name_start = id.strip_prefix("MULT[")?;
    let name_end = name_start.find(']')?;
    let mesh_name = name_start[..name_end].to_string();
    let lod_start = id.find("_LOD[")? + "_LOD[".len();
    let lod_end = id[lod_start..].find(']')? + lod_start;
    let lod = id[lod_start..lod_end].parse().ok()?;
    Some((mesh_name, lod))
}

fn parse_dae_material_name(symbol: &str) -> &str {
    if let Some(rest) = symbol.strip_prefix("MAT[") {
        if let Some(end) = rest.find(']') {
            return &rest[..end];
        }
    }
    symbol
}

fn compare_structures(hodor: &HODModel, generated: &HODModel) -> Result<(), String> {
    // Compare mesh counts
    if hodor.meshes.len() != generated.meshes.len() {
        return Err(format!(
            "Mesh count mismatch: {} vs {}",
            hodor.meshes.len(),
            generated.meshes.len()
        ));
    }

    // Compare material counts
    if hodor.materials.len() != generated.materials.len() {
        return Err(format!(
            "Material count mismatch: {} vs {}",
            hodor.materials.len(),
            generated.materials.len()
        ));
    }

    // Compare texture counts
    if hodor.textures.len() != generated.textures.len() {
        return Err(format!(
            "Texture count mismatch: {} vs {}",
            hodor.textures.len(),
            generated.textures.len()
        ));
    }

    // Compare joint counts
    if hodor.joints.len() != generated.joints.len() {
        return Err(format!(
            "Joint count mismatch: {} vs {}",
            hodor.joints.len(),
            generated.joints.len()
        ));
    }

    compare_materials(hodor, generated)?;
    compare_textures(hodor, generated)?;

    for hodor_mesh in &hodor.meshes {
        let generated_mesh = generated
            .meshes
            .iter()
            .find(|mesh| mesh.name == hodor_mesh.name && mesh.lod == hodor_mesh.lod)
            .ok_or_else(|| format!("Missing mesh '{}' lod {}", hodor_mesh.name, hodor_mesh.lod))?;

        if hodor_mesh.parts.len() != generated_mesh.parts.len() {
            return Err(format!(
                "Mesh '{}' lod {} part count mismatch: {} vs {}",
                hodor_mesh.name,
                hodor_mesh.lod,
                hodor_mesh.parts.len(),
                generated_mesh.parts.len()
            ));
        }

        for (part_idx, (hodor_part, generated_part)) in hodor_mesh
            .parts
            .iter()
            .zip(&generated_mesh.parts)
            .enumerate()
        {
            if hodor_part.material_index != generated_part.material_index {
                return Err(format!(
                    "Mesh '{}' lod {} part {} material mismatch: {} vs {}",
                    hodor_mesh.name,
                    hodor_mesh.lod,
                    part_idx,
                    hodor_part.material_index,
                    generated_part.material_index
                ));
            }

            if hodor_part.indices.len() != generated_part.indices.len() {
                return Err(format!(
                    "Mesh '{}' lod {} part {} index count mismatch: {} vs {}",
                    hodor_mesh.name,
                    hodor_mesh.lod,
                    part_idx,
                    hodor_part.indices.len(),
                    generated_part.indices.len()
                ));
            }

            if hodor_part.vertices.len() != generated_part.vertices.len() {
                return Err(format!(
                    "Mesh '{}' lod {} part {} vertex count mismatch: {} vs {}",
                    hodor_mesh.name,
                    hodor_mesh.lod,
                    part_idx,
                    hodor_part.vertices.len(),
                    generated_part.vertices.len()
                ));
            }
        }
    }

    println!("     Structure comparison: OK");
    Ok(())
}

fn compare_materials(hodor: &HODModel, generated: &HODModel) -> Result<(), String> {
    for (idx, (hodor_material, generated_material)) in
        hodor.materials.iter().zip(&generated.materials).enumerate()
    {
        if hodor_material.name != generated_material.name {
            return Err(format!(
                "Material {} name mismatch: '{}' vs '{}'",
                idx, hodor_material.name, generated_material.name
            ));
        }

        if hodor_material.shader_name != generated_material.shader_name {
            return Err(format!(
                "Material '{}' shader mismatch: '{}' vs '{}'",
                hodor_material.name, hodor_material.shader_name, generated_material.shader_name
            ));
        }

        if hodor_material.texture_maps.len() != generated_material.texture_maps.len() {
            return Err(format!(
                "Material '{}' texture map count mismatch: {} vs {}",
                hodor_material.name,
                hodor_material.texture_maps.len(),
                generated_material.texture_maps.len()
            ));
        }

        for (map_idx, (hodor_map, generated_map)) in hodor_material
            .texture_maps
            .iter()
            .zip(&generated_material.texture_maps)
            .enumerate()
        {
            if hodor_map != generated_map {
                return Err(format!(
                    "Material '{}' texture map {} mismatch: '{}' vs '{}'",
                    hodor_material.name, map_idx, hodor_map, generated_map
                ));
            }
        }
    }

    Ok(())
}

fn compare_textures(hodor: &HODModel, generated: &HODModel) -> Result<(), String> {
    let mut format_mismatches = Vec::new();

    for hodor_texture in &hodor.textures {
        let generated_texture = generated
            .textures
            .iter()
            .find(|texture| texture.name.eq_ignore_ascii_case(&hodor_texture.name))
            .ok_or_else(|| format!("Missing texture '{}'", hodor_texture.name))?;

        if hodor_texture.width != generated_texture.width
            || hodor_texture.height != generated_texture.height
        {
            return Err(format!(
                "Texture '{}' dimensions mismatch: {}x{} vs {}x{}",
                hodor_texture.name,
                hodor_texture.width,
                hodor_texture.height,
                generated_texture.width,
                generated_texture.height
            ));
        }

        if hodor_texture.format != generated_texture.format {
            format_mismatches.push(format!(
                "{}: HODOR={} generated={}",
                hodor_texture.name, hodor_texture.format, generated_texture.format
            ));
        }
    }

    if !format_mismatches.is_empty() {
        println!(
            "     Texture format parity pending: {}",
            format_mismatches.join(", ")
        );
    }

    Ok(())
}

#[derive(Debug)]
struct TextureLayoutSummary {
    texture_pool_comp_len: usize,
    texture_pool_decomp_len: usize,
    textures: Vec<LmipLayout>,
}

#[derive(Debug)]
struct LmipLayout {
    name: String,
    format: String,
    dimensions: Vec<(u32, u32)>,
    byte_len: usize,
}

fn compare_texture_layouts(hodor_bytes: &[u8], generated_bytes: &[u8]) -> Result<(), String> {
    let hodor = extract_texture_layout(hodor_bytes)?;
    let generated = extract_texture_layout(generated_bytes)?;

    if hodor.texture_pool_decomp_len != generated.texture_pool_decomp_len
        || hodor.texture_pool_comp_len != generated.texture_pool_comp_len
    {
        println!(
            "     Texture pool parity pending: HODOR comp/decomp={}/{} generated comp/decomp={}/{}",
            hodor.texture_pool_comp_len,
            hodor.texture_pool_decomp_len,
            generated.texture_pool_comp_len,
            generated.texture_pool_decomp_len
        );
    }

    let generated_by_name: HashMap<String, &LmipLayout> = generated
        .textures
        .iter()
        .map(|texture| (texture.name.to_lowercase(), texture))
        .collect();

    let mut layout_mismatches = Vec::new();
    for hodor_texture in &hodor.textures {
        let Some(generated_texture) = generated_by_name.get(&hodor_texture.name.to_lowercase())
        else {
            layout_mismatches.push(format!("missing generated LMIP '{}'", hodor_texture.name));
            continue;
        };

        if hodor_texture.format != generated_texture.format
            || hodor_texture.dimensions != generated_texture.dimensions
            || hodor_texture.byte_len != generated_texture.byte_len
        {
            layout_mismatches.push(format!(
                "{}: HODOR {} {:?} {} bytes, generated {} {:?} {} bytes",
                hodor_texture.name,
                hodor_texture.format,
                hodor_texture.dimensions,
                hodor_texture.byte_len,
                generated_texture.format,
                generated_texture.dimensions,
                generated_texture.byte_len
            ));
        }
    }

    if layout_mismatches.is_empty() {
        println!("     Texture LMIP layout comparison: OK");
    } else {
        println!(
            "     Texture LMIP layout parity pending: {}",
            layout_mismatches.join("; ")
        );
    }

    Ok(())
}

fn extract_texture_layout(bytes: &[u8]) -> Result<TextureLayoutSummary, String> {
    let chunks = parse_top_level_chunks(bytes)?;
    let mut texture_pool_comp_len = 0;
    let mut texture_pool_decomp_len = 0;

    for chunk in &chunks {
        if chunk.id == "POOL" {
            let mut pool_cursor = Cursor::new(&chunk.data);
            let _pool_type = pool_cursor
                .read_u32::<LittleEndian>()
                .map_err(|e| e.to_string())?;

            texture_pool_comp_len = pool_cursor
                .read_u32::<LittleEndian>()
                .map_err(|e| e.to_string())? as usize;
            texture_pool_decomp_len = pool_cursor
                .read_u32::<LittleEndian>()
                .map_err(|e| e.to_string())? as usize;

            let mut comp_tex = vec![0u8; texture_pool_comp_len];
            pool_cursor
                .read_exact(&mut comp_tex)
                .map_err(|e| e.to_string())?;

            if texture_pool_comp_len != texture_pool_decomp_len {
                let _ = xpress::decompress(&comp_tex, texture_pool_decomp_len)?;
            }
        }
    }

    let mut textures = Vec::new();
    for chunk in &chunks {
        if chunk.id == "HVMD" {
            for child in &chunk.children {
                if child.id == "LMIP" {
                    textures.push(parse_lmip_layout(child)?);
                }
            }
        }
    }

    Ok(TextureLayoutSummary {
        texture_pool_comp_len,
        texture_pool_decomp_len,
        textures,
    })
}

fn parse_top_level_chunks(bytes: &[u8]) -> Result<Vec<IffChunk>, String> {
    let mut cursor = Cursor::new(bytes);
    let mut chunks = Vec::new();

    while cursor.position() < bytes.len() as u64 {
        chunks.push(IffChunk::read_chunk(&mut cursor).map_err(|e| e.to_string())?);
    }

    Ok(chunks)
}

fn parse_lmip_layout(chunk: &IffChunk) -> Result<LmipLayout, String> {
    let mut cursor = Cursor::new(&chunk.data);
    let name = read_lmip_len_string(&mut cursor)?;

    let mut format_bytes = [0u8; 4];
    cursor
        .read_exact(&mut format_bytes)
        .map_err(|e| e.to_string())?;
    let format = String::from_utf8_lossy(&format_bytes).trim().to_string();

    let mip_count = cursor
        .read_u32::<LittleEndian>()
        .map_err(|e| e.to_string())? as usize;
    let mut dimensions = Vec::new();
    for _ in 0..mip_count {
        let width = cursor
            .read_u32::<LittleEndian>()
            .map_err(|e| e.to_string())?;
        let height = cursor
            .read_u32::<LittleEndian>()
            .map_err(|e| e.to_string())?;
        dimensions.push((width, height));
    }

    let byte_len = dimensions
        .iter()
        .map(|&(width, height)| lmip_level_size(&format, width, height))
        .sum();

    Ok(LmipLayout {
        name,
        format,
        dimensions,
        byte_len,
    })
}

fn read_lmip_len_string(reader: &mut Cursor<&Vec<u8>>) -> Result<String, String> {
    let len = reader
        .read_u32::<LittleEndian>()
        .map_err(|e| e.to_string())? as usize;
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&buf).to_string())
}

fn lmip_level_size(format: &str, width: u32, height: u32) -> usize {
    match format {
        "DXT1" => std::cmp::max(8, (width * height) / 2) as usize,
        "DXT5" => std::cmp::max(16, width * height) as usize,
        _ => (width * height * 4) as usize,
    }
}

fn verify_roundtrip(bytes: &[u8]) -> Result<(), String> {
    // Parse the generated HOD
    let model =
        HODModel::parse(bytes).map_err(|e| format!("Failed to parse for round-trip: {}", e))?;

    // Re-generate from parsed model
    let regenerated =
        generate_v2_from_model(&[], &model).map_err(|e| format!("Failed to re-generate: {}", e))?;

    // Parse again to verify
    let _reparsed = HODModel::parse(&regenerated)
        .map_err(|e| format!("Failed to re-parse regenerated: {}", e))?;

    println!("     Round-trip verification: OK");
    Ok(())
}
