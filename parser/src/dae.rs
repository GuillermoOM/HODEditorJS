use crate::hod::*;
use roxmltree::{Document, Node};
use std::collections::HashMap;

/// Hashable key for vertex deduplication: (position_index, normal_index, uv_index)
type VertexKey = (usize, usize, usize);

fn parse_float_array(node: Node) -> Result<Vec<f32>, String> {
    if let Some(text) = node.text() {
        let floats: Result<Vec<f32>, _> =
            text.split_whitespace().map(|s| s.parse::<f32>()).collect();
        floats.map_err(|e| format!("Failed to parse floats: {}", e))
    } else {
        Ok(Vec::new())
    }
}

fn parse_int_array(node: Node) -> Result<Vec<usize>, String> {
    if let Some(text) = node.text() {
        let ints: Result<Vec<usize>, _> = text
            .split_whitespace()
            .map(|s| s.parse::<usize>())
            .collect();
        ints.map_err(|e| format!("Failed to parse ints: {}", e))
    } else {
        Ok(Vec::new())
    }
}

pub fn parse_dae(xml_str: &str) -> Result<HODModel, String> {
    let doc = Document::parse(xml_str).map_err(|e| e.to_string())?;

    let mut model = HODModel::new();
    model.is_v2 = true;
    model.version = 1400; // Force HOD 2.0 version

    // Parse materials first (just names for now)
    let mut material_names = Vec::new();
    for node in doc.descendants() {
        if node.has_tag_name("material") {
            if let Some(name) = node.attribute("name") {
                // e.g. MAT[fighter2S-01]_SHD[ship]
                material_names.push(name.to_string());
                model.materials.push(HODMaterial {
                    name: name.to_string(),
                    shader_name: "default".to_string(),
                    texture_maps: Vec::new(),
                });
            }
        }
    }

    // Build meshes from library_geometries
    let mut parsed_meshes_with_lod: Vec<(String, i32, String, HODMeshPart)> = Vec::new();

    for geometry in doc.descendants().filter(|n| n.has_tag_name("geometry")) {
        let geom_id = geometry.attribute("id").unwrap_or("unknown");
        let geom_name = geometry.attribute("name").unwrap_or(geom_id);

        if let Some(mesh) = geometry.children().find(|n| n.has_tag_name("mesh")) {
            let mut source_map: HashMap<String, Vec<f32>> = HashMap::new();

            for source in mesh.children().filter(|n| n.has_tag_name("source")) {
                let id = source.attribute("id").unwrap_or("");
                if let Some(float_array) = source.children().find(|n| n.has_tag_name("float_array"))
                {
                    if let Ok(floats) = parse_float_array(float_array) {
                        source_map.insert(format!("#{}", id), floats);
                    }
                }
            }

            // Extract the position source ID
            let mut pos_source_id = String::new();
            if let Some(vertices) = mesh.children().find(|n| n.has_tag_name("vertices")) {
                if let Some(input) = vertices.children().find(|n| {
                    n.has_tag_name("input") && n.attribute("semantic") == Some("POSITION")
                }) {
                    pos_source_id = input.attribute("source").unwrap_or("").to_string();
                }
            }

            // Parse ALL triangles/polylist groups (one per material)
            for triangles in mesh
                .children()
                .filter(|n| n.has_tag_name("triangles") || n.has_tag_name("polylist"))
            {
                let mut mesh_part = HODMeshPart {
                    material_index: 0,
                    vertex_mask: 0x01 | 0x02 | 0x08 | 0x2000 | 0x4000,
                    vertices: Vec::new(),
                    indices: Vec::new(),
                };

                if let Some(mat) = triangles.attribute("material") {
                    if let Some(idx) = material_names.iter().position(|m| m == mat) {
                        mesh_part.material_index = idx;
                    }
                }

                let mut pos_offset = 0;
                let mut norm_offset = -1;
                let mut uv_offset = -1;
                let mut max_offset = 0;

                let mut norm_source_id = String::new();
                let mut uv_source_id = String::new();

                for input in triangles.children().filter(|n| n.has_tag_name("input")) {
                    let semantic = input.attribute("semantic").unwrap_or("");
                    let offset: i32 = input
                        .attribute("offset")
                        .unwrap_or("0")
                        .parse()
                        .unwrap_or(0);
                    let source = input.attribute("source").unwrap_or("");

                    if offset > max_offset {
                        max_offset = offset;
                    }

                    if semantic == "VERTEX" {
                        pos_offset = offset;
                    } else if semantic == "NORMAL" {
                        norm_offset = offset;
                        norm_source_id = source.to_string();
                    } else if semantic == "TEXCOORD" {
                        uv_offset = offset;
                        uv_source_id = source.to_string();
                    }
                }

                let stride = (max_offset + 1) as usize;

                if let Some(p) = triangles.children().find(|n| n.has_tag_name("p")) {
                    if let Ok(indices) = parse_int_array(p) {
                        let pos_data = source_map.get(&pos_source_id);
                        let norm_data = source_map.get(&norm_source_id);
                        let uv_data = source_map.get(&uv_source_id);

                        let mut v_idx = 0;
                        while v_idx + stride <= indices.len() {
                            let p_i = indices[v_idx + pos_offset as usize];
                            let mut vertex = HODVertex {
                                position: Vector3 {
                                    x: 0.0,
                                    y: 0.0,
                                    z: 0.0,
                                },
                                normal: Some(Vector3 {
                                    x: 0.0,
                                    y: 1.0,
                                    z: 0.0,
                                }),
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
                                uv: Some(Vector2 { u: 0.0, v: 0.0 }),
                                color: Some(0xFFFFFFFF),
                                skinning_data: None,
                            };

                            if let Some(pd) = pos_data {
                                if p_i * 3 + 2 < pd.len() {
                                    vertex.position.x = pd[p_i * 3];
                                    vertex.position.y = pd[p_i * 3 + 1];
                                    vertex.position.z = pd[p_i * 3 + 2];
                                }
                            }

                            if norm_offset >= 0 {
                                let n_i = indices[v_idx + norm_offset as usize];
                                if let Some(nd) = norm_data {
                                    if n_i * 3 + 2 < nd.len() {
                                        if let Some(ref mut n) = vertex.normal {
                                            n.x = nd[n_i * 3];
                                            n.y = nd[n_i * 3 + 1];
                                            n.z = nd[n_i * 3 + 2];
                                        }
                                    }
                                }
                            }

                            if uv_offset >= 0 {
                                let u_i = indices[v_idx + uv_offset as usize];
                                if let Some(ud) = uv_data {
                                    if u_i * 2 + 1 < ud.len() {
                                        if let Some(ref mut uv_coords) = vertex.uv {
                                            uv_coords.u = ud[u_i * 2];
                                            uv_coords.v = ud[u_i * 2 + 1];
                                        }
                                    }
                                }
                            }

                            let idx = mesh_part.vertices.len() as u16;
                            mesh_part.vertices.push(vertex);
                            mesh_part.indices.push(idx);
                            v_idx += stride;
                        }
                    }
                }

                // Extract the MULT[name] tag and LOD
                let mut mesh_target_name = geom_name.to_string();
                let mut lod = 0i32;
                if mesh_target_name.starts_with("MULT[") {
                    if let Some(end) = mesh_target_name.find("]") {
                        mesh_target_name = mesh_target_name[5..end].to_string();
                    }
                    // Extract LOD from "MULT[name]_LOD[N]"
                    if let Some(lod_start) = geom_name.find("_LOD[") {
                        let lod_part = &geom_name[lod_start + 5..];
                        if let Some(lod_end) = lod_part.find("]") {
                            lod = lod_part[..lod_end].parse().unwrap_or(0);
                        }
                    }
                }

                let key = format!("{}_{}", mesh_target_name, lod);
                parsed_meshes_with_lod.push((mesh_target_name, lod, key, mesh_part));
            }
        }
    }

    // Group mesh parts into full HODMeshes by (name, lod)
    let mut mesh_map: HashMap<String, HODMesh> = HashMap::new();
    for (name, lod, key, part) in parsed_meshes_with_lod {
        if let Some(mesh) = mesh_map.get_mut(&key) {
            mesh.parts.push(part);
        } else {
            mesh_map.insert(
                key.clone(),
                HODMesh {
                    name: name.clone(),
                    parent_name: "Root".to_string(),
                    lod,
                    has_mult_tags: false,
                    parts: vec![part],
                },
            );
        }
    }

    for (_, mesh) in mesh_map {
        model.meshes.push(mesh);
    }

    // Parse visual scenes to construct hierarchy
    if let Some(visual_scene) = doc.descendants().find(|n| n.has_tag_name("visual_scene")) {
        for child in visual_scene.children().filter(|n| n.has_tag_name("node")) {
            parse_scene_node(child, None, &mut model);
        }
    }

    Ok(model)
}

