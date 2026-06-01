use base64::prelude::*;
use hwr_hod_parser::hod::{
    compose_transform_matrix, generate_v2_from_model, HODJoint, HODMaterial, HODMesh, HODMeshPart,
    HODModel, HODTexture, HODVertex, Vector2, Vector3,
};
use hwr_hod_parser::shader_map::ShadersMap;
use serde_json;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

fn main() -> Result<(), String> {
    let root = Path::new("../testing");
    for name in ["pebble_0", "pebble_1", "pebble_2"] {
        let dir = root.join(name);
        println!("\n=== {} ===", name);
        run_fixture(&dir, name)?;
    }
    Ok(())
}

fn run_fixture(dir: &Path, name: &str) -> Result<(), String> {
    let vanilla_path = dir.join(format!("{}_vanilla.hod", name));
    let vanilla_bytes = fs::read(&vanilla_path).map_err(|e| e.to_string())?;
    let vanilla_model = HODModel::parse_with_external(
        &vanilla_bytes,
        Some(vanilla_path.to_string_lossy().as_ref()),
        Some(dir.to_string_lossy().as_ref()),
    )?;

    let asset_model = build_model_from_assets(dir)?;
    let mut asset_model_with_kdop = asset_model;
    // Copy preserved KDOP from vanilla so from-assets has it
    for chunk in &vanilla_model.preserved_chunks {
        if chunk.id == "KDOP" || chunk.id == "COLD" || chunk.id == "INFO" {
            println!(
                "  copying preserved {} from vanilla (data={} children={})",
                chunk.id,
                chunk.data.len(),
                chunk.children.len()
            );
            asset_model_with_kdop.preserved_chunks.push(chunk.clone());
        }
    }
    println!(
        "  asset_model preserved_chunks count={}",
        asset_model_with_kdop.preserved_chunks.len()
    );
    for chunk in &asset_model_with_kdop.preserved_chunks {
        println!("    preserved: {} data={}", chunk.id, chunk.data.len());
    }
    print_model_summary("vanilla", &vanilla_model);
    print_model_summary("assets", &asset_model_with_kdop);
    compare_models(&vanilla_model, &asset_model_with_kdop);

    let generated = generate_v2_from_model(&[], &asset_model_with_kdop)?;
    let out_path = dir.join(format!("{}_from_assets.hod", name));
    fs::write(&out_path, &generated).map_err(|e| e.to_string())?;
    println!(
        "generated_from_assets={} bytes -> {}",
        generated.len(),
        out_path.display()
    );

    // Also copy to game directory for testing (only if directory exists)
    let game_dir = format!(
        "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/pebble/{}",
        name
    );
    if Path::new(&game_dir).exists() {
        let game_path = format!("{}/{}.hod", game_dir, name);
        fs::write(&game_path, &generated).map_err(|e| e.to_string())?;
        println!("copied to game directory: {}", game_path);
    } else {
        println!("game directory does not exist: {}", game_dir);
    }
    let reparsed = HODModel::parse_with_external(
        &generated,
        Some(out_path.to_string_lossy().as_ref()),
        Some(dir.to_string_lossy().as_ref()),
    )?;
    print_model_summary("reparsed_assets", &reparsed);
    compare_models(&vanilla_model, &reparsed);
    Ok(())
}

