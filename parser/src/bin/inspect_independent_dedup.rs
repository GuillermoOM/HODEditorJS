use hwr_hod_parser::dae::parse_dae;
use std::collections::HashSet;
use std::fs;

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
struct FullAttrKey {
    px: u32,
    py: u32,
    pz: u32,
    nx: u32,
    ny: u32,
    nz: u32,
    u: u32,
    v: u32,
    tx: u32,
    ty: u32,
    tz: u32,
    bx: u32,
    by: u32,
    bz: u32,
}

fn f32_to_u32(f: f32) -> u32 {
    if f.is_nan() {
        0
    } else {
        f.to_bits()
    }
}

fn main() -> Result<(), String> {
    let path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/mod-tools/HODEditorJS/testing/ter_centaur/ter_centaur.DAE";
    let xml = fs::read_to_string(path).map_err(|e| e.to_string())?;

    // Parse DAE using our own parse_dae (which generates flat vertices per group)
    let model = parse_dae(&xml)?;

    println!("Model has {} meshes", model.meshes.len());

    // Let's find Mesh 'Root_mesh' LOD 0
    let mesh = model
        .meshes
        .iter()
        .find(|m| m.name == "Root_mesh" && m.lod == 0)
        .ok_or_else(|| "Mesh not found".to_string())?;

    println!(
        "Root_mesh LOD 0 has {} parts before merging consecutive parts",
        mesh.parts.len()
    );

    for (pi, part) in mesh.parts.iter().enumerate() {
        println!(
            "\nPart {}: material_index = {}, flat vertices = {}",
            pi,
            part.material_index,
            part.vertices.len()
        );

        // Let's compute tangent space for this flat part
        let mut vertices = part.vertices.clone();

        // The DAE parser doesn't compute tangents, so we compute them
        hwr_hod_parser::hod::generate_v2_from_model(&[], &model).unwrap(); // Trigger internal updates or do it manually

        // Let's manually compute tangents for this part
        // (the code in compiler.rs does compute_tangent_space)
        // Since we want to use the exact same logic:
        let indices: Vec<u16> = (0..vertices.len() as u16).collect();
        // Since we are not editing source files, let's write a local tangent space calculator:
        compute_local_tangent_space(&mut vertices, &indices);

        // Deduplicate
        let mut unique_verts = HashSet::new();
        for v in &vertices {
            let key = FullAttrKey {
                px: f32_to_u32(v.position.x),
                py: f32_to_u32(v.position.y),
                pz: f32_to_u32(v.position.z),
                nx: f32_to_u32(v.normal.as_ref().map(|n| n.x).unwrap_or(0.0)),
                ny: f32_to_u32(v.normal.as_ref().map(|n| n.y).unwrap_or(0.0)),
                nz: f32_to_u32(v.normal.as_ref().map(|n| n.z).unwrap_or(0.0)),
                u: f32_to_u32(v.uv.as_ref().map(|uv| uv.u).unwrap_or(0.0)),
                v: f32_to_u32(v.uv.as_ref().map(|uv| uv.v).unwrap_or(0.0)),
                tx: f32_to_u32(v.tangent.as_ref().map(|t| t.x).unwrap_or(0.0)),
                ty: f32_to_u32(v.tangent.as_ref().map(|t| t.y).unwrap_or(0.0)),
                tz: f32_to_u32(v.tangent.as_ref().map(|t| t.z).unwrap_or(0.0)),
                bx: f32_to_u32(v.binormal.as_ref().map(|b| b.x).unwrap_or(0.0)),
                by: f32_to_u32(v.binormal.as_ref().map(|b| b.y).unwrap_or(0.0)),
                bz: f32_to_u32(v.binormal.as_ref().map(|b| b.z).unwrap_or(0.0)),
            };
            unique_verts.insert(key);
        }

        println!("  Unique vertices count: {}", unique_verts.len());
    }

    Ok(())
}

