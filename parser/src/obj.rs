use crate::hod::{HODModel, HODMesh, HODMeshPart, HODVertex, HODMaterial, HODJoint, Matrix4, Vector3, Vector2, HODTexture};
use std::path::Path;

pub fn parse_obj(obj_path: &Path) -> Result<HODModel, String> {
    let (models, materials_res) = tobj::load_obj(
        obj_path,
        &tobj::LoadOptions {
            single_index: true,
            triangulate: true,
            ignore_points: true,
            ignore_lines: true,
        },
    ).map_err(|e| format!("Failed to load OBJ: {}", e))?;

    let mut hod_materials = Vec::new();
    let mut mat_names = Vec::new();

    if let Ok(materials) = materials_res {
        for m in materials {
            let mut texture_maps = Vec::new();
            if let Some(diff) = m.diffuse_texture {
                texture_maps.push(diff);
            }
            hod_materials.push(HODMaterial {
                name: m.name.clone(),
                shader_name: "ship".to_string(),
                texture_maps,
            });
            mat_names.push(m.name);
        }
    }

    if hod_materials.is_empty() {
        hod_materials.push(HODMaterial {
            name: "default".to_string(),
            shader_name: "ship".to_string(),
            texture_maps: vec![],
        });
        mat_names.push("default".to_string());
    }

    let mut meshes = Vec::new();
    
    // Create a Root joint since HOD models require it
    let joints = vec![
        HODJoint {
            name: "Root".to_string(),
            parent_name: None,
            local_transform: Matrix4 { m: [[1.0,0.0,0.0,0.0], [0.0,1.0,0.0,0.0], [0.0,0.0,1.0,0.0], [0.0,0.0,0.0,1.0]] },
            position: None,
            rotation: None,
            scale: None,
        }
    ];

    for m in models {
        let mesh = m.mesh;
        let mat_idx = mesh.material_id.unwrap_or(0);
        let mut parts = Vec::new();

        let has_normals = !mesh.normals.is_empty();
        let has_uvs = !mesh.texcoords.is_empty();

        let vertex_mask = 0x01 // position
            | (if has_normals { 0x02 } else { 0 })
            | (if has_uvs { 0x04 } else { 0 })
            | (if has_normals && has_uvs { 0x18 } else { 0 }); // tangent + bitangent

        let total_vertices = mesh.positions.len() / 3;

        // Split mesh into multiple parts if vertex count > 65535 to fit in u16 indices
        let mut current_vertices = Vec::new();
        let mut current_indices = Vec::new();
        let mut global_to_local = std::collections::HashMap::new();

        for i in (0..mesh.indices.len()).step_by(3) {
            let idx0 = mesh.indices[i] as usize;
            let idx1 = mesh.indices[i+1] as usize;
            let idx2 = mesh.indices[i+2] as usize;

            let mut needed = 0;
            if !global_to_local.contains_key(&idx0) { needed += 1; }
            if !global_to_local.contains_key(&idx1) { needed += 1; }
            if !global_to_local.contains_key(&idx2) { needed += 1; }

            if current_vertices.len() + needed > 65535 {
                parts.push(HODMeshPart {
                    material_index: mat_idx,
                    vertex_mask,
                    vertices: current_vertices.clone(),
                    indices: current_indices.clone(),
                });

                current_vertices.clear();
                current_indices.clear();
                global_to_local.clear();
            }

            let mut add_vertex = |g_idx: usize| -> u16 {
                if let Some(&l_idx) = global_to_local.get(&g_idx) {
                    l_idx
                } else {
                    let l_idx = current_vertices.len() as u16;
                    global_to_local.insert(g_idx, l_idx);

                    let vx = mesh.positions[g_idx * 3];
                    let vy = mesh.positions[g_idx * 3 + 1];
                    let vz = mesh.positions[g_idx * 3 + 2];
                    
                    let nx = if has_normals { mesh.normals[g_idx * 3] } else { 0.0 };
                    let ny = if has_normals { mesh.normals[g_idx * 3 + 1] } else { 0.0 };
                    let nz = if has_normals { mesh.normals[g_idx * 3 + 2] } else { 1.0 };
                    
                    let tu = if has_uvs { mesh.texcoords[g_idx * 2] } else { 0.0 };
                    let tv = if has_uvs { 1.0 - mesh.texcoords[g_idx * 2 + 1] } else { 0.0 };

                    current_vertices.push(HODVertex {
                        position: Vector3 { x: vx, y: vy, z: vz },
                        normal: Some(Vector3 { x: nx, y: ny, z: nz }),
                        color: None,
                        uv: Some(Vector2 { u: tu, v: tv }),
                        tangent: Some(Vector3 { x: 0.0, y: 0.0, z: 0.0 }),
                        binormal: Some(Vector3 { x: 0.0, y: 0.0, z: 0.0 }),
                        skinning_data: None,
                    });
                    l_idx
                }
            };

            let l0 = add_vertex(idx0);
            let l1 = add_vertex(idx1);
            let l2 = add_vertex(idx2);

            current_indices.push(l0);
            current_indices.push(l1);
            current_indices.push(l2);
        }

        if !current_vertices.is_empty() {
            parts.push(HODMeshPart {
                material_index: mat_idx,
                vertex_mask,
                vertices: current_vertices,
                indices: current_indices,
            });
        }

        // Calculate tangents/bitangents for each part
        if has_normals && has_uvs {
            for part in &mut parts {
                for i in (0..part.indices.len()).step_by(3) {
                    let i0 = part.indices[i] as usize;
                    let i1 = part.indices[i+1] as usize;
                    let i2 = part.indices[i+2] as usize;

                    let v0 = &part.vertices[i0];
                    let v1 = &part.vertices[i1];
                    let v2 = &part.vertices[i2];

                    let p0 = &v0.position;
                    let p1 = &v1.position;
                    let p2 = &v2.position;

                    let uv0 = v0.uv.as_ref().unwrap();
                    let uv1 = v1.uv.as_ref().unwrap();
                    let uv2 = v2.uv.as_ref().unwrap();

                    let dx1 = p1.x - p0.x;
                    let dy1 = p1.y - p0.y;
                    let dz1 = p1.z - p0.z;

                    let dx2 = p2.x - p0.x;
                    let dy2 = p2.y - p0.y;
                    let dz2 = p2.z - p0.z;

                    let du1 = uv1.u - uv0.u;
                    let dv1 = uv1.v - uv0.v;
                    let du2 = uv2.u - uv0.u;
                    let dv2 = uv2.v - uv0.v;

                    let r = 1.0 / (du1 * dv2 - dv1 * du2 + 1e-6);

                    let tx = (dv2 * dx1 - dv1 * dx2) * r;
                    let ty = (dv2 * dy1 - dv1 * dy2) * r;
                    let tz = (dv2 * dz1 - dv1 * dz2) * r;

                    let bx = (du1 * dx2 - du2 * dx1) * r;
                    let by = (du1 * dy2 - du2 * dy1) * r;
                    let bz = (du1 * dz2 - du2 * dz1) * r;

                    for &idx in &[i0, i1, i2] {
                        if let Some(t) = &mut part.vertices[idx].tangent {
                            t.x += tx; t.y += ty; t.z += tz;
                        }
                        if let Some(b) = &mut part.vertices[idx].binormal {
                            b.x += bx; b.y += by; b.z += bz;
                        }
                    }
                }

                // Normalize
                for v in &mut part.vertices {
                    let normalize = |vec: &mut Vector3| {
                        let len = (vec.x*vec.x + vec.y*vec.y + vec.z*vec.z).sqrt();
                        if len > 1e-6 {
                            vec.x /= len;
                            vec.y /= len;
                            vec.z /= len;
                        }
                    };
                    if let Some(t) = &mut v.tangent { normalize(t); }
                    if let Some(b) = &mut v.binormal { normalize(b); }
                }
            }
        }

        let mut final_name = m.name;
        let mut suffix_idx = 1;
        while meshes.iter().any(|mesh: &HODMesh| mesh.name == final_name) {
            final_name = format!("{}_{}", final_name.split('_').next().unwrap_or(&final_name), suffix_idx);
            suffix_idx += 1;
        }

        meshes.push(HODMesh {
            name: final_name,
            parent_name: "Root".to_string(),
            lod: 0,
            has_mult_tags: false,
            parts,
        });
    }

    Ok(HODModel {
        version: 0x40000,
        is_v2: true,
        name: obj_path.file_stem().unwrap_or_default().to_string_lossy().into_owned(),
        textures: Vec::new(),
        materials: hod_materials,
        meshes,
        joints,
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