fn build_model_from_assets(dir: &Path) -> Result<HODModel, String> {
    // Load SHADERS.MAP for shader detection
    let shader_map_path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/HODOR/SHADERS.MAP";
    let shaders_map = ShadersMap::from_file(Path::new(shader_map_path)).unwrap_or_else(|e| {
        println!("Warning: Could not load SHADERS.MAP: {}", e);
        println!("Using default shader detection");
        ShadersMap {
            mappings: std::collections::HashMap::new(),
            default_mapping: None,
        }
    });

    // Load materials from JSON
    let materials_path = dir.join("Homeworld2 Multi Mesh File_materials.json");
    let mut materials: Vec<HODMaterial> =
        serde_json::from_slice(&fs::read(&materials_path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;

    // Auto-detect shader type from texture files
    let mut texture_names: Vec<String> = fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("tga"))
        .filter_map(|path| path.file_stem()?.to_str()?.to_string().into())
        .collect();
    texture_names.sort();

    let detected_shader = ShadersMap::detect_shader_type(&texture_names);
    println!("  Detected shader type: {}", detected_shader);

    // Update materials with detected shader if not set
    for material in &mut materials {
        if material.shader_name.is_empty() {
            material.shader_name = detected_shader.clone();
        }
    }

    validate_mtl_sources(dir, &materials, &texture_names)?;

    let material_indices = build_material_index(&materials);

    // Load textures
    let mut textures = Vec::new();
    for tex_name in &texture_names {
        textures.push(load_tga_texture(
            &dir.join(format!("{}.tga", tex_name)),
            tex_name,
        )?);
    }

    // Load OBJ meshes
    let mut obj_paths: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("obj"))
        .collect();
    obj_paths.sort();

    let mut meshes = Vec::new();
    for path in obj_paths {
        if let Some((base_name, lod)) = parse_lod_filename(&path) {
            meshes.push(parse_obj_mesh(&path, &base_name, lod, &material_indices)?);
        }
    }

    let root_pos = Vector3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let root_rot = Vector3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let root_scale = Vector3 {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    };

    Ok(HODModel {
        version: 512,
        is_v2: true,
        name: "Homeworld2 Multi Mesh File".to_string(),
        textures,
        materials,
        meshes,
        joints: vec![HODJoint {
            name: "Root".to_string(),
            parent_name: None,
            local_transform: compose_transform_matrix(
                root_pos.clone(),
                root_rot.clone(),
                root_scale.clone(),
            ),
            position: Some(root_pos),
            rotation: Some(root_rot),
            scale: Some(root_scale),
        }],
        markers: Vec::new(),
        nav_lights: Vec::new(),
        engine_burns: Vec::new(),
        engine_glows: Vec::new(),
        engine_shapes: Vec::new(),
        collision_meshes: Vec::new(),
        dockpaths: Vec::new(),
        animations: Vec::new(),
        preserved_chunks: Vec::new(),
    })
}

