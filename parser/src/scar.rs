//! `SCAR` (battle-scar reference mesh) generator.
//!
//! The engine uses the SCAR chunk (`DTRM/SCAR`, DEFA-style `NRML` version 0x7DF in
//! vanilla) to project battle-damage decals onto a ship. It is a welded
//! copy of the LOD0 hull mesh plus a uniform spatial grid of triangle lists.
//! The chunk body holds the header/grid; the vertex, normal and triangle arrays
//! live in the POOL mesh/face streams (appended after the COLD collision data).
//!
//! Layout (little endian), reverse engineered from ~170 vanilla HODs and the
//! `SCAR/NRML` handler (see `docs/hod2-reverse-engineering/hod2_reverse_engineering_knowledge_base.md`):
//!
//! ```text
//! len-string  mesh name (LOD0 mesh)      len-string  parent joint
//! u32         1
//! f32[3] min  f32[3] max                 (bounding box of the welded positions)
//! f32[3] center  f32 radius              (bounding sphere)
//! u32 nv  u32 nn  u32 nt  u32 nl         (positions, normals, triangles, list length)
//! u16[nl]     triangle index list        (concatenated per-cell lists)
//! u32[3]      grid dims (x, y, z)
//! u16[x*y*z]  per-cell list length       (cell = (dimY*z + y)*dimX + x)
//! u32         trailing id/hash
//! ```
//!
//! POOL: mesh stream += `nv` x f32[3] positions, then `nn` x f32[3] normals;
//! face stream += `nt` x (u16[3] position idx, u16[3] normal idx). Normals are the
//! per-corner vertex normals of the hull mesh, deduplicated by exact value.
use crate::types::HODModel;
use std::collections::HashMap;

/// Generated SCAR chunk body plus the POOL data it references.
pub struct ScarData {
    pub chunk: Vec<u8>,
    pub mesh_pool: Vec<u8>,
    pub face_pool: Vec<u8>,
}

fn tri_count(m: &crate::types::HODMesh) -> usize {
    m.parts.iter().map(|p| p.indices.len() / 3).sum()
}

/// Resolves the meshes to scar: one per name in `model.scar_meshes` (LOD0, case
/// insensitive; names that no longer match a mesh are dropped). If the model had a SCAR
/// but none of its meshes survive, the LOD0 mesh with the most triangles is used.
fn pick_meshes(model: &HODModel) -> Vec<&crate::types::HODMesh> {
    let mut out: Vec<&crate::types::HODMesh> = Vec::new();
    for name in &model.scar_meshes {
        if let Some(m) = model
            .meshes
            .iter()
            .find(|m| m.lod == 0 && m.name.eq_ignore_ascii_case(name) && tri_count(m) > 0)
        {
            out.push(m);
        }
    }
    if out.is_empty() && !model.scar_meshes.is_empty() {
        let mut best: Option<(&crate::types::HODMesh, usize)> = None;
        for m in model.meshes.iter().filter(|m| m.lod == 0) {
            let n = tri_count(m);
            if n > 0 && best.map_or(true, |(_, bn)| n > bn) {
                best = Some((m, n));
            }
        }
        out.extend(best.map(|(m, _)| m));
    }
    out
}

