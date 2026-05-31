use image::{Rgba, RgbaImage};

pub fn pack_glow(
    refl: Option<&RgbaImage>,
    glow: Option<&RgbaImage>,
    spec: Option<&RgbaImage>,
    width: u32,
    height: u32,
) -> RgbaImage {
    let mut packed = RgbaImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let r = if let Some(img) = refl {
                if x < img.width() && y < img.height() {
                    img.get_pixel(x, y)[0]
                } else {
                    0
                }
            } else {
                0
            };

            let g = if let Some(img) = glow {
                if x < img.width() && y < img.height() {
                    img.get_pixel(x, y)[1]
                } else {
                    0
                }
            } else {
                0
            };

            let b = if let Some(img) = spec {
                if x < img.width() && y < img.height() {
                    img.get_pixel(x, y)[2]
                } else {
                    0
                }
            } else {
                0
            };

            packed.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }
    packed
}

pub fn pack_team(
    team: Option<&RgbaImage>,
    strp: Option<&RgbaImage>,
    pain: Option<&RgbaImage>,
    width: u32,
    height: u32,
) -> RgbaImage {
    let mut packed = RgbaImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let r = if let Some(img) = team {
                if x < img.width() && y < img.height() {
                    img.get_pixel(x, y)[0]
                } else {
                    0
                }
            } else {
                0
            };

            let g = if let Some(img) = strp {
                if x < img.width() && y < img.height() {
                    img.get_pixel(x, y)[1] // Note: DAEnerys uses the channel directly if it's grayscale, or G channel. We'll use R since single-channel might be in R, but Assimp/DevIL loads TGA to RGBA so any channel works. Actually, if it's grayscale, R=G=B. We will use 0 for R, 1 for G, 2 for B to match standard packing where they might put it in those channels. Wait, HWTexture.MakeMultTexture takes whole images and extracts R from first, G from second, B from third. Yes, so index 0, 1, 2 respectively.
                } else {
                    0
                }
            } else {
                0
            };

            let b = if let Some(img) = pain {
                if x < img.width() && y < img.height() {
                    img.get_pixel(x, y)[2]
                } else {
                    0
                }
            } else {
                0
            };

            packed.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }
    packed
}

// Fallback for combining diff with alpha
pub fn pack_diffuse_with_alpha(
    diff: Option<&RgbaImage>,
    alpha: Option<&RgbaImage>,
    width: u32,
    height: u32,
) -> RgbaImage {
    let mut packed = RgbaImage::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let mut rgba = Rgba([0, 0, 0, 255]);
            if let Some(img) = diff {
                if x < img.width() && y < img.height() {
                    let p = img.get_pixel(x, y);
                    rgba[0] = p[0];
                    rgba[1] = p[1];
                    rgba[2] = p[2];
                    rgba[3] = p[3];
                }
            }
            if let Some(img) = alpha {
                if x < img.width() && y < img.height() {
                    rgba[3] = img.get_pixel(x, y)[0]; // use R channel of alpha map as alpha
                }
            }
            packed.put_pixel(x, y, rgba);
        }
    }
    packed
}

use std::path::Path;
use crate::hod::{HODModel, HODTexture};
use image::ImageReader;
use base64::{engine::general_purpose, Engine as _};
use image::ImageEncoder;

fn load_image_if_exists(base_dir: &Path, base_name: &str, suffix: &str) -> Option<RgbaImage> {
    let tga_path = base_dir.join(format!("{}{}.tga", base_name, suffix));
    let tga_upper_path = base_dir.join(format!("{}{}.TGA", base_name, suffix));
    let png_path = base_dir.join(format!("{}{}.png", base_name, suffix));
    
    let path = if tga_path.exists() {
        tga_path
    } else if tga_upper_path.exists() {
        tga_upper_path
    } else if png_path.exists() {
        png_path
    } else {
        return None;
    };

    if let Ok(reader) = ImageReader::open(&path) {
        if let Ok(img) = reader.decode() {
            return Some(img.to_rgba8());
        }
    }
    None
}

