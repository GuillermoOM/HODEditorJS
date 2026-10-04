use crate::iff::IffChunk;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Vector3 {
    #[serde(default)]
    pub x: f32,
    #[serde(default)]
    pub y: f32,
    #[serde(default)]
    pub z: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Vector2 {
    #[serde(default)]
    pub u: f32,
    #[serde(default)]
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

/// One generic STAT shader parameter record.
/// Layout on disk: `u32 param_type`, `u32 value_size`, `value[value_size]`, then
/// (in the common named variant) a length-prefixed name string.
/// `param_type == 5` holds a texture pool index in the first 4 value bytes;
/// `param_type == 4` holds a 4 x f32 (16 byte) colour/vector.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HODMaterialParam {
    pub param_type: u32,
    pub value: Vec<u8>,
    #[serde(default)]
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HODMaterial {
    pub name: String,
    pub shader_name: String,
    pub texture_maps: Vec<String>,
    #[serde(default)]
    pub parameters: Vec<u8>,
    /// Full generic shader parameter list, preserved verbatim for lossless saves.
    #[serde(default)]
    pub params: Vec<HODMaterialParam>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HODNavLight {
    pub name: String,
    pub section: u32,
    #[serde(default)]
    pub size: f32,
    #[serde(default)]
    pub phase: f32,
    #[serde(default)]
    pub frequency: f32,
    pub style: String,
    pub color: Vector3,
    #[serde(default)]
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
    #[serde(default)]
    pub radius: f32,
    pub mesh: HODMesh,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HODDockpoint {
    pub position: Vector3,
    pub rotation: Matrix4,
    #[serde(default)]
    pub tolerance: f32,
    #[serde(default)]
    pub max_speed: f32,
    #[serde(default)]
    pub extra1: u32,
    #[serde(default)]
    pub extra2: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HODDockpath {
    pub name: String,
    pub parent_name: String,
    pub points: Vec<HODDockpoint>,
    #[serde(default)]
    pub val1: u32,
    #[serde(default)]
    pub val2: u32,
    #[serde(default)]
    pub val3: u32,
    #[serde(default)]
    pub val4: u32,
    #[serde(default)]
    pub val5: u32,
    #[serde(default)]
    pub compatible_ships: String,
    #[serde(default)]
    pub padding1: u32,
    #[serde(default)]
    pub padding2: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HODQuaternion {
    #[serde(default)]
    pub x: f32,
    #[serde(default)]
    pub y: f32,
    #[serde(default)]
    pub z: f32,
    #[serde(default)]
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