fn compute_local_tangent_space(vertices: &mut [hwr_hod_parser::hod::HODVertex], indices: &[u16]) {
    use hwr_hod_parser::hod::Vector3;
    let mut tangents = vec![
        Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0
        };
        vertices.len()
    ];
    let mut binormals = vec![
        Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0
        };
        vertices.len()
    ];

    for tri in indices.chunks_exact(3) {
        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;

        let p0 = &vertices[i0].position;
        let p1 = &vertices[i1].position;
        let p2 = &vertices[i2].position;
        let uv0 = vertices[i0].uv.as_ref().unwrap();
        let uv1 = vertices[i1].uv.as_ref().unwrap();
        let uv2 = vertices[i2].uv.as_ref().unwrap();

        let edge1 = Vector3 {
            x: p1.x - p0.x,
            y: p1.y - p0.y,
            z: p1.z - p0.z,
        };
        let edge2 = Vector3 {
            x: p2.x - p0.x,
            y: p2.y - p0.y,
            z: p2.z - p0.z,
        };
        let du1 = uv1.u - uv0.u;
        let dv1 = uv1.v - uv0.v;
        let du2 = uv2.u - uv0.u;
        let dv2 = uv2.v - uv0.v;
        let denom = du1 * dv2 - du2 * dv1;
        if denom.abs() < 0.000001 {
            continue;
        }
        let inv = 1.0 / denom;
        let tangent = Vector3 {
            x: (edge1.x * dv2 - edge2.x * dv1) * inv,
            y: (edge1.y * dv2 - edge2.y * dv1) * inv,
            z: (edge1.z * dv2 - edge2.z * dv1) * inv,
        };
        let binormal = Vector3 {
            x: (edge2.x * du1 - edge1.x * du2) * inv,
            y: (edge2.y * du1 - edge1.y * du2) * inv,
            z: (edge2.z * du1 - edge1.z * du2) * inv,
        };
        for idx in [i0, i1, i2] {
            tangents[idx].x += tangent.x;
            tangents[idx].y += tangent.y;
            tangents[idx].z += tangent.z;
            binormals[idx].x += binormal.x;
            binormals[idx].y += binormal.y;
            binormals[idx].z += binormal.z;
        }
    }

    for (idx, vertex) in vertices.iter_mut().enumerate() {
        let normal = vertex.normal.clone().unwrap();
        let tangent = tangents[idx].clone();
        let binormal = binormals[idx].clone();

        let t_dot_n = tangent.x * normal.x + tangent.y * normal.y + tangent.z * normal.z;
        let t_ortho = Vector3 {
            x: tangent.x - normal.x * t_dot_n,
            y: tangent.y - normal.y * t_dot_n,
            z: tangent.z - normal.z * t_dot_n,
        };
        let t_len = (t_ortho.x * t_ortho.x + t_ortho.y * t_ortho.y + t_ortho.z * t_ortho.z).sqrt();
        let t_norm = if t_len > 0.000001 {
            Vector3 {
                x: t_ortho.x / t_len,
                y: t_ortho.y / t_len,
                z: t_ortho.z / t_len,
            }
        } else {
            Vector3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            }
        };

        let b_cross = Vector3 {
            x: normal.y * t_norm.z - normal.z * t_norm.y,
            y: normal.z * t_norm.x - normal.x * t_norm.z,
            z: normal.x * t_norm.y - normal.y * t_norm.x,
        };
        let b_len = (b_cross.x * b_cross.x + b_cross.y * b_cross.y + b_cross.z * b_cross.z).sqrt();
        let b_norm = if b_len > 0.000001 {
            Vector3 {
                x: b_cross.x / b_len,
                y: b_cross.y / b_len,
                z: b_cross.z / b_len,
            }
        } else {
            Vector3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            }
        };

        let cross_n_t = Vector3 {
            x: normal.y * t_norm.z - normal.z * t_norm.y,
            y: normal.z * t_norm.x - normal.x * t_norm.z,
            z: normal.x * t_norm.y - normal.y * t_norm.x,
        };
        let dot_cross_b =
            cross_n_t.x * binormal.x + cross_n_t.y * binormal.y + cross_n_t.z * binormal.z;
        let handedness = if dot_cross_b >= 0.0 { 1.0 } else { -1.0 };

        vertex.tangent = Some(t_norm);
        vertex.binormal = Some(Vector3 {
            x: b_norm.x * handedness,
            y: b_norm.y * handedness,
            z: b_norm.z * handedness,
        });
    }
}