fn load_tga_texture(path: &Path, name: &str) -> Result<HODTexture, String> {
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
    img.write_to(&mut Cursor::new(&mut png_bytes), image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    let b64 = BASE64_STANDARD.encode(&png_bytes);
    Ok(HODTexture {
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

fn build_material_index(materials: &[HODMaterial]) -> HashMap<String, usize> {
    let mut material_indices = HashMap::new();
    for (idx, material) in materials.iter().enumerate() {
        material_indices.insert(material.name.clone(), idx);
        material_indices.insert(material.name.to_lowercase(), idx);
    }
    material_indices
}

fn validate_mtl_sources(
    dir: &Path,
    materials: &[HODMaterial],
    texture_names: &[String],
) -> Result<(), String> {
    let material_lookup: HashMap<String, &HODMaterial> = materials
        .iter()
        .map(|material| (material.name.to_lowercase(), material))
        .collect();
    let texture_lookup: HashSet<String> = texture_names
        .iter()
        .map(|name| name.to_lowercase())
        .collect();

    let mut mtl_paths: Vec<PathBuf> = fs::read_dir(dir)
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
                            "{} defines material '{}' missing from material JSON",
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
                            "{} material '{}' maps texture '{}' not present in material JSON texture_maps",
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

    println!("  MTL/material/texture consistency: OK");
    Ok(())
}

struct ObjPartBuilder {
    part: HODMeshPart,
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
        part: HODMeshPart {
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
    vertex: HODVertex,
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
) -> Result<HODMesh, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut positions: Vec<Vector3> = Vec::new();
    let mut uvs: Vec<Vector2> = Vec::new();
    let mut normals: Vec<Vector3> = Vec::new();
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
                let vals = parse_floats(parts)?;
                if vals.len() >= 3 {
                    positions.push(Vector3 {
                        x: vals[0],
                        y: vals[1],
                        z: vals[2],
                    });
                }
            }
            Some("vt") => {
                let vals = parse_floats(parts)?;
                if vals.len() >= 2 {
                    uvs.push(Vector2 {
                        u: vals[0],
                        v: 1.0 - vals[1],
                    });
                }
            }
            Some("vn") => {
                let vals = parse_floats(parts)?;
                if vals.len() >= 3 {
                    normals.push(Vector3 {
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

    let parts: Vec<HODMeshPart> = parts_out.into_iter().map(|part| part.part).collect();

    Ok(HODMesh {
        name: name.to_string(),
        parent_name: "Root".to_string(),
        lod,
        has_mult_tags: false,
        parts,
    })
}

fn parse_floats<'a>(parts: impl Iterator<Item = &'a str>) -> Result<Vec<f32>, String> {
    parts
        .map(|part| part.parse::<f32>().map_err(|e| e.to_string()))
        .collect()
}

fn parse_obj_vertex(
    token: &str,
    positions: &[Vector3],
    uvs: &[Vector2],
    normals: &[Vector3],
) -> Result<((usize, Option<usize>, Option<usize>), HODVertex), String> {
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
    let vertex = HODVertex {
        position: positions
            .get(vi)
            .cloned()
            .ok_or_else(|| "position index out of range".to_string())?,
        normal: ni
            .and_then(|idx| normals.get(idx).cloned())
            .or(Some(Vector3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            })),
        color: None,
        uv: ti
            .and_then(|idx| uvs.get(idx).cloned())
            .or(Some(Vector2 { u: 0.0, v: 0.0 })),
        tangent: Some(Vector3 {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        }),
        binormal: Some(Vector3 {
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

fn print_model_summary(label: &str, model: &HODModel) {
    println!(
        "{}: meshes={} materials={} textures={} joints={} preserved={}",
        label,
        model.meshes.len(),
        model.materials.len(),
        model.textures.len(),
        model.joints.len(),
        model.preserved_chunks.len()
    );
    for mesh in &model.meshes {
        let verts: usize = mesh.parts.iter().map(|p| p.vertices.len()).sum();
        let indices: usize = mesh.parts.iter().map(|p| p.indices.len()).sum();
        println!(
            "  mesh={} lod={} parts={} verts={} indices={} tags={}",
            mesh.name,
            mesh.lod,
            mesh.parts.len(),
            verts,
            indices,
            mesh.has_mult_tags
        );
    }
}

fn compare_models(vanilla: &HODModel, other: &HODModel) {
    for vm in &vanilla.meshes {
        let Some(om) = other
            .meshes
            .iter()
            .find(|m| m.name == vm.name && m.lod == vm.lod)
        else {
            println!("  missing mesh {} lod {}", vm.name, vm.lod);
            continue;
        };
        for (part_idx, vp) in vm.parts.iter().enumerate() {
            let Some(op) = om.parts.get(part_idx) else {
                println!("  missing part {} for {} lod {}", part_idx, vm.name, vm.lod);
                continue;
            };
            let mut pos_mismatch = 0;
            let mut norm_mismatch = 0;
            let mut uv_mismatch = 0;
            let mut tangent_mismatch = 0;
            for (vv, ov) in vp.vertices.iter().zip(&op.vertices) {
                if !vec3_close(&vv.position, &ov.position) {
                    pos_mismatch += 1;
                }
                if !opt_vec3_close(&vv.normal, &ov.normal) {
                    norm_mismatch += 1;
                }
                if !opt_vec2_close(&vv.uv, &ov.uv) {
                    uv_mismatch += 1;
                }
                if !opt_vec3_close(&vv.tangent, &ov.tangent) {
                    tangent_mismatch += 1;
                }
            }
            println!(
                "  compare {} lod {} part {}: verts {}->{} indices {}->{} pos={} norm={} uv={} tangent={}",
                vm.name,
                vm.lod,
                part_idx,
                vp.vertices.len(),
                op.vertices.len(),
                vp.indices.len(),
                op.indices.len(),
                pos_mismatch,
                norm_mismatch,
                uv_mismatch,
                tangent_mismatch
            );
        }
    }
}

fn vec3_close(a: &Vector3, b: &Vector3) -> bool {
    (a.x - b.x).abs() < 0.0001 && (a.y - b.y).abs() < 0.0001 && (a.z - b.z).abs() < 0.0001
}

fn vec2_close(a: &Vector2, b: &Vector2) -> bool {
    (a.u - b.u).abs() < 0.0001 && (a.v - b.v).abs() < 0.0001
}

fn opt_vec3_close(a: &Option<Vector3>, b: &Option<Vector3>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => vec3_close(a, b),
        (None, None) => true,
        _ => false,
    }
}

fn opt_vec2_close(a: &Option<Vector2>, b: &Option<Vector2>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => vec2_close(a, b),
        (None, None) => true,
        _ => false,
    }
}