/// Builds every SCAR chunk (and its pool arrays) the saved HOD needs, in chunk order.
/// Empty when the loaded HOD had no SCAR.
pub fn generate_scars(model: &HODModel) -> Vec<ScarData> {
    pick_meshes(model)
        .into_iter()
        .filter_map(|m| build_scar(m, None))
        .collect()
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Akenine-Moller triangle/AABB overlap test. `c` = box center, `h` = half extents.
fn tri_box(v: &[[f64; 3]; 3], c: [f64; 3], h: [f64; 3]) -> bool {
    let a = sub(v[0], c);
    let b = sub(v[1], c);
    let d = sub(v[2], c);
    for ax in 0..3 {
        let lo = a[ax].min(b[ax]).min(d[ax]);
        let hi = a[ax].max(b[ax]).max(d[ax]);
        if lo > h[ax] || hi < -h[ax] {
            return false;
        }
    }
    let e = [sub(b, a), sub(d, b), sub(a, d)];
    let n = cross(e[0], e[1]);
    let r = h[0] * n[0].abs() + h[1] * n[1].abs() + h[2] * n[2].abs();
    if dot(n, a).abs() > r {
        return false;
    }
    for ed in &e {
        let axes = [
            [0.0, -ed[2], ed[1]],
            [ed[2], 0.0, -ed[0]],
            [-ed[1], ed[0], 0.0],
        ];
        for axis in axes {
            let p0 = dot(axis, a);
            let p1 = dot(axis, b);
            let p2 = dot(axis, d);
            let r = h[0] * axis[0].abs() + h[1] * axis[1].abs() + h[2] * axis[2].abs();
            if p0.min(p1).min(p2) > r || p0.max(p1).max(p2) < -r {
                return false;
            }
        }
    }
    true
}

/// Grid resolution. Vanilla uses 3*2^k cells per axis; the cell edge is roughly
/// 1.5x the edge of a cube holding one triangle each (fitted to vanilla files; any
/// resolution is valid for the engine's cell lookup, this only affects query speed).
fn choose_dims(extent: [f64; 3], tri_count: usize) -> [u32; 3] {
    let volume = extent[0].max(1e-6) * extent[1].max(1e-6) * extent[2].max(1e-6);
    let cell = 1.5 * (volume / tri_count.max(1) as f64).cbrt();
    let mut dims = [3u32; 3];
    for a in 0..3 {
        let mut best = 3u32;
        let mut best_err = f64::MAX;
        for k in 0..5 {
            let d = 3u32 << k;
            let err = ((extent[a].max(1e-6) / d as f64) / cell).ln().abs();
            if err < best_err {
                best_err = err;
                best = d;
            }
        }
        dims[a] = best;
    }
    dims
}

fn fnv1a(bytes: &[u8]) -> u32 {
    let mut h: u32 = 0x811c9dc5;
    for &b in bytes {
        h ^= b as u32;
        h = h.wrapping_mul(0x01000193);
    }
    h
}

fn put_str(out: &mut Vec<u8>, s: &str) {
    out.extend_from_slice(&(s.len() as u32).to_le_bytes());
    out.extend_from_slice(s.as_bytes());
}

/// Builds the SCAR for the model's first scar mesh (convenience for tests).
pub fn generate_scar(model: &HODModel) -> Option<ScarData> {
    generate_scars(model).into_iter().next()
}

/// Like [`generate_scar`], optionally forcing the grid resolution (used by tests to
/// compare cell contents against a vanilla grid).
pub fn generate_scar_with_dims(model: &HODModel, dims_override: Option<[u32; 3]>) -> Option<ScarData> {
    build_scar(pick_meshes(model).into_iter().next()?, dims_override)
}

/// Builds the SCAR chunk body and pool data from one LOD0 hull mesh.
/// Returns `None` when the mesh is unusable (or exceeds the u16 limits).
pub fn build_scar(mesh: &crate::types::HODMesh, dims_override: Option<[u32; 3]>) -> Option<ScarData> {
    // Weld positions (exact value match, so -0.0 == 0.0; first-use order); normals are the per-corner vertex
    // normals, deduplicated by exact value in first-use order over triangle corners.
    let mut pos_map: HashMap<[u32; 3], u16> = HashMap::new();
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut nrm_map: HashMap<[u32; 3], u16> = HashMap::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut tris: Vec<[u16; 3]> = Vec::new();
    let mut tri_nrm: Vec<[u16; 3]> = Vec::new();
    for part in &mesh.parts {
        for t in part.indices.chunks_exact(3) {
            let mut ids = [0u16; 3];
            let mut nids = [0u16; 3];
            let corner = |k: usize| part.vertices.get(t[k] as usize);
            // Face normal fallback for vertices without a stored normal.
            let face_n = {
                let (a, b, c) = (corner(0)?, corner(1)?, corner(2)?);
                let p = |v: &crate::types::HODVertex| {
                    [v.position.x as f64, v.position.y as f64, v.position.z as f64]
                };
                let n = cross(sub(p(b), p(a)), sub(p(c), p(a)));
                let l = dot(n, n).sqrt();
                if l > 0.0 {
                    [(n[0] / l) as f32, (n[1] / l) as f32, (n[2] / l) as f32]
                } else {
                    [0.0, 1.0, 0.0]
                }
            };
            for k in 0..3 {
                let v = corner(k)?;
                let p = [v.position.x, v.position.y, v.position.z];
                let key = [(p[0] + 0.0).to_bits(), (p[1] + 0.0).to_bits(), (p[2] + 0.0).to_bits()];
                ids[k] = match pos_map.get(&key) {
                    Some(&id) => id,
                    None => {
                        if positions.len() >= u16::MAX as usize {
                            return None;
                        }
                        let id = positions.len() as u16;
                        positions.push(p);
                        pos_map.insert(key, id);
                        id
                    }
                };
                let n = v.normal.as_ref().map_or(face_n, |n| [n.x, n.y, n.z]);
                let nkey = [(n[0] + 0.0).to_bits(), (n[1] + 0.0).to_bits(), (n[2] + 0.0).to_bits()];
                nids[k] = match nrm_map.get(&nkey) {
                    Some(&id) => id,
                    None => {
                        if normals.len() >= u16::MAX as usize {
                            return None;
                        }
                        let id = normals.len() as u16;
                        normals.push(n);
                        nrm_map.insert(nkey, id);
                        id
                    }
                };
            }
            tris.push(ids);
            tri_nrm.push(nids);
        }
    }
    if tris.is_empty() || tris.len() > u16::MAX as usize {
        return None;
    }

    // Bounds + bounding sphere (bbox center; radius = farthest welded position).
    let mut mn = [f32::MAX; 3];
    let mut mx = [f32::MIN; 3];
    for p in &positions {
        for a in 0..3 {
            mn[a] = mn[a].min(p[a]);
            mx[a] = mx[a].max(p[a]);
        }
    }
    let center = [
        (mn[0] + mx[0]) * 0.5,
        (mn[1] + mx[1]) * 0.5,
        (mn[2] + mx[2]) * 0.5,
    ];
    let mut radius = 0.0f64;
    for p in &positions {
        let d = [
            (p[0] - center[0]) as f64,
            (p[1] - center[1]) as f64,
            (p[2] - center[2]) as f64,
        ];
        radius = radius.max(dot(d, d).sqrt());
    }

    // Uniform grid; a triangle joins every cell whose box (inflated 2%) it overlaps.
    let extent = [
        (mx[0] - mn[0]) as f64,
        (mx[1] - mn[1]) as f64,
        (mx[2] - mn[2]) as f64,
    ];
    let dims = dims_override.unwrap_or_else(|| choose_dims(extent, tris.len()));
    let size = [
        extent[0] / dims[0] as f64,
        extent[1] / dims[1] as f64,
        extent[2] / dims[2] as f64,
    ];
    let total_cells = (dims[0] * dims[1] * dims[2]) as usize;
    let mut cells: Vec<Vec<u16>> = vec![Vec::new(); total_cells];
    for (ti, t) in tris.iter().enumerate() {
        let v = [0, 1, 2].map(|k| {
            let q = positions[t[k] as usize];
            [q[0] as f64, q[1] as f64, q[2] as f64]
        });
        let mut lo = [0usize; 3];
        let mut hi = [0usize; 3];
        for a in 0..3 {
            let (tmin, tmax) = (
                v[0][a].min(v[1][a]).min(v[2][a]),
                v[0][a].max(v[1][a]).max(v[2][a]),
            );
            if size[a] <= 0.0 {
                lo[a] = 0;
                hi[a] = dims[a] as usize - 1;
            } else {
                let l = ((tmin - mn[a] as f64) / size[a]).floor() as i64 - 1;
                let h = ((tmax - mn[a] as f64) / size[a]).floor() as i64 + 1;
                lo[a] = l.clamp(0, dims[a] as i64 - 1) as usize;
                hi[a] = h.clamp(0, dims[a] as i64 - 1) as usize;
            }
        }
        for z in lo[2]..=hi[2] {
            for y in lo[1]..=hi[1] {
                for x in lo[0]..=hi[0] {
                    let c = [
                        mn[0] as f64 + (x as f64 + 0.5) * size[0],
                        mn[1] as f64 + (y as f64 + 0.5) * size[1],
                        mn[2] as f64 + (z as f64 + 0.5) * size[2],
                    ];
                    let h = [
                        size[0] * 0.5 * 1.02,
                        size[1] * 0.5 * 1.02,
                        size[2] * 0.5 * 1.02,
                    ];
                    if tri_box(&v, c, h) {
                        cells[(dims[1] as usize * z + y) * dims[0] as usize + x].push(ti as u16);
                    }
                }
            }
        }
    }
    let list_len: usize = cells.iter().map(|c| c.len()).sum();
    if cells.iter().any(|c| c.len() > u16::MAX as usize) || list_len > u32::MAX as usize {
        return None;
    }

    // Chunk body.
    let mut body = Vec::new();
    put_str(&mut body, &mesh.name);
    put_str(&mut body, &mesh.parent_name);
    body.extend_from_slice(&1u32.to_le_bytes());
    for v in mn.iter().chain(mx.iter()).chain(center.iter()) {
        body.extend_from_slice(&v.to_le_bytes());
    }
    body.extend_from_slice(&(radius as f32).to_le_bytes());
    for n in [positions.len(), normals.len(), tris.len(), list_len] {
        body.extend_from_slice(&(n as u32).to_le_bytes());
    }
    for c in &cells {
        for &t in c {
            body.extend_from_slice(&t.to_le_bytes());
        }
    }
    for d in dims {
        body.extend_from_slice(&d.to_le_bytes());
    }
    for c in &cells {
        body.extend_from_slice(&(c.len() as u16).to_le_bytes());
    }
    let id = fnv1a(&body);
    body.extend_from_slice(&id.to_le_bytes());

    // Pool data.
    let mut mesh_pool = Vec::with_capacity((positions.len() + normals.len()) * 12);
    for v in positions.iter().chain(normals.iter()) {
        for f in v {
            mesh_pool.extend_from_slice(&f.to_le_bytes());
        }
    }
    let mut face_pool = Vec::with_capacity(tris.len() * 12);
    for (t, n) in tris.iter().zip(&tri_nrm) {
        for &i in t.iter().chain(n.iter()) {
            face_pool.extend_from_slice(&i.to_le_bytes());
        }
    }

    Some(ScarData {
        chunk: body,
        mesh_pool,
        face_pool,
    })
}
