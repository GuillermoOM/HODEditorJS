use crate::iff::ChunkType;
use crate::iff::IffChunk;
pub use crate::types::*;
pub use crate::math::*;
pub use crate::texture_codec::*;
pub use crate::animation::*;
pub use crate::model_methods::*;
pub use crate::engine_nozzles::*;
pub use crate::parse::*;
use crate::parse::find_tga_recursive;
use crate::xpress;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read, Write};


pub(crate) fn parse_texture(chunk: &IffChunk, context: &mut ParsingContext) -> Result<HODTexture, String> {
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
        // Split a trailing "[N]" selector off BEFORE stripping the extension:
        // "fth_export.psd[1]" / "fth_export.psd[2]" are DIFF/GLOW layers of one
        // PSD, and a plain rsplit_once('.') discards the selector, colliding
        // both layers onto one name (save then loses the binding, because
        // textures are re-resolved by name). "Capital04-01[1].dds" needs no
        // handling: its selector is already inside the stem.
        let (base_name, selector) = match file_name.rfind('[') {
            Some(i) if i > 0 && file_name.ends_with(']') => (&file_name[..i], &file_name[i..]),
            _ => (file_name, ""),
        };
        let stem = base_name
            .rsplit_once('.')
            .map(|(s, _)| s)
            .unwrap_or(base_name);
        let stem = format!("{}{}", stem, selector);
        if stem.ends_with(&format_str) {
            stem
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
pub(crate) fn write_len_string<W: Write>(writer: &mut W, s: &str) -> Result<(), String> {
    let bytes = s.as_bytes();
    writer
        .write_u32::<LittleEndian>(bytes.len() as u32)
        .map_err(|e| e.to_string())?;
    writer.write_all(bytes).map_err(|e| e.to_string())?;
    Ok(())
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
pub(crate) fn base64_encode(data: &[u8]) -> String {
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
    // Resolve the current texture bindings (slot order) to pool indices.
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

    // Rebuild the generic parameter list: keep every non-texture parameter
    // (colours, scalars, ...) verbatim so classic HW2 materials survive a
    // load/save round-trip, and refresh the texture parameters from the
    // current bindings while preserving their original slot names ($mask, ...).
    let mut records: Vec<HODMaterialParam> = Vec::new();
    let mut pending_textures = texture_indices.iter();

    if mat.params.is_empty() {
        for (texture_index, param_name) in texture_indices.iter() {
            records.push(HODMaterialParam {
                param_type: 5,
                value: texture_index.to_le_bytes().to_vec(),
                name: param_name.clone(),
            });
        }
    } else {
        for param in &mat.params {
            if param.param_type == 5 {
                let Some((texture_index, fallback_name)) = pending_textures.next() else {
                    // Slot was cleared in the editor: drop the stale binding.
                    continue;
                };
                let name = if param.name.is_empty() {
                    fallback_name.clone()
                } else {
                    param.name.clone()
                };
                records.push(HODMaterialParam {
                    param_type: 5,
                    value: texture_index.to_le_bytes().to_vec(),
                    name,
                });
            } else {
                records.push(param.clone());
            }
        }
        // Textures added in the editor beyond the original texture slots.
        for (texture_index, param_name) in pending_textures {
            records.push(HODMaterialParam {
                param_type: 5,
                value: texture_index.to_le_bytes().to_vec(),
                name: param_name.clone(),
            });
        }
    }

    writer
        .write_u32::<LittleEndian>(records.len() as u32)
        .map_err(|e| e.to_string())?;
    for record in &records {
        writer
            .write_u32::<LittleEndian>(record.param_type)
            .map_err(|e| e.to_string())?;
        writer
            .write_u32::<LittleEndian>(record.value.len() as u32)
            .map_err(|e| e.to_string())?;
        writer.write_all(&record.value).map_err(|e| e.to_string())?;
        write_len_string(writer, &record.name)?;
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
                params: Vec::new(),
            }
        } else {
            HODMaterial {
                name: "default_mat".to_string(),
                shader_name: "default_shader".to_string(),
                texture_maps: Vec::new(),
                parameters: Vec::new(),
                params: Vec::new(),
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
