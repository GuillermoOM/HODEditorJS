use std::collections::HashMap;
use crate::hod::{HODModel, HODVertex, Vector3, Vector2, HODMeshPart};
use crate::iff::IffChunk;

#[derive(Hash, PartialEq, Eq, Clone)]
struct HashableVertex {
    px: u32, py: u32, pz: u32,
    nx: u32, ny: u32, nz: u32,
    c: u32,
    u: u32, v: u32,
    tx: u32, ty: u32, tz: u32,
    bx: u32, by: u32, bz: u32,
    skinning: Option<Vec<u8>>,
}

fn f32_to_u32(f: f32) -> u32 {
    if f.is_nan() { 0 } else { f.to_bits() }
}

impl HashableVertex {
    fn from_hod(v: &HODVertex) -> Self {
        let (nx, ny, nz) = v.normal.as_ref().map_or((0, 0, 0), |n| (f32_to_u32(n.x), f32_to_u32(n.y), f32_to_u32(n.z)));
        let (u, v_uv) = v.uv.as_ref().map_or((0, 0), |uv| (f32_to_u32(uv.u), f32_to_u32(uv.v)));
        let (tx, ty, tz) = v.tangent.as_ref().map_or((0, 0, 0), |t| (f32_to_u32(t.x), f32_to_u32(t.y), f32_to_u32(t.z)));
        let (bx, by, bz) = v.binormal.as_ref().map_or((0, 0, 0), |b| (f32_to_u32(b.x), f32_to_u32(b.y), f32_to_u32(b.z)));
        
        Self {
            px: f32_to_u32(v.position.x), py: f32_to_u32(v.position.y), pz: f32_to_u32(v.position.z),
            nx, ny, nz,
            c: v.color.unwrap_or(0),
            u, v: v_uv,
            tx, ty, tz,
            bx, by, bz,
            skinning: v.skinning_data.clone(),
        }
    }
}

pub struct MeshDeduplicator {
    vertices: Vec<HODVertex>,
    vertex_map: HashMap<HashableVertex, u16>,
}

impl MeshDeduplicator {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            vertex_map: HashMap::new(),
        }
    }

    pub fn add_vertex(&mut self, v: &HODVertex) -> u16 {
        let hv = HashableVertex::from_hod(v);
        if let Some(&idx) = self.vertex_map.get(&hv) {
            return idx;
        }
        let idx = self.vertices.len() as u16;
        self.vertices.push(v.clone());
        self.vertex_map.insert(hv, idx);
        idx
    }

    pub fn get_vertices(&self) -> &[HODVertex] {
        &self.vertices
    }
}

use byteorder::{LittleEndian, BigEndian, WriteBytesExt};
use std::io::{Cursor, Write};

pub struct CompiledMesh {
    pub name: String,
    pub parent_name: String,
    pub lod: i32,
    pub new_parts: Vec<HODMeshPart>,
}

fn normalize_hwrm_vertex_mask(mask: u32) -> u32 {
    let mut normalized = mask & !0x30;
    if (mask & 0x10) != 0 {
        normalized |= 0x08;
    }
    normalized
}

fn write_len_string<W: Write>(writer: &mut W, s: &str) -> std::io::Result<()> {
    writer.write_u32::<LittleEndian>(s.len() as u32)?;
    writer.write_all(s.as_bytes())?;
    Ok(())
}

pub fn compile_model_meshes(model: &HODModel) -> Vec<CompiledMesh> {
    let mut compiled_meshes = Vec::new();

    for mesh in &model.meshes {
        let mut new_parts = Vec::new();

        for part in &mesh.parts {
            if !part.indices.is_empty() {
                new_parts.push(HODMeshPart {
                    material_index: part.material_index,
                    vertex_mask: normalize_hwrm_vertex_mask(part.vertex_mask),
                    vertices: part.vertices.clone(),
                    indices: part.indices.clone(),
                });
            } else {
                let mut dedup = MeshDeduplicator::new();
                let mut new_indices = Vec::new();
                
                // If there are no indices, assume it's a flat triangle list
                for (idx, v) in part.vertices.iter().enumerate() {
                    let new_idx = dedup.add_vertex(v);
                    new_indices.push(new_idx);
                }

                new_parts.push(HODMeshPart {
                    material_index: part.material_index,
                    vertex_mask: normalize_hwrm_vertex_mask(part.vertex_mask),
                    vertices: dedup.get_vertices().to_vec(),
                    indices: new_indices,
                });
            }
        }

        compiled_meshes.push(CompiledMesh {
            name: mesh.name.clone(),
            parent_name: mesh.parent_name.clone(),
            lod: mesh.lod,
            new_parts,
        });
    }

    compiled_meshes
}