fn parse_matrix(node: Node) -> Matrix4 {
    let mut mat = Matrix4 { m: [[0.0; 4]; 4] };
    mat.m[0][0] = 1.0;
    mat.m[1][1] = 1.0;
    mat.m[2][2] = 1.0;
    mat.m[3][3] = 1.0;

    if let Some(text) = node.text() {
        let floats: Vec<f32> = text
            .split_whitespace()
            .filter_map(|s| s.parse::<f32>().ok())
            .collect();
        if floats.len() == 16 {
            // Collada matrices are row-major
            mat.m[0] = [floats[0], floats[4], floats[8], floats[12]];
            mat.m[1] = [floats[1], floats[5], floats[9], floats[13]];
            mat.m[2] = [floats[2], floats[6], floats[10], floats[14]];
            mat.m[3] = [floats[3], floats[7], floats[11], floats[15]];
        }
    }
    mat
}

fn parse_scene_node(node: Node, parent_name: Option<&str>, model: &mut HODModel) {
    let mut name = node.attribute("name").unwrap_or("unknown").to_string();
    let mut is_joint = false;
    let mut is_navl = false;
    let mut is_burn = false;
    let mut is_mark = false;

    // Extract real name and type from prefixes
    if name.starts_with("JNT[") {
        if let Some(end) = name.find("]") {
            name = name[4..end].to_string();
            is_joint = true;
        }
    } else if name.starts_with("MARK[") {
        if let Some(end) = name.find("]") {
            name = name[5..end].to_string();
            is_mark = true;
        }
    } else if name.starts_with("NAVL[") {
        if let Some(end) = name.find("]") {
            name = name[5..end].to_string();
            is_navl = true;
        }
    } else if name.starts_with("BURN[") {
        if let Some(end) = name.find("]") {
            name = name[5..end].to_string();
            is_burn = true;
        }
    }

    let mut transform = Matrix4 { m: [[0.0; 4]; 4] };
    transform.m[0][0] = 1.0;
    transform.m[1][1] = 1.0;
    transform.m[2][2] = 1.0;
    transform.m[3][3] = 1.0;

    if let Some(matrix_node) = node.children().find(|n| n.has_tag_name("matrix")) {
        transform = parse_matrix(matrix_node);
    }

    let p_name = parent_name.map(|s| s.to_string());

    if is_joint {
        model.joints.push(HODJoint {
            name: name.clone(),
            parent_name: p_name.clone(),
            local_transform: transform.clone(),
            position: None,
            rotation: None,
            scale: None,
        });
    } else if is_mark {
        model.markers.push(HODMarker {
            name: name.clone(),
            parent_joint: p_name.clone().unwrap_or_default(),
            position: Vector3 {
                x: transform.m[3][0],
                y: transform.m[3][1],
                z: transform.m[3][2],
            },
            rotation: transform.clone(),
            rotation_euler: None,
        });
    } else if is_navl {
        model.nav_lights.push(HODNavLight {
            name: name.clone(),
            section: 0,
            size: 10.0,
            phase: 0.0,
            frequency: 0.0,
            style: "default".to_string(),
            color: Vector3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            distance: 1000.0,
            sprite_visible: true,
            high_end_only: false,
        });
        // Real parsing of [Sz_xx] from name string can be done later if needed
    } else if is_burn {
        model.engine_burns.push(HODEngineBurn {
            name: name.clone(),
            parent_name: p_name.clone().unwrap_or_default(),
            num_divisions: 5,
            num_flames: 1,
            vertices: Vec::new(),
        });
    } else {
        // Just a generic node (e.g. ROOT_COL, MULT[Root_mesh])
        // Let's add it as a joint so its transform is kept in the hierarchy
        if !name.starts_with("MULT[") && !name.starts_with("Flame[") && !name.starts_with("Class[")
        {
            model.joints.push(HODJoint {
                name: name.clone(),
                parent_name: p_name.clone(),
                local_transform: transform.clone(),
                position: None,
                rotation: None,
                scale: None,
            });
        }
    }

    // Parse children
    for child in node.children().filter(|n| n.has_tag_name("node")) {
        parse_scene_node(child, Some(&name), model);
    }
}
