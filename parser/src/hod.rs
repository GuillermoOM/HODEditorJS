use crate::iff::ChunkType;
use crate::iff::IffChunk;
use crate::xpress;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Vector2 {
    pub u: f32,
    pub v: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Matrix4 {
    pub m: [[f32; 4]; 4],
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HODVertex {
    pub position: Vector3,
    pub normal: Option<Vector3>,
    pub color: Option<u32>,
    pub uv: Option<Vector2>,
    pub tangent: Option<Vector3>,
    pub binormal: Option<Vector3>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skinning_data: Option<Vec<u8>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HODMeshPart {
    pub material_index: usize,
    pub vertex_mask: u32,
    pub vertices: Vec<HODVertex>,
    pub indices: Vec<u16>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HODMesh {
    pub name: String,
    pub parent_name: String,
    pub lod: i32,
    #[serde(default)]
    pub has_mult_tags: bool,
    pub parts: Vec<HODMeshPart>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HODJoint {
    pub name: String,
    pub parent_name: Option<String>,
    pub local_transform: Matrix4,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Vector3>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<Vector3>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<Vector3>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HODMarker {
    pub name: String,
    pub parent_joint: String,
    pub position: Vector3,
    pub rotation: Matrix4,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation_euler: Option<Vector3>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HODTexture {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub format: String,              // e.g. "DXT1", "DXT3", "DXT5", "RGBA"
    pub png_preview: Option<String>, // Base64 encoded PNG for React UI thumbnails (max 128px)
    pub png_data: Option<String>, // Base64 encoded PNG for WebGL high-resolution rendering (max 1024px)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>, // Original path if imported from TGA
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HODMaterial {
    pub name: String,
    pub shader_name: String,
    pub texture_maps: Vec<String>,
    #[serde(default)]
    pub parameters: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HODNavLight {
    pub name: String,
    pub section: u32,
    pub size: f32,
    pub phase: f32,
    pub frequency: f32,
    pub style: String,
    pub color: Vector3,
    pub distance: f32,
    pub sprite_visible: bool,
    pub high_end_only: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HODEngineBurn {
    pub name: String,
    pub parent_name: String,
    pub num_divisions: i32,
    pub num_flames: i32,
    pub vertices: Vec<Vector3>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HODEngineGlow {
    pub name: String,
    pub parent_name: String,
    pub lod: i32,
    pub mesh: HODMesh,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HODEngineShape {
    pub name: String,
    pub parent_name: String,
    pub mesh: HODMesh,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HODCollisionMesh {
    pub name: String,
    pub min_extents: Vector3,
    pub max_extents: Vector3,
    pub center: Vector3,
    pub radius: f32,
    pub mesh: HODMesh,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HODDockpoint {
    pub position: Vector3,
    pub rotation: Matrix4,
    pub tolerance: f32,
    pub max_speed: f32,
    pub extra1: u32,
    pub extra2: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HODDockpath {
    pub name: String,
    pub parent_name: String,
    pub points: Vec<HODDockpoint>,
    pub val1: u32,
    pub val2: u32,
    pub val3: u32,
    pub val4: u32,
    pub val5: u32,
    pub compatible_ships: String,
    pub padding1: u32,
    pub padding2: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HODQuaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HODKeyframe {
    pub time: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Vector3>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<HODQuaternion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation_euler: Option<Vector3>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<Vector3>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HODAnimationTrack {
    pub joint_name: String,
    pub keyframes: Vec<HODKeyframe>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HODAnimation {
    pub name: String,
    pub duration: f64,
    pub tracks: Vec<HODAnimationTrack>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HODModel {
    pub version: u32,
    pub is_v2: bool,
    pub name: String,
    pub textures: Vec<HODTexture>,
    pub materials: Vec<HODMaterial>,
    pub meshes: Vec<HODMesh>,
    pub joints: Vec<HODJoint>,
    pub markers: Vec<HODMarker>,
    pub nav_lights: Vec<HODNavLight>,
    pub engine_burns: Vec<HODEngineBurn>,
    pub engine_glows: Vec<HODEngineGlow>,
    pub engine_shapes: Vec<HODEngineShape>,
    pub collision_meshes: Vec<HODCollisionMesh>,
    pub dockpaths: Vec<HODDockpath>,
    #[serde(default)]
    pub animations: Vec<HODAnimation>,
    #[serde(default)]
    pub textures_modified: bool,
    #[serde(skip)]
    pub preserved_chunks: Vec<IffChunk>,
}

pub struct ParsingContext {
    pub texture_pool: Cursor<Vec<u8>>,
    pub mesh_pool: Cursor<Vec<u8>>,
    pub face_pool: Cursor<Vec<u8>>,
    pub is_v2: bool,
    pub hod_dir: Option<PathBuf>,
    pub uncompressed_dir: Option<PathBuf>,
    pub hod_file_path: Option<PathBuf>,
}

fn companion_mad_candidates(hod_path: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    candidates.push(hod_path.with_extension("mad"));

    let parent = match hod_path.parent() {
        Some(parent) => parent,
        None => return candidates,
    };

    let stem = match hod_path.file_stem().and_then(|s| s.to_str()) {
        Some(stem) => stem,
        None => return candidates,
    };

    let mut normalized_stems = Vec::new();
    for suffix in [
        "_2.0_original",
        "_1.0_original",
        "_from_2.0_to_2.0",
        "_from_1.0_to_2.0",
    ] {
        if let Some(base) = stem.strip_suffix(suffix) {
            normalized_stems.push(base.to_string());
        }
    }

    if let Some((base, _)) = stem.split_once("_from_") {
        normalized_stems.push(base.to_string());
    }

    for normalized_stem in normalized_stems {
        let candidate = parent.join(format!("{}.mad", normalized_stem));
        if !candidates.iter().any(|existing| existing == &candidate) {
            candidates.push(candidate);
        }
    }

    candidates
}

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
    pub fn new() -> Self {
        Self {
            version: 0,
            is_v2: false,
            name: String::new(),
            textures: Vec::new(),
            materials: Vec::new(),
            meshes: Vec::new(),
            joints: Vec::new(),
            markers: Vec::new(),
            nav_lights: Vec::new(),
            engine_burns: Vec::new(),
            engine_glows: Vec::new(),
            engine_shapes: Vec::new(),
            collision_meshes: Vec::new(),
            dockpaths: Vec::new(),
            animations: Vec::new(),
            textures_modified: false,
            preserved_chunks: Vec::new(),
        }
    }

    /// Legacy parser fallback
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        Self::parse_with_external(bytes, None, None)
    }

    /// Native parser that ingests raw HOD file bytes and extracts the complete model representation with external TGA mapping
    pub fn parse_with_external(
        bytes: &[u8],
        hod_file_path: Option<&str>,
        uncompressed_path: Option<&str>,
    ) -> Result<Self, String> {
        println!(
            "[RUST] HODModel::parse: Initiating with {} bytes.",
            bytes.len()
        );
        let mut cursor = Cursor::new(bytes);

        // Step 1: Parse the top-level IFF container
        let mut chunks = Vec::new();
        while cursor.position() < bytes.len() as u64 {
            match IffChunk::read_chunk(&mut cursor) {
                Ok(chunk) => chunks.push(chunk),
                Err(e) => {
                    println!("[RUST] IFF boundary read error: {}", e);
                    return Err(format!("IFF boundary read error: {}", e));
                }
            }
        }
        println!("[RUST] HODModel::parse: Chunks parsed: {}", chunks.len());

        let hod_file_path_buf = hod_file_path.map(|p| std::path::Path::new(p).to_path_buf());
        let hod_dir = hod_file_path_buf
            .as_ref()
            .and_then(|p| p.parent().map(|parent| parent.to_path_buf()));
        let uncompressed_dir = uncompressed_path.map(|p| std::path::Path::new(p).to_path_buf());

        // Step 2: Detect the POOL chunk (indicates HWR HOD v2.0)
        let mut context = ParsingContext {
            texture_pool: Cursor::new(Vec::new()),
            mesh_pool: Cursor::new(Vec::new()),
            face_pool: Cursor::new(Vec::new()),
            is_v2: chunks.iter().any(|c| c.id == "POOL"),
            hod_dir,
            uncompressed_dir,
            hod_file_path: hod_file_path_buf,
        };

        for chunk in &chunks {
            if chunk.id == "POOL" {
                println!("[RUST] POOL chunk detected. Decompressing streams...");
                let mut pool_cursor = Cursor::new(&chunk.data);
                let _pool_type = pool_cursor
                    .read_u32::<LittleEndian>()
                    .map_err(|e| e.to_string())?;

                // Texture data stream
                let comp_tex_len = pool_cursor
                    .read_u32::<LittleEndian>()
                    .map_err(|e| e.to_string())? as usize;
                let decomp_tex_len = pool_cursor
                    .read_u32::<LittleEndian>()
                    .map_err(|e| e.to_string())? as usize;
                println!(
                    "[RUST]   Texture pool: compressed={}, decompressed={}",
                    comp_tex_len, decomp_tex_len
                );
                let mut comp_tex = vec![0u8; comp_tex_len];
                pool_cursor
                    .read_exact(&mut comp_tex)
                    .map_err(|e| e.to_string())?;
                let decomp_tex = if comp_tex_len == decomp_tex_len {
                    comp_tex
                } else {
                    xpress::decompress(&comp_tex, decomp_tex_len)?
                };
                context.texture_pool = Cursor::new(decomp_tex);

                // Mesh data stream
                let comp_mesh_len = pool_cursor
                    .read_u32::<LittleEndian>()
                    .map_err(|e| e.to_string())? as usize;
                let decomp_mesh_len = pool_cursor
                    .read_u32::<LittleEndian>()
                    .map_err(|e| e.to_string())? as usize;
                println!(
                    "[RUST]   Mesh pool: compressed={}, decompressed={}",
                    comp_mesh_len, decomp_mesh_len
                );
                let mut comp_mesh = vec![0u8; comp_mesh_len];
                pool_cursor
                    .read_exact(&mut comp_mesh)
                    .map_err(|e| e.to_string())?;
                let decomp_mesh = if comp_mesh_len == decomp_mesh_len {
                    comp_mesh
                } else {
                    xpress::decompress(&comp_mesh, decomp_mesh_len)?
                };
                println!("[RUST] First 256 bytes of decompressed mesh pool:");
                for i in 0..16 {
                    let offset = i * 16;
                    if offset + 16 <= decomp_mesh.len() {
                        let slice = &decomp_mesh[offset..offset + 16];
                        let floats: Vec<f32> = slice
                            .chunks(4)
                            .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                            .collect();
                        println!("  {:04X}: {:02x?} | {:?}", offset, slice, floats);
                    }
                }
                context.mesh_pool = Cursor::new(decomp_mesh);

                // Face data stream
                let comp_face_len = pool_cursor
                    .read_u32::<LittleEndian>()
                    .map_err(|e| e.to_string())? as usize;
                let decomp_face_len = pool_cursor
                    .read_u32::<LittleEndian>()
                    .map_err(|e| e.to_string())? as usize;
                println!(
                    "[RUST]   Face pool: compressed={}, decompressed={}",
                    comp_face_len, decomp_face_len
                );
                let mut comp_face = vec![0u8; comp_face_len];
                pool_cursor
                    .read_exact(&mut comp_face)
                    .map_err(|e| e.to_string())?;
                let decomp_face = if comp_face_len == decomp_face_len {
                    comp_face
                } else {
                    xpress::decompress(&comp_face, decomp_face_len)?
                };
                println!("[RUST] First 128 u16s of decompressed face pool:");
                for chunk in decomp_face.chunks(32).take(8) {
                    let u16s: Vec<u16> = chunk
                        .chunks(2)
                        .map(|c| {
                            if c.len() == 2 {
                                u16::from_le_bytes([c[0], c[1]])
                            } else {
                                0
                            }
                        })
                        .collect();
                    println!("  {:?}", u16s);
                }
                context.face_pool = Cursor::new(decomp_face);

                println!("[RUST] POOL decompression complete.");
                break;
            }
        }

        // Step 3: Parse standard chunks (VERS, NAME, HVMD, DTRM)
        println!("[RUST] Step 3: Parsing standard chunks (VERS, NAME, HVMD, DTRM)...");
        let mut model_version = 0x200;
        let mut model_name = String::new();
        let mut textures = Vec::new();
        let mut materials = Vec::new();
        let mut meshes = Vec::new();
        let mut joints = Vec::new();
        let mut markers = Vec::new();
        let mut nav_lights = Vec::new();
        let mut engine_burns = Vec::new();
        let mut engine_glows = Vec::new();
        let mut engine_shapes = Vec::new();
        let mut collision_meshes = Vec::new();
        let mut dockpaths = Vec::new();
        let mut preserved_chunks = Vec::new();

        for chunk in &chunks {
            println!(
                "[RUST]   Processing chunk ID: '{}' (type={:?}, size={})",
                chunk.id,
                chunk.chunk_type,
                chunk.data.len()
            );
            match chunk.id.as_str() {
                "VERS" => {
                    println!("[RUST]     Parsing VERS...");
                    let mut r = Cursor::new(&chunk.data);
                    if let Ok(ver) = r.read_u32::<LittleEndian>() {
                        model_version = ver;
                    }
                }
                "NAME" => {
                    println!("[RUST]     Parsing NAME...");
                    model_name = String::from_utf8_lossy(&chunk.data)
                        .trim_matches('\0')
                        .to_string();
                }
                "HVMD" => {
                    println!(
                        "[RUST]     Parsing HVMD container children (count: {})...",
                        chunk.children.len()
                    );
                    for sub_chunk in &chunk.children {
                        println!(
                            "[RUST]       HVMD child: ID='{}', type={:?}, data_len={}",
                            sub_chunk.id,
                            sub_chunk.chunk_type,
                            sub_chunk.data.len()
                        );
                    }
                    // Pass 1: Parse all texture chunks first
                    for sub_chunk in &chunk.children {
                        match sub_chunk.id.trim() {
                            "LMIP" => {
                                // Extract textures from LMIP
                                if sub_chunk.data.len() < 12 {
                                    println!(
                                        "[RUST]       Skipping tiny LMIP chunk (len={})",
                                        sub_chunk.data.len()
                                    );
                                    continue;
                                }
                                println!("[RUST]       Parsing LMIP texture chunk...");
                                let tex = parse_texture(sub_chunk, &mut context)
                                    .map_err(|e| format!("Error in LMIP texture: {}", e))?;
                                println!("[RUST]         Successfully parsed texture: name='{}', format='{}'", tex.name, tex.format);
                                textures.push(tex);
                            }
                            "TEXM" => {
                                // Extract textures
                                let tex = parse_texture(sub_chunk, &mut context)
                                    .map_err(|e| format!("Error in TEXM: {}", e))?;
                                textures.push(tex);
                            }
                            _ => {}
                        }
                    }

                    // Pass 2: Parse all materials and meshes
                    for sub_chunk in &chunk.children {
                        match sub_chunk.id.trim() {
                            "STAT" => {
                                // Extract materials from STAT
                                let mat = parse_stat_material(sub_chunk, &textures)
                                    .map_err(|e| format!("Error in STAT material: {}", e))?;
                                materials.push(mat);
                            }
                            "MATT" => {
                                // Extract materials
                                let mat = parse_stat_material(sub_chunk, &textures)
                                    .map_err(|e| format!("Error in MATT: {}", e))?;
                                materials.push(mat);
                            }
                            "MSHL" => {
                                // Extract meshes
                                println!(
                                    "[RUST]         MSHL children count: {}",
                                    sub_chunk.children.len()
                                );
                                for mesh_chunk in &sub_chunk.children {
                                    if mesh_chunk.id.trim() == "BMSH" {
                                        let mut mesh = parse_basic_mesh(mesh_chunk, &mut context)
                                            .map_err(|e| {
                                            format!("Error in BMSH under MSHL: {}", e)
                                        })?;
                                        mesh.parent_name = "Root".to_string();
                                        meshes.push(mesh);
                                    }
                                }
                            }
                            "MULT" | "GOBG" => {
                                let mut reader = Cursor::new(&sub_chunk.data);
                                let total_len = sub_chunk.data.len();

                                // Read name string
                                let mut len_bytes = [0u8; 4];
                                if reader.read_exact(&mut len_bytes).is_ok() {
                                    let len = u32::from_le_bytes(len_bytes) as usize;
                                    println!(
                                        "[RUST]         {} named sub-mesh string length: {}",
                                        sub_chunk.id, len
                                    );

                                    if len < total_len {
                                        let mut name_bytes = vec![0u8; len];
                                        let _ = reader.read_exact(&mut name_bytes);
                                        let name = String::from_utf8_lossy(&name_bytes).to_string();

                                        // Read parent name string
                                        if reader.read_exact(&mut len_bytes).is_ok() {
                                            let parent_len = u32::from_le_bytes(len_bytes) as usize;

                                            let remaining_after_parent_len = total_len
                                                .saturating_sub(reader.position() as usize);
                                            if parent_len <= remaining_after_parent_len {
                                                println!(
                                                    "[RUST]         parent_len string length: {}",
                                                    parent_len
                                                );
                                                let mut parent_bytes = vec![0u8; parent_len];
                                                let _ = reader.read_exact(&mut parent_bytes);
                                                let parent_name =
                                                    String::from_utf8_lossy(&parent_bytes)
                                                        .trim_matches('\0')
                                                        .to_string();

                                                // Read LODCount
                                                let _lod_count =
                                                    reader.read_u32::<LittleEndian>().unwrap_or(0);

                                                // Parse remaining bytes as child chunks
                                                let current_pos = reader.position() as usize;
                                                let mut sub_chunks = Vec::new();
                                                let mut sub_cursor =
                                                    Cursor::new(&sub_chunk.data[current_pos..]);
                                                println!("[RUST]         MULT payload current_pos={}, total_len={}, remaining={}", current_pos, sub_chunk.data.len(), sub_chunk.data.len() - current_pos);
                                                let mut debug_bytes = [0u8; 16];
                                                let pos = sub_cursor.position();
                                                let _ = sub_cursor.read(&mut debug_bytes);
                                                println!("[RUST]         Next 16 bytes at pos {}: {:02x?}", pos, debug_bytes);
                                                sub_cursor.set_position(pos);

                                                while sub_cursor.position()
                                                    < sub_cursor.get_ref().len() as u64
                                                {
                                                    match IffChunk::read_chunk(&mut sub_cursor) {
                                                        Ok(c) => sub_chunks.push(c),
                                                        Err(e) => {
                                                            println!("[RUST] WARNING: Failed to read chunk inside MULT: {}", e);
                                                            break;
                                                        }
                                                    }
                                                }

                                                println!(
                                                    "[RUST]         {} child chunks parsed: {}",
                                                    sub_chunk.id,
                                                    sub_chunks.len()
                                                );
                                                let has_mult_tags = sub_chunks
                                                    .iter()
                                                    .any(|child| child.id.trim() == "TAGS");
                                                for child in &sub_chunks {
                                                    println!("[RUST]         Checking child chunk ID='{}' (len={}) against BMSH", child.id, child.data.len());

                                                    let is_nrml_bmsh = child.id.trim() == "NRML"
                                                        && child.data.starts_with(b"BMSH");
                                                    let actual_child = if is_nrml_bmsh {
                                                        println!("[RUST]         Unwrapping BMSH from NRML chunk");
                                                        std::borrow::Cow::Owned(IffChunk {
                                                            id: "BMSH".to_string(),
                                                            chunk_type:
                                                                crate::iff::ChunkType::Default,
                                                            version: 0,
                                                            data: child.data[4..].to_vec(),
                                                            children: Vec::new(),
                                                        })
                                                    } else {
                                                        std::borrow::Cow::Borrowed(child)
                                                    };

                                                    if actual_child.id.trim() == "BMSH" {
                                                        let mut mesh = parse_basic_mesh(
                                                            &actual_child,
                                                            &mut context,
                                                        )
                                                        .map_err(|e| {
                                                            format!(
                                                                "Error in BMSH under {}: {}",
                                                                sub_chunk.id, e
                                                            )
                                                        })?;
                                                        mesh.name = name.clone();
                                                        mesh.parent_name = parent_name.clone();
                                                        mesh.has_mult_tags = has_mult_tags;
                                                        println!("[RUST]         PUSHING MESH '{}' to meshes list. Previous meshes.len()={}", mesh.name, meshes.len());
                                                        meshes.push(mesh);
                                                    }
                                                }
                                            } else {
                                                println!("[RUST]         WARNING: Suspicious parent_len {} (remaining={}), skipping sub-mesh hierarchy parse for {}", parent_len, remaining_after_parent_len, sub_chunk.id);
                                            }
                                        }
                                    } else {
                                        println!("[RUST]         WARNING: Suspicious name string length {} (total_len={}), skipping sub-mesh hierarchy parse for {}", len, total_len, sub_chunk.id);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                "DTRM" => {
                    println!(
                        "[RUST]     Parsing DTRM container children (count: {})...",
                        chunk.children.len()
                    );
                    // Data logical structures
                    let mut pending_kdop_collision: Option<HODCollisionMesh> = None;
                    let mut pending_cold_collision: Option<HODCollisionMesh> = None;
                    for sub_chunk in &chunk.children {
                        println!(
                            "[RUST]       DTRM Sub-chunk: '{}' (size={})",
                            sub_chunk.id,
                            sub_chunk.data.len()
                        );
                        match sub_chunk.id.trim() {
                            "HIER" => {
                                // Parse joints skeleton hierarchy
                                joints = parse_joints(sub_chunk)
                                    .map_err(|e| format!("Error in HIER: {}", e))?;
                            }
                            "MRKS" => {
                                // HOD 2.0 markers chunk
                                let m = parse_markers_v2(sub_chunk)
                                    .map_err(|e| format!("Error in MRKS: {}", e))?;
                                markers.extend(m);
                            }
                            "MRKR" => {
                                // HOD 1.0 marker container
                                if let Some(head) = sub_chunk.find_child("HEAD") {
                                    let m = parse_marker_v1(head)
                                        .map_err(|e| format!("Error in MRKR: {}", e))?;
                                    markers.push(m);
                                }
                            }
                            "NAVL" => {
                                let mut parse_navl = || -> Result<(), String> {
                                    let mut r = Cursor::new(&sub_chunk.data);
                                    let count =
                                        r.read_u32::<LittleEndian>().map_err(|e| e.to_string())?;
                                    let remaining_bytes = r.get_ref().len() as u64 - r.position();
                                    let max_possible_navs = remaining_bytes / 32;
                                    if count as u64 > max_possible_navs {
                                        return Err("NAVL count exceeds buffer space".to_string());
                                    }
                                    for _ in 0..count {
                                        let name = read_len_string(&mut r)?;
                                        let section = r
                                            .read_u32::<LittleEndian>()
                                            .map_err(|e| e.to_string())?;
                                        let size = r
                                            .read_f32::<LittleEndian>()
                                            .map_err(|e| e.to_string())?;
                                        let phase = r
                                            .read_f32::<LittleEndian>()
                                            .map_err(|e| e.to_string())?;
                                        let frequency = r
                                            .read_f32::<LittleEndian>()
                                            .map_err(|e| e.to_string())?;
                                        let style = read_len_string(&mut r)?;
                                        let cx = r
                                            .read_f32::<LittleEndian>()
                                            .map_err(|e| e.to_string())?;
                                        let cy = r
                                            .read_f32::<LittleEndian>()
                                            .map_err(|e| e.to_string())?;
                                        let cz = r
                                            .read_f32::<LittleEndian>()
                                            .map_err(|e| e.to_string())?;
                                        let color = Vector3 {
                                            x: cx,
                                            y: cy,
                                            z: cz,
                                        };
                                        let _unused = r
                                            .read_f32::<LittleEndian>()
                                            .map_err(|e| e.to_string())?;
                                        let distance = r
                                            .read_f32::<LittleEndian>()
                                            .map_err(|e| e.to_string())?;
                                        let sprite_visible =
                                            r.read_u8().map_err(|e| e.to_string())? != 0;
                                        let high_end_only =
                                            r.read_u8().map_err(|e| e.to_string())? != 0;
                                        nav_lights.push(HODNavLight {
                                            name,
                                            section,
                                            size,
                                            phase,
                                            frequency,
                                            style,
                                            color,
                                            distance,
                                            sprite_visible,
                                            high_end_only,
                                        });
                                    }
                                    Ok(())
                                };
                                parse_navl().map_err(|e| {
                                    format!(
                                        "Error in NAVL (data size={}): {}",
                                        sub_chunk.data.len(),
                                        e
                                    )
                                })?;
                            }
                            "BURN" => {
                                let mut parse_burn = || -> Result<(), String> {
                                    let mut r = Cursor::new(&sub_chunk.data);
                                    let name = read_len_string(&mut r)?;
                                    let parent_name = read_len_string(&mut r)?;
                                    let num_divisions =
                                        r.read_i32::<LittleEndian>().map_err(|e| e.to_string())?;
                                    let num_flames =
                                        r.read_i32::<LittleEndian>().map_err(|e| e.to_string())?;
                                    if num_divisions < 0 || num_flames < 0 {
                                        return Err(
                                            "Negative divisions or flames in BURN".to_string()
                                        );
                                    }
                                    let total_vertices = (num_divisions * num_flames) as usize;
                                    let remaining_bytes = r.get_ref().len() as u64 - r.position();
                                    let max_possible_verts = remaining_bytes / 12; // 3 floats = 12 bytes
                                    if total_vertices as u64 > max_possible_verts {
                                        return Err(
                                            "BURN total_vertices exceeds buffer space".to_string()
                                        );
                                    }
                                    let mut vertices = Vec::with_capacity(total_vertices);
                                    for _ in 0..total_vertices {
                                        let vx = r
                                            .read_f32::<LittleEndian>()
                                            .map_err(|e| e.to_string())?;
                                        let vy = r
                                            .read_f32::<LittleEndian>()
                                            .map_err(|e| e.to_string())?;
                                        let vz = r
                                            .read_f32::<LittleEndian>()
                                            .map_err(|e| e.to_string())?;
                                        vertices.push(Vector3 {
                                            x: vx,
                                            y: vy,
                                            z: vz,
                                        });
                                    }
                                    engine_burns.push(HODEngineBurn {
                                        name,
                                        parent_name,
                                        num_divisions,
                                        num_flames,
                                        vertices,
                                    });
                                    Ok(())
                                };
                                parse_burn().map_err(|e| {
                                    format!(
                                        "Error in BURN (data size={}): {}",
                                        sub_chunk.data.len(),
                                        e
                                    )
                                })?;
                            }
                            "DOCK" => match parse_dock_chunk(&sub_chunk.data, context.is_v2) {
                                Ok(parsed_dockpaths) => dockpaths.extend(parsed_dockpaths),
                                Err(e) => {
                                    println!("[RUST] WARNING: Failed to parse DOCK chunk: {}", e);
                                    let preview = sub_chunk.data.iter().take(128).map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ");
                                    println!("[RUST] DOCK chunk size: {}, preview: {}", sub_chunk.data.len(), preview);
                                }
                            },
                            "GLOW" => {
                                let mut parse_glow =
                                    |context: &mut ParsingContext| -> Result<(), String> {
                                        if let Some(info_chunk) = sub_chunk.find_child("INFO") {
                                            let mut r = Cursor::new(&info_chunk.data);
                                            let name = read_len_string(&mut r)?;
                                            let parent_name = read_len_string(&mut r)?;
                                            let lod = r
                                                .read_i32::<LittleEndian>()
                                                .map_err(|e| e.to_string())?;

                                            if let Some(bmsh_chunk) =
                                                find_child_trimmed(sub_chunk, "BMSH")
                                            {
                                                let mesh = parse_basic_mesh(bmsh_chunk, context)?;
                                                engine_glows.push(HODEngineGlow {
                                                    name,
                                                    parent_name,
                                                    lod,
                                                    mesh,
                                                });
                                            }
                                        }
                                        Ok(())
                                    };
                                parse_glow(&mut context)
                                    .map_err(|e| format!("Error in GLOW: {}", e))?;
                            }
                            "ETSH" => {
                                let mut parse_etsh =
                                    |context: &mut ParsingContext| -> Result<(), String> {
                                        let mut r = Cursor::new(&sub_chunk.data);
                                        let name = read_len_string(&mut r)?;
                                        let parent_name = read_len_string(&mut r)?;

                                        let mut vertices = Vec::new();
                                        let mut indices = Vec::new();
                                        let mut current_vertex_idx = 0;

                                        while r.position() < sub_chunk.data.len() as u64 {
                                            if r.position() + 4 > sub_chunk.data.len() as u64 {
                                                break;
                                            }
                                            let indice_count = r
                                                .read_i32::<LittleEndian>()
                                                .map_err(|e| e.to_string())?
                                                as usize;
                                            for _ in 0..indice_count {
                                                if r.position() + 12 > sub_chunk.data.len() as u64 {
                                                    break;
                                                }
                                                let vx = r
                                                    .read_f32::<LittleEndian>()
                                                    .map_err(|e| e.to_string())?;
                                                let vy = r
                                                    .read_f32::<LittleEndian>()
                                                    .map_err(|e| e.to_string())?;
                                                let vz = r
                                                    .read_f32::<LittleEndian>()
                                                    .map_err(|e| e.to_string())?;
                                                vertices.push(HODVertex {
                                                    position: Vector3 {
                                                        x: vx,
                                                        y: vy,
                                                        z: vz,
                                                    },
                                                    normal: None,
                                                    color: None,
                                                    uv: None,
                                                    tangent: None,
                                                    binormal: None,
                                                    skinning_data: None,
                                                });
                                                indices.push(current_vertex_idx as u16);
                                                current_vertex_idx += 1;
                                            }
                                        }

                                        let parts = vec![HODMeshPart {
                                            material_index: 0,
                                            vertex_mask: 1, // position only
                                            vertices,
                                            indices,
                                        }];

                                        engine_shapes.push(HODEngineShape {
                                            name,
                                            parent_name,
                                            mesh: HODMesh {
                                                name: "EngineShape".to_string(),
                                                parent_name: String::new(),
                                                lod: 0,
                                                has_mult_tags: false,
                                                parts,
                                            },
                                        });

                                        Ok(())
                                    };
                                parse_etsh(&mut context)
                                    .map_err(|e| format!("Error in ETSH: {}", e))?;
                            }
                            "KDOP" => {
                                match parse_kdop_collision_mesh(&sub_chunk.data, "KDOP") {
                                    Ok(kdop_collision) => {
                                        pending_kdop_collision = Some(kdop_collision);
                                    }
                                    Err(e) => {
                                        println!(
                                            "[RUST] WARNING: Failed to parse KDOP collision mesh: {}",
                                            e
                                        );
                                    }
                                }
                                preserved_chunks.push(sub_chunk.clone());
                            }
                            "COLD" => {
                                let mut parse_cold = || -> Result<HODCollisionMesh, String> {
                                    let mut r_prefix = Cursor::new(&sub_chunk.data);
                                    let name = if sub_chunk.data.len() >= 4 {
                                        read_len_string(&mut r_prefix)
                                            .unwrap_or_else(|_| "CollisionMesh".to_string())
                                    } else {
                                        "CollisionMesh".to_string()
                                    };

                                    let mut min_extents = Vector3 {
                                        x: 0.0,
                                        y: 0.0,
                                        z: 0.0,
                                    };
                                    let mut max_extents = Vector3 {
                                        x: 0.0,
                                        y: 0.0,
                                        z: 0.0,
                                    };
                                    let mut center = Vector3 {
                                        x: 0.0,
                                        y: 0.0,
                                        z: 0.0,
                                    };
                                    let mut radius = 0.0;
                                    let mut vertices = Vec::new();
                                    let mut indices = Vec::new();

                                    if r_prefix.position() + 40 <= sub_chunk.data.len() as u64 {
                                        min_extents = Vector3 {
                                            x: r_prefix
                                                .read_f32::<LittleEndian>()
                                                .map_err(|e| e.to_string())?,
                                            y: r_prefix
                                                .read_f32::<LittleEndian>()
                                                .map_err(|e| e.to_string())?,
                                            z: r_prefix
                                                .read_f32::<LittleEndian>()
                                                .map_err(|e| e.to_string())?,
                                        };
                                        max_extents = Vector3 {
                                            x: r_prefix
                                                .read_f32::<LittleEndian>()
                                                .map_err(|e| e.to_string())?,
                                            y: r_prefix
                                                .read_f32::<LittleEndian>()
                                                .map_err(|e| e.to_string())?,
                                            z: r_prefix
                                                .read_f32::<LittleEndian>()
                                                .map_err(|e| e.to_string())?,
                                        };
                                        center = Vector3 {
                                            x: r_prefix
                                                .read_f32::<LittleEndian>()
                                                .map_err(|e| e.to_string())?,
                                            y: r_prefix
                                                .read_f32::<LittleEndian>()
                                                .map_err(|e| e.to_string())?,
                                            z: r_prefix
                                                .read_f32::<LittleEndian>()
                                                .map_err(|e| e.to_string())?,
                                        };
                                        radius = r_prefix
                                            .read_f32::<LittleEndian>()
                                            .map_err(|e| e.to_string())?;
                                    }

                                    for child in &sub_chunk.children {
                                        match child.id.as_str() {
                                            "BBOX" => {
                                                let mut r_box = Cursor::new(&child.data);
                                                if child.data.len() >= 24 {
                                                    let min_x = r_box
                                                        .read_f32::<LittleEndian>()
                                                        .map_err(|e| e.to_string())?;
                                                    let min_y = r_box
                                                        .read_f32::<LittleEndian>()
                                                        .map_err(|e| e.to_string())?;
                                                    let min_z = r_box
                                                        .read_f32::<LittleEndian>()
                                                        .map_err(|e| e.to_string())?;
                                                    min_extents = Vector3 {
                                                        x: min_x,
                                                        y: min_y,
                                                        z: min_z,
                                                    };

                                                    let max_x = r_box
                                                        .read_f32::<LittleEndian>()
                                                        .map_err(|e| e.to_string())?;
                                                    let max_y = r_box
                                                        .read_f32::<LittleEndian>()
                                                        .map_err(|e| e.to_string())?;
                                                    let max_z = r_box
                                                        .read_f32::<LittleEndian>()
                                                        .map_err(|e| e.to_string())?;
                                                    max_extents = Vector3 {
                                                        x: max_x,
                                                        y: max_y,
                                                        z: max_z,
                                                    };
                                                }
                                            }
                                            "BSPH" => {
                                                let mut r_sph = Cursor::new(&child.data);
                                                if child.data.len() >= 16 {
                                                    let cx = r_sph
                                                        .read_f32::<LittleEndian>()
                                                        .map_err(|e| e.to_string())?;
                                                    let cy = r_sph
                                                        .read_f32::<LittleEndian>()
                                                        .map_err(|e| e.to_string())?;
                                                    let cz = r_sph
                                                        .read_f32::<LittleEndian>()
                                                        .map_err(|e| e.to_string())?;
                                                    center = Vector3 {
                                                        x: cx,
                                                        y: cy,
                                                        z: cz,
                                                    };
                                                    radius = r_sph
                                                        .read_f32::<LittleEndian>()
                                                        .map_err(|e| e.to_string())?;
                                                }
                                            }
                                            "TRIS" => {
                                                let mut r_tris = Cursor::new(&child.data);
                                                if child.data.len() >= 4 {
                                                    let vertex_count = r_tris
                                                        .read_i32::<LittleEndian>()
                                                        .map_err(|e| e.to_string())?
                                                        as usize;
                                                    for _ in 0..vertex_count {
                                                        if r_tris.position() + 12
                                                            > child.data.len() as u64
                                                        {
                                                            break;
                                                        }
                                                        let vx = r_tris
                                                            .read_f32::<LittleEndian>()
                                                            .map_err(|e| e.to_string())?;
                                                        let vy = r_tris
                                                            .read_f32::<LittleEndian>()
                                                            .map_err(|e| e.to_string())?;
                                                        let vz = r_tris
                                                            .read_f32::<LittleEndian>()
                                                            .map_err(|e| e.to_string())?;
                                                        vertices.push(HODVertex {
                                                            position: Vector3 {
                                                                x: vx,
                                                                y: vy,
                                                                z: vz,
                                                            },
                                                            normal: None,
                                                            color: None,
                                                            uv: None,
                                                            tangent: None,
                                                            binormal: None,
                                                            skinning_data: None,
                                                        });
                                                    }

                                                    if r_tris.position() + 4
                                                        <= child.data.len() as u64
                                                    {
                                                        let idx_count = r_tris
                                                            .read_i32::<LittleEndian>()
                                                            .map_err(|e| e.to_string())?
                                                            as usize;
                                                        for _ in 0..idx_count {
                                                            if r_tris.position() + 2
                                                                > child.data.len() as u64
                                                            {
                                                                break;
                                                            }
                                                            let idx = r_tris
                                                                .read_u16::<LittleEndian>()
                                                                .map_err(|e| e.to_string())?;
                                                            indices.push(idx);
                                                        }
                                                    }
                                                }
                                            }
                                            _ => {
                                                preserved_chunks.push(child.clone());
                                            }
                                        }
                                    }

                                    let parts = vec![HODMeshPart {
                                        material_index: 0,
                                        vertex_mask: 1, // position only
                                        vertices,
                                        indices,
                                    }];

                                    Ok(HODCollisionMesh {
                                        name: name.clone(),
                                        min_extents,
                                        max_extents,
                                        center,
                                        radius,
                                        mesh: HODMesh {
                                            name: "CollisionMesh".to_string(),
                                            parent_name: name,
                                            lod: 0,
                                            has_mult_tags: false,
                                            parts,
                                        },
                                    })
                                };
                                let cold_collision =
                                    parse_cold().map_err(|e| format!("Error in COLD: {}", e))?;
                                pending_cold_collision = Some(cold_collision);
                            }
                            _ => {
                                // Preserve unparsed DTRM sub-chunks (KDOP, SCAR, etc.)
                                preserved_chunks.push(sub_chunk.clone());
                            }
                        }
                    }
                    if let Some(mut kdop_collision) = pending_kdop_collision.take() {
                        if let Some(cold_collision) = pending_cold_collision.take() {
                            kdop_collision.name = cold_collision.name.clone();
                            kdop_collision.mesh.name = "CollisionMesh".to_string();
                            kdop_collision.mesh.parent_name = cold_collision.name.clone();
                            if cold_collision
                                .mesh
                                .parts
                                .iter()
                                .any(|part| !part.vertices.is_empty() && !part.indices.is_empty())
                            {
                                kdop_collision.mesh.parts = cold_collision.mesh.parts;
                            }
                            if cold_collision.radius > 0.0 {
                                kdop_collision.min_extents = cold_collision.min_extents;
                                kdop_collision.max_extents = cold_collision.max_extents;
                                kdop_collision.center = cold_collision.center;
                                kdop_collision.radius = cold_collision.radius;
                            }
                        } else {
                            let root_name = joints
                                .first()
                                .map(|j| j.name.clone())
                                .unwrap_or_else(|| "Root".to_string());
                            kdop_collision.name = "CollisionMesh".to_string();
                            kdop_collision.mesh.parent_name = root_name;
                        }
                        collision_meshes.push(kdop_collision);
                    } else if let Some(cold_collision) = pending_cold_collision.take() {
                        collision_meshes.push(cold_collision);
                    }
                }
                _ => {
                    // Preserve unparsed root chunks (INFO, etc.)
                    if chunk.id == "INFO" {
                        preserved_chunks.push(chunk.clone());
                    }
                }
            }
        }

        println!("[RUST] Finished parsing! Name='{}', Meshes={}, Joints={}, Markers={}, NavLights={}, EngineBurns={}, EngineGlows={}, EngineShapes={}, CollisionMeshes={}, Dockpaths={}", 
                 model_name, meshes.len(), joints.len(), markers.len(), nav_lights.len(), engine_burns.len(), engine_glows.len(), engine_shapes.len(), collision_meshes.len(), dockpaths.len());

        let mut model = Self {
            version: model_version,
            is_v2: context.is_v2,
            name: model_name,
            textures,
            materials,
            meshes,
            joints,
            markers,
            nav_lights,
            engine_burns,
            engine_glows,
            engine_shapes,
            collision_meshes,
            dockpaths,
            animations: Vec::new(),
            textures_modified: false,
            preserved_chunks,
        };

        // Phase 3: Animation Loading & parsing companion .mad or legacy KEYF
        if let Some(ref path_buf) = context.hod_file_path {
            if let Some(mad_path) = companion_mad_candidates(path_buf)
                .into_iter()
                .find(|candidate| candidate.exists())
            {
                println!(
                    "[RUST] Found companion .mad file: {:?}. Loading...",
                    mad_path
                );
                if let Ok(mad_bytes) = std::fs::read(&mad_path) {
                    match parse_mad_bytes(&mad_bytes, &model.joints) {
                        Ok(anims) => {
                            println!(
                                "[RUST] Loaded {} animations from companion MAD file.",
                                anims.len()
                            );
                            model.animations = anims;
                        }
                        Err(e) => {
                            println!("[RUST] WARNING: Failed to parse companion MAD file: {}", e);
                        }
                    }
                }
            }
        }

        if !context.is_v2 && model.animations.is_empty() {
            // HOD 1.0 embedded KEYF animations inside MRKR chunks
            let mut parsed_tracks = Vec::new();
            let mut global_max_time = 0.0f64;

            for chunk in &chunks {
                if chunk.id == "DTRM" {
                    for sub_chunk in &chunk.children {
                        if sub_chunk.id == "MRKR" {
                            if let Some(head) = sub_chunk.find_child("HEAD") {
                                if let Ok(m) = parse_marker_v1(head) {
                                    if let Some(keyf) = sub_chunk.find_child("KEYF") {
                                        if let Ok(curves) = parse_keyf_chunk(keyf) {
                                            if !curves.is_empty() {
                                                // Gather unique times
                                                let mut unique_times: Vec<f64> = Vec::new();
                                                for curve in &curves {
                                                    for kf in &curve.keyframes {
                                                        if !unique_times
                                                            .iter()
                                                            .any(|&t| (t - kf.time).abs() < 1e-5)
                                                        {
                                                            unique_times.push(kf.time);
                                                        }
                                                        if kf.time > global_max_time {
                                                            global_max_time = kf.time;
                                                        }
                                                    }
                                                }
                                                unique_times
                                                    .sort_by(|a, b| a.partial_cmp(b).unwrap());

                                                let default_pos = m.position.clone();
                                                let default_euler =
                                                    m.rotation_euler.clone().unwrap_or(Vector3 {
                                                        x: 0.0,
                                                        y: 0.0,
                                                        z: 0.0,
                                                    });
                                                let default_scale = Vector3 {
                                                    x: 1.0,
                                                    y: 1.0,
                                                    z: 1.0,
                                                };

                                                let tx_curve = curves.iter().find(|c| {
                                                    c.name.ends_with("translateX")
                                                        || c.name == "translateX"
                                                });
                                                let ty_curve = curves.iter().find(|c| {
                                                    c.name.ends_with("translateY")
                                                        || c.name == "translateY"
                                                });
                                                let tz_curve = curves.iter().find(|c| {
                                                    c.name.ends_with("translateZ")
                                                        || c.name == "translateZ"
                                                });
                                                let rx_curve = curves.iter().find(|c| {
                                                    c.name.ends_with("rotateX")
                                                        || c.name == "rotateX"
                                                });
                                                let ry_curve = curves.iter().find(|c| {
                                                    c.name.ends_with("rotateY")
                                                        || c.name == "rotateY"
                                                });
                                                let rz_curve = curves.iter().find(|c| {
                                                    c.name.ends_with("rotateZ")
                                                        || c.name == "rotateZ"
                                                });

                                                let mut keyframes = Vec::new();
                                                for &time in &unique_times {
                                                    let tx = evaluate_curve(
                                                        tx_curve
                                                            .map(|c| &c.keyframes[..])
                                                            .unwrap_or(&[]),
                                                        time,
                                                        default_pos.x as f64,
                                                    );
                                                    let ty = evaluate_curve(
                                                        ty_curve
                                                            .map(|c| &c.keyframes[..])
                                                            .unwrap_or(&[]),
                                                        time,
                                                        default_pos.y as f64,
                                                    );
                                                    let tz = evaluate_curve(
                                                        tz_curve
                                                            .map(|c| &c.keyframes[..])
                                                            .unwrap_or(&[]),
                                                        time,
                                                        default_pos.z as f64,
                                                    );

                                                    let rx = evaluate_curve(
                                                        rx_curve
                                                            .map(|c| &c.keyframes[..])
                                                            .unwrap_or(&[]),
                                                        time,
                                                        default_euler.x as f64,
                                                    );
                                                    let ry = evaluate_curve(
                                                        ry_curve
                                                            .map(|c| &c.keyframes[..])
                                                            .unwrap_or(&[]),
                                                        time,
                                                        default_euler.y as f64,
                                                    );
                                                    let rz = evaluate_curve(
                                                        rz_curve
                                                            .map(|c| &c.keyframes[..])
                                                            .unwrap_or(&[]),
                                                        time,
                                                        default_euler.z as f64,
                                                    );

                                                    let pos_vec = Vector3 {
                                                        x: tx as f32,
                                                        y: ty as f32,
                                                        z: tz as f32,
                                                    };
                                                    let rot_euler = Vector3 {
                                                        x: rx as f32,
                                                        y: ry as f32,
                                                        z: rz as f32,
                                                    };
                                                    let rot_quat = euler_to_quaternion(&rot_euler);

                                                    keyframes.push(HODKeyframe {
                                                        time,
                                                        position: Some(pos_vec),
                                                        rotation: Some(rot_quat),
                                                        rotation_euler: Some(rot_euler),
                                                        scale: None,
                                                    });
                                                }

                                                parsed_tracks.push(HODAnimationTrack {
                                                    joint_name: m.name.clone(),
                                                    keyframes,
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if !parsed_tracks.is_empty() {
                model.animations.push(HODAnimation {
                    name: "DefaultAnimation".to_string(),
                    duration: if global_max_time > 0.0 {
                        global_max_time
                    } else {
                        4.0
                    },
                    tracks: parsed_tracks,
                });
                println!(
                    "[RUST] Loaded HOD 1.0 embedded anims with {} tracks.",
                    model.animations[0].tracks.len()
                );
            }
        }

        model.auto_repair_assembly_names();
        model.clean_hierarchy();
        normalize_collision_meshes(&mut model);

        // Upgrade V1 to V2 in memory
        if !model.is_v2 {
            model.upgrade_v1_to_v2();
            normalize_collision_meshes(&mut model);
        }

        Ok(model)
    }

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
            let new_heading = crate::hod::HODJoint {
                name: head_name.clone(),
                parent_name: Some(pos_name.clone()),
                local_transform: compose_transform_matrix(
                    crate::hod::Vector3 {
                        x: 0.0,
                        y: 0.0,
                        z: 5.0,
                    },
                    crate::hod::Vector3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    crate::hod::Vector3 {
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
            let new_muzzle = crate::hod::HODJoint {
                name: muzzle_name,
                parent_name: Some(head_name), // Parented to Heading
                local_transform: compose_transform_matrix(
                    crate::hod::Vector3 {
                        x: 0.0,
                        y: 0.0,
                        z: 5.0,
                    },
                    crate::hod::Vector3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    crate::hod::Vector3 {
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

                            let mut new_transform = crate::hod::Matrix4 { m: [[0.0; 4]; 4] };
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

// Helpers for Sub-chunk Parsing

fn find_child_trimmed<'a>(chunk: &'a IffChunk, id: &str) -> Option<&'a IffChunk> {
    if chunk.id.trim() == id {
        return Some(chunk);
    }
    for child in &chunk.children {
        if let Some(found) = find_child_trimmed(child, id) {
            return Some(found);
        }
    }
    None
}

fn find_tga_recursive(dir: &std::path::Path, filename_lower: &str) -> Option<std::path::PathBuf> {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                if let Some(found) = find_tga_recursive(&path, filename_lower) {
                    return Some(found);
                }
            } else if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.to_lowercase() == filename_lower {
                        return Some(path);
                    }
                }
            }
        }
    }
    None
}

fn rgb565_to_u16(r: u8, g: u8, b: u8) -> u16 {
    let ri = (r as u32 * 31 / 255) as u16;
    let gi = (g as u32 * 63 / 255) as u16;
    let bi = (b as u32 * 31 / 255) as u16;
    (ri << 11) | (gi << 5) | bi
}

fn u16_to_rgb565(val: u16) -> (u8, u8, u8) {
    let r = (((val >> 11) & 0x1F) as u32 * 255 / 31) as u8;
    let g = (((val >> 5) & 0x3F) as u32 * 255 / 63) as u8;
    let b = ((val & 0x1F) as u32 * 255 / 31) as u8;
    (r, g, b)
}

fn color_error(r0: u8, g0: u8, b0: u8, r1: u8, g1: u8, b1: u8) -> u32 {
    let dr = r0 as i32 - r1 as i32;
    let dg = g0 as i32 - g1 as i32;
    let db = b0 as i32 - b1 as i32;
    (dr * dr + dg * dg + db * db) as u32
}

fn find_best_endpoints(block: &[[u8; 4]; 16]) -> (u16, u16) {
    let mut min_r = 255u8;
    let mut max_r = 0u8;
    let mut min_g = 255u8;
    let mut max_g = 0u8;
    let mut min_b = 255u8;
    let mut max_b = 0u8;
    for px in block {
        if px[0] < min_r {
            min_r = px[0];
        }
        if px[0] > max_r {
            max_r = px[0];
        }
        if px[1] < min_g {
            min_g = px[1];
        }
        if px[1] > max_g {
            max_g = px[1];
        }
        if px[2] < min_b {
            min_b = px[2];
        }
        if px[2] > max_b {
            max_b = px[2];
        }
    }

    let c0 = rgb565_to_u16(min_r, min_g, min_b);
    let c1 = rgb565_to_u16(max_r, max_g, max_b);
    let (er, eg, eb) = u16_to_rgb565(c0);
    let (fr, fg, fb) = u16_to_rgb565(c1);

    let mut best_err = u32::MAX;
    let mut best_c0 = c0;
    let mut best_c1 = c1;

    let candidates = [
        (min_r, min_g, min_b),
        (max_r, max_g, max_b),
        (er, eg, eb),
        (fr, fg, fb),
    ];
    for &(a_r, a_g, a_b) in &candidates {
        for &(b_r, b_g, b_b) in &candidates {
            let c_a = rgb565_to_u16(a_r, a_g, a_b);
            let c_b = rgb565_to_u16(b_r, b_g, b_b);
            let (ar, ag, ab) = u16_to_rgb565(c_a);
            let (br, bg, bb) = u16_to_rgb565(c_b);
            let palette = [
                (ar, ag, ab),
                (br, bg, bb),
                (
                    (ar as u32 * 2 + br as u32) as u8 / 3,
                    (ag as u32 * 2 + bg as u32) as u8 / 3,
                    (ab as u32 * 2 + bb as u32) as u8 / 3,
                ),
                (
                    (ar as u32 + br as u32 * 2) as u8 / 3,
                    (ag as u32 + bg as u32 * 2) as u8 / 3,
                    (ab as u32 + bb as u32 * 2) as u8 / 3,
                ),
            ];
            let mut total = 0u32;
            for px in block {
                let mut best_d = u32::MAX;
                for &(pr, pg, pb) in &palette {
                    let d = color_error(px[0], px[1], px[2], pr, pg, pb);
                    if d < best_d {
                        best_d = d;
                    }
                    if best_d == 0 {
                        break;
                    }
                }
                total += best_d;
                if total >= best_err {
                    break;
                }
            }
            if total < best_err {
                best_err = total;
                best_c0 = c_a;
                best_c1 = c_b;
            }
        }
    }

    if best_c0 <= best_c1 {
        (best_c1, best_c0)
    } else {
        (best_c0, best_c1)
    }
}

fn compress_dxt1_block(block: &[[u8; 4]; 16]) -> [u8; 8] {
    let (c0, c1) = find_best_endpoints(block);
    let (r0, g0, b0) = u16_to_rgb565(c0);
    let (r1, g1, b1) = u16_to_rgb565(c1);
    let palette = [
        (r0, g0, b0),
        (r1, g1, b1),
        if c0 > c1 {
            (
                (r0 as u32 * 2 + r1 as u32) as u8 / 3,
                (g0 as u32 * 2 + g1 as u32) as u8 / 3,
                (b0 as u32 * 2 + b1 as u32) as u8 / 3,
            )
        } else {
            (
                (r0 as u32 + r1 as u32) as u8 / 2,
                (g0 as u32 + g1 as u32) as u8 / 2,
                (b0 as u32 + b1 as u32) as u8 / 2,
            )
        },
        if c0 > c1 {
            (
                (r0 as u32 + r1 as u32 * 2) as u8 / 3,
                (g0 as u32 + g1 as u32 * 2) as u8 / 3,
                (b0 as u32 + b1 as u32 * 2) as u8 / 3,
            )
        } else {
            (0, 0, 0)
        },
    ];
    let mut code = 0u32;
    for (i, px) in block.iter().enumerate() {
        let mut best_idx = 0u32;
        let mut best_d = u32::MAX;
        for (j, &(pr, pg, pb)) in palette.iter().enumerate() {
            let d = color_error(px[0], px[1], px[2], pr, pg, pb);
            if d < best_d {
                best_d = d;
                best_idx = j as u32;
            }
            if best_d == 0 {
                break;
            }
        }
        code |= best_idx << (2 * i as u32);
    }
    let mut out = [0u8; 8];
    out[0..2].copy_from_slice(&c0.to_le_bytes());
    out[2..4].copy_from_slice(&c1.to_le_bytes());
    out[4..8].copy_from_slice(&code.to_le_bytes());
    out
}

fn compress_dxt1(rgba: &[u8], width: usize, height: usize) -> Vec<u8> {
    let blocks_x = (width + 3) / 4;
    let blocks_y = (height + 3) / 4;
    let mut out = Vec::with_capacity(blocks_x * blocks_y * 8);

    for by in 0..blocks_y {
        for bx in 0..blocks_x {
            let mut block = [[0u8; 4]; 16];
            for y in 0..4 {
                for x in 0..4 {
                    let px = bx * 4 + x;
                    let py = by * 4 + y;
                    let idx = (py * width + px) * 4;
                    if px < width && py < height && idx + 3 < rgba.len() {
                        block[y * 4 + x] = [rgba[idx], rgba[idx + 1], rgba[idx + 2], rgba[idx + 3]];
                    }
                }
            }
            out.extend_from_slice(&compress_dxt1_block(&block));
        }
    }
    out
}

fn compress_dxt5_block(block: &[[u8; 4]; 16]) -> [u8; 16] {
    let mut min_alpha = 255u8;
    let mut max_alpha = 0u8;
    for px in block {
        min_alpha = min_alpha.min(px[3]);
        max_alpha = max_alpha.max(px[3]);
    }

    let alpha0 = max_alpha;
    let alpha1 = if max_alpha == min_alpha && max_alpha > 0 {
        max_alpha - 1
    } else {
        min_alpha
    };

    let mut alpha_palette = [0u8; 8];
    alpha_palette[0] = alpha0;
    alpha_palette[1] = alpha1;
    if alpha0 > alpha1 {
        for i in 1..7 {
            alpha_palette[i + 1] =
                (((7 - i) as u32 * alpha0 as u32 + i as u32 * alpha1 as u32) / 7) as u8;
        }
    } else {
        for i in 1..5 {
            alpha_palette[i + 1] =
                (((5 - i) as u32 * alpha0 as u32 + i as u32 * alpha1 as u32) / 5) as u8;
        }
        alpha_palette[6] = 0;
        alpha_palette[7] = 255;
    }

    let mut alpha_code = 0u64;
    for (i, px) in block.iter().enumerate() {
        let mut best_idx = 0u64;
        let mut best_d = u16::MAX;
        for (idx, alpha) in alpha_palette.iter().enumerate() {
            let d = px[3].abs_diff(*alpha) as u16;
            if d < best_d {
                best_d = d;
                best_idx = idx as u64;
            }
            if best_d == 0 {
                break;
            }
        }
        alpha_code |= best_idx << (3 * i as u64);
    }

    let color = compress_dxt1_block(block);
    let mut out = [0u8; 16];
    out[0] = alpha0;
    out[1] = alpha1;
    for i in 0..6 {
        out[2 + i] = ((alpha_code >> (8 * i)) & 0xFF) as u8;
    }
    out[8..16].copy_from_slice(&color);
    out
}

fn compress_dxt5(rgba: &[u8], width: usize, height: usize) -> Vec<u8> {
    let blocks_x = (width + 3) / 4;
    let blocks_y = (height + 3) / 4;
    let mut out = Vec::with_capacity(blocks_x * blocks_y * 16);

    for by in 0..blocks_y {
        for bx in 0..blocks_x {
            let mut block = [[0u8; 4]; 16];
            for y in 0..4 {
                for x in 0..4 {
                    let px = bx * 4 + x;
                    let py = by * 4 + y;
                    let idx = (py * width + px) * 4;
                    if px < width && py < height && idx + 3 < rgba.len() {
                        block[y * 4 + x] = [rgba[idx], rgba[idx + 1], rgba[idx + 2], rgba[idx + 3]];
                    }
                }
            }
            out.extend_from_slice(&compress_dxt5_block(&block));
        }
    }
    out
}

fn compress_dxt3_block(block: &[[u8; 4]; 16]) -> [u8; 16] {
    let mut out = [0u8; 16];

    for (i, px) in block.iter().enumerate() {
        let alpha4 = ((px[3] as u16 * 15 + 127) / 255) as u8 & 0x0F;
        if i % 2 == 0 {
            out[i / 2] |= alpha4;
        } else {
            out[i / 2] |= alpha4 << 4;
        }
    }

    let color = compress_dxt1_block(block);
    out[8..16].copy_from_slice(&color);
    out
}

fn compress_dxt3(rgba: &[u8], width: usize, height: usize) -> Vec<u8> {
    let blocks_x = (width + 3) / 4;
    let blocks_y = (height + 3) / 4;
    let mut out = Vec::with_capacity(blocks_x * blocks_y * 16);

    for by in 0..blocks_y {
        for bx in 0..blocks_x {
            let mut block = [[0u8; 4]; 16];
            for y in 0..4 {
                for x in 0..4 {
                    let px = bx * 4 + x;
                    let py = by * 4 + y;
                    let idx = (py * width + px) * 4;
                    if px < width && py < height && idx + 3 < rgba.len() {
                        block[y * 4 + x] = [rgba[idx], rgba[idx + 1], rgba[idx + 2], rgba[idx + 3]];
                    }
                }
            }
            out.extend_from_slice(&compress_dxt3_block(&block));
        }
    }
    out
}

fn generate_mip_chain(
    rgba: &[u8],
    width: usize,
    height: usize,
    max_mips: usize,
) -> Vec<(Vec<u8>, usize, usize)> {
    let mut mips = Vec::new();
    let mut cur_rgba = rgba.to_vec();
    let mut cur_w = width;
    let mut cur_h = height;

    for _ in 0..max_mips {
        mips.push((cur_rgba.clone(), cur_w, cur_h));
        if cur_w <= 1 && cur_h <= 1 {
            break;
        }
        let next_w = std::cmp::max(1, cur_w / 2);
        let next_h = std::cmp::max(1, cur_h / 2);
        
        if let Some(img) = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(cur_w as u32, cur_h as u32, cur_rgba.clone()) {
            let resized = image::imageops::resize(
                &img,
                next_w as u32,
                next_h as u32,
                image::imageops::FilterType::Lanczos3,
            );
            cur_rgba = resized.into_raw();
        }
        
        cur_w = next_w;
        cur_h = next_h;
    }
    mips
}

fn decompress_dxt1(data: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut rgba = vec![0u8; width * height * 4];
    let blocks_x = (width + 3) / 4;
    let blocks_y = (height + 3) / 4;
    let mut offset = 0;

    for by in 0..blocks_y {
        for bx in 0..blocks_x {
            if offset + 8 > data.len() {
                break;
            }
            let color0 = u16::from_le_bytes([data[offset], data[offset + 1]]);
            let color1 = u16::from_le_bytes([data[offset + 2], data[offset + 3]]);
            let code = u32::from_le_bytes([
                data[offset + 4],
                data[offset + 5],
                data[offset + 6],
                data[offset + 7],
            ]);
            offset += 8;

            let mut r = [0u8; 4];
            let mut g = [0u8; 4];
            let mut b = [0u8; 4];
            let mut a = [255u8; 4];

            // Decode RGB565
            r[0] = (((color0 >> 11) & 0x1F) as u32 * 255 / 31) as u8;
            g[0] = (((color0 >> 5) & 0x3F) as u32 * 255 / 63) as u8;
            b[0] = ((color0 & 0x1F) as u32 * 255 / 31) as u8;

            r[1] = (((color1 >> 11) & 0x1F) as u32 * 255 / 31) as u8;
            g[1] = (((color1 >> 5) & 0x3F) as u32 * 255 / 63) as u8;
            b[1] = ((color1 & 0x1F) as u32 * 255 / 31) as u8;

            if color0 > color1 {
                r[2] = ((r[0] as u32 * 2 + r[1] as u32) / 3) as u8;
                g[2] = ((g[0] as u32 * 2 + g[1] as u32) / 3) as u8;
                b[2] = ((b[0] as u32 * 2 + b[1] as u32) / 3) as u8;

                r[3] = ((r[0] as u32 + r[1] as u32 * 2) / 3) as u8;
                g[3] = ((g[0] as u32 + g[1] as u32 * 2) / 3) as u8;
                b[3] = ((b[0] as u32 + b[1] as u32 * 2) / 3) as u8;
            } else {
                r[2] = ((r[0] as u32 + r[1] as u32) / 2) as u8;
                g[2] = ((g[0] as u32 + g[1] as u32) / 2) as u8;
                b[2] = ((b[0] as u32 + b[1] as u32) / 2) as u8;

                r[3] = 0;
                g[3] = 0;
                b[3] = 0;
                a[3] = 0;
            }

            for y in 0..4 {
                for x in 0..4 {
                    let px = bx * 4 + x;
                    let py = by * 4 + y;
                    if px < width && py < height {
                        let i = y * 4 + x;
                        let select = ((code >> (2 * i)) & 0x03) as usize;
                        let pixel_offset = (py * width + px) * 4;
                        rgba[pixel_offset] = r[select];
                        rgba[pixel_offset + 1] = g[select];
                        rgba[pixel_offset + 2] = b[select];
                        rgba[pixel_offset + 3] = a[select];
                    }
                }
            }
        }
    }
    rgba
}

fn decompress_dxt5(data: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut rgba = vec![0u8; width * height * 4];
    let blocks_x = (width + 3) / 4;
    let blocks_y = (height + 3) / 4;
    let mut offset = 0;

    for by in 0..blocks_y {
        for bx in 0..blocks_x {
            if offset + 16 > data.len() {
                break;
            }

            // 1. Decode Alpha Block (8 bytes)
            let alpha0 = data[offset];
            let alpha1 = data[offset + 1];

            // Extract 48-bit value for 16 3-bit indices
            let mut alpha_mask = 0u64;
            for i in 0..6 {
                alpha_mask |= (data[offset + 2 + i] as u64) << (i * 8);
            }
            offset += 8;

            let mut alphas = [0u8; 8];
            alphas[0] = alpha0;
            alphas[1] = alpha1;
            if alpha0 > alpha1 {
                for i in 1..7 {
                    alphas[i + 1] =
                        (((7 - i) as u32 * alpha0 as u32 + i as u32 * alpha1 as u32) / 7) as u8;
                }
            } else {
                for i in 1..5 {
                    alphas[i + 1] =
                        (((5 - i) as u32 * alpha0 as u32 + i as u32 * alpha1 as u32) / 5) as u8;
                }
                alphas[6] = 0;
                alphas[7] = 255;
            }

            // 2. Decode Color Block (8 bytes, like DXT1 but color 2/3 are always interpolated)
            let color0 = u16::from_le_bytes([data[offset], data[offset + 1]]);
            let color1 = u16::from_le_bytes([data[offset + 2], data[offset + 3]]);
            let code = u32::from_le_bytes([
                data[offset + 4],
                data[offset + 5],
                data[offset + 6],
                data[offset + 7],
            ]);
            offset += 8;

            let mut r = [0u8; 4];
            let mut g = [0u8; 4];
            let mut b = [0u8; 4];

            r[0] = (((color0 >> 11) & 0x1F) as u32 * 255 / 31) as u8;
            g[0] = (((color0 >> 5) & 0x3F) as u32 * 255 / 63) as u8;
            b[0] = ((color0 & 0x1F) as u32 * 255 / 31) as u8;

            r[1] = (((color1 >> 11) & 0x1F) as u32 * 255 / 31) as u8;
            g[1] = (((color1 >> 5) & 0x3F) as u32 * 255 / 63) as u8;
            b[1] = ((color1 & 0x1F) as u32 * 255 / 31) as u8;

            r[2] = ((r[0] as u32 * 2 + r[1] as u32) / 3) as u8;
            g[2] = ((g[0] as u32 * 2 + g[1] as u32) / 3) as u8;
            b[2] = ((b[0] as u32 * 2 + b[1] as u32) / 3) as u8;

            r[3] = ((r[0] as u32 + r[1] as u32 * 2) / 3) as u8;
            g[3] = ((g[0] as u32 + g[1] as u32 * 2) / 3) as u8;
            b[3] = ((b[0] as u32 + b[1] as u32 * 2) / 3) as u8;

            // 3. Write pixels
            for y in 0..4 {
                for x in 0..4 {
                    let px = bx * 4 + x;
                    let py = by * 4 + y;
                    if px < width && py < height {
                        let i = y * 4 + x;
                        let select = ((code >> (2 * i)) & 0x03) as usize;
                        let alpha_select = ((alpha_mask >> (3 * i)) & 0x07) as usize;

                        let pixel_offset = (py * width + px) * 4;
                        rgba[pixel_offset] = r[select];
                        rgba[pixel_offset + 1] = g[select];
                        rgba[pixel_offset + 2] = b[select];
                        rgba[pixel_offset + 3] = alphas[alpha_select];
                    }
                }
            }
        }
    }
    rgba
}

fn decompress_dxt3(data: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut rgba = vec![0u8; width * height * 4];
    let blocks_x = (width + 3) / 4;
    let blocks_y = (height + 3) / 4;
    let mut offset = 0;

    for by in 0..blocks_y {
        for bx in 0..blocks_x {
            if offset + 16 > data.len() {
                break;
            }

            let alpha_offset = offset;
            let color_offset = offset + 8;
            offset += 16;

            let color0 = u16::from_le_bytes([data[color_offset], data[color_offset + 1]]);
            let color1 = u16::from_le_bytes([data[color_offset + 2], data[color_offset + 3]]);
            let code = u32::from_le_bytes([
                data[color_offset + 4],
                data[color_offset + 5],
                data[color_offset + 6],
                data[color_offset + 7],
            ]);

            let mut r = [0u8; 4];
            let mut g = [0u8; 4];
            let mut b = [0u8; 4];

            r[0] = (((color0 >> 11) & 0x1F) as u32 * 255 / 31) as u8;
            g[0] = (((color0 >> 5) & 0x3F) as u32 * 255 / 63) as u8;
            b[0] = ((color0 & 0x1F) as u32 * 255 / 31) as u8;

            r[1] = (((color1 >> 11) & 0x1F) as u32 * 255 / 31) as u8;
            g[1] = (((color1 >> 5) & 0x3F) as u32 * 255 / 63) as u8;
            b[1] = ((color1 & 0x1F) as u32 * 255 / 31) as u8;

            r[2] = ((r[0] as u32 * 2 + r[1] as u32) / 3) as u8;
            g[2] = ((g[0] as u32 * 2 + g[1] as u32) / 3) as u8;
            b[2] = ((b[0] as u32 * 2 + b[1] as u32) / 3) as u8;

            r[3] = ((r[0] as u32 + r[1] as u32 * 2) / 3) as u8;
            g[3] = ((g[0] as u32 + g[1] as u32 * 2) / 3) as u8;
            b[3] = ((b[0] as u32 + b[1] as u32 * 2) / 3) as u8;

            for y in 0..4 {
                for x in 0..4 {
                    let px = bx * 4 + x;
                    let py = by * 4 + y;
                    if px < width && py < height {
                        let i = y * 4 + x;
                        let alpha_byte = data[alpha_offset + i / 2];
                        let alpha4 = if i % 2 == 0 {
                            alpha_byte & 0x0F
                        } else {
                            alpha_byte >> 4
                        };
                        let select = ((code >> (2 * i)) & 0x03) as usize;

                        let pixel_offset = (py * width + px) * 4;
                        rgba[pixel_offset] = r[select];
                        rgba[pixel_offset + 1] = g[select];
                        rgba[pixel_offset + 2] = b[select];
                        rgba[pixel_offset + 3] = alpha4 * 17;
                    }
                }
            }
        }
    }
    rgba
}

fn compressed_texture_mip_size(format: &str, width: usize, height: usize) -> usize {
    let blocks_x = (width + 3) / 4;
    let blocks_y = (height + 3) / 4;
    match format {
        "DXT1" => blocks_x * blocks_y * 8,
        "DXT3" | "DXT5" => blocks_x * blocks_y * 16,
        _ => width * height * 4,
    }
}

fn encode_b64_png_thumbnail(rgba: &[u8], width: u32, height: u32, max_dim: u32) -> Option<String> {
    if rgba.is_empty() || width == 0 || height == 0 {
        return None;
    }
    let (target_w, target_h) = if width > max_dim || height > max_dim {
        let aspect = width as f32 / height as f32;
        if width > height {
            (max_dim, ((max_dim as f32 / aspect) as u32).max(1))
        } else {
            (((max_dim as f32 * aspect) as u32).max(1), max_dim)
        }
    } else {
        (width, height)
    };

    let rgba_to_encode = if target_w != width || target_h != height {
        if let Some(img) =
            image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(width, height, rgba.to_vec())
        {
            let resized = image::imageops::resize(
                &img,
                target_w,
                target_h,
                image::imageops::FilterType::Lanczos3,
            );
            resized.into_raw()
        } else {
            rgba.to_vec()
        }
    } else {
        rgba.to_vec()
    };

    let mut png_bytes = Vec::new();
    if let Ok(_) = image::codecs::png::PngEncoder::new(&mut png_bytes).encode(
        &rgba_to_encode,
        target_w,
        target_h,
        image::ColorType::Rgba8,
    ) {
        Some(format!(
            "data:image/png;base64,{}",
            base64_encode(&png_bytes)
        ))
    } else {
        None
    }
}

fn parse_texture(chunk: &IffChunk, context: &mut ParsingContext) -> Result<HODTexture, String> {
    let mut reader = Cursor::new(&chunk.data);

    let name_base = read_len_string(&mut reader)?;
    let mut format_bytes = [0u8; 4];
    reader
        .read_exact(&mut format_bytes)
        .map_err(|e| e.to_string())?;
    let format_str = String::from_utf8_lossy(&format_bytes).trim().to_string();

    let mip_count = reader
        .read_u32::<LittleEndian>()
        .map_err(|e| e.to_string())? as usize;

    let mut width = 0u32;
    let mut height = 0u32;
    if mip_count > 0 {
        width = reader
            .read_u32::<LittleEndian>()
            .map_err(|e| e.to_string())?;
        height = reader
            .read_u32::<LittleEndian>()
            .map_err(|e| e.to_string())?;
        if context.is_v2 {
            // HOD 2.0 LMIP stores dimensions for each mip. Legacy HOD 1.0 inline
            // LMIP/TEXM stores only the base dimensions, then compressed mip bytes.
            for _ in 1..mip_count {
                let _ = reader
                    .read_u32::<LittleEndian>()
                    .map_err(|e| e.to_string())?;
                let _ = reader
                    .read_u32::<LittleEndian>()
                    .map_err(|e| e.to_string())?;
            }
        }
    }

    let name = if context.is_v2 {
        if name_base.ends_with(&format_str) {
            name_base.clone()
        } else {
            format!("{}{}", name_base, format_str)
        }
    } else {
        // HOD 1.0 names are often full paths (e.g. "G:\GOG.com\centaur\support01.dds"). Extract just the stem.
        // std::path::Path::new fails on Linux for Windows paths, so split manually.
        let file_name = name_base
            .split('\\')
            .last()
            .unwrap_or(&name_base)
            .split('/')
            .last()
            .unwrap_or(&name_base);
        let stem = file_name
            .rsplit_once('.')
            .map(|(s, _)| s)
            .unwrap_or(file_name);
        if stem.ends_with(&format_str) {
            stem.to_string()
        } else {
            format!("{}{}", stem, format_str)
        }
    };

    let format = format_str;
    let _mip_levels = mip_count as u32;

    let mut raw_pixels = Vec::new();
    if context.is_v2 {
        let mut expected_size = 0;
        let mut cur_w = width as usize;
        let mut cur_h = height as usize;
        let pool_pos_before = context.texture_pool.position();
        println!(
            "[TEX-DEBUG] Parsing texture '{}' ({}x{}, {} mips, format={}) at pool position {}",
            name, width, height, mip_count, format, pool_pos_before
        );

        for mip_idx in 0..mip_count {
            let mip_size = compressed_texture_mip_size(&format, cur_w, cur_h);
            expected_size += mip_size;
            println!(
                "[TEX-DEBUG]   Mip {}: {}x{} = {} bytes (cumulative: {})",
                mip_idx, cur_w, cur_h, mip_size, expected_size
            );
            if cur_w == 1 && cur_h == 1 {
                break;
            }
            cur_w = std::cmp::max(1, cur_w / 2);
            cur_h = std::cmp::max(1, cur_h / 2);
        }

        let mut buf = vec![0u8; expected_size.min(1024 * 1024 * 8)]; // clamp to safe limits
        if let Ok(_) = context.texture_pool.read_exact(&mut buf) {
            let pool_pos_after = context.texture_pool.position();
            println!(
                "[TEX-DEBUG] Read {} bytes, pool position now: {}",
                expected_size, pool_pos_after
            );
            raw_pixels = buf;
        } else {
            println!(
                "[TEX-DEBUG] FAILED to read {} bytes from texture pool!",
                expected_size
            );
        }
    } else {
        // Read raw inline mips from child chunks if present, else read inline
        if let Some(mips_chunk) = chunk.find_child("MIPS") {
            raw_pixels = mips_chunk.data.clone();
        } else {
            let current_pos = reader.position() as usize;
            if current_pos < chunk.data.len() {
                raw_pixels = chunk.data[current_pos..].to_vec();
            }
        }
    }

    // Try to convert raw RGBA/TGA/DDS mips to a Base64 encoded PNG preview and data
    let mut decoded_rgba = None;
    if !raw_pixels.is_empty() {
        decoded_rgba = if format == "RGBA" {
            Some(raw_pixels.clone())
        } else if format == "DXT1" {
            Some(decompress_dxt1(
                &raw_pixels,
                width as usize,
                height as usize,
            ))
        } else if format == "DXT3" {
            Some(decompress_dxt3(
                &raw_pixels,
                width as usize,
                height as usize,
            ))
        } else if format == "DXT5" {
            Some(decompress_dxt5(
                &raw_pixels,
                width as usize,
                height as usize,
            ))
        } else {
            None
        };
    }

    let mut png_preview = None;
    let mut png_data = None;

    if let Some(mut rgba) = decoded_rgba {
        // All HOD textures (1.0 and 2.0) are stored flipped in the DXT stream (DirectX convention).
        // Un-flip them here so the editor UI preview shows them correctly.
        flip_rgba_vertical_in_place(&mut rgba, width, height);
        png_preview = encode_b64_png_thumbnail(&rgba, width, height, 128);
        png_data = encode_b64_png_thumbnail(&rgba, width, height, 8192);
    }

    // If png_preview is None, we attempt to load from disk (.TGA files) inside the uncompressed folder or HOD directory!
    if png_preview.is_none() {
        let filename_lower = format!("{}.tga", name.to_lowercase());
        let mut tga_path: Option<std::path::PathBuf> = None;

        // 1. Look next to the HOD file
        if let Some(ref h_dir) = context.hod_dir {
            // Check direct match
            let direct = h_dir.join(&filename_lower);
            if direct.is_file() {
                tga_path = Some(direct);
            } else if let Some(found) = find_tga_recursive(h_dir, &filename_lower) {
                tga_path = Some(found);
            }
        }

        // 2. Look recursively inside uncompressed_dir if still not found
        if tga_path.is_none() {
            if let Some(ref u_dir) = context.uncompressed_dir {
                if let Some(found) = find_tga_recursive(u_dir, &filename_lower) {
                    tga_path = Some(found);
                }
            }
        }

        // 3. Fallback: Search for project ship converted directory recursively upwards (e.g. WorkshopTool/current_project_processing/ship_converted/[file_stem])
        if tga_path.is_none() {
            if let Some(ref h_file) = context.hod_file_path {
                if let Some(file_stem) = h_file.file_stem().and_then(|s| s.to_str()) {
                    let mut current = h_file.parent();
                    while let Some(parent) = current {
                        // Check sibling folder "Homeworld 347380" inside common
                        let sibling_tool = parent
                            .join("Homeworld 347380")
                            .join("GBXTools")
                            .join("WorkshopTool")
                            .join("current_project_processing")
                            .join("ship_converted")
                            .join(file_stem);
                        if sibling_tool.is_dir() {
                            if let Some(found) = find_tga_recursive(&sibling_tool, &filename_lower)
                            {
                                tga_path = Some(found);
                                break;
                            }
                        }

                        let direct_tool = parent
                            .join("current_project_processing")
                            .join("ship_converted")
                            .join(file_stem);
                        if direct_tool.is_dir() {
                            if let Some(found) = find_tga_recursive(&direct_tool, &filename_lower) {
                                tga_path = Some(found);
                                break;
                            }
                        }
                        current = parent.parent();
                    }
                }
            }
        }

        // 3. Load and convert the TGA file to PNG!
        if let Some(path) = tga_path {
            println!(
                "[RUST] Found TGA texture on disk: {}",
                path.to_string_lossy()
            );
            if let Ok(tga_bytes) = std::fs::read(&path) {
                if let Ok(img) =
                    image::load_from_memory_with_format(&tga_bytes, image::ImageFormat::Tga)
                {
                    let mut png_bytes = Vec::new();
                    let mut cursor = std::io::Cursor::new(&mut png_bytes);
                    if let Ok(_) = img.write_to(&mut cursor, image::ImageFormat::Png) {
                        let full_b64 =
                            format!("data:image/png;base64,{}", base64_encode(&png_bytes));
                        png_preview = Some(full_b64.clone());
                        png_data = Some(full_b64);
                        println!(
                            "[RUST] Successfully converted TGA to PNG preview! Size: {} bytes",
                            png_bytes.len()
                        );
                    }
                }
            }
        }
    }

    Ok(HODTexture {
        name,
        width,
        height,
        format,
        png_preview,
        png_data,
        source_path: None,
    })
}

fn parse_stat_material(chunk: &IffChunk, textures: &[HODTexture]) -> Result<HODMaterial, String> {
    let mut r = Cursor::new(&chunk.data);

    let material_name =
        read_len_string(&mut r).map_err(|e| format!("Failed to read material name: {}", e))?;
    let shader_name =
        read_len_string(&mut r).map_err(|e| format!("Failed to read shader name: {}", e))?;

    let param_count = r.read_u32::<LittleEndian>().map_err(|e| e.to_string())? as usize;

    let mut texture_maps = Vec::new();

    if param_count > 0 {
        let _extra1 = r.read_u32::<LittleEndian>().map_err(|e| e.to_string())?;
        let _extra2 = r.read_u32::<LittleEndian>().map_err(|e| e.to_string())?;

        for i in 0..param_count {
            let texture_index = if i == 0 {
                r.read_u32::<LittleEndian>().map_err(|e| e.to_string())? as usize
            } else {
                let _extra3 = r.read_u32::<LittleEndian>().map_err(|e| e.to_string())?;
                let _extra4 = r.read_u32::<LittleEndian>().map_err(|e| e.to_string())?;
                r.read_u32::<LittleEndian>().map_err(|e| e.to_string())? as usize
            };

            let _param_name =
                read_len_string(&mut r).map_err(|e| format!("Failed to read param name: {}", e))?;

            if texture_index < textures.len() {
                texture_maps.push(textures[texture_index].name.clone());
            }
        }
    }

    let pos = r.position() as usize;
    let remaining = &chunk.data[pos..];
    let parameters = remaining.to_vec();

    Ok(HODMaterial {
        name: material_name,
        shader_name,
        texture_maps,
        parameters,
    })
}

fn read_vertex<R: Read>(
    vertex_reader: &mut R,
    vertex_mask: u32,
    version: u32,
    stride: u32,
    is_v2: bool,
) -> Result<HODVertex, String> {
    let mut bytes_read = 0;
    let mut pos_x = 0.0;
    let mut pos_y = 0.0;
    let mut pos_z = 0.0;
    if (vertex_mask & 0x1) != 0 {
        pos_x = vertex_reader
            .read_f32::<LittleEndian>()
            .map_err(|e| e.to_string())?;
        pos_y = vertex_reader
            .read_f32::<LittleEndian>()
            .map_err(|e| e.to_string())?;
        pos_z = vertex_reader
            .read_f32::<LittleEndian>()
            .map_err(|e| e.to_string())?;
        let _pos_w = vertex_reader
            .read_f32::<LittleEndian>()
            .map_err(|e| e.to_string())?;
        bytes_read += 16;

        if !pos_x.is_finite() {
            pos_x = 0.0;
        }
        if !pos_y.is_finite() {
            pos_y = 0.0;
        }
        if !pos_z.is_finite() {
            pos_z = 0.0;
        }
    }

    let mut normal = None;
    if (vertex_mask & 0x2) != 0 {
        let nx = vertex_reader
            .read_f32::<LittleEndian>()
            .map_err(|e| e.to_string())?;
        let ny = vertex_reader
            .read_f32::<LittleEndian>()
            .map_err(|e| e.to_string())?;
        let nz = vertex_reader
            .read_f32::<LittleEndian>()
            .map_err(|e| e.to_string())?;
        let _normal_w = vertex_reader
            .read_f32::<LittleEndian>()
            .map_err(|e| e.to_string())?;
        bytes_read += 16;

        normal = Some(Vector3 {
            x: nx,
            y: ny,
            z: nz,
        });
    }

    let mut color = None;
    if (vertex_mask & 0x4) != 0 {
        color = Some(
            vertex_reader
                .read_u32::<LittleEndian>()
                .map_err(|e| e.to_string())?,
        );
        bytes_read += 4;
    }

    let mut uv = None;
    if (vertex_mask & 0x8) != 0 {
        let u = vertex_reader
            .read_f32::<LittleEndian>()
            .map_err(|e| e.to_string())?;
        let v = vertex_reader
            .read_f32::<LittleEndian>()
            .map_err(|e| e.to_string())?;
        bytes_read += 8;
        uv = Some(Vector2 { u, v });
    }

    if version == 1401 {
        if (vertex_mask & 0x10) != 0 {
            let _u1 = vertex_reader
                .read_f32::<LittleEndian>()
                .map_err(|e| e.to_string())?;
            let _v1 = vertex_reader
                .read_f32::<LittleEndian>()
                .map_err(|e| e.to_string())?;
            bytes_read += 8;
        }
        if (vertex_mask & 0x20) != 0 {
            let _u2 = vertex_reader
                .read_f32::<LittleEndian>()
                .map_err(|e| e.to_string())?;
            let _v2 = vertex_reader
                .read_f32::<LittleEndian>()
                .map_err(|e| e.to_string())?;
            bytes_read += 8;
        }
    }

    let mut tangent = None;
    if (vertex_mask & 0x2000) != 0 {
        let tx = vertex_reader
            .read_f32::<LittleEndian>()
            .map_err(|e| e.to_string())?;
        let ty = vertex_reader
            .read_f32::<LittleEndian>()
            .map_err(|e| e.to_string())?;
        let tz = vertex_reader
            .read_f32::<LittleEndian>()
            .map_err(|e| e.to_string())?;
        bytes_read += 12;
        tangent = Some(Vector3 {
            x: tx,
            y: ty,
            z: tz,
        });
    }

    let mut binormal = None;
    if (vertex_mask & 0x4000) != 0 {
        let bx = vertex_reader
            .read_f32::<LittleEndian>()
            .map_err(|e| e.to_string())?;
        let by = vertex_reader
            .read_f32::<LittleEndian>()
            .map_err(|e| e.to_string())?;
        let bz = vertex_reader
            .read_f32::<LittleEndian>()
            .map_err(|e| e.to_string())?;
        bytes_read += 12;
        binormal = Some(Vector3 {
            x: bx,
            y: by,
            z: bz,
        });
    }

    let mut skinning_data = None;
    if stride > bytes_read {
        let padding_needed = stride - bytes_read;
        let mut padding = vec![0u8; padding_needed as usize];
        vertex_reader.read_exact(&mut padding).map_err(|e| format!("failed to read padding in read_vertex: {} (padding_needed={}, bytes_read={}, stride={})", e, padding_needed, bytes_read, stride))?;
        skinning_data = Some(padding);
    }

    Ok(HODVertex {
        position: Vector3 {
            x: pos_x,
            y: pos_y,
            z: pos_z,
        },
        normal,
        color,
        uv,
        tangent,
        binormal,
        skinning_data,
    })
}

fn safe_read_u16(face_pool: &mut Cursor<Vec<u8>>) -> Result<u16, String> {
    let pos = face_pool.position() as usize;
    let data = face_pool.get_ref();
    if pos >= data.len() {
        return Ok(0);
    }
    if pos + 1 >= data.len() {
        let val = data[pos] as u16;
        face_pool.set_position((pos + 1) as u64);
        return Ok(val);
    }
    let val = u16::from_le_bytes([data[pos], data[pos + 1]]);
    face_pool.set_position((pos + 2) as u64);
    Ok(val)
}

fn append_hod1_primitive_indices(indices: &mut Vec<u16>, prim_type: u32, raw: &[u16]) {
    match prim_type {
        514 => indices.extend_from_slice(raw),
        515 => {
            for i in 0..raw.len().saturating_sub(2) {
                if i % 2 == 0 {
                    indices.extend_from_slice(&[raw[i], raw[i + 1], raw[i + 2]]);
                } else {
                    indices.extend_from_slice(&[raw[i + 1], raw[i], raw[i + 2]]);
                }
            }
        }
        516 => {
            for i in 1..raw.len().saturating_sub(1) {
                indices.extend_from_slice(&[raw[0], raw[i], raw[i + 1]]);
            }
        }
        517 => {
            for quad in raw.chunks_exact(4) {
                indices.extend_from_slice(&[quad[0], quad[1], quad[2], quad[2], quad[3], quad[0]]);
            }
        }
        _ => indices.extend_from_slice(raw),
    }
}

fn parse_basic_mesh(chunk: &IffChunk, context: &mut ParsingContext) -> Result<HODMesh, String> {
    println!(
        "[RUST] BMSH raw data (len={}): {:02x?}",
        chunk.data.len(),
        &chunk.data[..std::cmp::min(128, chunk.data.len())]
    );
    let mut reader = Cursor::new(&chunk.data);
    let lod = reader
        .read_i32::<LittleEndian>()
        .map_err(|e| e.to_string())?;
    let part_count = reader
        .read_i32::<LittleEndian>()
        .map_err(|e| e.to_string())? as usize;

    println!(
        "  parse_basic_mesh ID={} lod={} parts={}",
        chunk.id, lod, part_count
    );

    let mut parts = Vec::new();
    let mut cumulative_vertex_offset: usize = 0;

    for p_idx in 0..part_count {
        let material_index = reader
            .read_i32::<LittleEndian>()
            .map_err(|e| e.to_string())? as usize;
        let vertex_mask = reader
            .read_u32::<LittleEndian>()
            .map_err(|e| e.to_string())?;
        let vertex_count = reader
            .read_i32::<LittleEndian>()
            .map_err(|e| e.to_string())? as usize;

        // Compute vertex stride from vertex_mask so we can convert global mesh-pool
        // byte offsets to local 0-based vertex indices.
        //   bit 0  (pos):      4 floats = 16 bytes
        //   bit 1  (normal):   4 floats = 16 bytes
        //   bit 2  (color):    1 u32    =  4 bytes
        //   bit 3  (uv0):      2 floats =  8 bytes
        //   bit 4  (uv1):      2 floats =  8 bytes  (v1401 only)
        //   bit 5  (uv2):      2 floats =  8 bytes  (v1401 only)
        //   bit 13 (tangent):  3 floats = 12 bytes
        //   bit 14 (binormal): 3 floats = 12 bytes
        let is_v2 = context.is_v2;
        let pos_size = 16;
        let normal_size = 16;
        let mut vertex_stride: u32 = 0;
        if (vertex_mask & 0x01) != 0 {
            vertex_stride += pos_size;
        }
        if (vertex_mask & 0x02) != 0 {
            vertex_stride += normal_size;
        }
        if (vertex_mask & 0x04) != 0 {
            vertex_stride += 4;
        }
        if (vertex_mask & 0x08) != 0 {
            vertex_stride += 8;
        }
        if chunk.version == 1401 {
            if (vertex_mask & 0x10) != 0 {
                vertex_stride += 8;
            }
            if (vertex_mask & 0x20) != 0 {
                vertex_stride += 8;
            }
        }
        if (vertex_mask & 0x2000) != 0 {
            vertex_stride += 12;
        }
        if (vertex_mask & 0x4000) != 0 {
            vertex_stride += 12;
        }
        if vertex_stride == 0 {
            vertex_stride = 1;
        } // guard against div-by-zero

        let mut vertices = Vec::new();
        let mut indices: Vec<u16> = Vec::new();

        if context.is_v2 {
            let _prim_group_count = reader
                .read_i16::<LittleEndian>()
                .map_err(|e| e.to_string())? as i32;
            let indice_count = reader
                .read_i32::<LittleEndian>()
                .map_err(|e| e.to_string())? as usize;
            println!("    Part {} - verts={} indices={} mesh_pos={} face_pos={} mask=0x{:X} stride={} cumulative_vert={}", p_idx, vertex_count, indice_count, context.mesh_pool.position(), context.face_pool.position(), vertex_mask, vertex_stride, cumulative_vertex_offset);

            for v_idx in 0..vertex_count {
                let v = read_vertex(
                    &mut context.mesh_pool,
                    vertex_mask,
                    chunk.version,
                    vertex_stride,
                    is_v2,
                )
                .map_err(|e| {
                    format!(
                        "failed at vertex {}/{}: {} (mesh_pool pos={}, total_len={})",
                        v_idx,
                        vertex_count,
                        e,
                        context.mesh_pool.position(),
                        context.mesh_pool.get_ref().len()
                    )
                })?;
                vertices.push(v);
            }

            let mut raw_indices = Vec::new();
            for _ in 0..indice_count {
                // HOD 2.0 face pool stores plain little-endian u16 indices, local to each part.
                let raw = safe_read_u16(&mut context.face_pool)?;
                raw_indices.push(raw);
                let safe_idx = if vertex_count > 0 {
                    std::cmp::min(raw, (vertex_count - 1) as u16)
                } else {
                    0
                };
                indices.push(safe_idx);
            }
            println!(
                "      First 20 parsed indices of Part {}: {:?}",
                p_idx,
                &raw_indices[..std::cmp::min(20, raw_indices.len())]
            );
        } else {
            for _ in 0..vertex_count {
                let v = read_vertex(
                    &mut reader,
                    vertex_mask,
                    chunk.version,
                    vertex_stride,
                    is_v2,
                )?;
                vertices.push(v);
            }

            let prim_group_count = reader
                .read_i16::<LittleEndian>()
                .map_err(|e| e.to_string())? as i32;
            if prim_group_count < 0 {
                return Err(format!(
                    "Negative primitive group count in HOD 1.0 BMSH part {}",
                    p_idx
                ));
            }
            let remaining_bytes = reader.get_ref().len() as u64 - reader.position();
            if prim_group_count as u64 > remaining_bytes / 10 {
                return Err(format!(
                    "Primitive group count exceeds buffer in HOD 1.0 BMSH part {}: groups={}, remaining={}",
                    p_idx, prim_group_count, remaining_bytes
                ));
            }

            for group_idx in 0..prim_group_count {
                let prim_type = reader.read_u32::<LittleEndian>().unwrap_or(514);
                let indice_count = reader
                    .read_i32::<LittleEndian>()
                    .map_err(|e| e.to_string())? as usize;
                let remaining_bytes = reader.get_ref().len() as u64 - reader.position();
                if indice_count as u64 > remaining_bytes / 2 {
                    return Err(format!(
                        "Index count exceeds buffer in HOD 1.0 BMSH part {} group {}: indices={}, remaining={}",
                        p_idx, group_idx, indice_count, remaining_bytes
                    ));
                }

                let mut raw_indices = Vec::with_capacity(indice_count);
                for _ in 0..indice_count {
                    // v1 HOD: direct per-part vertex indices already
                    raw_indices.push(
                        reader
                            .read_u16::<LittleEndian>()
                            .map_err(|e| e.to_string())?,
                    );
                }
                append_hod1_primitive_indices(&mut indices, prim_type, &raw_indices);
            }
        }

        cumulative_vertex_offset += vertex_count;

        parts.push(HODMeshPart {
            material_index,
            vertex_mask,
            vertices,
            indices,
        });
    }

    Ok(HODMesh {
        name: chunk.id.clone(),
        parent_name: String::new(),
        lod,
        has_mult_tags: false,
        parts,
    })
}

pub fn read_len_string<R: Read>(reader: &mut R) -> Result<String, String> {
    let len = reader
        .read_u32::<LittleEndian>()
        .map_err(|e| e.to_string())? as usize;
    if len == 0 {
        return Ok(String::new());
    }
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&buf).to_string())
}

fn parse_dock_chunk(data: &[u8], is_v2: bool) -> Result<Vec<HODDockpath>, String> {
    if is_v2 {
        parse_dock_extended(data).or_else(|extended_err| {
            parse_dock_legacy(data).map_err(|legacy_err| {
                format!(
                    "extended layout failed: {}; legacy layout failed: {}",
                    extended_err, legacy_err
                )
            })
        })
    } else {
        parse_dock_legacy(data).or_else(|legacy_err| {
            parse_dock_extended(data).map_err(|extended_err| {
                format!(
                    "legacy layout failed: {}; extended layout failed: {}",
                    legacy_err, extended_err
                )
            })
        })
    }
}

fn parse_dock_extended(data: &[u8]) -> Result<Vec<HODDockpath>, String> {
    let mut r = Cursor::new(data);
    let count = r.read_u32::<LittleEndian>().map_err(|e| e.to_string())?;

    let remaining_bytes = r.get_ref().len() as u64 - r.position();
    let max_possible_paths = remaining_bytes / 12;
    if count as u64 > max_possible_paths {
        return Err("DOCK count exceeds buffer space".to_string());
    }

    let mut dockpaths = Vec::new();
    for _ in 0..count {
        let name = read_len_string(&mut r)?;
        let parent_name = read_len_string(&mut r)?;
        let val1 = r.read_u32::<LittleEndian>().map_err(|e| e.to_string())?;
        let val2 = r.read_u32::<LittleEndian>().map_err(|e| e.to_string())?;
        let val3 = r.read_u32::<LittleEndian>().map_err(|e| e.to_string())?;
        let val4 = r.read_u32::<LittleEndian>().map_err(|e| e.to_string())?;
        let val5 = r.read_u32::<LittleEndian>().map_err(|e| e.to_string())?;
        let compatible_ships = read_len_string(&mut r)?;
        let padding1 = r.read_u32::<LittleEndian>().map_err(|e| e.to_string())?;
        let padding2 = read_len_string(&mut r)?;
        let num_points = r.read_i32::<LittleEndian>().map_err(|e| e.to_string())?;
        if num_points < 0 {
            return Err("Negative num_points in DOCK".to_string());
        }
        let num_points = num_points as usize;

        let remaining_bytes = r.get_ref().len() as u64 - r.position();
        println!("[DOCK DEBUG] path='{}', num_points={}, remaining={}", name, num_points, remaining_bytes);
        
        let max_possible_points = remaining_bytes / 50;
        if num_points as u64 > max_possible_points {
            return Err("DOCK num_points exceeds buffer space".to_string());
        }

        let mut points = Vec::with_capacity(num_points);
        for i in 0..num_points {
            let pt = read_dockpoint_matrix3_extra(&mut r).map_err(|e| format!("path '{}' point {} err: {}", name, i, e))?;
            points.push(pt);
        }

        dockpaths.push(HODDockpath {
            name,
            parent_name,
            points,
            val1,
            val2,
            val3,
            val4,
            val5,
            compatible_ships,
            padding1,
            padding2,
        });
    }

    Ok(dockpaths)
}

fn parse_dock_legacy(data: &[u8]) -> Result<Vec<HODDockpath>, String> {
    parse_dock_legacy_with_layout(data, true).or_else(|matrix4_err| {
        parse_dock_legacy_with_layout(data, false).map_err(|matrix3_err| {
            format!(
                "matrix4 layout failed: {}; matrix3 layout failed: {}",
                matrix4_err, matrix3_err
            )
        })
    })
}

fn parse_dock_legacy_with_layout(
    data: &[u8],
    use_matrix4: bool,
) -> Result<Vec<HODDockpath>, String> {
    let mut r = Cursor::new(data);
    let count = r.read_u32::<LittleEndian>().map_err(|e| e.to_string())?;
    let remaining_bytes = r.get_ref().len() as u64 - r.position();
    let max_possible_paths = remaining_bytes / 12;
    if count as u64 > max_possible_paths {
        return Err("DOCK count exceeds buffer space".to_string());
    }

    let mut dockpaths = Vec::new();
    for _ in 0..count {
        let name = read_len_string(&mut r)?;
        let parent_name = read_len_string(&mut r)?;
        let num_points = r.read_i32::<LittleEndian>().map_err(|e| e.to_string())?;
        if num_points < 0 {
            return Err("Negative num_points in DOCK".to_string());
        }
        let num_points = num_points as usize;

        let point_record_size = if use_matrix4 { 84 } else { 64 };
        let remaining_bytes = r.get_ref().len() as u64 - r.position();
        let max_possible_points = remaining_bytes / point_record_size;
        if num_points as u64 > max_possible_points {
            return Err("DOCK num_points exceeds buffer space".to_string());
        }

        let mut points = Vec::with_capacity(num_points);
        for _ in 0..num_points {
            let point = if use_matrix4 {
                read_dockpoint_matrix4(&mut r)?
            } else {
                read_dockpoint_matrix3_extra(&mut r)?
            };
            points.push(point);
        }

        dockpaths.push(HODDockpath {
            name,
            parent_name,
            points,
            val1: 0,
            val2: 0,
            val3: 0,
            val4: 0,
            val5: 0,
            compatible_ships: String::new(),
            padding1: 0,
            padding2: String::new(),
        });
    }

    let trailing = r.get_ref().len() as u64 - r.position();
    if trailing > 8 {
        return Err(format!("DOCK trailing bytes after parse: {}", trailing));
    }

    Ok(dockpaths)
}

fn read_dockpoint_matrix3_extra<R: Read>(r: &mut R) -> Result<HODDockpoint, String> {
    let position = Vector3 {
        x: r.read_f32::<LittleEndian>().map_err(|e| e.to_string())?,
        y: r.read_f32::<LittleEndian>().map_err(|e| e.to_string())?,
        z: r.read_f32::<LittleEndian>().map_err(|e| e.to_string())?,
    };
    let mut m = [[0.0f32; 4]; 4];
    for row in 0..3 {
        for col in 0..3 {
            m[row][col] = r.read_f32::<LittleEndian>().map_err(|e| e.to_string())?;
        }
    }
    m[3][3] = 1.0;
    let tolerance = r.read_f32::<LittleEndian>().map_err(|e| e.to_string())?;
    let max_speed = r.read_f32::<LittleEndian>().map_err(|e| e.to_string())?;
    let extra1 = r.read_u32::<LittleEndian>().map_err(|e| e.to_string())?;
    let extra2 = r.read_u32::<LittleEndian>().map_err(|e| e.to_string())?;

    Ok(HODDockpoint {
        position,
        rotation: Matrix4 { m },
        tolerance,
        max_speed,
        extra1,
        extra2,
    })
}

fn read_dockpoint_matrix4<R: Read>(r: &mut R) -> Result<HODDockpoint, String> {
    let position = Vector3 {
        x: r.read_f32::<LittleEndian>().map_err(|e| e.to_string())?,
        y: r.read_f32::<LittleEndian>().map_err(|e| e.to_string())?,
        z: r.read_f32::<LittleEndian>().map_err(|e| e.to_string())?,
    };
    let mut m = [[0.0f32; 4]; 4];
    for row in 0..4 {
        for col in 0..4 {
            m[row][col] = r.read_f32::<LittleEndian>().map_err(|e| e.to_string())?;
        }
    }
    let tolerance = r.read_f32::<LittleEndian>().map_err(|e| e.to_string())?;
    let max_speed = r.read_f32::<LittleEndian>().map_err(|e| e.to_string())?;

    Ok(HODDockpoint {
        position,
        rotation: Matrix4 { m },
        tolerance,
        max_speed,
        extra1: 0,
        extra2: 0,
    })
}

fn parse_kdop_collision_mesh(data: &[u8], name: &str) -> Result<HODCollisionMesh, String> {
    const KDOP_HEADER_SIZE: u64 = 28 + 13 * 32;
    if data.len() < KDOP_HEADER_SIZE as usize + 8 {
        return Err(format!("KDOP payload too small: {} bytes", data.len()));
    }

    let mut r = Cursor::new(data);
    let radius = r.read_f32::<LittleEndian>().map_err(|e| e.to_string())?;
    let min_extents = Vector3 {
        x: r.read_f32::<LittleEndian>().map_err(|e| e.to_string())?,
        y: r.read_f32::<LittleEndian>().map_err(|e| e.to_string())?,
        z: r.read_f32::<LittleEndian>().map_err(|e| e.to_string())?,
    };
    let max_extents = Vector3 {
        x: r.read_f32::<LittleEndian>().map_err(|e| e.to_string())?,
        y: r.read_f32::<LittleEndian>().map_err(|e| e.to_string())?,
        z: r.read_f32::<LittleEndian>().map_err(|e| e.to_string())?,
    };
    let center = Vector3 {
        x: (min_extents.x + max_extents.x) * 0.5,
        y: (min_extents.y + max_extents.y) * 0.5,
        z: (min_extents.z + max_extents.z) * 0.5,
    };

    r.set_position(KDOP_HEADER_SIZE);
    let vertex_count = r.read_u32::<LittleEndian>().map_err(|e| e.to_string())? as usize;
    if vertex_count > u16::MAX as usize + 1 {
        return Err(format!("KDOP vertex count is too large: {}", vertex_count));
    }

    let vertex_bytes = vertex_count
        .checked_mul(12)
        .ok_or_else(|| "KDOP vertex byte count overflow".to_string())?;
    if r.position() as usize + vertex_bytes + 4 > data.len() {
        return Err(format!(
            "KDOP vertex buffer exceeds payload: vertices={}, bytes={}",
            vertex_count,
            data.len()
        ));
    }

    let mut vertices = Vec::with_capacity(vertex_count);
    for _ in 0..vertex_count {
        vertices.push(HODVertex {
            position: Vector3 {
                x: r.read_f32::<LittleEndian>().map_err(|e| e.to_string())?,
                y: r.read_f32::<LittleEndian>().map_err(|e| e.to_string())?,
                z: r.read_f32::<LittleEndian>().map_err(|e| e.to_string())?,
            },
            normal: None,
            color: None,
            uv: None,
            tangent: None,
            binormal: None,
            skinning_data: None,
        });
    }

    let face_count = r.read_u32::<LittleEndian>().map_err(|e| e.to_string())? as usize;
    let index_count = face_count
        .checked_mul(3)
        .ok_or_else(|| "KDOP index count overflow".to_string())?;
    let index_bytes = index_count
        .checked_mul(2)
        .ok_or_else(|| "KDOP index byte count overflow".to_string())?;
    if r.position() as usize + index_bytes > data.len() {
        return Err(format!(
            "KDOP index buffer exceeds payload: faces={}, bytes={}",
            face_count,
            data.len()
        ));
    }

    let mut indices = Vec::with_capacity(index_count);
    for _ in 0..index_count {
        let idx = r.read_u16::<LittleEndian>().map_err(|e| e.to_string())?;
        if idx as usize >= vertex_count {
            return Err(format!(
                "KDOP index {} is out of bounds for {} vertices",
                idx, vertex_count
            ));
        }
        indices.push(idx);
    }

    Ok(HODCollisionMesh {
        name: name.to_string(),
        min_extents,
        max_extents,
        center,
        radius,
        mesh: HODMesh {
            name: "CollisionMesh".to_string(),
            parent_name: name.to_string(),
            lod: 0,
            has_mult_tags: false,
            parts: vec![HODMeshPart {
                material_index: 0,
                vertex_mask: 1,
                vertices,
                indices,
            }],
        },
    })
}

fn write_len_string<W: Write>(writer: &mut W, s: &str) -> Result<(), String> {
    let bytes = s.as_bytes();
    writer
        .write_u32::<LittleEndian>(bytes.len() as u32)
        .map_err(|e| e.to_string())?;
    writer.write_all(bytes).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn synthesize_engine_nozzles_v1(model: &mut HODModel) {
    if !model.engine_burns.is_empty() {
        return;
    }

    let mut burns = Vec::new();
    let mut remaining_navs = Vec::new();

    for nav in &model.nav_lights {
        if nav.name.starts_with("e")
            && nav.name.len() >= 2
            && nav.name[1..].chars().all(char::is_numeric)
        {
            let size = if nav.size > 0.0 { nav.size } else { 4.0 };
            burns.push(crate::hod::HODEngineBurn {
                name: nav.name.clone(),
                parent_name: nav.name.clone(),
                num_divisions: 8,
                num_flames: 1,
                vertices: vec![
                    crate::hod::Vector3 {
                        x: -size,
                        y: -size,
                        z: 0.0,
                    },
                    crate::hod::Vector3 {
                        x: size,
                        y: -size,
                        z: 0.0,
                    },
                    crate::hod::Vector3 {
                        x: size,
                        y: size,
                        z: 0.0,
                    },
                    crate::hod::Vector3 {
                        x: -size,
                        y: size,
                        z: 0.0,
                    },
                ],
            });
        } else {
            remaining_navs.push(nav.clone());
        }
    }
    model.nav_lights = remaining_navs;
    model.engine_burns.extend(burns);
}

pub fn reverse_engine_nozzles_v1(model: &mut HODModel) {
    if model.engine_burns.is_empty() {
        return;
    }

    let mut new_navs = Vec::new();
    let mut i = 0;
    while i < model.engine_burns.len() {
        let burn = &model.engine_burns[i];
        if burn.name.starts_with("e")
            && burn.name.len() >= 2
            && burn.name[1..].chars().all(char::is_numeric)
        {
            new_navs.push(crate::hod::HODNavLight {
                name: burn.name.clone(),
                section: 0,
                size: 4.0, // Best guess fallback
                phase: 0.0,
                frequency: 1.0,
                style: "engine".to_string(),
                color: crate::hod::Vector3 {
                    x: 1.0,
                    y: 1.0,
                    z: 1.0,
                },
                distance: 0.0,
                sprite_visible: true,
                high_end_only: false,
            });
            model.engine_burns.remove(i);
        } else {
            i += 1;
        }
    }

    model.nav_lights.extend(new_navs);
}

pub fn validate_marker_parents(model: &mut HODModel) {
    for marker in &mut model.markers {
        if marker.parent_joint.is_empty() {
            marker.parent_joint = "Root".to_string();
        }
    }
}

pub fn generate_collision_mesh(model: &mut HODModel) {
    normalize_collision_meshes(model);

    // Always generate box vertices for collision meshes that have extents but no vertices
    for cm in &mut model.collision_meshes {
        if cm.mesh.parts.is_empty() || cm.mesh.parts.iter().all(|p| p.vertices.is_empty()) {
            let min = &cm.min_extents;
            let max = &cm.max_extents;

            println!(
                "[RUST] Generating collision mesh box vertices from extents: min=({},{},{}), max=({},{},{})",
                min.x, min.y, min.z, max.x, max.y, max.z
            );

            // Box vertices: 8 corners
            let vertices = vec![
                crate::hod::HODVertex {
                    position: crate::hod::Vector3 {
                        x: min.x,
                        y: min.y,
                        z: min.z,
                    },
                    normal: None,
                    color: None,
                    uv: None,
                    tangent: None,
                    binormal: None,
                    skinning_data: None,
                },
                crate::hod::HODVertex {
                    position: crate::hod::Vector3 {
                        x: max.x,
                        y: min.y,
                        z: min.z,
                    },
                    normal: None,
                    color: None,
                    uv: None,
                    tangent: None,
                    binormal: None,
                    skinning_data: None,
                },
                crate::hod::HODVertex {
                    position: crate::hod::Vector3 {
                        x: max.x,
                        y: max.y,
                        z: min.z,
                    },
                    normal: None,
                    color: None,
                    uv: None,
                    tangent: None,
                    binormal: None,
                    skinning_data: None,
                },
                crate::hod::HODVertex {
                    position: crate::hod::Vector3 {
                        x: min.x,
                        y: max.y,
                        z: min.z,
                    },
                    normal: None,
                    color: None,
                    uv: None,
                    tangent: None,
                    binormal: None,
                    skinning_data: None,
                },
                crate::hod::HODVertex {
                    position: crate::hod::Vector3 {
                        x: min.x,
                        y: min.y,
                        z: max.z,
                    },
                    normal: None,
                    color: None,
                    uv: None,
                    tangent: None,
                    binormal: None,
                    skinning_data: None,
                },
                crate::hod::HODVertex {
                    position: crate::hod::Vector3 {
                        x: max.x,
                        y: min.y,
                        z: max.z,
                    },
                    normal: None,
                    color: None,
                    uv: None,
                    tangent: None,
                    binormal: None,
                    skinning_data: None,
                },
                crate::hod::HODVertex {
                    position: crate::hod::Vector3 {
                        x: max.x,
                        y: max.y,
                        z: max.z,
                    },
                    normal: None,
                    color: None,
                    uv: None,
                    tangent: None,
                    binormal: None,
                    skinning_data: None,
                },
                crate::hod::HODVertex {
                    position: crate::hod::Vector3 {
                        x: min.x,
                        y: max.y,
                        z: max.z,
                    },
                    normal: None,
                    color: None,
                    uv: None,
                    tangent: None,
                    binormal: None,
                    skinning_data: None,
                },
            ];

            // Box indices: 12 triangles (6 faces × 2 triangles)
            let indices = vec![
                0, 1, 2, 0, 2, 3, // front
                4, 6, 5, 4, 7, 6, // back
                0, 4, 5, 0, 5, 1, // left
                2, 6, 7, 2, 7, 3, // right
                0, 3, 7, 0, 7, 4, // top
                1, 5, 6, 1, 6, 2, // bottom
            ];

            cm.mesh.parts = vec![crate::hod::HODMeshPart {
                material_index: 0,
                vertex_mask: 1, // position only
                vertices,
                indices,
            }];

            println!(
                "[RUST] Generated collision mesh: {} vertices, {} indices",
                cm.mesh.parts[0].vertices.len(),
                cm.mesh.parts[0].indices.len()
            );
        }
    }
}

pub fn compose_transform_matrix(pos: Vector3, rot: Vector3, scale: Vector3) -> Matrix4 {
    let cx = rot.x.cos();
    let sx = rot.x.sin();
    let cy = rot.y.cos();
    let sy = rot.y.sin();
    let cz = rot.z.cos();
    let sz = rot.z.sin();

    // Enforce a minimum scale of 1.0 if the parsed scale is zero or near-zero
    // to prevent matrix multiplication from collapsing child translations to (0,0,0)
    let s_x = if scale.x.abs() < 0.0001 { 1.0 } else { scale.x };
    let s_y = if scale.y.abs() < 0.0001 { 1.0 } else { scale.y };
    let s_z = if scale.z.abs() < 0.0001 { 1.0 } else { scale.z };

    let mut ms = [[0.0f32; 4]; 4];
    ms[0][0] = s_x;
    ms[1][1] = s_y;
    ms[2][2] = s_z;
    ms[3][3] = 1.0;

    let mut rx = [[0.0f32; 4]; 4];
    rx[0][0] = 1.0;
    rx[1][1] = cx;
    rx[1][2] = sx;
    rx[2][1] = -sx;
    rx[2][2] = cx;
    rx[3][3] = 1.0;

    let mut ry = [[0.0f32; 4]; 4];
    ry[0][0] = cy;
    ry[0][2] = -sy;
    ry[1][1] = 1.0;
    ry[2][0] = sy;
    ry[2][2] = cy;
    ry[3][3] = 1.0;

    let mut rz = [[0.0f32; 4]; 4];
    rz[0][0] = cz;
    rz[0][1] = sz;
    rz[1][0] = -sz;
    rz[1][1] = cz;
    rz[2][2] = 1.0;
    rz[3][3] = 1.0;

    fn mat_mul(a: [[f32; 4]; 4], b: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
        let mut res = [[0.0f32; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                res[i][j] =
                    a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j] + a[i][3] * b[3][j];
            }
        }
        res
    }

    let m_s_rx = mat_mul(ms, rx);
    let m_s_rx_ry = mat_mul(m_s_rx, ry);
    let mut final_rot = mat_mul(m_s_rx_ry, rz);

    final_rot[3][0] = pos.x;
    final_rot[3][1] = pos.y;
    final_rot[3][2] = pos.z;
    final_rot[3][3] = 1.0;

    Matrix4 { m: final_rot }
}

pub fn decompose_matrix(matrix: Matrix4) -> (Vector3, Vector3, Vector3) {
    let m = matrix.m;
    let pos = Vector3 {
        x: m[3][0],
        y: m[3][1],
        z: m[3][2],
    };

    let sx = (m[0][0] * m[0][0] + m[0][1] * m[0][1] + m[0][2] * m[0][2]).sqrt();
    let sy = (m[1][0] * m[1][0] + m[1][1] * m[1][1] + m[1][2] * m[1][2]).sqrt();
    let sz = (m[2][0] * m[2][0] + m[2][1] * m[2][1] + m[2][2] * m[2][2]).sqrt();
    let scale = Vector3 {
        x: sx,
        y: sy,
        z: sz,
    };

    let r13 = if sx > 0.0 { m[0][2] / sx } else { 0.0 };
    let r12 = if sx > 0.0 { m[0][1] / sx } else { 0.0 };
    let r11 = if sx > 0.0 { m[0][0] / sx } else { 0.0 };
    let r23 = if sy > 0.0 { m[1][2] / sy } else { 0.0 };
    let r33 = if sz > 0.0 { m[2][2] / sz } else { 0.0 };
    let r21 = if sy > 0.0 { m[1][0] / sy } else { 0.0 };
    let r22 = if sy > 0.0 { m[1][1] / sy } else { 0.0 };

    let mut rx = 0.0;
    let mut ry = 0.0;
    let mut rz = 0.0;

    if r13.abs() < 0.99999 {
        ry = (-r13).asin();
        rx = r23.atan2(r33);
        rz = r12.atan2(r11);
    } else {
        ry = if r13 < 0.0 {
            std::f32::consts::FRAC_PI_2
        } else {
            -std::f32::consts::FRAC_PI_2
        };
        rx = -r21.atan2(r22);
        rz = 0.0;
    }

    let rot = Vector3 {
        x: rx,
        y: ry,
        z: rz,
    };
    (pos, rot, scale)
}

fn parse_joints(chunk: &IffChunk) -> Result<Vec<HODJoint>, String> {
    let mut reader = Cursor::new(&chunk.data);
    let mut joints = Vec::new();

    if chunk.data.len() < 4 {
        return Ok(joints);
    }

    let first_val = reader
        .read_u32::<LittleEndian>()
        .map_err(|e| e.to_string())?;
    println!("[RUST] HIER chunk first_val: 0x{:08X}", first_val);

    // HOD 2.0 stores the joint count as a full signed negative value.
    // Large hierarchies can exceed 255 joints (e.g. -298 = 0xFFFFFED6),
    // so this must not be detected via a 0xFFFFFF00 byte mask.
    if (first_val as i32) < 0 {
        // HOD 2.0 joints hierarchy
        while reader.position() + 4 <= chunk.data.len() as u64 {
            let remaining = chunk.data.len() as u64 - reader.position();
            if remaining < 44 {
                break;
            }
            let name = read_len_string(&mut reader)?;
            if name.is_empty() {
                break;
            }
            let parent_raw = read_len_string(&mut reader)?;
            let parent_name = if parent_raw.is_empty() {
                None
            } else {
                Some(parent_raw)
            };

            let px = reader
                .read_f32::<LittleEndian>()
                .map_err(|e| e.to_string())?;
            let py = reader
                .read_f32::<LittleEndian>()
                .map_err(|e| e.to_string())?;
            let pz = reader
                .read_f32::<LittleEndian>()
                .map_err(|e| e.to_string())?;

            let rx = reader
                .read_f32::<LittleEndian>()
                .map_err(|e| e.to_string())?;
            let ry = reader
                .read_f32::<LittleEndian>()
                .map_err(|e| e.to_string())?;
            let rz = reader
                .read_f32::<LittleEndian>()
                .map_err(|e| e.to_string())?;

            let mut sx = reader
                .read_f32::<LittleEndian>()
                .map_err(|e| e.to_string())?;
            let mut sy = reader
                .read_f32::<LittleEndian>()
                .map_err(|e| e.to_string())?;
            let mut sz = reader
                .read_f32::<LittleEndian>()
                .map_err(|e| e.to_string())?;

            // Retroactive fix: Any existing HOD 2.0 file corrupted by our previous (1.0, 1.0, 1.0) fallback bug
            // will be repaired upon load so it saves correctly with zeroed gimbal limits.
            if (sx - 1.0).abs() < 0.001 && (sy - 1.0).abs() < 0.001 && (sz - 1.0).abs() < 0.001 {
                sx = 0.0;
                sy = 0.0;
                sz = 0.0;
            }

            let local_transform = compose_transform_matrix(
                Vector3 {
                    x: px,
                    y: py,
                    z: pz,
                },
                Vector3 {
                    x: rx,
                    y: ry,
                    z: rz,
                },
                Vector3 {
                    x: sx,
                    y: sy,
                    z: sz,
                },
            );

            joints.push(HODJoint {
                name,
                parent_name,
                local_transform,
                position: Some(Vector3 {
                    x: px,
                    y: py,
                    z: pz,
                }),
                rotation: Some(Vector3 {
                    x: rx,
                    y: ry,
                    z: rz,
                }),
                scale: Some(Vector3 {
                    x: sx,
                    y: sy,
                    z: sz,
                }),
            });
        }
    } else {
        // HOD 1.0 joints hierarchy
        let count = first_val as usize;
        for _ in 0..count {
            let name = read_len_string(&mut reader)?;
            let parent_raw = read_len_string(&mut reader)?;
            let parent_name = if parent_raw.is_empty() {
                None
            } else {
                Some(parent_raw)
            };

            let px = reader
                .read_f32::<LittleEndian>()
                .map_err(|e| e.to_string())?;
            let py = reader
                .read_f32::<LittleEndian>()
                .map_err(|e| e.to_string())?;
            let pz = reader
                .read_f32::<LittleEndian>()
                .map_err(|e| e.to_string())?;

            let rx = reader
                .read_f32::<LittleEndian>()
                .map_err(|e| e.to_string())?;
            let ry = reader
                .read_f32::<LittleEndian>()
                .map_err(|e| e.to_string())?;
            let rz = reader
                .read_f32::<LittleEndian>()
                .map_err(|e| e.to_string())?;

            let sx = reader
                .read_f32::<LittleEndian>()
                .map_err(|e| e.to_string())?;
            let sy = reader
                .read_f32::<LittleEndian>()
                .map_err(|e| e.to_string())?;
            let sz = reader
                .read_f32::<LittleEndian>()
                .map_err(|e| e.to_string())?;

            let _ax = reader
                .read_f32::<LittleEndian>()
                .map_err(|e| e.to_string())?;
            let _ay = reader
                .read_f32::<LittleEndian>()
                .map_err(|e| e.to_string())?;
            let _az = reader
                .read_f32::<LittleEndian>()
                .map_err(|e| e.to_string())?;

            let mut dof = [0u8; 3];
            reader.read_exact(&mut dof).map_err(|e| e.to_string())?;

            let local_transform = compose_transform_matrix(
                Vector3 {
                    x: px,
                    y: py,
                    z: pz,
                },
                Vector3 {
                    x: rx,
                    y: ry,
                    z: rz,
                },
                Vector3 {
                    x: sx,
                    y: sy,
                    z: sz,
                },
            );

            joints.push(HODJoint {
                name,
                parent_name,
                local_transform,
                position: Some(Vector3 {
                    x: px,
                    y: py,
                    z: pz,
                }),
                rotation: Some(Vector3 {
                    x: rx,
                    y: ry,
                    z: rz,
                }),
                scale: Some(Vector3 {
                    x: sx,
                    y: sy,
                    z: sz,
                }),
            });
        }
    }

    Ok(joints)
}

fn parse_marker_single<R: Read>(reader: &mut R, is_v2: bool) -> Result<HODMarker, String> {
    let name = read_len_string(reader)?;
    let parent_joint = read_len_string(reader)?;

    if !is_v2 {
        let _start_time = reader
            .read_f32::<LittleEndian>()
            .map_err(|e| e.to_string())?;
        let _end_time = reader
            .read_f32::<LittleEndian>()
            .map_err(|e| e.to_string())?;
    }

    let px = reader
        .read_f64::<LittleEndian>()
        .map_err(|e| e.to_string())? as f32;
    let py = reader
        .read_f64::<LittleEndian>()
        .map_err(|e| e.to_string())? as f32;
    let pz = reader
        .read_f64::<LittleEndian>()
        .map_err(|e| e.to_string())? as f32;

    let rx = reader
        .read_f64::<LittleEndian>()
        .map_err(|e| e.to_string())? as f32;
    let ry = reader
        .read_f64::<LittleEndian>()
        .map_err(|e| e.to_string())? as f32;
    let rz = reader
        .read_f64::<LittleEndian>()
        .map_err(|e| e.to_string())? as f32;

    let cx = rx.cos();
    let sx = rx.sin();
    let cy = ry.cos();
    let sy = ry.sin();
    let cz = rz.cos();
    let sz = rz.sin();

    let mut rot = [[0.0f32; 4]; 4];
    let r11 = cy * cz;
    let r12 = cy * sz;
    let r13 = -sy;

    let r21 = sx * sy * cz - cx * sz;
    let r22 = sx * sy * sz + cx * cz;
    let r23 = sx * cy;

    let r31 = cx * sy * cz + sx * sz;
    let r32 = cx * sy * sz - sx * cz;
    let r33 = cx * cy;

    rot[0][0] = r11;
    rot[0][1] = r12;
    rot[0][2] = r13;
    rot[1][0] = r21;
    rot[1][1] = r22;
    rot[1][2] = r23;
    rot[2][0] = r31;
    rot[2][1] = r32;
    rot[2][2] = r33;
    rot[3][3] = 1.0;

    Ok(HODMarker {
        name,
        parent_joint,
        position: Vector3 {
            x: px,
            y: py,
            z: pz,
        },
        rotation: Matrix4 { m: rot },
        rotation_euler: Some(Vector3 {
            x: rx,
            y: ry,
            z: rz,
        }),
    })
}

fn parse_marker_v1(chunk: &IffChunk) -> Result<HODMarker, String> {
    let mut reader = Cursor::new(&chunk.data);
    parse_marker_single(&mut reader, false)
}

fn parse_markers_v2(chunk: &IffChunk) -> Result<Vec<HODMarker>, String> {
    let mut reader = Cursor::new(&chunk.data);
    let count = reader.read_u32::<LittleEndian>().unwrap_or(0) as usize;
    let mut markers = Vec::new();
    for _ in 0..count {
        let m = parse_marker_single(&mut reader, true)?;
        markers.push(m);
    }
    Ok(markers)
}

fn serialize_single_marker(marker: &HODMarker, is_v2: bool) -> Result<Vec<u8>, String> {
    let mut data = Vec::new();
    write_len_string(&mut data, &marker.name)?;
    write_len_string(&mut data, &marker.parent_joint)?;

    if !is_v2 {
        data.write_f32::<LittleEndian>(0.0)
            .map_err(|e| e.to_string())?;
        data.write_f32::<LittleEndian>(0.0)
            .map_err(|e| e.to_string())?;
    }

    data.write_f64::<LittleEndian>(marker.position.x as f64)
        .map_err(|e| e.to_string())?;
    data.write_f64::<LittleEndian>(marker.position.y as f64)
        .map_err(|e| e.to_string())?;
    data.write_f64::<LittleEndian>(marker.position.z as f64)
        .map_err(|e| e.to_string())?;

    let rot_euler = if let Some(ref euler) = marker.rotation_euler {
        euler.clone()
    } else {
        let (_, r, _) = decompose_matrix(marker.rotation.clone());
        r
    };

    data.write_f64::<LittleEndian>(rot_euler.x as f64)
        .map_err(|e| e.to_string())?;
    data.write_f64::<LittleEndian>(rot_euler.y as f64)
        .map_err(|e| e.to_string())?;
    data.write_f64::<LittleEndian>(rot_euler.z as f64)
        .map_err(|e| e.to_string())?;

    Ok(data)
}

// Low-overhead base64 encoder for image base64 streaming previews
fn base64_encode(data: &[u8]) -> String {
    const CHARSET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    let mut i = 0;
    while i < data.len() {
        let byte1 = data[i];
        let byte2 = if i + 1 < data.len() {
            Some(data[i + 1])
        } else {
            None
        };
        let byte3 = if i + 2 < data.len() {
            Some(data[i + 2])
        } else {
            None
        };

        let val = ((byte1 as u32) << 16)
            | ((byte2.unwrap_or(0) as u32) << 8)
            | (byte3.unwrap_or(0) as u32);

        let char1 = CHARSET[((val >> 18) & 63) as usize] as char;
        let char2 = CHARSET[((val >> 12) & 63) as usize] as char;
        let char3 = if byte2.is_some() {
            CHARSET[((val >> 6) & 63) as usize] as char
        } else {
            '='
        };
        let char4 = if byte3.is_some() {
            CHARSET[(val & 63) as usize] as char
        } else {
            '='
        };

        result.push(char1);
        result.push(char2);
        result.push(char3);
        result.push(char4);

        i += 3;
    }
    result
}

pub fn write_vertex<W: Write>(
    writer: &mut W,
    vertex: &HODVertex,
    vertex_mask: u32,
    version: u32,
    stride: u32,
) -> Result<(), String> {
    let mut bytes_written = 0;
    if (vertex_mask & 0x1) != 0 {
        writer
            .write_f32::<LittleEndian>(vertex.position.x)
            .map_err(|e| e.to_string())?;
        writer
            .write_f32::<LittleEndian>(vertex.position.y)
            .map_err(|e| e.to_string())?;
        writer
            .write_f32::<LittleEndian>(vertex.position.z)
            .map_err(|e| e.to_string())?;
        writer
            .write_f32::<LittleEndian>(1.0)
            .map_err(|e| e.to_string())?;
        bytes_written += 16;
    }
    if (vertex_mask & 0x2) != 0 {
        let n = vertex.normal.clone().unwrap_or(Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        });
        writer
            .write_f32::<LittleEndian>(n.x)
            .map_err(|e| e.to_string())?;
        writer
            .write_f32::<LittleEndian>(n.y)
            .map_err(|e| e.to_string())?;
        writer
            .write_f32::<LittleEndian>(n.z)
            .map_err(|e| e.to_string())?;
        writer
            .write_f32::<LittleEndian>(1.0)
            .map_err(|e| e.to_string())?;
        bytes_written += 16;
    }
    if (vertex_mask & 0x4) != 0 {
        let col = vertex.color.unwrap_or(0xFFFFFFFF);
        writer
            .write_u32::<LittleEndian>(col)
            .map_err(|e| e.to_string())?;
        bytes_written += 4;
    }
    if (vertex_mask & 0x8) != 0 {
        let uv = vertex.uv.clone().unwrap_or(Vector2 { u: 0.0, v: 0.0 });
        writer
            .write_f32::<LittleEndian>(uv.u)
            .map_err(|e| e.to_string())?;
        writer
            .write_f32::<LittleEndian>(uv.v)
            .map_err(|e| e.to_string())?;
        bytes_written += 8;
    }
    if version == 1401 {
        if (vertex_mask & 0x10) != 0 {
            writer
                .write_f32::<LittleEndian>(0.0)
                .map_err(|e| e.to_string())?;
            writer
                .write_f32::<LittleEndian>(0.0)
                .map_err(|e| e.to_string())?;
            bytes_written += 8;
        }
        if (vertex_mask & 0x20) != 0 {
            writer
                .write_f32::<LittleEndian>(0.0)
                .map_err(|e| e.to_string())?;
            writer
                .write_f32::<LittleEndian>(0.0)
                .map_err(|e| e.to_string())?;
            bytes_written += 8;
        }
    }
    if (vertex_mask & 0x2000) != 0 {
        let t = vertex.tangent.clone().unwrap_or(Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        });
        writer
            .write_f32::<LittleEndian>(t.x)
            .map_err(|e| e.to_string())?;
        writer
            .write_f32::<LittleEndian>(t.y)
            .map_err(|e| e.to_string())?;
        writer
            .write_f32::<LittleEndian>(t.z)
            .map_err(|e| e.to_string())?;
        bytes_written += 12;
    }
    if (vertex_mask & 0x4000) != 0 {
        let b = vertex.binormal.clone().unwrap_or(Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        });
        writer
            .write_f32::<LittleEndian>(b.x)
            .map_err(|e| e.to_string())?;
        writer
            .write_f32::<LittleEndian>(b.y)
            .map_err(|e| e.to_string())?;
        writer
            .write_f32::<LittleEndian>(b.z)
            .map_err(|e| e.to_string())?;
        bytes_written += 12;
    }
    if stride > bytes_written {
        let padding_needed = stride - bytes_written;
        if let Some(ref padding) = vertex.skinning_data {
            if padding.len() == padding_needed as usize {
                writer.write_all(padding).map_err(|e| e.to_string())?;
            } else {
                writer
                    .write_all(&vec![0u8; padding_needed as usize])
                    .map_err(|e| e.to_string())?;
            }
        } else {
            writer
                .write_all(&vec![0u8; padding_needed as usize])
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn update_mesh_chunks(
    chunks: &mut [IffChunk],
    updated_model: &HODModel,
    is_v2: bool,
    new_mesh_pool: &mut Vec<u8>,
    new_face_pool: &mut Vec<u8>,
    parent_name: &str,
) -> Result<(), String> {
    for chunk in chunks {
        let mut current_parent_name = parent_name.to_string();

        if chunk.id == "MULT" || chunk.id == "GOBG" {
            let mut reader = Cursor::new(&chunk.data);
            let total_len = chunk.data.len();
            let mut len_bytes = [0u8; 4];

            let mut extracted_mesh_name = String::new();
            let mut extracted_parent_name = String::new();
            let mut current_pos = 0;

            if reader.read_exact(&mut len_bytes).is_ok() {
                let len = u32::from_le_bytes(len_bytes) as usize;
                if len < total_len {
                    let mut name_bytes = vec![0u8; len];
                    if reader.read_exact(&mut name_bytes).is_ok() {
                        extracted_mesh_name = String::from_utf8_lossy(&name_bytes).to_string();
                        current_parent_name = extracted_mesh_name.clone();

                        if reader.read_exact(&mut len_bytes).is_ok() {
                            let parent_len = u32::from_le_bytes(len_bytes) as usize;
                            let remaining = total_len.saturating_sub(reader.position() as usize);
                            if parent_len <= remaining {
                                let mut parent_bytes = vec![0u8; parent_len];
                                if reader.read_exact(&mut parent_bytes).is_ok() {
                                    extracted_parent_name = String::from_utf8_lossy(&parent_bytes)
                                        .trim_matches('\0')
                                        .to_string();
                                }
                            }
                        }

                        let _ = reader.read_u32::<LittleEndian>();
                        current_pos = reader.position() as usize;
                    }
                }
            }

            if !extracted_mesh_name.is_empty() && current_pos < total_len {
                let mut sub_chunks = Vec::new();
                let mut sub_cursor = Cursor::new(&chunk.data[current_pos..]);
                while sub_cursor.position() < sub_cursor.get_ref().len() as u64 {
                    if let Ok(c) = IffChunk::read_chunk(&mut sub_cursor) {
                        sub_chunks.push(c);
                    } else {
                        break;
                    }
                }

                // Detect NRML MULT: inline BMSH data, not pool-based
                let is_nrml_mult = chunk.chunk_type == ChunkType::Normal && chunk.data.len() > 20;
                update_mesh_chunks(
                    &mut sub_chunks,
                    updated_model,
                    is_v2 && !is_nrml_mult,
                    new_mesh_pool,
                    new_face_pool,
                    &extracted_mesh_name,
                )?;

                let original_chunk_type = chunk.chunk_type.clone();
                let mut new_mult_data = Vec::new();
                write_len_string(&mut new_mult_data, &extracted_mesh_name)?;
                write_len_string(&mut new_mult_data, &extracted_parent_name)?;

                let lod_count = sub_chunks.iter().filter(|c| c.id.trim() == "BMSH").count() as u32;
                new_mult_data
                    .write_u32::<LittleEndian>(lod_count)
                    .map_err(|e| e.to_string())?;

                if original_chunk_type == ChunkType::Form {
                    // FORM: children are serialized separately during write_chunk
                    chunk.data = new_mult_data;
                    chunk.children = sub_chunks;
                } else {
                    // NRML: serialize children into data
                    for child in &sub_chunks {
                        child
                            .write_chunk(&mut new_mult_data)
                            .map_err(|e| e.to_string())?;
                    }
                    chunk.data = new_mult_data;
                    chunk.children = Vec::new();
                }
                chunk.chunk_type = original_chunk_type;
            }
        } else if chunk.id == "GLOW" {
            let mut extracted_glow_name = String::new();
            if let Some(info_chunk) = chunk.children.iter().find(|c| c.id == "INFO") {
                let mut r = Cursor::new(&info_chunk.data);
                if let Ok(name) = read_len_string(&mut r) {
                    extracted_glow_name = name;
                }
            }
            current_parent_name = format!("GLOW:{}", extracted_glow_name);
        } else if chunk.id == "ETSH" || chunk.id == "COLD" {
            current_parent_name = chunk.id.clone();
        } else if chunk.id.trim() == "BMSH" {
            let mut reader = Cursor::new(&chunk.data);
            let lod = reader.read_i32::<LittleEndian>().unwrap_or(0);

            let mesh_name =
                if parent_name == "MSHL" || parent_name == "Root" || parent_name.is_empty() {
                    "BMSH"
                } else {
                    parent_name
                };

            let matched_mesh = if parent_name.starts_with("GLOW:") {
                let glow_name = &parent_name[5..];
                updated_model
                    .engine_glows
                    .iter()
                    .find(|g| g.name == glow_name)
                    .map(|g| &g.mesh)
            } else {
                updated_model.meshes.iter().find(|m| {
                    m.lod == lod && (m.name == mesh_name || m.name == "BMSH" || mesh_name == "BMSH")
                })
            };

            if let Some(mesh) = matched_mesh {
                let mut new_bmsh_data = Vec::new();
                new_bmsh_data
                    .write_i32::<LittleEndian>(mesh.lod)
                    .map_err(|e| e.to_string())?;
                new_bmsh_data
                    .write_i32::<LittleEndian>(mesh.parts.len() as i32)
                    .map_err(|e| e.to_string())?;

                for part in &mesh.parts {
                    new_bmsh_data
                        .write_i32::<LittleEndian>(part.material_index as i32)
                        .map_err(|e| e.to_string())?;
                    new_bmsh_data
                        .write_u32::<LittleEndian>(part.vertex_mask)
                        .map_err(|e| e.to_string())?;
                    new_bmsh_data
                        .write_i32::<LittleEndian>(part.vertices.len() as i32)
                        .map_err(|e| e.to_string())?;

                    if is_v2 {
                        let mut vertex_stride = 0;
                        if (part.vertex_mask & 0x01) != 0 {
                            vertex_stride += 16;
                        }
                        if (part.vertex_mask & 0x02) != 0 {
                            vertex_stride += 16;
                        }
                        if (part.vertex_mask & 0x04) != 0 {
                            vertex_stride += 4;
                        }
                        if (part.vertex_mask & 0x08) != 0 {
                            vertex_stride += 8;
                        }
                        if chunk.version == 1401 {
                            if (part.vertex_mask & 0x10) != 0 {
                                vertex_stride += 8;
                            }
                            if (part.vertex_mask & 0x20) != 0 {
                                vertex_stride += 8;
                            }
                        }
                        if (part.vertex_mask & 0x2000) != 0 {
                            vertex_stride += 12;
                        }
                        if (part.vertex_mask & 0x4000) != 0 {
                            vertex_stride += 12;
                        }
                        if vertex_stride == 0 {
                            vertex_stride = 1;
                        }

                        for vertex in &part.vertices {
                            write_vertex(
                                new_mesh_pool,
                                vertex,
                                part.vertex_mask,
                                chunk.version,
                                vertex_stride,
                            )?;
                        }

                        new_bmsh_data
                            .write_i16::<LittleEndian>(-1)
                            .map_err(|e| e.to_string())?;
                        new_bmsh_data
                            .write_i32::<LittleEndian>(part.indices.len() as i32)
                            .map_err(|e| e.to_string())?;
                        for &idx in &part.indices {
                            new_face_pool
                                .write_u16::<LittleEndian>(idx)
                                .map_err(|e| e.to_string())?;
                        }
                    } else {
                        let mut vertex_stride = 0;
                        if (part.vertex_mask & 0x01) != 0 {
                            vertex_stride += 16;
                        }
                        if (part.vertex_mask & 0x02) != 0 {
                            vertex_stride += 16;
                        }
                        if (part.vertex_mask & 0x04) != 0 {
                            vertex_stride += 4;
                        }
                        if (part.vertex_mask & 0x08) != 0 {
                            vertex_stride += 8;
                        }
                        if chunk.version == 1401 {
                            if (part.vertex_mask & 0x10) != 0 {
                                vertex_stride += 8;
                            }
                            if (part.vertex_mask & 0x20) != 0 {
                                vertex_stride += 8;
                            }
                        }
                        if (part.vertex_mask & 0x2000) != 0 {
                            vertex_stride += 12;
                        }
                        if (part.vertex_mask & 0x4000) != 0 {
                            vertex_stride += 12;
                        }
                        if vertex_stride == 0 {
                            vertex_stride = 1;
                        }

                        for vertex in &part.vertices {
                            write_vertex(
                                &mut new_bmsh_data,
                                vertex,
                                part.vertex_mask,
                                chunk.version,
                                vertex_stride,
                            )?;
                        }
                        new_bmsh_data
                            .write_i16::<LittleEndian>(1)
                            .map_err(|e| e.to_string())?;
                        new_bmsh_data
                            .write_u32::<LittleEndian>(514)
                            .map_err(|e| e.to_string())?;
                        new_bmsh_data
                            .write_i32::<LittleEndian>(part.indices.len() as i32)
                            .map_err(|e| e.to_string())?;
                        for &idx in &part.indices {
                            new_bmsh_data
                                .write_u16::<LittleEndian>(idx)
                                .map_err(|e| e.to_string())?;
                        }
                    }
                }
                chunk.data = new_bmsh_data;
            } else {
                let matched_mesh_fallback = if parent_name.starts_with("GLOW:") {
                    let glow_name = &parent_name[5..];
                    updated_model
                        .engine_glows
                        .iter()
                        .find(|g| g.name == glow_name)
                        .map(|g| &g.mesh)
                } else {
                    updated_model.meshes.iter().find(|m| m.lod == lod)
                };
                if let Some(mesh) = matched_mesh_fallback {
                    let mut new_bmsh_data = Vec::new();
                    new_bmsh_data
                        .write_i32::<LittleEndian>(mesh.lod)
                        .map_err(|e| e.to_string())?;
                    new_bmsh_data
                        .write_i32::<LittleEndian>(mesh.parts.len() as i32)
                        .map_err(|e| e.to_string())?;

                    for part in &mesh.parts {
                        new_bmsh_data
                            .write_i32::<LittleEndian>(part.material_index as i32)
                            .map_err(|e| e.to_string())?;
                        new_bmsh_data
                            .write_u32::<LittleEndian>(part.vertex_mask)
                            .map_err(|e| e.to_string())?;
                        new_bmsh_data
                            .write_i32::<LittleEndian>(part.vertices.len() as i32)
                            .map_err(|e| e.to_string())?;

                        if is_v2 {
                            let mut vertex_stride = 0;
                            if (part.vertex_mask & 0x01) != 0 {
                                vertex_stride += 16;
                            }
                            if (part.vertex_mask & 0x02) != 0 {
                                vertex_stride += 16;
                            }
                            if (part.vertex_mask & 0x04) != 0 {
                                vertex_stride += 4;
                            }
                            if (part.vertex_mask & 0x08) != 0 {
                                vertex_stride += 8;
                            }
                            if chunk.version == 1401 {
                                if (part.vertex_mask & 0x10) != 0 {
                                    vertex_stride += 8;
                                }
                                if (part.vertex_mask & 0x20) != 0 {
                                    vertex_stride += 8;
                                }
                            }
                            if (part.vertex_mask & 0x2000) != 0 {
                                vertex_stride += 12;
                            }
                            if (part.vertex_mask & 0x4000) != 0 {
                                vertex_stride += 12;
                            }
                            if vertex_stride == 0 {
                                vertex_stride = 1;
                            }

                            for vertex in &part.vertices {
                                write_vertex(
                                    new_mesh_pool,
                                    vertex,
                                    part.vertex_mask,
                                    chunk.version,
                                    vertex_stride,
                                )?;
                            }
                            new_bmsh_data
                                .write_i16::<LittleEndian>(-1)
                                .map_err(|e| e.to_string())?;
                            new_bmsh_data
                                .write_i32::<LittleEndian>(part.indices.len() as i32)
                                .map_err(|e| e.to_string())?;
                            for &idx in &part.indices {
                                new_face_pool
                                    .write_u16::<LittleEndian>(idx)
                                    .map_err(|e| e.to_string())?;
                            }
                        } else {
                            let mut vertex_stride = 0;
                            if (part.vertex_mask & 0x01) != 0 {
                                vertex_stride += 16;
                            }
                            if (part.vertex_mask & 0x02) != 0 {
                                vertex_stride += 16;
                            }
                            if (part.vertex_mask & 0x04) != 0 {
                                vertex_stride += 4;
                            }
                            if (part.vertex_mask & 0x08) != 0 {
                                vertex_stride += 8;
                            }
                            if chunk.version == 1401 {
                                if (part.vertex_mask & 0x10) != 0 {
                                    vertex_stride += 8;
                                }
                                if (part.vertex_mask & 0x20) != 0 {
                                    vertex_stride += 8;
                                }
                            }
                            if (part.vertex_mask & 0x2000) != 0 {
                                vertex_stride += 12;
                            }
                            if (part.vertex_mask & 0x4000) != 0 {
                                vertex_stride += 12;
                            }
                            if vertex_stride == 0 {
                                vertex_stride = 1;
                            }

                            for vertex in &part.vertices {
                                write_vertex(
                                    &mut new_bmsh_data,
                                    vertex,
                                    part.vertex_mask,
                                    chunk.version,
                                    vertex_stride,
                                )?;
                            }
                            new_bmsh_data
                                .write_i16::<LittleEndian>(1)
                                .map_err(|e| e.to_string())?;
                            new_bmsh_data
                                .write_u32::<LittleEndian>(514)
                                .map_err(|e| e.to_string())?;
                            new_bmsh_data
                                .write_i32::<LittleEndian>(part.indices.len() as i32)
                                .map_err(|e| e.to_string())?;
                            for &idx in &part.indices {
                                new_bmsh_data
                                    .write_u16::<LittleEndian>(idx)
                                    .map_err(|e| e.to_string())?;
                            }
                        }
                    }
                    chunk.data = new_bmsh_data;
                }
            }
        }

        if !chunk.children.is_empty() {
            update_mesh_chunks(
                &mut chunk.children,
                updated_model,
                is_v2,
                new_mesh_pool,
                new_face_pool,
                &current_parent_name,
            )?;
        }
    }
    Ok(())
}

fn sanitize_prim_group_counts(chunks: &mut [IffChunk]) -> Result<(), String> {
    for chunk in chunks {
        if chunk.id == "MULT" || chunk.id == "GOBG" {
            let mut reader = Cursor::new(&chunk.data);
            let total_len = chunk.data.len();
            let mut len_bytes = [0u8; 4];
            let mut current_pos = 0;

            if reader.read_exact(&mut len_bytes).is_ok() {
                let len = u32::from_le_bytes(len_bytes) as usize;
                if len < total_len {
                    let mut name_bytes = vec![0u8; len];
                    if reader.read_exact(&mut name_bytes).is_ok() {
                        if reader.read_exact(&mut len_bytes).is_ok() {
                            let parent_len = u32::from_le_bytes(len_bytes) as usize;
                            let remaining = total_len.saturating_sub(reader.position() as usize);
                            if parent_len <= remaining {
                                let mut parent_bytes = vec![0u8; parent_len];
                                if reader.read_exact(&mut parent_bytes).is_ok() {
                                    let _ = reader.read_u32::<LittleEndian>();
                                    current_pos = reader.position() as usize;
                                }
                            }
                        }
                    }
                }
            }

            if current_pos > 0 && current_pos < total_len {
                let mut sub_chunks = Vec::new();
                let mut sub_cursor = Cursor::new(&chunk.data[current_pos..]);
                while sub_cursor.position() < sub_cursor.get_ref().len() as u64 {
                    if let Ok(c) = IffChunk::read_chunk(&mut sub_cursor) {
                        sub_chunks.push(c);
                    } else {
                        break;
                    }
                }

                sanitize_prim_group_counts(&mut sub_chunks)?;

                let mut new_mult_data = chunk.data[..current_pos].to_vec();
                for child in &sub_chunks {
                    child
                        .write_chunk(&mut new_mult_data)
                        .map_err(|e| e.to_string())?;
                }
                chunk.data = new_mult_data;
            }
        } else if chunk.id.trim() == "BMSH" {
            let mut data = chunk.data.clone();
            if data.len() >= 8 {
                let mut reader = Cursor::new(&data);
                let _lod = reader.read_i32::<LittleEndian>().unwrap_or(0);
                let part_count = reader.read_i32::<LittleEndian>().unwrap_or(0) as usize;
                for p_idx in 0..part_count {
                    let offset = 20 + p_idx * 18;
                    if offset + 2 <= data.len() {
                        let val = u16::from_le_bytes([data[offset], data[offset + 1]]);
                        if val == 1 {
                            data[offset] = 0xFF;
                            data[offset + 1] = 0xFF;
                        }
                    }
                }
            }
            chunk.data = data;
        }

        if !chunk.children.is_empty() {
            sanitize_prim_group_counts(&mut chunk.children)?;
        }
    }
    Ok(())
}

fn texture_name_key(name: &str) -> String {
    name.to_lowercase()
        .trim_end_matches(".tga")
        .trim_end_matches(".png")
        .trim_end_matches(".dds")
        .trim_end_matches(".bmp")
        .trim_end_matches(".jpg")
        .trim_end_matches(".jpeg")
        .trim()
        .to_string()
}

fn decode_texture_png_rgba(texture: &HODTexture) -> Result<Option<(Vec<u8>, u32, u32)>, String> {
    use base64::{engine::general_purpose, Engine as _};

    let Some(encoded) = texture.png_data.as_ref().or(texture.png_preview.as_ref()) else {
        return Ok(None);
    };
    let payload = encoded
        .split_once(',')
        .map(|(_, data)| data)
        .unwrap_or(encoded.as_str());
    let bytes = general_purpose::STANDARD
        .decode(payload)
        .map_err(|e| format!("Failed to decode texture '{}': {}", texture.name, e))?;
    let img = image::load_from_memory(&bytes)
        .map_err(|e| format!("Failed to decode texture image '{}': {}", texture.name, e))?
        .to_rgba8();
    let (width, height) = img.dimensions();

    Ok(Some((img.into_raw(), width, height)))
}

fn flip_rgba_vertical_in_place(rgba: &mut [u8], width: u32, height: u32) {
    let row_len = width as usize * 4;
    if row_len == 0 || rgba.len() < row_len * height as usize {
        return;
    }
    for y in 0..(height as usize / 2) {
        let top = y * row_len;
        let bottom = (height as usize - 1 - y) * row_len;
        for x in 0..row_len {
            rgba.swap(top + x, bottom + x);
        }
    }
}

fn generate_lmip_texture_chunks_and_pool(
    textures: &[HODTexture],
) -> Result<(Vec<IffChunk>, Vec<u8>), String> {
    let mut chunks = Vec::new();
    let mut texture_pool = Vec::new();

    for texture in textures {
        let (mut rgba, width, height) = match decode_texture_png_rgba(texture) {
            Ok(Some((rgba, w, h))) if w > 0 && h > 0 => (rgba, w, h),
            _ => {
                println!("[TEX-DEBUG-SAVE] WARNING: Texture '{}' failed to decode or was empty. Injecting 4x4 magenta fallback to prevent material index desync.", texture.name);
                let w = 4;
                let h = 4;
                let mut fallback = vec![0u8; w * h * 4];
                for i in 0..(w * h) {
                    fallback[i * 4] = 255; // R
                    fallback[i * 4 + 1] = 0; // G
                    fallback[i * 4 + 2] = 255; // B
                    fallback[i * 4 + 3] = 255; // A
                }
                (fallback, w as u32, h as u32)
            }
        };
        println!(
            "[TEX-DEBUG-SAVE] Writing texture '{}' ({}x{}, format={})",
            texture.name, width, height, texture.format
        );

        // HOD 2.0 POOL chunks store textures bottom-up (DirectX convention).
        // Our internal format (from TGA/PNG or extracted) is always top-down.
        // So we ALWAYS flip it before compression.
        flip_rgba_vertical_in_place(&mut rgba, width, height);

        let mut mip_count = 0usize;
        let mut mip_width = width;
        let mut mip_height = height;
        while mip_width >= 8 && mip_height >= 8 {
            mip_count += 1;
            mip_width = std::cmp::max(1, mip_width / 2);
            mip_height = std::cmp::max(1, mip_height / 2);
        }
        if mip_count == 0 {
            mip_count = 1;
        }
        println!("[TEX-DEBUG-SAVE] Generated {} mip levels", mip_count);

        let mips = generate_mip_chain(&rgba, width as usize, height as usize, mip_count as usize);

        let output_format = match texture.format.as_str() {
            "DXT1" => "DXT1",
            "DXT3" => "DXT3",
            "DXT5" => "DXT5",
            _ => {
                let has_alpha = rgba.chunks_exact(4).any(|pixel| pixel[3] < 250);
                if has_alpha {
                    "DXT5"
                } else {
                    "DXT1"
                }
            }
        };

        let mut data = Vec::new();
        // V2 format: string name, 4-byte format name, u32 mip_count, and then mip dimensions (width, height as u32 LittleEndian)
        write_len_string(&mut data, &texture.name)?;

        let format_name = output_format; // "DXT1", "DXT3", or "DXT5"
        let mut format_bytes = [b' '; 4];
        let bytes = format_name.as_bytes();
        let copy_len = bytes.len().min(4);
        format_bytes[..copy_len].copy_from_slice(&bytes[..copy_len]);
        data.write_all(&format_bytes).map_err(|e| e.to_string())?;

        data.write_u32::<LittleEndian>(mip_count as u32)
            .map_err(|e| e.to_string())?;

        for &(_, mw, mh) in &mips {
            data.write_u32::<LittleEndian>(mw as u32)
                .map_err(|e| e.to_string())?;
            data.write_u32::<LittleEndian>(mh as u32)
                .map_err(|e| e.to_string())?;
        }

        for (mip_rgba, mw, mh) in &mips {
            let dxt = match output_format {
                "DXT3" => compress_dxt3(mip_rgba, *mw, *mh),
                "DXT5" => compress_dxt5(mip_rgba, *mw, *mh),
                _ => compress_dxt1(mip_rgba, *mw, *mh),
            };
            texture_pool.extend_from_slice(&dxt);
        }

        chunks.push(IffChunk {
            id: "LMIP".to_string(),
            chunk_type: crate::iff::ChunkType::Default,
            version: 0,
            data,
            children: Vec::new(),
        });
    }

    Ok((chunks, texture_pool))
}

fn shader_texture_param_name(shader_name: &str, texture_name: &str, slot_index: usize) -> String {
    let tex_key = texture_name_key(texture_name);
    let shader_key = shader_name.to_lowercase();

    if tex_key.contains("diff_off") || tex_key.contains("diffoff") {
        return "$diffuseoff".to_string();
    }
    if tex_key.contains("glow_off") || tex_key.contains("glowoff") {
        return "$glowoff".to_string();
    }
    if tex_key.contains("diff") || tex_key.contains("albedo") || tex_key.contains("base") {
        return "$diffuse".to_string();
    }
    if tex_key.contains("glow") || tex_key.contains("emit") {
        return "$glow".to_string();
    }
    if tex_key.contains("team") || tex_key.contains("stripe") {
        return "$team".to_string();
    }
    if tex_key.contains("norm") || tex_key.contains("normal") || tex_key.contains("bump") {
        return "$normal".to_string();
    }
    if tex_key.contains("spec") || tex_key.contains("rough") || tex_key.contains("metal") {
        return "$specular".to_string();
    }

    let ship_slots = ["$diffuse", "$glow", "$team", "$normal", "$specular"];
    let thruster_slots = [
        "$diffuse",
        "$glow",
        "$team",
        "$normal",
        "$diffuseoff",
        "$glowoff",
    ];
    let default_slots = ["$diffuse", "$glow", "$team", "$normal"];

    let slots = if shader_key.contains("thruster") {
        &thruster_slots[..]
    } else if shader_key.contains("ship") || shader_key.contains("bay") {
        &ship_slots[..]
    } else {
        &default_slots[..]
    };

    slots.get(slot_index).unwrap_or(&"$diffuse").to_string()
}

fn write_stat_texture_params<W: Write>(
    writer: &mut W,
    mat: &HODMaterial,
    textures: &[HODTexture],
) -> Result<(), String> {
    let mut texture_indices = Vec::new();
    for (slot_index, tex_name) in mat.texture_maps.iter().enumerate() {
        let tex_key = texture_name_key(tex_name);
        if let Some(index) = textures
            .iter()
            .position(|t| texture_name_key(&t.name) == tex_key)
        {
            let param_name = shader_texture_param_name(&mat.shader_name, tex_name, slot_index);
            texture_indices.push((index as u32, param_name));
        }
    }

    writer
        .write_u32::<LittleEndian>(texture_indices.len() as u32)
        .map_err(|e| e.to_string())?;
    if texture_indices.is_empty() {
        return Ok(());
    }

    writer
        .write_u32::<LittleEndian>(5)
        .map_err(|e| e.to_string())?;
    writer
        .write_u32::<LittleEndian>(4)
        .map_err(|e| e.to_string())?;
    for (idx, (texture_index, param_name)) in texture_indices.iter().enumerate() {
        if idx > 0 {
            writer
                .write_u32::<LittleEndian>(5)
                .map_err(|e| e.to_string())?;
            writer
                .write_u32::<LittleEndian>(4)
                .map_err(|e| e.to_string())?;
        }
        writer
            .write_u32::<LittleEndian>(*texture_index)
            .map_err(|e| e.to_string())?;
        write_len_string(writer, param_name)?;
    }

    if !mat.parameters.is_empty() {
        writer
            .write_all(&mat.parameters)
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn original_needs_full_v2_regeneration(original_bytes: &[u8], updated_model: &HODModel) -> bool {
    if original_bytes.is_empty() {
        return true;
    }
    if !updated_model.is_v2 {
        return false;
    }

    let mut cursor = Cursor::new(original_bytes);
    let mut has_pool = false;
    let mut has_lmip = false;
    while cursor.position() < original_bytes.len() as u64 {
        let Ok(chunk) = IffChunk::read_chunk(&mut cursor) else {
            return false;
        };
        if chunk.id == "HVMD" {
            has_lmip = chunk.children.iter().any(|child| child.id == "LMIP");
        }
        if chunk.id == "POOL" {
            has_pool = true;
            let mut pool_cursor = Cursor::new(&chunk.data);
            let Ok(_pool_type) = pool_cursor.read_u32::<LittleEndian>() else {
                return true;
            };
            let Ok(comp_tex_len) = pool_cursor.read_u32::<LittleEndian>() else {
                return true;
            };
            let Ok(decomp_tex_len) = pool_cursor.read_u32::<LittleEndian>() else {
                return true;
            };
            if !updated_model.textures.is_empty() && (comp_tex_len == 0 || decomp_tex_len == 0) {
                return true;
            }
            let skip_tex_to = pool_cursor.position().saturating_add(comp_tex_len as u64);
            if skip_tex_to > chunk.data.len() as u64 {
                return true;
            }
            pool_cursor.set_position(skip_tex_to);
            let Ok(comp_mesh_len) = pool_cursor.read_u32::<LittleEndian>() else {
                return true;
            };
            let Ok(decomp_mesh_len) = pool_cursor.read_u32::<LittleEndian>() else {
                return true;
            };
            if !updated_model.meshes.is_empty() && (comp_mesh_len == 0 || decomp_mesh_len == 0) {
                return true;
            }
            break;
        }
    }

    if !updated_model.textures.is_empty() && !has_lmip {
        return true;
    }

    !has_pool
}

pub fn generate_v2_from_model(original_bytes: &[u8], model: &HODModel) -> Result<Vec<u8>, String> {
    use crate::iff::{ChunkType, IffChunk};
    use crate::xpress;
    let mut model = model.clone();
    generate_collision_mesh(&mut model);
    let mut compiled = crate::compiler::compile_model_meshes(&mut model);

    // Sort meshes by LOD within each base name group so POOL and BMSH ordering
    // matches HODOR's expected sequential LOD layout.
    compiled.sort_by(|a, b| {
        let base_a = a.name.split("_LOD").next().unwrap_or(&a.name);
        let base_b = b.name.split("_LOD").next().unwrap_or(&b.name);
        base_a.cmp(base_b).then(a.lod.cmp(&b.lod))
    });

    let mut comp_tex_buf = Vec::new();
    let mut decomp_tex_len_val = 0;
    let mut extracted_pool_type = if model.is_v2 { 3518 } else { 0 };

    // Try to extract original compressed texture data from the original POOL
    let mut original_tex_preserved = false;
    let mut original_lmip_chunks: Vec<IffChunk> = Vec::new();
    let mut original_stat_chunks: Vec<IffChunk> = Vec::new();
    let mut original_comp_mesh_buf = Vec::new();
    let mut original_decomp_mesh_len_val = 0u32;
    let mut original_comp_face_buf = Vec::new();
    let mut original_decomp_face_len_val = 0u32;
    let mut original_mesh_pool_preserved = false;
    if !original_bytes.is_empty() {
        let mut cursor = Cursor::new(original_bytes);
        while cursor.position() < original_bytes.len() as u64 {
            if let Ok(chunk) = IffChunk::read_chunk(&mut cursor) {
                if chunk.id == "POOL" && !chunk.data.is_empty() && !original_tex_preserved {
                    let mut pool_cursor = Cursor::new(&chunk.data);
                    if let Ok(pool_type) = pool_cursor.read_u32::<LittleEndian>() {
                        extracted_pool_type = pool_type;
                        if let Ok(comp_tex_len) = pool_cursor.read_u32::<LittleEndian>() {
                            if let Ok(decomp_tex_len) = pool_cursor.read_u32::<LittleEndian>() {
                                let mut comp_tex = vec![0u8; comp_tex_len as usize];
                                if pool_cursor.read_exact(&mut comp_tex).is_ok()
                                    && !comp_tex.is_empty()
                                {
                                    comp_tex_buf = comp_tex;
                                    decomp_tex_len_val = decomp_tex_len;
                                    original_tex_preserved = true;
                                }
                                // Also extract mesh and face pool data
                                if let Ok(cm_len) = pool_cursor.read_u32::<LittleEndian>() {
                                    if let Ok(dm_len) = pool_cursor.read_u32::<LittleEndian>() {
                                        let mut comp_mesh = vec![0u8; cm_len as usize];
                                        if pool_cursor.read_exact(&mut comp_mesh).is_ok() {
                                            original_comp_mesh_buf = comp_mesh;
                                            original_decomp_mesh_len_val = dm_len;
                                        }
                                        if let Ok(cf_len) = pool_cursor.read_u32::<LittleEndian>() {
                                            if let Ok(df_len) =
                                                pool_cursor.read_u32::<LittleEndian>()
                                            {
                                                let mut comp_face = vec![0u8; cf_len as usize];
                                                if pool_cursor.read_exact(&mut comp_face).is_ok() {
                                                    original_comp_face_buf = comp_face;
                                                    original_decomp_face_len_val = df_len;
                                                    original_mesh_pool_preserved = true;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else if chunk.id == "HVMD" {
                    // Extract original LMIP/TEXM and STAT/MATT chunks from HVMD for preservation
                    for child in &chunk.children {
                        if child.id == "LMIP" || child.id == "TEXM" {
                            let mut child_clone = child.clone();
                            child_clone.id = "LMIP".to_string();
                            original_lmip_chunks.push(child_clone);
                        }
                    }
                }
            } else {
                break;
            }
        }
    }

    // Fall back to regenerating textures from model data if original wasn't available
    // OR if the user explicitly modified textures (added, deleted, or toggled properties).
    if model.textures_modified {
        original_tex_preserved = false;
        // Strip out any original LMIP chunks since we are regenerating them
        original_lmip_chunks.clear();
    }

    let mut generated_texture_chunks = Vec::new();
    if !original_tex_preserved {
        let (gen_chunks, generated_texture_pool) =
            generate_lmip_texture_chunks_and_pool(&model.textures)?;
        generated_texture_chunks = gen_chunks;
        if !generated_texture_pool.is_empty() {
            comp_tex_buf = xpress::compress_or_raw(&generated_texture_pool);
            decomp_tex_len_val = generated_texture_pool.len() as u32;
        }
    }
    if model.is_v2 {
        extracted_pool_type = 3518;
    }

    let mut pool_data = crate::compiler::generate_pool_data(
        &compiled,
        &comp_tex_buf,
        decomp_tex_len_val,
        extracted_pool_type,
    )
    .map_err(|e| e.to_string())?;

    let has_preserved_collision_chunks = model.preserved_chunks.iter().any(|chunk| {
        chunk.id == "KDOP"
            || chunk.id == "COLD"
            || (chunk.id == "DTRM"
                && chunk
                    .children
                    .iter()
                    .any(|child| child.id == "KDOP" || child.id == "COLD"))
    });

    // Append collision mesh vertices/indices to pool data only for regenerated collision data.
    // Parsed HOD 2.0 KDOP/COLD chunks are preserved raw and do not reference POOL offsets.
    if !model.collision_meshes.is_empty() && !has_preserved_collision_chunks {
        // The pool_data already contains tex/mesh/face streams.
        // Collision mesh vertices go into the mesh stream, indices into the face stream.
        // We need to re-generate the pool with collision mesh data appended.
        let mut pool_cursor = Cursor::new(&mut pool_data);
        let _pool_type = pool_cursor
            .read_u32::<LittleEndian>()
            .map_err(|e| e.to_string())?;

        // Read existing texture stream
        let comp_tex_len = pool_cursor
            .read_u32::<LittleEndian>()
            .map_err(|e| e.to_string())? as usize;
        let decomp_tex_len = pool_cursor
            .read_u32::<LittleEndian>()
            .map_err(|e| e.to_string())? as usize;
        let mut comp_tex = vec![0u8; comp_tex_len];
        pool_cursor
            .read_exact(&mut comp_tex)
            .map_err(|e| e.to_string())?;

        // Read existing mesh stream
        let comp_mesh_len = pool_cursor
            .read_u32::<LittleEndian>()
            .map_err(|e| e.to_string())? as usize;
        let decomp_mesh_len = pool_cursor
            .read_u32::<LittleEndian>()
            .map_err(|e| e.to_string())? as usize;
        let mut comp_mesh = vec![0u8; comp_mesh_len];
        pool_cursor
            .read_exact(&mut comp_mesh)
            .map_err(|e| e.to_string())?;
        let mut decomp_mesh = if comp_mesh_len == decomp_mesh_len {
            comp_mesh.clone()
        } else {
            xpress::decompress(&comp_mesh, decomp_mesh_len)?
        };

        // Read existing face stream
        let comp_face_len = pool_cursor
            .read_u32::<LittleEndian>()
            .map_err(|e| e.to_string())? as usize;
        let decomp_face_len = pool_cursor
            .read_u32::<LittleEndian>()
            .map_err(|e| e.to_string())? as usize;
        let mut comp_face = vec![0u8; comp_face_len];
        pool_cursor
            .read_exact(&mut comp_face)
            .map_err(|e| e.to_string())?;
        let mut decomp_face = if comp_face_len == decomp_face_len {
            comp_face.clone()
        } else {
            xpress::decompress(&comp_face, decomp_face_len)?
        };

        println!("[RUST] Collision mesh pool append: {} collision meshes, decomp_mesh={} bytes, decomp_face={} bytes", model.collision_meshes.len(), decomp_mesh.len(), decomp_face.len());

        // Append collision mesh vertices to mesh stream
        for cm in &model.collision_meshes {
            for part in &cm.mesh.parts {
                println!(
                    "[RUST]   Collision part: {} vertices, {} indices, mask=0x{:x}",
                    part.vertices.len(),
                    part.indices.len(),
                    part.vertex_mask
                );
                let mut vertex_stride = 0u32;
                if (part.vertex_mask & 0x01) != 0 {
                    vertex_stride += 16;
                }
                if (part.vertex_mask & 0x02) != 0 {
                    vertex_stride += 16;
                }
                if (part.vertex_mask & 0x04) != 0 {
                    vertex_stride += 4;
                }
                if (part.vertex_mask & 0x08) != 0 {
                    vertex_stride += 8;
                }
                if (part.vertex_mask & 0x2000) != 0 {
                    vertex_stride += 12;
                }
                if (part.vertex_mask & 0x4000) != 0 {
                    vertex_stride += 12;
                }
                if vertex_stride == 0 {
                    vertex_stride = 1;
                }

                for v in &part.vertices {
                    let _ = crate::hod::write_vertex(
                        &mut decomp_mesh,
                        v,
                        part.vertex_mask,
                        1400,
                        vertex_stride,
                    );
                }
                // Align face pool to 2 bytes
                if decomp_face.len() % 2 != 0 {
                    decomp_face.push(0);
                }
                for &idx in &part.indices {
                    let _ = decomp_face.write_u16::<LittleEndian>(idx);
                }
            }
        }

        // Raw (uncompressed) mesh and face streams
        let new_comp_mesh = decomp_mesh.clone();
        let new_comp_face = decomp_face.clone();

        println!("[RUST] After collision append: decomp_mesh={} bytes, decomp_face={} bytes, new_comp_mesh={} bytes, new_comp_face={} bytes", decomp_mesh.len(), decomp_face.len(), new_comp_mesh.len(), new_comp_face.len());

        let mut new_pool = Vec::new();
        new_pool
            .write_u32::<LittleEndian>(_pool_type)
            .map_err(|e| e.to_string())?;
        new_pool
            .write_u32::<LittleEndian>(comp_tex.len() as u32)
            .map_err(|e| e.to_string())?;
        new_pool
            .write_u32::<LittleEndian>(decomp_tex_len as u32)
            .map_err(|e| e.to_string())?;
        new_pool.extend_from_slice(&comp_tex);
        new_pool
            .write_u32::<LittleEndian>(new_comp_mesh.len() as u32)
            .map_err(|e| e.to_string())?;
        new_pool
            .write_u32::<LittleEndian>(decomp_mesh.len() as u32)
            .map_err(|e| e.to_string())?;
        new_pool.extend_from_slice(&new_comp_mesh);
        new_pool
            .write_u32::<LittleEndian>(new_comp_face.len() as u32)
            .map_err(|e| e.to_string())?;
        new_pool
            .write_u32::<LittleEndian>(decomp_face.len() as u32)
            .map_err(|e| e.to_string())?;
        new_pool.extend_from_slice(&new_comp_face);

        pool_data = new_pool;
    }
    let mut hvmd_children = Vec::new();

    // Add LMIP chunks: preserve originals if texture pool was preserved, otherwise use generated
    if original_tex_preserved && !original_lmip_chunks.is_empty() {
        hvmd_children.extend(original_lmip_chunks);
    } else {
        hvmd_children.extend(generated_texture_chunks);
    }

    // Generate STAT chunks directly from materials so UI edits apply
    if !model.materials.is_empty() {
        for mat in &model.materials {
            let mut stat_buf = Vec::new();
            let mut stat_cursor = Cursor::new(&mut stat_buf);

            write_len_string(&mut stat_cursor, &mat.name).unwrap();
            write_len_string(&mut stat_cursor, &mat.shader_name).unwrap();
            write_stat_texture_params(&mut stat_cursor, mat, &model.textures).unwrap();

            hvmd_children.push(IffChunk {
                id: "STAT".to_string(),
                chunk_type: ChunkType::Normal,
                version: 1001,
                data: stat_buf,
                children: Vec::new(),
            });
        }
    } else if model.materials.is_empty() {
        // Auto-generate a material from imported textures when no material exists
        let auto_mat = if !model.textures.is_empty() {
            let texture_maps: Vec<String> = model.textures.iter().map(|t| t.name.clone()).collect();
            HODMaterial {
                name: "default_mat".to_string(),
                shader_name: "ship".to_string(),
                texture_maps,
                parameters: Vec::new(),
            }
        } else {
            HODMaterial {
                name: "default_mat".to_string(),
                shader_name: "default_shader".to_string(),
                texture_maps: Vec::new(),
                parameters: Vec::new(),
            }
        };
        let mut stat_buf = Vec::new();
        let mut stat_cursor = Cursor::new(&mut stat_buf);
        write_len_string(&mut stat_cursor, &auto_mat.name).unwrap();
        write_len_string(&mut stat_cursor, &auto_mat.shader_name).unwrap();
        write_stat_texture_params(&mut stat_cursor, &auto_mat, &model.textures).unwrap();

        hvmd_children.push(IffChunk {
            id: "STAT".to_string(),
            chunk_type: ChunkType::Normal,
            version: 1001,
            data: stat_buf,
            children: Vec::new(),
        });
    }

    // Add MULT chunks
    let mult_chunks = crate::compiler::generate_mult_chunks(&compiled)
        .map_err(|e: std::io::Error| e.to_string())?;
    for mult in mult_chunks {
        hvmd_children.push(mult);
    }

    let hvmd_chunk = IffChunk {
        id: "HVMD".to_string(),
        chunk_type: ChunkType::Form,
        version: 0,
        data: Vec::new(),
        children: hvmd_children,
    };

    // Build DTRM children
    let mut dtrm_children = Vec::new();

    // Bug 1 fix: Compute first_val dynamically from joint count
    // Bug 2 fix: Use ChunkType::Form (FORM HIER), not ChunkType::Normal (NRML HIER)
    // Bug 3 fix: Use actual joint rotation/scale data, not hardcoded zeros/ones
    let mut hier_buf = Vec::new();
    let default_root_pos = Vector3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let default_root_rot = Vector3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let default_root_scale = Vector3 {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    };
    let default_root_joint = HODJoint {
        name: "Root".to_string(),
        parent_name: None,
        local_transform: compose_transform_matrix(
            default_root_pos.clone(),
            default_root_rot.clone(),
            default_root_scale.clone(),
        ),
        position: Some(default_root_pos),
        rotation: Some(default_root_rot),
        scale: Some(default_root_scale),
    };
    let joints_to_write: Vec<&HODJoint> = if model.joints.is_empty() {
        vec![&default_root_joint]
    } else {
        model.joints.iter().collect()
    };
    {
        let joint_count = joints_to_write.len() as i32;
        // HOD 2.0 HIER uses a full two's-complement negative joint count.
        // Do not mask to the low byte; files can legitimately exceed 255 joints.
        let first_val = (-joint_count) as u32;
        hier_buf
            .write_u32::<LittleEndian>(first_val)
            .map_err(|e| e.to_string())?;
    }
    for joint in &joints_to_write {
        write_len_string(&mut hier_buf, &joint.name)?;
        let parent_str = joint.parent_name.clone().unwrap_or_default();
        write_len_string(&mut hier_buf, &parent_str)?;

        let (pos, rot, scale) = if let (Some(ref p), Some(ref r), Some(ref s)) =
            (&joint.position, &joint.rotation, &joint.scale)
        {
            (p.clone(), r.clone(), s.clone())
        } else {
            let (p, r, s) = decompose_matrix(joint.local_transform.clone());
            (p, r, s)
        };

        hier_buf
            .write_f32::<LittleEndian>(pos.x)
            .map_err(|e| e.to_string())?;
        hier_buf
            .write_f32::<LittleEndian>(pos.y)
            .map_err(|e| e.to_string())?;
        hier_buf
            .write_f32::<LittleEndian>(pos.z)
            .map_err(|e| e.to_string())?;

        hier_buf
            .write_f32::<LittleEndian>(rot.x)
            .map_err(|e| e.to_string())?;
        hier_buf
            .write_f32::<LittleEndian>(rot.y)
            .map_err(|e| e.to_string())?;
        hier_buf
            .write_f32::<LittleEndian>(rot.z)
            .map_err(|e| e.to_string())?;

        hier_buf
            .write_f32::<LittleEndian>(scale.x)
            .map_err(|e| e.to_string())?;
        hier_buf
            .write_f32::<LittleEndian>(scale.y)
            .map_err(|e| e.to_string())?;
        hier_buf
            .write_f32::<LittleEndian>(scale.z)
            .map_err(|e| e.to_string())?;
    }
    dtrm_children.push(IffChunk {
        id: "HIER".to_string(),
        chunk_type: ChunkType::Form,
        version: 0,
        data: hier_buf,
        children: Vec::new(),
    });

    if !model.markers.is_empty() {
        let mut mrkr_buf = Vec::new();
        mrkr_buf
            .write_u32::<LittleEndian>(model.markers.len() as u32)
            .map_err(|e| e.to_string())?;
        for mrkr in &model.markers {
            let m_bytes = serialize_single_marker(mrkr, true)?;
            mrkr_buf.write_all(&m_bytes).map_err(|e| e.to_string())?;
        }
        dtrm_children.push(IffChunk {
            id: "MRKS".to_string(),
            chunk_type: ChunkType::Default,
            version: 0,
            data: mrkr_buf,
            children: Vec::new(),
        });
    }

    // Bug 4 fix: Write individual BURN chunks with ChunkType::Default, not one consolidated NRML BURN
    for burn in &model.engine_burns {
        let mut data = Vec::new();
        write_len_string(&mut data, &burn.name)?;
        write_len_string(&mut data, &burn.parent_name)?;
        data.write_i32::<LittleEndian>(burn.num_divisions)
            .map_err(|e| e.to_string())?;
        data.write_i32::<LittleEndian>(burn.num_flames)
            .map_err(|e| e.to_string())?;
        for v in &burn.vertices {
            data.write_f32::<LittleEndian>(v.x)
                .map_err(|e| e.to_string())?;
            data.write_f32::<LittleEndian>(v.y)
                .map_err(|e| e.to_string())?;
            data.write_f32::<LittleEndian>(v.z)
                .map_err(|e| e.to_string())?;
        }
        dtrm_children.push(IffChunk {
            id: "BURN".to_string(),
            chunk_type: ChunkType::Default,
            version: 0,
            data,
            children: Vec::new(),
        });
    }

    // Bug 5 fix: Regenerate NAVL from model data instead of preserving stale original
    if !model.nav_lights.is_empty() {
        let mut navl_data = Vec::new();
        navl_data
            .write_u32::<LittleEndian>(model.nav_lights.len() as u32)
            .map_err(|e| e.to_string())?;
        for nav in &model.nav_lights {
            write_len_string(&mut navl_data, &nav.name)?;
            navl_data
                .write_u32::<LittleEndian>(nav.section)
                .map_err(|e| e.to_string())?;
            navl_data
                .write_f32::<LittleEndian>(nav.size)
                .map_err(|e| e.to_string())?;
            navl_data
                .write_f32::<LittleEndian>(nav.phase)
                .map_err(|e| e.to_string())?;
            navl_data
                .write_f32::<LittleEndian>(nav.frequency)
                .map_err(|e| e.to_string())?;
            write_len_string(&mut navl_data, &nav.style)?;
            navl_data
                .write_f32::<LittleEndian>(nav.color.x)
                .map_err(|e| e.to_string())?;
            navl_data
                .write_f32::<LittleEndian>(nav.color.y)
                .map_err(|e| e.to_string())?;
            navl_data
                .write_f32::<LittleEndian>(nav.color.z)
                .map_err(|e| e.to_string())?;
            navl_data
                .write_f32::<LittleEndian>(1.0)
                .map_err(|e| e.to_string())?; // _unused f32
            navl_data
                .write_f32::<LittleEndian>(nav.distance)
                .map_err(|e| e.to_string())?;
            navl_data
                .write_u8(if nav.sprite_visible { 1 } else { 0 })
                .map_err(|e| e.to_string())?;
            navl_data
                .write_u8(if nav.high_end_only { 1 } else { 0 })
                .map_err(|e| e.to_string())?;
        }
        dtrm_children.push(IffChunk {
            id: "NAVL".to_string(),
            chunk_type: ChunkType::Normal,
            version: 3,
            data: navl_data,
            children: Vec::new(),
        });
    }

    if !model.dockpaths.is_empty() {
        let mut dock_data = Vec::new();
        dock_data
            .write_u32::<LittleEndian>(model.dockpaths.len() as u32)
            .map_err(|e| e.to_string())?;
        for path in &model.dockpaths {
            write_len_string(&mut dock_data, &path.name)?;
            write_len_string(&mut dock_data, &path.parent_name)?;
            dock_data
                .write_u32::<LittleEndian>(path.val1)
                .map_err(|e| e.to_string())?;
            dock_data
                .write_u32::<LittleEndian>(path.val2)
                .map_err(|e| e.to_string())?;
            dock_data
                .write_u32::<LittleEndian>(path.val3)
                .map_err(|e| e.to_string())?;
            dock_data
                .write_u32::<LittleEndian>(path.val4)
                .map_err(|e| e.to_string())?;
            dock_data
                .write_u32::<LittleEndian>(path.val5)
                .map_err(|e| e.to_string())?;
            write_len_string(&mut dock_data, &path.compatible_ships)?;
            dock_data
                .write_u32::<LittleEndian>(path.padding1)
                .map_err(|e| e.to_string())?;
            write_len_string(&mut dock_data, &path.padding2)?;
            dock_data
                .write_i32::<LittleEndian>(path.points.len() as i32)
                .map_err(|e| e.to_string())?;

            for pt in &path.points {
                dock_data
                    .write_f32::<LittleEndian>(pt.position.x)
                    .map_err(|e| e.to_string())?;
                dock_data
                    .write_f32::<LittleEndian>(pt.position.y)
                    .map_err(|e| e.to_string())?;
                dock_data
                    .write_f32::<LittleEndian>(pt.position.z)
                    .map_err(|e| e.to_string())?;

                dock_data
                    .write_f32::<LittleEndian>(pt.rotation.m[0][0])
                    .map_err(|e| e.to_string())?;
                dock_data
                    .write_f32::<LittleEndian>(pt.rotation.m[0][1])
                    .map_err(|e| e.to_string())?;
                dock_data
                    .write_f32::<LittleEndian>(pt.rotation.m[0][2])
                    .map_err(|e| e.to_string())?;

                dock_data
                    .write_f32::<LittleEndian>(pt.rotation.m[1][0])
                    .map_err(|e| e.to_string())?;
                dock_data
                    .write_f32::<LittleEndian>(pt.rotation.m[1][1])
                    .map_err(|e| e.to_string())?;
                dock_data
                    .write_f32::<LittleEndian>(pt.rotation.m[1][2])
                    .map_err(|e| e.to_string())?;

                dock_data
                    .write_f32::<LittleEndian>(pt.rotation.m[2][0])
                    .map_err(|e| e.to_string())?;
                dock_data
                    .write_f32::<LittleEndian>(pt.rotation.m[2][1])
                    .map_err(|e| e.to_string())?;
                dock_data
                    .write_f32::<LittleEndian>(pt.rotation.m[2][2])
                    .map_err(|e| e.to_string())?;

                dock_data
                    .write_f32::<LittleEndian>(pt.tolerance)
                    .map_err(|e| e.to_string())?;
                dock_data
                    .write_f32::<LittleEndian>(pt.max_speed)
                    .map_err(|e| e.to_string())?;
                dock_data
                    .write_u32::<LittleEndian>(pt.extra1)
                    .map_err(|e| e.to_string())?;
                dock_data
                    .write_u32::<LittleEndian>(pt.extra2)
                    .map_err(|e| e.to_string())?;
            }
        }
        dtrm_children.push(IffChunk {
            id: "DOCK".to_string(),
            chunk_type: ChunkType::Default,
            version: 0,
            data: dock_data,
            children: Vec::new(),
        });
    }

    let dtrm_sub_chunk_ids = [
        "HIER", "MRKR", "BURN", "NAVL", "MRKS", "DOCK", "COLD", "KDOP", "SCAR", "BNDV", "ETSH",
        "BSRM", "PATH", "PNTS", "MAD",
    ];
    if let Some(dtrm) = model.preserved_chunks.iter().find(|c| c.id == "DTRM") {
        for child in &dtrm.children {
            if dtrm_sub_chunk_ids.contains(&child.id.as_str()) {
                if child.id == "COLD" || child.id == "KDOP" {
                    // Do not preserve old collision chunks; they will be regenerated
                    continue;
                }
                dtrm_children.push(child.clone());
            }
        }
    } else {
        // Handle case where DTRM was not preserved but its children were top-level
        for chunk in &model.preserved_chunks {
            if ["SCAR", "BNDV", "ETSH"].contains(&chunk.id.as_str()) {
                dtrm_children.push(chunk.clone());
            }
        }
    }

    // Generate COLD chunk from collision mesh data if not preserved
    if !model.collision_meshes.is_empty() {
        for cm in &model.collision_meshes {
            let mut cold_children = Vec::new();

            // BBOX
            let mut bbox_data = Vec::new();
            bbox_data
                .write_f32::<LittleEndian>(cm.min_extents.x)
                .map_err(|e| e.to_string())?;
            bbox_data
                .write_f32::<LittleEndian>(cm.min_extents.y)
                .map_err(|e| e.to_string())?;
            bbox_data
                .write_f32::<LittleEndian>(cm.min_extents.z)
                .map_err(|e| e.to_string())?;
            bbox_data
                .write_f32::<LittleEndian>(cm.max_extents.x)
                .map_err(|e| e.to_string())?;
            bbox_data
                .write_f32::<LittleEndian>(cm.max_extents.y)
                .map_err(|e| e.to_string())?;
            bbox_data
                .write_f32::<LittleEndian>(cm.max_extents.z)
                .map_err(|e| e.to_string())?;
            cold_children.push(IffChunk {
                id: "BBOX".to_string(),
                chunk_type: ChunkType::Default,
                version: 0,
                data: bbox_data,
                children: Vec::new(),
            });

            // BSPH
            let mut bsph_data = Vec::new();
            bsph_data
                .write_f32::<LittleEndian>(cm.center.x)
                .map_err(|e| e.to_string())?;
            bsph_data
                .write_f32::<LittleEndian>(cm.center.y)
                .map_err(|e| e.to_string())?;
            bsph_data
                .write_f32::<LittleEndian>(cm.center.z)
                .map_err(|e| e.to_string())?;
            bsph_data
                .write_f32::<LittleEndian>(cm.radius)
                .map_err(|e| e.to_string())?;
            cold_children.push(IffChunk {
                id: "BSPH".to_string(),
                chunk_type: ChunkType::Default,
                version: 0,
                data: bsph_data,
                children: Vec::new(),
            });

            // TRIS (vertex data is in mesh pool, just write vertex/index counts)
            if let Some(part) = cm.mesh.parts.first() {
                let mut tris_data = Vec::new();
                tris_data
                    .write_i32::<LittleEndian>(part.vertices.len() as i32)
                    .map_err(|e| e.to_string())?;
                for v in &part.vertices {
                    tris_data
                        .write_f32::<LittleEndian>(v.position.x)
                        .map_err(|e| e.to_string())?;
                    tris_data
                        .write_f32::<LittleEndian>(v.position.y)
                        .map_err(|e| e.to_string())?;
                    tris_data
                        .write_f32::<LittleEndian>(v.position.z)
                        .map_err(|e| e.to_string())?;
                }
                tris_data
                    .write_i32::<LittleEndian>(part.indices.len() as i32)
                    .map_err(|e| e.to_string())?;
                for &idx in &part.indices {
                    tris_data
                        .write_u16::<LittleEndian>(idx)
                        .map_err(|e| e.to_string())?;
                }
                cold_children.push(IffChunk {
                    id: "FORM".to_string(),
                    chunk_type: ChunkType::Form,
                    version: 0,
                    data: Vec::new(),
                    children: vec![IffChunk {
                        id: "TRIS".to_string(),
                        chunk_type: ChunkType::Default,
                        version: 0,
                        data: tris_data,
                        children: Vec::new(),
                    }],
                });
            }

            let mut cold_data = Vec::new();
            write_len_string(&mut cold_data, &cm.name).map_err(|e| e.to_string())?;
            cold_data
                .write_f32::<LittleEndian>(cm.min_extents.x)
                .map_err(|e| e.to_string())?;
            cold_data
                .write_f32::<LittleEndian>(cm.min_extents.y)
                .map_err(|e| e.to_string())?;
            cold_data
                .write_f32::<LittleEndian>(cm.min_extents.z)
                .map_err(|e| e.to_string())?;
            cold_data
                .write_f32::<LittleEndian>(cm.max_extents.x)
                .map_err(|e| e.to_string())?;
            cold_data
                .write_f32::<LittleEndian>(cm.max_extents.y)
                .map_err(|e| e.to_string())?;
            cold_data
                .write_f32::<LittleEndian>(cm.max_extents.z)
                .map_err(|e| e.to_string())?;
            cold_data
                .write_f32::<LittleEndian>(cm.center.x)
                .map_err(|e| e.to_string())?;
            cold_data
                .write_f32::<LittleEndian>(cm.center.y)
                .map_err(|e| e.to_string())?;
            cold_data
                .write_f32::<LittleEndian>(cm.center.z)
                .map_err(|e| e.to_string())?;
            cold_data
                .write_f32::<LittleEndian>(cm.radius)
                .map_err(|e| e.to_string())?;

            dtrm_children.push(IffChunk {
                id: "COLD".to_string(),
                chunk_type: ChunkType::Form,
                version: 0,
                data: cold_data,
                children: cold_children,
            });
        }
    }

    if !model.collision_meshes.is_empty() {
        for cm in &model.collision_meshes {
            if let Some(part) = cm.mesh.parts.first() {
                let verts: Vec<[f32; 3]> = part
                    .vertices
                    .iter()
                    .map(|v| [v.position.x, v.position.y, v.position.z])
                    .collect();
                let kdop_data = crate::kdop::generate_kdop(&verts, &part.indices);
                dtrm_children.push(IffChunk {
                    id: "KDOP".to_string(),
                    chunk_type: ChunkType::Default,
                    version: 0,
                    data: kdop_data,
                    children: Vec::new(),
                });
            }
        }
    }

    let dtrm_chunk = IffChunk {
        id: "DTRM".to_string(),
        chunk_type: ChunkType::Form,
        version: 0,
        data: Vec::new(),
        children: dtrm_children,
    };

    // Assemble top-level chunks
    let mut top_chunks = Vec::new();

    // VERS
    let mut vers_data = Vec::new();
    vers_data.write_u32::<LittleEndian>(model.version).unwrap(); // Use LittleEndian to match original HOD 2.0 files
    top_chunks.push(IffChunk {
        id: "VERS".to_string(),
        chunk_type: ChunkType::Form,
        version: 0,
        data: vers_data,
        children: Vec::new(),
    });

    // HOD 2.0 files use this fixed NAME payload; do not serialize the UI model name here.
    let name_bytes = b"Homeworld2 Multi Mesh File".to_vec();
    top_chunks.push(IffChunk {
        id: "NAME".to_string(),
        chunk_type: ChunkType::Form,
        version: 0,
        data: name_bytes,
        children: Vec::new(),
    });

    // POOL
    top_chunks.push(IffChunk {
        id: "POOL".to_string(),
        chunk_type: ChunkType::Default,
        version: 0,
        data: pool_data,
        children: Vec::new(),
    });

    // HVMD & DTRM
    top_chunks.push(hvmd_chunk);
    top_chunks.push(dtrm_chunk);

    // Add INFO chunk if not already present in preserved chunks
    let has_info = model.preserved_chunks.iter().any(|c| c.id == "INFO");
    if !has_info {
        let author = b"HODEditorJS";
        let mut ownr_data = Vec::new();
        ownr_data
            .write_u32::<LittleEndian>(author.len() as u32)
            .map_err(|e| e.to_string())?;
        ownr_data.extend_from_slice(author);
        ownr_data.push(0);
        let ownr_chunk = IffChunk {
            id: "OWNR".to_string(),
            chunk_type: ChunkType::Normal,
            version: 38,
            data: ownr_data,
            children: Vec::new(),
        };
        top_chunks.push(IffChunk {
            id: "INFO".to_string(),
            chunk_type: ChunkType::Form,
            version: 0,
            data: Vec::new(),
            children: vec![ownr_chunk],
        });
    }

    // Add preserved Root children (like INFO) — exclude DTRM sub-chunks and top-level chunks
    let top_level_ids = [
        "VERS", "NAME", "POOL", "HVMD", "DTRM", "KDOP", "COLD", "SCAR", "BNDV", "ETSH",
    ];
    for chunk in &model.preserved_chunks {
        if !top_level_ids.contains(&chunk.id.as_str()) {
            top_chunks.push(chunk.clone());
        }
    }

    let mut hod_buf = Vec::new();
    for chunk in &top_chunks {
        chunk.write_chunk(&mut hod_buf).map_err(|e| e.to_string())?;
    }

    Ok(hod_buf)
}

pub fn save_edits(original_bytes: &[u8], updated_model: &HODModel) -> Result<Vec<u8>, String> {
    // For HOD 2.0, always generate from scratch
    if updated_model.is_v2 {
        // Check if original has a POOL chunk (valid HOD 2.0 with pool data)
        let has_pool = if !original_bytes.is_empty() {
            let mut cursor = Cursor::new(original_bytes);
            let mut found = false;
            while cursor.position() < original_bytes.len() as u64 {
                match IffChunk::read_chunk(&mut cursor) {
                    Ok(chunk) => {
                        if chunk.id == "POOL" {
                            found = true;
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
            found
        } else {
            false
        };
        // If no POOL or empty original, generate from scratch
        if !has_pool || original_bytes.is_empty() {
            return generate_v2_from_model(&[], updated_model);
        }
    }
    if original_needs_full_v2_regeneration(original_bytes, updated_model) {
        return generate_v2_from_model(&[], updated_model);
    }

    let mut chunks = Vec::new();
    if original_bytes.is_empty() {
        let mut vers_data = Vec::new();
        vers_data
            .write_u32::<BigEndian>(updated_model.version)
            .map_err(|e| e.to_string())?;
        chunks.push(IffChunk {
            id: "VERS".to_string(),
            chunk_type: crate::iff::ChunkType::Form,
            version: 0,
            data: vers_data,
            children: Vec::new(),
        });

        chunks.push(IffChunk {
            id: "NAME".to_string(),
            chunk_type: crate::iff::ChunkType::Form,
            version: 0,
            data: b"Homeworld2 Multi Mesh File".to_vec(),
            children: Vec::new(),
        });

        chunks.push(IffChunk {
            id: "POOL".to_string(),
            chunk_type: crate::iff::ChunkType::Default,
            version: 0,
            data: Vec::new(),
            children: Vec::new(),
        });

        let mut hvmd_children = Vec::new();
        for mesh in &updated_model.meshes {
            let mut mult_data = Vec::new();
            write_len_string(&mut mult_data, &mesh.name)?;
            write_len_string(&mut mult_data, &mesh.parent_name)?;
            mult_data
                .write_u32::<LittleEndian>(1)
                .map_err(|e| e.to_string())?;

            let bmsh_chunk = IffChunk {
                id: "BMSH".to_string(),
                chunk_type: crate::iff::ChunkType::Normal,
                version: 0,
                data: Vec::new(),
                children: Vec::new(),
            };

            hvmd_children.push(IffChunk {
                id: "MULT".to_string(),
                chunk_type: crate::iff::ChunkType::Form,
                version: 0,
                data: mult_data,
                children: vec![bmsh_chunk],
            });
        }

        chunks.push(IffChunk {
            id: "HVMD".to_string(),
            chunk_type: crate::iff::ChunkType::Form,
            version: 0,
            data: Vec::new(),
            children: hvmd_children,
        });

        chunks.push(IffChunk {
            id: "DTRM".to_string(),
            chunk_type: crate::iff::ChunkType::Form,
            version: 0,
            data: Vec::new(),
            children: Vec::new(),
        });
    } else {
        let mut cursor = Cursor::new(original_bytes);
        while cursor.position() < original_bytes.len() as u64 {
            match IffChunk::read_chunk(&mut cursor) {
                Ok(chunk) => chunks.push(chunk),
                Err(e) => return Err(format!("IFF boundary read error: {}", e)),
            }
        }
    }

    let is_v2 = chunks.iter().any(|c| c.id == "POOL") || updated_model.version >= 0x200;

    let mut original_pool_type = 0;
    let mut original_comp_tex = Vec::new();
    let mut original_decomp_tex_len = 0;
    let mut original_comp_mesh = Vec::new();
    let mut original_decomp_mesh_len = 0;
    let mut original_comp_face = Vec::new();
    let mut original_decomp_face_len = 0;

    let mut original_texture_pool = Vec::new();
    let mut original_mesh_pool = Vec::new();
    let mut original_face_pool = Vec::new();

    if is_v2 {
        for chunk in &chunks {
            if chunk.id == "POOL" {
                if !chunk.data.is_empty() {
                    let mut pool_cursor = Cursor::new(&chunk.data);
                    if let Ok(pool_type) = pool_cursor.read_u32::<LittleEndian>() {
                        original_pool_type = pool_type;
                        if let Ok(comp_tex_len) = pool_cursor.read_u32::<LittleEndian>() {
                            if let Ok(decomp_tex_len) = pool_cursor.read_u32::<LittleEndian>() {
                                let mut comp_tex = vec![0u8; comp_tex_len as usize];
                                if pool_cursor.read_exact(&mut comp_tex).is_ok() {
                                    original_comp_tex = comp_tex.clone();
                                    original_decomp_tex_len = decomp_tex_len;
                                    if comp_tex_len == decomp_tex_len {
                                        original_texture_pool = comp_tex.clone();
                                    } else if let Ok(decomp_tex) =
                                        xpress::decompress(&comp_tex, decomp_tex_len as usize)
                                    {
                                        original_texture_pool = decomp_tex;
                                    }
                                }
                                if let Ok(comp_mesh_len) = pool_cursor.read_u32::<LittleEndian>() {
                                    if let Ok(decomp_mesh_len) =
                                        pool_cursor.read_u32::<LittleEndian>()
                                    {
                                        let mut comp_mesh = vec![0u8; comp_mesh_len as usize];
                                        if pool_cursor.read_exact(&mut comp_mesh).is_ok() {
                                            original_comp_mesh = comp_mesh.clone();
                                            original_decomp_mesh_len = decomp_mesh_len;
                                            if comp_mesh_len == decomp_mesh_len {
                                                original_mesh_pool = comp_mesh.clone();
                                            } else if let Ok(decomp_mesh) = xpress::decompress(
                                                &comp_mesh,
                                                decomp_mesh_len as usize,
                                            ) {
                                                original_mesh_pool = decomp_mesh;
                                            }
                                        }
                                    }
                                }
                                if let Ok(comp_face_len) = pool_cursor.read_u32::<LittleEndian>() {
                                    if let Ok(decomp_face_len) =
                                        pool_cursor.read_u32::<LittleEndian>()
                                    {
                                        let mut comp_face = vec![0u8; comp_face_len as usize];
                                        if pool_cursor.read_exact(&mut comp_face).is_ok() {
                                            original_comp_face = comp_face.clone();
                                            original_decomp_face_len = decomp_face_len;
                                            if comp_face_len == decomp_face_len {
                                                original_face_pool = comp_face.clone();
                                            } else if let Ok(decomp_face) = xpress::decompress(
                                                &comp_face,
                                                decomp_face_len as usize,
                                            ) {
                                                original_face_pool = decomp_face;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                break;
            }
        }
    }

    let mut meshes_modified = true;
    if !original_bytes.is_empty() {
        if let Ok(orig) = HODModel::parse(original_bytes) {
            let orig_total_verts: usize = orig
                .meshes
                .iter()
                .map(|m| m.parts.iter().map(|p| p.vertices.len()).sum::<usize>())
                .sum();
            let updated_total_verts: usize = updated_model
                .meshes
                .iter()
                .map(|m| m.parts.iter().map(|p| p.vertices.len()).sum::<usize>())
                .sum();
            if orig_total_verts == updated_total_verts {
                meshes_modified = false;
            }
        }
    }

    let mut new_mesh_pool = Vec::new();
    let mut new_face_pool = Vec::new();

    for chunk in &mut chunks {
        if chunk.id == "HVMD" {
            for sub_chunk in &mut chunk.children {
                if sub_chunk.id.trim() == "GOBG" {
                    sub_chunk.id = "MULT".to_string();
                }
            }
        }
    }

    // For HOD 2.0, always regenerate mesh pool from model data to ensure correct vertex attributes.
    // For HOD 1.0, only update if meshes were actually modified (mesh data is inline in BMSH).
    // IMPORTANT: When original_bytes is empty, the template BMSH chunks have empty data
    // (all read as lod=0), so update_mesh_chunks would match the wrong mesh.
    // In that case, leave new_mesh_pool empty so the POOL chunk built by
    // generate_pool_data stays intact.
    if is_v2
        && updated_model.meshes.iter().any(|m| !m.parts.is_empty())
        && !original_bytes.is_empty()
    {
        // Target HVMD children specifically, not top-level chunks
        for chunk in &mut chunks {
            if chunk.id == "HVMD" {
                update_mesh_chunks(
                    &mut chunk.children,
                    updated_model,
                    is_v2,
                    &mut new_mesh_pool,
                    &mut new_face_pool,
                    "",
                )?;

                chunk.children.retain(|c| c.id != "STAT" && c.id != "MATT");

                for mat in &updated_model.materials {
                    let mut stat_buf = Vec::new();
                    let mut stat_cursor = Cursor::new(&mut stat_buf);

                    write_len_string(&mut stat_cursor, &mat.name).unwrap();
                    write_len_string(&mut stat_cursor, &mat.shader_name).unwrap();
                    write_stat_texture_params(&mut stat_cursor, mat, &updated_model.textures)
                        .unwrap();

                    chunk.children.push(IffChunk {
                        id: "STAT".to_string(),
                        chunk_type: ChunkType::Normal,
                        version: 1001,
                        data: stat_buf,
                        children: Vec::new(),
                    });
                }
            }
        }

        // Append collision mesh vertices to mesh pool
        for cm in &updated_model.collision_meshes {
            for part in &cm.mesh.parts {
                for v in &part.vertices {
                    let _ = crate::hod::write_vertex(
                        &mut new_mesh_pool,
                        v,
                        part.vertex_mask,
                        1400,
                        if (part.vertex_mask & 0x01) != 0 {
                            16
                        } else {
                            0
                        } + if (part.vertex_mask & 0x02) != 0 {
                            16
                        } else {
                            0
                        } + if (part.vertex_mask & 0x08) != 0 { 8 } else { 0 }
                            + if (part.vertex_mask & 0x2000) != 0 {
                                12
                            } else {
                                0
                            }
                            + if (part.vertex_mask & 0x4000) != 0 {
                                12
                            } else {
                                0
                            },
                    );
                }
                for &idx in &part.indices {
                    let _ = new_face_pool.write_u16::<LittleEndian>(idx);
                }
            }
        }
    } else if meshes_modified && !(is_v2 && original_bytes.is_empty()) {
        update_mesh_chunks(
            &mut chunks,
            updated_model,
            is_v2,
            &mut new_mesh_pool,
            &mut new_face_pool,
            "",
        )?;
    } else {
        new_mesh_pool = original_mesh_pool;
        new_face_pool = original_face_pool;
        sanitize_prim_group_counts(&mut chunks)?;
    }

    if is_v2 && !new_mesh_pool.is_empty() {
        let mut pool_data = Vec::new();
        pool_data
            .write_u32::<LittleEndian>(original_pool_type)
            .map_err(|e| e.to_string())?;

        pool_data
            .write_u32::<LittleEndian>(original_comp_tex.len() as u32)
            .map_err(|e| e.to_string())?;
        pool_data
            .write_u32::<LittleEndian>(original_decomp_tex_len)
            .map_err(|e| e.to_string())?;
        pool_data.extend_from_slice(&original_comp_tex);

        // For v2, always use regenerated mesh/face pools to ensure correct vertex data
        // DANGER: Our custom LZXpress compressor seems to cause glitches in the Homeworld engine!
        // We will store the mesh and face pools as UNCOMPRESSED (comp_len == decomp_len).
        let comp_mesh = new_mesh_pool.clone();
        let comp_face = new_face_pool.clone();

        pool_data
            .write_u32::<LittleEndian>(comp_mesh.len() as u32)
            .map_err(|e| e.to_string())?;
        pool_data
            .write_u32::<LittleEndian>(new_mesh_pool.len() as u32)
            .map_err(|e| e.to_string())?;
        pool_data.extend_from_slice(&comp_mesh);

        pool_data
            .write_u32::<LittleEndian>(comp_face.len() as u32)
            .map_err(|e| e.to_string())?;
        pool_data
            .write_u32::<LittleEndian>(new_face_pool.len() as u32)
            .map_err(|e| e.to_string())?;
        pool_data.extend_from_slice(&comp_face);

        for chunk in &mut chunks {
            if chunk.id == "POOL" {
                chunk.data = pool_data;
                break;
            }
        }
    }

    let default_root_pos = Vector3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let default_root_rot = Vector3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let default_root_scale = Vector3 {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    };
    let default_root_joint = HODJoint {
        name: "Root".to_string(),
        parent_name: None,
        local_transform: compose_transform_matrix(
            default_root_pos.clone(),
            default_root_rot.clone(),
            default_root_scale.clone(),
        ),
        position: Some(default_root_pos),
        rotation: Some(default_root_rot),
        scale: Some(default_root_scale),
    };
    let joints_to_write: Vec<&HODJoint> = if updated_model.joints.is_empty() {
        vec![&default_root_joint]
    } else {
        updated_model.joints.iter().collect()
    };

    let mut hier_data = Vec::new();
    if is_v2 {
        let joint_count = joints_to_write.len() as i32;
        // HOD 2.0 HIER uses a full two's-complement negative joint count.
        // Do not mask to the low byte; files can legitimately exceed 255 joints.
        let first_val = (-joint_count) as u32;
        hier_data
            .write_u32::<LittleEndian>(first_val)
            .map_err(|e| e.to_string())?;
        for joint in &joints_to_write {
            write_len_string(&mut hier_data, &joint.name)?;
            let parent_str = joint.parent_name.clone().unwrap_or_default();
            write_len_string(&mut hier_data, &parent_str)?;

            let (pos, rot, scale) = if let (Some(ref p), Some(ref r), Some(ref s)) =
                (&joint.position, &joint.rotation, &joint.scale)
            {
                (p.clone(), r.clone(), s.clone())
            } else {
                let (p, r, s) = decompose_matrix(joint.local_transform.clone());
                (p, r, s)
            };

            hier_data
                .write_f32::<LittleEndian>(pos.x)
                .map_err(|e| e.to_string())?;
            hier_data
                .write_f32::<LittleEndian>(pos.y)
                .map_err(|e| e.to_string())?;
            hier_data
                .write_f32::<LittleEndian>(pos.z)
                .map_err(|e| e.to_string())?;

            hier_data
                .write_f32::<LittleEndian>(rot.x)
                .map_err(|e| e.to_string())?;
            hier_data
                .write_f32::<LittleEndian>(rot.y)
                .map_err(|e| e.to_string())?;
            hier_data
                .write_f32::<LittleEndian>(rot.z)
                .map_err(|e| e.to_string())?;

            hier_data
                .write_f32::<LittleEndian>(scale.x)
                .map_err(|e| e.to_string())?;
            hier_data
                .write_f32::<LittleEndian>(scale.y)
                .map_err(|e| e.to_string())?;
            hier_data
                .write_f32::<LittleEndian>(scale.z)
                .map_err(|e| e.to_string())?;
        }
    } else {
        hier_data
            .write_u32::<LittleEndian>(joints_to_write.len() as u32)
            .map_err(|e| e.to_string())?;
        for joint in &joints_to_write {
            write_len_string(&mut hier_data, &joint.name)?;
            let parent_str = joint.parent_name.clone().unwrap_or_default();
            write_len_string(&mut hier_data, &parent_str)?;

            let (pos, rot, scale) = if let (Some(ref p), Some(ref r), Some(ref s)) =
                (&joint.position, &joint.rotation, &joint.scale)
            {
                (p.clone(), r.clone(), s.clone())
            } else {
                let (p, r, s) = decompose_matrix(joint.local_transform.clone());
                (p, r, s)
            };

            hier_data
                .write_f32::<LittleEndian>(pos.x)
                .map_err(|e| e.to_string())?;
            hier_data
                .write_f32::<LittleEndian>(pos.y)
                .map_err(|e| e.to_string())?;
            hier_data
                .write_f32::<LittleEndian>(pos.z)
                .map_err(|e| e.to_string())?;

            hier_data
                .write_f32::<LittleEndian>(rot.x)
                .map_err(|e| e.to_string())?;
            hier_data
                .write_f32::<LittleEndian>(rot.y)
                .map_err(|e| e.to_string())?;
            hier_data
                .write_f32::<LittleEndian>(rot.z)
                .map_err(|e| e.to_string())?;

            hier_data
                .write_f32::<LittleEndian>(scale.x)
                .map_err(|e| e.to_string())?;
            hier_data
                .write_f32::<LittleEndian>(scale.y)
                .map_err(|e| e.to_string())?;
            hier_data
                .write_f32::<LittleEndian>(scale.z)
                .map_err(|e| e.to_string())?;

            // Default Axis (0.0, -0.0, 0.0)
            hier_data
                .write_f32::<LittleEndian>(0.0)
                .map_err(|e| e.to_string())?;
            hier_data
                .write_f32::<LittleEndian>(-0.0)
                .map_err(|e| e.to_string())?;
            hier_data
                .write_f32::<LittleEndian>(0.0)
                .map_err(|e| e.to_string())?;

            // Default DOF (1, 1, 1)
            hier_data.write_all(&[1, 1, 1]).map_err(|e| e.to_string())?;
        }
    }

    let mut mrkr_data = Vec::new();
    if is_v2 {
        mrkr_data
            .write_u32::<LittleEndian>(updated_model.markers.len() as u32)
            .map_err(|e| e.to_string())?;
        for marker in &updated_model.markers {
            let m_bytes = serialize_single_marker(marker, true)?;
            mrkr_data.write_all(&m_bytes).map_err(|e| e.to_string())?;
        }
    }

    let mut dtrm_found = false;
    for chunk in &mut chunks {
        if chunk.id == "DTRM" {
            dtrm_found = true;

            let mut navl_data = Vec::new();
            if !updated_model.nav_lights.is_empty() {
                navl_data
                    .write_u32::<LittleEndian>(updated_model.nav_lights.len() as u32)
                    .map_err(|e| e.to_string())?;
                for nav in &updated_model.nav_lights {
                    write_len_string(&mut navl_data, &nav.name)?;
                    navl_data
                        .write_u32::<LittleEndian>(nav.section)
                        .map_err(|e| e.to_string())?;
                    navl_data
                        .write_f32::<LittleEndian>(nav.size)
                        .map_err(|e| e.to_string())?;
                    navl_data
                        .write_f32::<LittleEndian>(nav.phase)
                        .map_err(|e| e.to_string())?;
                    navl_data
                        .write_f32::<LittleEndian>(nav.frequency)
                        .map_err(|e| e.to_string())?;
                    write_len_string(&mut navl_data, &nav.style)?;
                    navl_data
                        .write_f32::<LittleEndian>(nav.color.x)
                        .map_err(|e| e.to_string())?;
                    navl_data
                        .write_f32::<LittleEndian>(nav.color.y)
                        .map_err(|e| e.to_string())?;
                    navl_data
                        .write_f32::<LittleEndian>(nav.color.z)
                        .map_err(|e| e.to_string())?;
                    navl_data
                        .write_f32::<LittleEndian>(1.0)
                        .map_err(|e| e.to_string())?; // _unused f32
                    navl_data
                        .write_f32::<LittleEndian>(nav.distance)
                        .map_err(|e| e.to_string())?;
                    navl_data
                        .write_u8(if nav.sprite_visible { 1 } else { 0 })
                        .map_err(|e| e.to_string())?;
                    navl_data
                        .write_u8(if nav.high_end_only { 1 } else { 0 })
                        .map_err(|e| e.to_string())?;
                }
            }

            let mut burn_chunks = Vec::new();
            for burn in &updated_model.engine_burns {
                let mut data = Vec::new();
                write_len_string(&mut data, &burn.name)?;
                write_len_string(&mut data, &burn.parent_name)?;
                data.write_i32::<LittleEndian>(burn.num_divisions)
                    .map_err(|e| e.to_string())?;
                data.write_i32::<LittleEndian>(burn.num_flames)
                    .map_err(|e| e.to_string())?;
                for v in &burn.vertices {
                    data.write_f32::<LittleEndian>(v.x)
                        .map_err(|e| e.to_string())?;
                    data.write_f32::<LittleEndian>(v.y)
                        .map_err(|e| e.to_string())?;
                    data.write_f32::<LittleEndian>(v.z)
                        .map_err(|e| e.to_string())?;
                }
                burn_chunks.push(IffChunk {
                    id: "BURN".to_string(),
                    chunk_type: crate::iff::ChunkType::Default,
                    version: 0,
                    data,
                    children: Vec::new(),
                });
            }

            let mut dock_data = Vec::new();
            if !updated_model.dockpaths.is_empty() {
                let mut paths_payload = Vec::new();
                paths_payload
                    .write_u32::<LittleEndian>(updated_model.dockpaths.len() as u32)
                    .map_err(|e| e.to_string())?;
                for path in &updated_model.dockpaths {
                    write_len_string(&mut paths_payload, &path.name)?;
                    write_len_string(&mut paths_payload, &path.parent_name)?;
                    paths_payload
                        .write_u32::<LittleEndian>(path.val1)
                        .map_err(|e| e.to_string())?;
                    paths_payload
                        .write_u32::<LittleEndian>(path.val2)
                        .map_err(|e| e.to_string())?;
                    paths_payload
                        .write_u32::<LittleEndian>(path.val3)
                        .map_err(|e| e.to_string())?;
                    paths_payload
                        .write_u32::<LittleEndian>(path.val4)
                        .map_err(|e| e.to_string())?;
                    paths_payload
                        .write_u32::<LittleEndian>(path.val5)
                        .map_err(|e| e.to_string())?;
                    write_len_string(&mut paths_payload, &path.compatible_ships)?;
                    paths_payload
                        .write_u32::<LittleEndian>(path.padding1)
                        .map_err(|e| e.to_string())?;
                    write_len_string(&mut paths_payload, &path.padding2)?;
                    paths_payload
                        .write_i32::<LittleEndian>(path.points.len() as i32)
                        .map_err(|e| e.to_string())?;

                    for pt in &path.points {
                        paths_payload
                            .write_f32::<LittleEndian>(pt.position.x)
                            .map_err(|e| e.to_string())?;
                        paths_payload
                            .write_f32::<LittleEndian>(pt.position.y)
                            .map_err(|e| e.to_string())?;
                        paths_payload
                            .write_f32::<LittleEndian>(pt.position.z)
                            .map_err(|e| e.to_string())?;

                        paths_payload
                            .write_f32::<LittleEndian>(pt.rotation.m[0][0])
                            .map_err(|e| e.to_string())?;
                        paths_payload
                            .write_f32::<LittleEndian>(pt.rotation.m[0][1])
                            .map_err(|e| e.to_string())?;
                        paths_payload
                            .write_f32::<LittleEndian>(pt.rotation.m[0][2])
                            .map_err(|e| e.to_string())?;

                        paths_payload
                            .write_f32::<LittleEndian>(pt.rotation.m[1][0])
                            .map_err(|e| e.to_string())?;
                        paths_payload
                            .write_f32::<LittleEndian>(pt.rotation.m[1][1])
                            .map_err(|e| e.to_string())?;
                        paths_payload
                            .write_f32::<LittleEndian>(pt.rotation.m[1][2])
                            .map_err(|e| e.to_string())?;

                        paths_payload
                            .write_f32::<LittleEndian>(pt.rotation.m[2][0])
                            .map_err(|e| e.to_string())?;
                        paths_payload
                            .write_f32::<LittleEndian>(pt.rotation.m[2][1])
                            .map_err(|e| e.to_string())?;
                        paths_payload
                            .write_f32::<LittleEndian>(pt.rotation.m[2][2])
                            .map_err(|e| e.to_string())?;

                        paths_payload
                            .write_f32::<LittleEndian>(pt.tolerance)
                            .map_err(|e| e.to_string())?;
                        paths_payload
                            .write_f32::<LittleEndian>(pt.max_speed)
                            .map_err(|e| e.to_string())?;
                        paths_payload
                            .write_u32::<LittleEndian>(pt.extra1)
                            .map_err(|e| e.to_string())?;
                        paths_payload
                            .write_u32::<LittleEndian>(pt.extra2)
                            .map_err(|e| e.to_string())?;
                    }
                }
                if !is_v2 {
                    let total_size = (paths_payload.len() + 4) as u32;
                    dock_data
                        .write_u32::<BigEndian>(total_size)
                        .map_err(|e| e.to_string())?;
                }
                dock_data.extend_from_slice(&paths_payload);
            }

            let mut new_children = Vec::new();
            let mut hier_written = false;
            let mut mrks_written = false;
            let mut navl_written = false;
            let mut burn_written = false;
            let mut dock_written = false;

            for child in &chunk.children {
                match child.id.as_str() {
                    "HIER" => {
                        new_children.push(IffChunk {
                            id: "HIER".to_string(),
                            chunk_type: crate::iff::ChunkType::Form,
                            version: 0,
                            data: hier_data.clone(),
                            children: Vec::new(),
                        });
                        hier_written = true;
                    }
                    "MRKS" | "MRKR" => {
                        if is_v2 {
                            if !mrks_written {
                                new_children.push(IffChunk {
                                    id: "MRKS".to_string(),
                                    chunk_type: crate::iff::ChunkType::Default,
                                    version: 0,
                                    data: mrkr_data.clone(),
                                    children: Vec::new(),
                                });
                                mrks_written = true;
                            }
                        } else {
                            if !mrks_written {
                                for marker in &updated_model.markers {
                                    let head_data = serialize_single_marker(marker, false)?;
                                    let head_chunk = IffChunk {
                                        id: "HEAD".to_string(),
                                        chunk_type: crate::iff::ChunkType::Normal,
                                        version: 1,
                                        data: head_data,
                                        children: Vec::new(),
                                    };

                                    let mut anim_children = Vec::new();
                                    if let Some(default_anim) = updated_model
                                        .animations
                                        .iter()
                                        .find(|a| a.name == "DefaultAnimation")
                                    {
                                        if let Some(track) = default_anim.tracks.iter().find(|t| {
                                            t.joint_name.eq_ignore_ascii_case(&marker.name)
                                        }) {
                                            let mut has_pos = false;
                                            let mut has_rot = false;

                                            for kf in &track.keyframes {
                                                if kf.position.is_some() {
                                                    has_pos = true;
                                                }
                                                if kf.rotation.is_some() {
                                                    has_rot = true;
                                                }
                                            }

                                            if has_pos {
                                                let mut tx_vals = Vec::new();
                                                let mut ty_vals = Vec::new();
                                                let mut tz_vals = Vec::new();
                                                for kf in &track.keyframes {
                                                    if let Some(ref pos) = kf.position {
                                                        tx_vals.push((kf.time, pos.x as f64));
                                                        ty_vals.push((kf.time, pos.y as f64));
                                                        tz_vals.push((kf.time, pos.z as f64));
                                                    }
                                                }
                                                anim_children.push(serialize_anim_curve(
                                                    "translateX",
                                                    tx_vals,
                                                )?);
                                                anim_children.push(serialize_anim_curve(
                                                    "translateY",
                                                    ty_vals,
                                                )?);
                                                anim_children.push(serialize_anim_curve(
                                                    "translateZ",
                                                    tz_vals,
                                                )?);
                                            }

                                            if has_rot {
                                                let mut rx_vals = Vec::new();
                                                let mut ry_vals = Vec::new();
                                                let mut rz_vals = Vec::new();
                                                for kf in &track.keyframes {
                                                    let euler = if let Some(ref euler) =
                                                        kf.rotation_euler
                                                    {
                                                        euler.clone()
                                                    } else if let Some(ref rot) = kf.rotation {
                                                        quaternion_to_euler(rot)
                                                    } else {
                                                        Vector3 {
                                                            x: 0.0,
                                                            y: 0.0,
                                                            z: 0.0,
                                                        }
                                                    };
                                                    rx_vals.push((kf.time, euler.x as f64));
                                                    ry_vals.push((kf.time, euler.y as f64));
                                                    rz_vals.push((kf.time, euler.z as f64));
                                                }
                                                anim_children.push(serialize_anim_curve(
                                                    "rotateX", rx_vals,
                                                )?);
                                                anim_children.push(serialize_anim_curve(
                                                    "rotateY", ry_vals,
                                                )?);
                                                anim_children.push(serialize_anim_curve(
                                                    "rotateZ", rz_vals,
                                                )?);
                                            }
                                        }
                                    }

                                    let keyf_chunk = IffChunk {
                                        id: "KEYF".to_string(),
                                        chunk_type: crate::iff::ChunkType::Form,
                                        version: 0,
                                        data: Vec::new(),
                                        children: anim_children,
                                    };
                                    new_children.push(IffChunk {
                                        id: "MRKR".to_string(),
                                        chunk_type: crate::iff::ChunkType::Form,
                                        version: 0,
                                        data: Vec::new(),
                                        children: vec![head_chunk, keyf_chunk],
                                    });
                                }
                                mrks_written = true;
                            }
                        }
                    }
                    "NAVL" => {
                        if !navl_data.is_empty() && !navl_written {
                            new_children.push(IffChunk {
                                id: "NAVL".to_string(),
                                chunk_type: crate::iff::ChunkType::Normal,
                                version: 3,
                                data: navl_data.clone(),
                                children: Vec::new(),
                            });
                            navl_written = true;
                        }
                    }
                    "BURN" => {
                        if !burn_written {
                            for burn in &updated_model.engine_burns {
                                let mut data = Vec::new();
                                write_len_string(&mut data, &burn.name)?;
                                write_len_string(&mut data, &burn.parent_name)?;
                                data.write_i32::<LittleEndian>(burn.num_divisions)
                                    .map_err(|e| e.to_string())?;
                                data.write_i32::<LittleEndian>(burn.num_flames)
                                    .map_err(|e| e.to_string())?;
                                for v in &burn.vertices {
                                    data.write_f32::<LittleEndian>(v.x)
                                        .map_err(|e| e.to_string())?;
                                    data.write_f32::<LittleEndian>(v.y)
                                        .map_err(|e| e.to_string())?;
                                    data.write_f32::<LittleEndian>(v.z)
                                        .map_err(|e| e.to_string())?;
                                }
                                new_children.push(IffChunk {
                                    id: "BURN".to_string(),
                                    chunk_type: crate::iff::ChunkType::Default,
                                    version: 0,
                                    data,
                                    children: Vec::new(),
                                });
                            }
                            burn_written = true;
                        }
                    }
                    "DOCK" => {
                        if !dock_data.is_empty() && !dock_written {
                            new_children.push(IffChunk {
                                id: "DOCK".to_string(),
                                chunk_type: crate::iff::ChunkType::Default,
                                version: 0,
                                data: dock_data.clone(),
                                children: Vec::new(),
                            });
                            dock_written = true;
                        }
                    }
                    _ => {
                        new_children.push(child.clone());
                    }
                }
            }

            if !hier_written {
                new_children.push(IffChunk {
                    id: "HIER".to_string(),
                    chunk_type: crate::iff::ChunkType::Form,
                    version: 0,
                    data: hier_data.clone(),
                    children: Vec::new(),
                });
            }
            if is_v2 && !mrks_written && !updated_model.markers.is_empty() {
                new_children.push(IffChunk {
                    id: "MRKS".to_string(),
                    chunk_type: crate::iff::ChunkType::Default,
                    version: 0,
                    data: mrkr_data.clone(),
                    children: Vec::new(),
                });
            }
            if !navl_data.is_empty() && !navl_written {
                new_children.push(IffChunk {
                    id: "NAVL".to_string(),
                    chunk_type: crate::iff::ChunkType::Normal,
                    version: 3,
                    data: navl_data.clone(),
                    children: Vec::new(),
                });
            }
            if !burn_written {
                for burn in &updated_model.engine_burns {
                    let mut data = Vec::new();
                    write_len_string(&mut data, &burn.name)?;
                    write_len_string(&mut data, &burn.parent_name)?;
                    data.write_i32::<LittleEndian>(burn.num_divisions)
                        .map_err(|e| e.to_string())?;
                    data.write_i32::<LittleEndian>(burn.num_flames)
                        .map_err(|e| e.to_string())?;
                    for v in &burn.vertices {
                        data.write_f32::<LittleEndian>(v.x)
                            .map_err(|e| e.to_string())?;
                        data.write_f32::<LittleEndian>(v.y)
                            .map_err(|e| e.to_string())?;
                        data.write_f32::<LittleEndian>(v.z)
                            .map_err(|e| e.to_string())?;
                    }
                    new_children.push(IffChunk {
                        id: "BURN".to_string(),
                        chunk_type: crate::iff::ChunkType::Default,
                        version: 0,
                        data,
                        children: Vec::new(),
                    });
                }
            }
            if !dock_data.is_empty() && !dock_written {
                new_children.push(IffChunk {
                    id: "DOCK".to_string(),
                    chunk_type: crate::iff::ChunkType::Default,
                    version: 0,
                    data: dock_data.clone(),
                    children: Vec::new(),
                });
            }

            chunk.children = new_children;
        }
    }

    if !dtrm_found {
        return Err("DTRM logical container chunk not found in original HOD".to_string());
    }

    let mut out_buffer = Vec::new();
    for chunk in &chunks {
        chunk
            .write_chunk(&mut out_buffer)
            .map_err(|e| e.to_string())?;
    }

    Ok(out_buffer)
}

// =========================================================================
// Phase 3 Animation Loading, Saving & Handedness Math Helpers
// =========================================================================

struct ParsedCurve {
    name: String,
    keyframes: Vec<HODKeyframeCurveValue>,
    #[allow(dead_code)]
    pre_infinity: i32,
    #[allow(dead_code)]
    post_infinity: i32,
}

struct HODKeyframeCurveValue {
    time: f64,
    value: f64,
    in_tangent: [f32; 2],
    out_tangent: [f32; 2],
}

fn evaluate_curve(keyframes: &[HODKeyframeCurveValue], time: f64, default_val: f64) -> f64 {
    if keyframes.is_empty() {
        return default_val;
    }
    if keyframes.len() == 1 {
        return keyframes[0].value;
    }
    if time <= keyframes[0].time {
        return keyframes[0].value;
    }
    if time >= keyframes[keyframes.len() - 1].time {
        return keyframes[keyframes.len() - 1].value;
    }
    for i in 0..keyframes.len() - 1 {
        let k1 = &keyframes[i];
        let k2 = &keyframes[i + 1];
        if time >= k1.time && time <= k2.time {
            let dt = k2.time - k1.time;
            if dt.abs() < 1e-6 {
                return k1.value;
            }
            let alpha = (time - k1.time) / dt;
            return k1.value + alpha * (k2.value - k1.value);
        }
    }
    keyframes[0].value
}

fn quaternion_to_euler(q: &HODQuaternion) -> Vector3 {
    let x = q.x;
    let y = q.y;
    let z = q.z;
    let w = q.w;

    let mut m = [[0.0f32; 4]; 4];
    m[0][0] = 1.0 - 2.0 * (y * y + z * z);
    m[0][1] = 2.0 * (x * y + z * w);
    m[0][2] = 2.0 * (x * z - y * w);

    m[1][0] = 2.0 * (x * y - z * w);
    m[1][1] = 1.0 - 2.0 * (x * x + z * z);
    m[1][2] = 2.0 * (y * z + x * w);

    m[2][0] = 2.0 * (x * z + y * w);
    m[2][1] = 2.0 * (y * z - x * w);
    m[2][2] = 1.0 - 2.0 * (x * x + y * y);
    m[3][3] = 1.0;

    let (_, rot, _) = decompose_matrix(Matrix4 { m });
    rot
}

pub fn euler_to_quaternion(rot: &Vector3) -> HODQuaternion {
    let cx = rot.x.cos();
    let sx = rot.x.sin();
    let cy = rot.y.cos();
    let sy = rot.y.sin();
    let cz = rot.z.cos();
    let sz = rot.z.sin();

    let mut rx = [[0.0f32; 3]; 3];
    rx[0][0] = 1.0;
    rx[1][1] = cx;
    rx[1][2] = sx;
    rx[2][1] = -sx;
    rx[2][2] = cx;

    let mut ry = [[0.0f32; 3]; 3];
    ry[0][0] = cy;
    ry[0][2] = -sy;
    ry[1][1] = 1.0;
    ry[2][0] = sy;
    ry[2][2] = cy;

    let mut rz = [[0.0f32; 3]; 3];
    rz[0][0] = cz;
    rz[0][1] = sz;
    rz[1][0] = -sz;
    rz[1][1] = cz;
    rz[2][2] = 1.0;

    fn mat_mul_3(a: [[f32; 3]; 3], b: [[f32; 3]; 3]) -> [[f32; 3]; 3] {
        let mut res = [[0.0f32; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                res[i][j] = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j];
            }
        }
        res
    }

    let rot_m = mat_mul_3(mat_mul_3(rx, ry), rz);
    matrix_to_quaternion(rot_m)
}

fn matrix_to_quaternion(m: [[f32; 3]; 3]) -> HODQuaternion {
    let tr = m[0][0] + m[1][1] + m[2][2];

    let (qw, qx, qy, qz);
    if tr > 0.0 {
        let s = (tr + 1.0).sqrt() * 2.0;
        qw = 0.25 * s;
        qx = (m[1][2] - m[2][1]) / s;
        qy = (m[2][0] - m[0][2]) / s;
        qz = (m[0][1] - m[1][0]) / s;
    } else if (m[0][0] > m[1][1]) && (m[0][0] > m[2][2]) {
        let s = (1.0 + m[0][0] - m[1][1] - m[2][2]).sqrt() * 2.0;
        qw = (m[1][2] - m[2][1]) / s;
        qx = 0.25 * s;
        qy = (m[0][1] + m[1][0]) / s;
        qz = (m[2][0] + m[0][2]) / s;
    } else if m[1][1] > m[2][2] {
        let s = (1.0 + m[1][1] - m[0][0] - m[2][2]).sqrt() * 2.0;
        qw = (m[2][0] - m[0][2]) / s;
        qx = (m[0][1] + m[1][0]) / s;
        qy = 0.25 * s;
        qz = (m[1][2] + m[2][1]) / s;
    } else {
        let s = (1.0 + m[2][2] - m[0][0] - m[1][1]).sqrt() * 2.0;
        qw = (m[0][1] - m[1][0]) / s;
        qx = (m[2][0] + m[0][2]) / s;
        qy = (m[1][2] + m[2][1]) / s;
        qz = 0.25 * s;
    }

    HODQuaternion {
        x: qx,
        y: qy,
        z: qz,
        w: qw,
    }
}

fn parse_keyf_chunk(keyf: &IffChunk) -> Result<Vec<ParsedCurve>, String> {
    let mut curves = Vec::new();
    for anim in &keyf.children {
        if anim.id == "ANIM" {
            let mut r = Cursor::new(&anim.data);
            let name = read_len_string(&mut r)?;
            let keyframe_count = r.read_i32::<LittleEndian>().map_err(|e| e.to_string())? as usize;
            let mut keyframes = Vec::with_capacity(keyframe_count);
            for _ in 0..keyframe_count {
                let time = r.read_f64::<LittleEndian>().map_err(|e| e.to_string())?;
                let value = r.read_f64::<LittleEndian>().map_err(|e| e.to_string())?;
                let in_x = r.read_f32::<LittleEndian>().map_err(|e| e.to_string())?;
                let in_y = r.read_f32::<LittleEndian>().map_err(|e| e.to_string())?;
                let out_x = r.read_f32::<LittleEndian>().map_err(|e| e.to_string())?;
                let out_y = r.read_f32::<LittleEndian>().map_err(|e| e.to_string())?;
                keyframes.push(HODKeyframeCurveValue {
                    time,
                    value,
                    in_tangent: [in_x, in_y],
                    out_tangent: [out_x, out_y],
                });
            }
            keyframes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());

            let pre_infinity = if r.position() + 4 <= anim.data.len() as u64 {
                r.read_i32::<LittleEndian>().unwrap_or(0)
            } else {
                0
            };
            let post_infinity = if r.position() + 4 <= anim.data.len() as u64 {
                r.read_i32::<LittleEndian>().unwrap_or(0)
            } else {
                0
            };
            curves.push(ParsedCurve {
                name,
                keyframes,
                pre_infinity,
                post_infinity,
            });
        }
    }
    Ok(curves)
}

fn serialize_anim_curve(curve_name: &str, values: Vec<(f64, f64)>) -> Result<IffChunk, String> {
    let mut keyframes = Vec::new();
    for (time, val) in values {
        keyframes.push(HODKeyframeCurveValue {
            time,
            value: val,
            in_tangent: [1.0, 0.0],
            out_tangent: [1.0, 0.0],
        });
    }

    let n = keyframes.len();
    if n > 1 {
        for i in 0..n - 1 {
            let dt = keyframes[i + 1].time - keyframes[i].time;
            let dv = keyframes[i + 1].value - keyframes[i].value;
            let len = (dt * dt + dv * dv).sqrt();
            let dir = if len > 0.0 {
                [(dt / len) as f32, (dv / len) as f32]
            } else {
                [1.0, 0.0]
            };
            keyframes[i].out_tangent = dir;
            keyframes[i + 1].in_tangent = dir;
        }
    }

    let mut data = Vec::new();
    write_len_string(&mut data, curve_name)?;
    data.write_i32::<LittleEndian>(keyframes.len() as i32)
        .map_err(|e| e.to_string())?;
    for kf in &keyframes {
        data.write_f64::<LittleEndian>(kf.time)
            .map_err(|e| e.to_string())?;
        data.write_f64::<LittleEndian>(kf.value)
            .map_err(|e| e.to_string())?;
        data.write_f32::<LittleEndian>(kf.in_tangent[0])
            .map_err(|e| e.to_string())?;
        data.write_f32::<LittleEndian>(kf.in_tangent[1])
            .map_err(|e| e.to_string())?;
        data.write_f32::<LittleEndian>(kf.out_tangent[0])
            .map_err(|e| e.to_string())?;
        data.write_f32::<LittleEndian>(kf.out_tangent[1])
            .map_err(|e| e.to_string())?;
    }
    data.write_i32::<LittleEndian>(0)
        .map_err(|e| e.to_string())?;
    data.write_i32::<LittleEndian>(0)
        .map_err(|e| e.to_string())?;

    Ok(IffChunk {
        id: "ANIM".to_string(),
        chunk_type: crate::iff::ChunkType::Normal,
        version: 1,
        data,
        children: Vec::new(),
    })
}

pub fn parse_mad_bytes(bytes: &[u8], joints: &[HODJoint]) -> Result<Vec<HODAnimation>, String> {
    let mut cursor = Cursor::new(bytes);
    let mut root_chunks = Vec::new();
    while cursor.position() < bytes.len() as u64 {
        match IffChunk::read_chunk(&mut cursor) {
            Ok(chunk) => root_chunks.push(chunk),
            Err(e) => return Err(format!("MAD IFF parse error: {}", e)),
        }
    }

    let mad_form = root_chunks
        .iter()
        .find(|c| c.id.trim() == "MAD")
        .ok_or_else(|| "Root MAD chunk not found in companion file".to_string())?;

    let info_chunk = mad_form
        .children
        .iter()
        .find(|c| c.id.trim() == "INFO")
        .ok_or_else(|| "INFO chunk not found in MAD file".to_string())?;
    let mut r_info = Cursor::new(&info_chunk.data);
    let _fps = r_info
        .read_i32::<LittleEndian>()
        .map_err(|e| e.to_string())?;
    let animation_count = r_info
        .read_i32::<LittleEndian>()
        .map_err(|e| e.to_string())? as usize;
    let curve_count = r_info
        .read_i32::<LittleEndian>()
        .map_err(|e| e.to_string())? as usize;

    let stri_chunk = mad_form
        .children
        .iter()
        .find(|c| c.id.trim() == "STRI")
        .ok_or_else(|| "STRI chunk not found in MAD file".to_string())?;
    let stri_bytes = &stri_chunk.data;

    let get_string_from_stri = |pos: usize| -> String {
        if pos >= stri_bytes.len() {
            return "Unknown".to_string();
        }
        let sub = &stri_bytes[pos..];
        if let Some(null_pos) = sub.iter().position(|&b| b == 0) {
            String::from_utf8_lossy(&sub[..null_pos]).to_string()
        } else {
            String::from_utf8_lossy(sub).to_string()
        }
    };

    let curv_chunk = mad_form
        .children
        .iter()
        .find(|c| c.id.trim() == "CURV")
        .ok_or_else(|| "CURV chunk not found in MAD file".to_string())?;
    let mut r_curv = Cursor::new(&curv_chunk.data);
    let mut curves = Vec::with_capacity(curve_count);
    for _ in 0..curve_count {
        let name_pos = r_curv
            .read_i32::<LittleEndian>()
            .map_err(|e| e.to_string())? as usize;
        let name = get_string_from_stri(name_pos);
        let keyframe_count = r_curv
            .read_i32::<LittleEndian>()
            .map_err(|e| e.to_string())? as usize;
        let mut keyframes = Vec::with_capacity(keyframe_count);
        for _ in 0..keyframe_count {
            let time = r_curv
                .read_f64::<LittleEndian>()
                .map_err(|e| e.to_string())?;
            let value = r_curv
                .read_f64::<LittleEndian>()
                .map_err(|e| e.to_string())?;
            let in_x = r_curv
                .read_f32::<LittleEndian>()
                .map_err(|e| e.to_string())?;
            let in_y = r_curv
                .read_f32::<LittleEndian>()
                .map_err(|e| e.to_string())?;
            let out_x = r_curv
                .read_f32::<LittleEndian>()
                .map_err(|e| e.to_string())?;
            let out_y = r_curv
                .read_f32::<LittleEndian>()
                .map_err(|e| e.to_string())?;
            keyframes.push(HODKeyframeCurveValue {
                time,
                value,
                in_tangent: [in_x, in_y],
                out_tangent: [out_x, out_y],
            });
        }
        keyframes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());

        println!(
            "[DEBUG] Curve '{}' has {} keyframes:",
            name,
            keyframes.len()
        );
        for kf in &keyframes {
            println!("  kf time={:.3}s value={:.6}", kf.time, kf.value);
        }

        let pre_infinity = r_curv
            .read_i32::<LittleEndian>()
            .map_err(|e| e.to_string())?;
        let post_infinity = r_curv
            .read_i32::<LittleEndian>()
            .map_err(|e| e.to_string())?;

        curves.push(ParsedCurve {
            name,
            keyframes,
            pre_infinity,
            post_infinity,
        });
    }

    let mark_chunk = mad_form
        .children
        .iter()
        .find(|c| c.id.trim() == "MARK")
        .ok_or_else(|| "MARK chunk not found in MAD file".to_string())?;
    let mut r_mark = Cursor::new(&mark_chunk.data);
    let mut animations = Vec::with_capacity(animation_count);

    for _ in 0..animation_count {
        let name_pos = r_mark
            .read_i32::<LittleEndian>()
            .map_err(|e| e.to_string())? as usize;
        let anim_name = get_string_from_stri(name_pos);

        let start_time = r_mark
            .read_f32::<LittleEndian>()
            .map_err(|e| e.to_string())? as f64;
        let end_time = r_mark
            .read_f32::<LittleEndian>()
            .map_err(|e| e.to_string())? as f64;
        let _loop_start = r_mark
            .read_f32::<LittleEndian>()
            .map_err(|e| e.to_string())?;
        let _loop_end = r_mark
            .read_f32::<LittleEndian>()
            .map_err(|e| e.to_string())?;

        let joint_count = r_mark
            .read_i32::<LittleEndian>()
            .map_err(|e| e.to_string())? as usize;
        let mut tracks = Vec::with_capacity(joint_count);

        for _ in 0..joint_count {
            let joint_name_pos = r_mark
                .read_i32::<LittleEndian>()
                .map_err(|e| e.to_string())? as usize;
            let joint_name = get_string_from_stri(joint_name_pos);
            let channel_count = r_mark
                .read_i32::<LittleEndian>()
                .map_err(|e| e.to_string())? as usize;
            let mut channel_indices = Vec::with_capacity(channel_count);
            for _ in 0..channel_count {
                let curve_idx = r_mark
                    .read_i32::<LittleEndian>()
                    .map_err(|e| e.to_string())? as usize;
                channel_indices.push(curve_idx);
            }

            let joint_obj = joints
                .iter()
                .find(|j| j.name.eq_ignore_ascii_case(&joint_name));
            let default_pos = joint_obj
                .and_then(|j| j.position.clone())
                .unwrap_or(Vector3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                });
            let default_euler = joint_obj
                .and_then(|j| j.rotation.clone())
                .unwrap_or(Vector3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                });
            let mut default_scale = joint_obj.and_then(|j| j.scale.clone()).unwrap_or(Vector3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            });
            if default_scale.x.abs() < 0.0001 {
                default_scale.x = 1.0;
            }
            if default_scale.y.abs() < 0.0001 {
                default_scale.y = 1.0;
            }
            if default_scale.z.abs() < 0.0001 {
                default_scale.z = 1.0;
            }

            let mut tx_curve: Option<&ParsedCurve> = None;
            let mut ty_curve: Option<&ParsedCurve> = None;
            let mut tz_curve: Option<&ParsedCurve> = None;
            let mut rx_curve: Option<&ParsedCurve> = None;
            let mut ry_curve: Option<&ParsedCurve> = None;
            let mut rz_curve: Option<&ParsedCurve> = None;
            let mut sx_curve: Option<&ParsedCurve> = None;
            let mut sy_curve: Option<&ParsedCurve> = None;
            let mut sz_curve: Option<&ParsedCurve> = None;

            for &idx in &channel_indices {
                if idx < curves.len() {
                    let curve = &curves[idx];
                    let name_lower = curve.name.to_lowercase();
                    if name_lower.ends_with("translatex") {
                        tx_curve = Some(curve);
                    } else if name_lower.ends_with("translatey") {
                        ty_curve = Some(curve);
                    } else if name_lower.ends_with("translatez") {
                        tz_curve = Some(curve);
                    } else if name_lower.ends_with("rotatex") {
                        rx_curve = Some(curve);
                    } else if name_lower.ends_with("rotatey") {
                        ry_curve = Some(curve);
                    } else if name_lower.ends_with("rotatez") {
                        rz_curve = Some(curve);
                    } else if name_lower.ends_with("scalex") {
                        sx_curve = Some(curve);
                    } else if name_lower.ends_with("scaley") {
                        sy_curve = Some(curve);
                    } else if name_lower.ends_with("scalez") {
                        sz_curve = Some(curve);
                    }
                }
            }

            let mut unique_times: Vec<f64> = Vec::new();
            let matched_curves = [
                tx_curve, ty_curve, tz_curve, rx_curve, ry_curve, rz_curve, sx_curve, sy_curve,
                sz_curve,
            ];
            for curve_opt in matched_curves.iter().flatten() {
                for kf in &curve_opt.keyframes {
                    if !unique_times.iter().any(|&t| (t - kf.time).abs() < 1e-5) {
                        unique_times.push(kf.time);
                    }
                }
            }
            unique_times.sort_by(|a, b| a.partial_cmp(b).unwrap());

            let mut keyframes = Vec::new();
            for &time in &unique_times {
                let tx = evaluate_curve(
                    tx_curve.map(|c| &c.keyframes[..]).unwrap_or(&[]),
                    time,
                    default_pos.x as f64,
                );
                let ty = evaluate_curve(
                    ty_curve.map(|c| &c.keyframes[..]).unwrap_or(&[]),
                    time,
                    default_pos.y as f64,
                );
                let tz = evaluate_curve(
                    tz_curve.map(|c| &c.keyframes[..]).unwrap_or(&[]),
                    time,
                    default_pos.z as f64,
                );

                let rx = evaluate_curve(
                    rx_curve.map(|c| &c.keyframes[..]).unwrap_or(&[]),
                    time,
                    default_euler.x as f64,
                );
                let ry = evaluate_curve(
                    ry_curve.map(|c| &c.keyframes[..]).unwrap_or(&[]),
                    time,
                    default_euler.y as f64,
                );
                let rz = evaluate_curve(
                    rz_curve.map(|c| &c.keyframes[..]).unwrap_or(&[]),
                    time,
                    default_euler.z as f64,
                );

                let sx = evaluate_curve(
                    sx_curve.map(|c| &c.keyframes[..]).unwrap_or(&[]),
                    time,
                    default_scale.x as f64,
                );
                let sy = evaluate_curve(
                    sy_curve.map(|c| &c.keyframes[..]).unwrap_or(&[]),
                    time,
                    default_scale.y as f64,
                );
                let sz = evaluate_curve(
                    sz_curve.map(|c| &c.keyframes[..]).unwrap_or(&[]),
                    time,
                    default_scale.z as f64,
                );

                let pos_vec = Vector3 {
                    x: tx as f32,
                    y: ty as f32,
                    z: tz as f32,
                };
                let rot_euler = Vector3 {
                    x: rx as f32,
                    y: ry as f32,
                    z: rz as f32,
                };
                let rot_quat = euler_to_quaternion(&rot_euler);
                let scale_vec = Vector3 {
                    x: sx as f32,
                    y: sy as f32,
                    z: sz as f32,
                };

                keyframes.push(HODKeyframe {
                    time,
                    position: Some(pos_vec),
                    rotation: Some(rot_quat),
                    rotation_euler: Some(rot_euler),
                    scale: Some(scale_vec),
                });
            }

            tracks.push(HODAnimationTrack {
                joint_name,
                keyframes,
            });
        }

        animations.push(HODAnimation {
            name: anim_name,
            duration: if end_time > 0.0 { end_time } else { 4.0 },
            tracks,
        });
    }

    Ok(animations)
}

pub fn serialize_mad_companion(model: &HODModel) -> Result<Vec<u8>, String> {
    let mut stri_data = Vec::new();
    let mut get_stri_pos = |s: &str| -> i32 {
        let pos = stri_data.len() as i32;
        stri_data.extend_from_slice(s.as_bytes());
        stri_data.push(0);
        pos
    };

    let mut anim_names_pos = Vec::new();
    for anim in &model.animations {
        anim_names_pos.push(get_stri_pos(&anim.name));
    }

    struct TempCurve {
        name_pos: i32,
        keyframes: Vec<HODKeyframeCurveValue>,
    }

    let mut temp_curves = Vec::new();

    struct TempTrack {
        joint_name_pos: i32,
        curve_indices: Vec<i32>,
    }

    struct TempAnim {
        name_pos: i32,
        duration: f64,
        tracks: Vec<TempTrack>,
    }

    let mut temp_anims = Vec::new();

    for (anim_idx, anim) in model.animations.iter().enumerate() {
        let name_pos = anim_names_pos[anim_idx];
        let mut temp_tracks = Vec::new();

        for track in &anim.tracks {
            let joint_name_pos = get_stri_pos(&track.joint_name);
            let mut curve_indices = Vec::new();

            let mut has_pos = false;
            let mut has_rot = false;
            let mut has_scale = false;

            for kf in &track.keyframes {
                if kf.position.is_some() {
                    has_pos = true;
                }
                if kf.rotation.is_some() {
                    has_rot = true;
                }
                if kf.scale.is_some() {
                    has_scale = true;
                }
            }

            let mut add_channel = |suffix: &str, values: Vec<(f64, f64)>| {
                let curve_name = format!("{}_{}", track.joint_name, suffix);
                let name_pos = get_stri_pos(&curve_name);

                let mut keyframes = Vec::new();
                for (time, val) in values {
                    keyframes.push(HODKeyframeCurveValue {
                        time,
                        value: val,
                        in_tangent: [1.0, 0.0],
                        out_tangent: [1.0, 0.0],
                    });
                }

                let n = keyframes.len();
                if n > 1 {
                    for i in 0..n - 1 {
                        let dt = keyframes[i + 1].time - keyframes[i].time;
                        let dv = keyframes[i + 1].value - keyframes[i].value;
                        let len = (dt * dt + dv * dv).sqrt();
                        let dir = if len > 0.0 {
                            [(dt / len) as f32, (dv / len) as f32]
                        } else {
                            [1.0, 0.0]
                        };
                        keyframes[i].out_tangent = dir;
                        keyframes[i + 1].in_tangent = dir;
                    }
                }

                let idx = temp_curves.len() as i32;
                temp_curves.push(TempCurve {
                    name_pos,
                    keyframes,
                });
                curve_indices.push(idx);
            };

            if has_pos {
                let mut tx_vals = Vec::new();
                let mut ty_vals = Vec::new();
                let mut tz_vals = Vec::new();
                for kf in &track.keyframes {
                    if let Some(ref pos) = kf.position {
                        tx_vals.push((kf.time, pos.x as f64));
                        ty_vals.push((kf.time, pos.y as f64));
                        tz_vals.push((kf.time, pos.z as f64));
                    }
                }
                add_channel("translateX", tx_vals);
                add_channel("translateY", ty_vals);
                add_channel("translateZ", tz_vals);
            }

            if has_rot {
                let mut rx_vals = Vec::new();
                let mut ry_vals = Vec::new();
                let mut rz_vals = Vec::new();
                for kf in &track.keyframes {
                    let euler = if let Some(ref euler) = kf.rotation_euler {
                        euler.clone()
                    } else if let Some(ref rot) = kf.rotation {
                        quaternion_to_euler(rot)
                    } else {
                        Vector3 {
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                        }
                    };
                    rx_vals.push((kf.time, euler.x as f64));
                    ry_vals.push((kf.time, euler.y as f64));
                    rz_vals.push((kf.time, euler.z as f64));
                }
                add_channel("rotateX", rx_vals);
                add_channel("rotateY", ry_vals);
                add_channel("rotateZ", rz_vals);
            }

            if has_scale {
                let mut sx_vals = Vec::new();
                let mut sy_vals = Vec::new();
                let mut sz_vals = Vec::new();
                for kf in &track.keyframes {
                    if let Some(ref sc) = kf.scale {
                        sx_vals.push((kf.time, sc.x as f64));
                        sy_vals.push((kf.time, sc.y as f64));
                        sz_vals.push((kf.time, sc.z as f64));
                    }
                }
                add_channel("scaleX", sx_vals);
                add_channel("scaleY", sy_vals);
                add_channel("scaleZ", sz_vals);
            }

            temp_tracks.push(TempTrack {
                joint_name_pos,
                curve_indices,
            });
        }

        temp_anims.push(TempAnim {
            name_pos,
            duration: anim.duration,
            tracks: temp_tracks,
        });
    }

    let mut mad_children = Vec::new();

    let mut vers_data = Vec::new();
    let _ = vers_data.write_u32::<BigEndian>(0x104);
    mad_children.push(IffChunk {
        id: "VERS".to_string(),
        chunk_type: crate::iff::ChunkType::Default,
        version: 0,
        data: vers_data,
        children: Vec::new(),
    });

    let mut name_data = Vec::new();
    let _ = name_data.write_all(b"Homeworld2 MAD File");
    mad_children.push(IffChunk {
        id: "NAME".to_string(),
        chunk_type: crate::iff::ChunkType::Default,
        version: 0,
        data: name_data,
        children: Vec::new(),
    });

    let mut info_data = Vec::new();
    let mut joint_map_count = 0;
    let mut total_channel_count = 0;
    for anim in &temp_anims {
        joint_map_count += anim.tracks.len();
        for track in &anim.tracks {
            total_channel_count += track.curve_indices.len();
        }
    }

    let _ = info_data.write_i32::<LittleEndian>(30);
    let _ = info_data.write_i32::<LittleEndian>(model.animations.len() as i32);
    let _ = info_data.write_i32::<LittleEndian>(temp_curves.len() as i32);
    let _ = info_data.write_i32::<LittleEndian>(joint_map_count as i32);
    let _ = info_data.write_i32::<LittleEndian>(total_channel_count as i32);

    mad_children.push(IffChunk {
        id: "INFO".to_string(),
        chunk_type: crate::iff::ChunkType::Default,
        version: 0,
        data: info_data,
        children: Vec::new(),
    });

    mad_children.push(IffChunk {
        id: "STRI".to_string(),
        chunk_type: crate::iff::ChunkType::Default,
        version: 0,
        data: stri_data,
        children: Vec::new(),
    });

    let mut mark_data = Vec::new();
    for anim in &temp_anims {
        let _ = mark_data.write_i32::<LittleEndian>(anim.name_pos);
        let _ = mark_data.write_f32::<LittleEndian>(0.0);
        let _ = mark_data.write_f32::<LittleEndian>(anim.duration as f32);
        let _ = mark_data.write_f32::<LittleEndian>(0.0);
        let _ = mark_data.write_f32::<LittleEndian>(anim.duration as f32);
        let _ = mark_data.write_i32::<LittleEndian>(anim.tracks.len() as i32);

        for track in &anim.tracks {
            let _ = mark_data.write_i32::<LittleEndian>(track.joint_name_pos);
            let _ = mark_data.write_i32::<LittleEndian>(track.curve_indices.len() as i32);
            for &idx in &track.curve_indices {
                let _ = mark_data.write_i32::<LittleEndian>(idx);
            }
        }
    }

    mad_children.push(IffChunk {
        id: "MARK".to_string(),
        chunk_type: crate::iff::ChunkType::Default,
        version: 0,
        data: mark_data,
        children: Vec::new(),
    });

    let mut curv_data = Vec::new();
    for curve in &temp_curves {
        let _ = curv_data.write_i32::<LittleEndian>(curve.name_pos);
        let _ = curv_data.write_i32::<LittleEndian>(curve.keyframes.len() as i32);
        for kf in &curve.keyframes {
            let _ = curv_data.write_f64::<LittleEndian>(kf.time);
            let _ = curv_data.write_f64::<LittleEndian>(kf.value);
            let _ = curv_data.write_f32::<LittleEndian>(kf.in_tangent[0]);
            let _ = curv_data.write_f32::<LittleEndian>(kf.in_tangent[1]);
            let _ = curv_data.write_f32::<LittleEndian>(kf.out_tangent[0]);
            let _ = curv_data.write_f32::<LittleEndian>(kf.out_tangent[1]);
        }
        let _ = curv_data.write_i32::<LittleEndian>(0);
        let _ = curv_data.write_i32::<LittleEndian>(0);
    }

    mad_children.push(IffChunk {
        id: "CURV".to_string(),
        chunk_type: crate::iff::ChunkType::Default,
        version: 0,
        data: curv_data,
        children: Vec::new(),
    });

    let mad_root = IffChunk {
        id: "MAD ".to_string(),
        chunk_type: crate::iff::ChunkType::Form,
        version: 0,
        data: Vec::new(),
        children: mad_children,
    };

    let mut output = Vec::new();
    mad_root
        .write_chunk(&mut output)
        .map_err(|e| e.to_string())?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decompress_dxt3_expands_explicit_alpha() {
        let mut block = [0u8; 16];
        for i in 0..16 {
            let alpha = i as u8;
            if i % 2 == 0 {
                block[i / 2] |= alpha;
            } else {
                block[i / 2] |= alpha << 4;
            }
        }

        let red565 = 0xF800u16;
        block[8..10].copy_from_slice(&red565.to_le_bytes());
        block[10..12].copy_from_slice(&red565.to_le_bytes());

        let rgba = decompress_dxt3(&block, 4, 4);

        for i in 0..16 {
            assert_eq!(rgba[i * 4], 255);
            assert_eq!(rgba[i * 4 + 1], 0);
            assert_eq!(rgba[i * 4 + 2], 0);
            assert_eq!(rgba[i * 4 + 3], i as u8 * 17);
        }
    }
}