pub fn generate_pool_data(compiled_meshes: &[CompiledMesh], comp_tex: &[u8], decomp_tex_len: u32, pool_type: u32) -> std::io::Result<Vec<u8>> {
    let mut decomp_mesh = Vec::new();
    let mut decomp_face = Vec::new();

    for mesh in compiled_meshes {
        for part in &mesh.new_parts {
            let mut vertex_stride = 0;
            if (part.vertex_mask & 0x01) != 0 { vertex_stride += 16; }
            if (part.vertex_mask & 0x02) != 0 { vertex_stride += 16; }
            if (part.vertex_mask & 0x04) != 0 { vertex_stride += 4; }
            if (part.vertex_mask & 0x08) != 0 { vertex_stride += 8; }
            if (part.vertex_mask & 0x10) != 0 { vertex_stride += 8; }
            if (part.vertex_mask & 0x20) != 0 { vertex_stride += 8; }
            if (part.vertex_mask & 0x2000) != 0 { vertex_stride += 12; }
            if (part.vertex_mask & 0x4000) != 0 { vertex_stride += 12; }
            
            if let Some(first_vert) = part.vertices.first() {
                if let Some(ref skin) = first_vert.skinning_data {
                    vertex_stride += skin.len() as u32;
                }
            }
            if vertex_stride == 0 { vertex_stride = 1; }
            
            for v in &part.vertices {
                let _ = crate::hod::write_vertex(&mut decomp_mesh, v, part.vertex_mask, 1400, vertex_stride);
            }
            
            // Align face pool to 2 bytes
            if decomp_face.len() % 2 != 0 {
                decomp_face.push(0);
            }
            let mut face_cursor = Cursor::new(&mut decomp_face);
            face_cursor.set_position(face_cursor.get_ref().len() as u64);
            for &idx in &part.indices {
                face_cursor.write_u16::<LittleEndian>(idx)?;
            }
        }
    }

    let comp_mesh = if decomp_mesh.is_empty() { Vec::new() } else { crate::xpress::compress_or_raw(&decomp_mesh) };
    
    let comp_face = if decomp_face.is_empty() { Vec::new() } else { crate::xpress::compress_or_raw(&decomp_face) };

    let mut pool_buf = Vec::new();
    let mut cursor = Cursor::new(&mut pool_buf);
    
    cursor.write_u32::<LittleEndian>(pool_type)?; // pool_type
    
    cursor.write_u32::<LittleEndian>(comp_tex.len() as u32)?;
    cursor.write_u32::<LittleEndian>(decomp_tex_len)?;
    cursor.write_all(comp_tex)?;

    cursor.write_u32::<LittleEndian>(comp_mesh.len() as u32)?;
    cursor.write_u32::<LittleEndian>(decomp_mesh.len() as u32)?;
    cursor.write_all(&comp_mesh)?;

    cursor.write_u32::<LittleEndian>(comp_face.len() as u32)?;
    cursor.write_u32::<LittleEndian>(decomp_face.len() as u32)?;
    cursor.write_all(&comp_face)?;

    Ok(pool_buf)
}

fn get_base_name(name: &str) -> String {
    if let Some(idx) = name.find("_lod_") {
        name[0..idx].to_string()
    } else if let Some(idx) = name.rfind("_LOD") {
        let suffix = &name[idx + 4..];
        if !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit()) {
            return name[0..idx].to_string();
        }
        name.to_string()
    } else if let Some(idx) = name.rfind("_lod") {
        let suffix = &name[idx + 4..];
        if !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit()) {
            return name[0..idx].to_string();
        }
        name.to_string()
    } else {
        name.to_string()
    }
}

pub fn generate_mult_chunks(compiled_meshes: &[CompiledMesh]) -> std::io::Result<Vec<IffChunk>> {
    let mut mult_chunks = Vec::new();
    let mut grouped_meshes: Vec<(String, String, Vec<&CompiledMesh>)> = Vec::new();

    for mesh in compiled_meshes {
        let base_name = get_base_name(&mesh.name);
        if let Some((_, _, lod_meshes)) = grouped_meshes
            .iter_mut()
            .find(|(name, parent_name, _)| name == &base_name && parent_name == &mesh.parent_name)
        {
            lod_meshes.push(mesh);
        } else {
            grouped_meshes.push((base_name, mesh.parent_name.clone(), vec![mesh]));
        }
    }
    
    for (name, parent_name, lod_meshes) in grouped_meshes {
        let mut mult_buf = Vec::new();
        let mut m_cursor = Cursor::new(&mut mult_buf);
        
        write_len_string(&mut m_cursor, &name)?;
        write_len_string(&mut m_cursor, &parent_name)?;
        m_cursor.write_u32::<LittleEndian>(lod_meshes.len() as u32)?;
        
        // Original HOD 2.0 MULT chunks count only the real TAGS payload here:
        // real id "TAGS" + u32 string length + "DoScars" = 15 bytes.
        // Do not include a counted padding byte; doing so shifts the following NRML.
        m_cursor.write_all(b"FORM")?;
        m_cursor.write_u32::<BigEndian>(15)?; // size
        m_cursor.write_all(b"TAGS")?;
        m_cursor.write_u32::<LittleEndian>(7)?; // string length
        m_cursor.write_all(b"DoScars")?;

        for mesh in lod_meshes {
            let bmsh_data_size = 8 + mesh.new_parts.len() * 18;
            let nrml_size = 8 + bmsh_data_size;

            // Wrap BMSH in NRML chunk.
            m_cursor.write_all(b"NRML")?;
            m_cursor.write_u32::<BigEndian>(nrml_size as u32)?;
            m_cursor.write_all(b"BMSH")?;
            m_cursor.write_u32::<BigEndian>(1400)?; // nested NRML BMSH version
            
            m_cursor.write_i32::<LittleEndian>(mesh.lod)?;
            m_cursor.write_i32::<LittleEndian>(mesh.new_parts.len() as i32)?;
            
            for part in &mesh.new_parts {
                m_cursor.write_i32::<LittleEndian>(part.material_index as i32)?;
                m_cursor.write_u32::<LittleEndian>(part.vertex_mask)?;
                m_cursor.write_i32::<LittleEndian>(part.vertices.len() as i32)?;
                m_cursor.write_i16::<LittleEndian>(-1)?; // prim_group_count
                m_cursor.write_i32::<LittleEndian>(part.indices.len() as i32)?;
            }
        }
        
        mult_chunks.push(IffChunk {
            id: "MULT".to_string(),
            chunk_type: crate::iff::ChunkType::Normal,
            version: 1400,
            data: mult_buf,
            children: Vec::new(),
        });
    }
    
    Ok(mult_chunks)
}