fn encode_rgba_to_b64_png(img: &RgbaImage) -> Option<String> {
    let mut png_bytes = Vec::new();
    if image::codecs::png::PngEncoder::new(&mut png_bytes)
        .write_image(
            img.as_raw(),
            img.width(),
            img.height(),
            image::ColorType::Rgba8.into(),
        )
        .is_ok()
    {
        Some(general_purpose::STANDARD.encode(&png_bytes))
    } else {
        None
    }
}

pub fn discover_and_pack_textures(model: &mut HODModel, base_dir: &Path) {
    let mut new_textures = Vec::new();
    let mut processed_bases = std::collections::HashSet::new();

    for mat in &mut model.materials {
        // Try to derive base_name from texture_maps if any, else mat.name
        let base_name = if !mat.texture_maps.is_empty() {
            let mut name = mat.texture_maps[0].clone();
            if name.to_lowercase().ends_with(".tga") || name.to_lowercase().ends_with(".png") {
                name.truncate(name.len() - 4);
            }
            if name.to_lowercase().ends_with("_diff") {
                name.truncate(name.len() - 5);
            }
            name
        } else {
            mat.name.clone()
        };

        if processed_bases.contains(&base_name) {
            continue;
        }
        processed_bases.insert(base_name.clone());

        // Load all possible component maps
        let diff = load_image_if_exists(base_dir, &base_name, "_DIFF")
            .or_else(|| load_image_if_exists(base_dir, &base_name, ""));
        let glow = load_image_if_exists(base_dir, &base_name, "_GLOW");
        let team = load_image_if_exists(base_dir, &base_name, "_TEAM");
        let norm = load_image_if_exists(base_dir, &base_name, "_NORM");
        let spec = load_image_if_exists(base_dir, &base_name, "_SPEC");
        let refl = load_image_if_exists(base_dir, &base_name, "_REFL");
        let strp = load_image_if_exists(base_dir, &base_name, "_STRP");
        let pain = load_image_if_exists(base_dir, &base_name, "_PAIN");

        let mut width = 0;
        let mut height = 0;
        let components = [&diff, &glow, &team, &norm, &spec, &refl, &strp, &pain];
        for comp in &components {
            if let Some(img) = comp {
                width = width.max(img.width());
                height = height.max(img.height());
            }
        }

        if width == 0 || height == 0 {
            continue; // No textures found
        }

        let mut add_texture = |name: &str, img: &RgbaImage| {
            if let Some(b64) = encode_rgba_to_b64_png(img) {
                let has_alpha = img.pixels().any(|p| p[3] < 250);
                new_textures.push(HODTexture {
                    name: name.to_string(),
                    width,
                    height,
                    format: if has_alpha { "DXT5".to_string() } else { "DXT1".to_string() },
                    png_preview: Some(format!("data:image/png;base64,{}", b64)),
                    png_data: Some(b64),
                    source_path: None,
                });
            }
        };

        // Diffuse
        if let Some(img) = &diff {
            add_texture(&format!("{}_DIFF", base_name), img);
        }
        
        // Normal
        if let Some(img) = &norm {
            add_texture(&format!("{}_NORM", base_name), img);
        }

        // Packed GLOW
        if glow.is_some() || refl.is_some() || spec.is_some() {
            let packed = pack_glow(refl.as_ref(), glow.as_ref(), spec.as_ref(), width, height);
            add_texture(&format!("{}_GLOW", base_name), &packed);
        } else if let Some(img) = &spec {
            add_texture(&format!("{}_SPEC", base_name), img);
        }

        // Packed TEAM
        if team.is_some() || strp.is_some() || pain.is_some() {
            let packed = pack_team(team.as_ref(), strp.as_ref(), pain.as_ref(), width, height);
            add_texture(&format!("{}_TEAM", base_name), &packed);
        }
    }

    model.textures.extend(new_textures);
}

