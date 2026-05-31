use image::RgbaImage;
use image_dds::{dds_from_image, ImageFormat, Mipmaps};

pub fn encode_to_dxt5(img: &RgbaImage) -> Result<Vec<u8>, String> {
    let dds = dds_from_image(
        img,
        ImageFormat::BC3RgbaUnorm,
        image_dds::Quality::Normal,
        Mipmaps::GeneratedAutomatic
    ).map_err(|e| format!("DDS conversion failed: {}", e))?;
    
    let mut out = Vec::new();
    dds.write(&mut out).map_err(|e| format!("DDS write failed: {}", e))?;
    Ok(out)
}

pub fn encode_to_dxt1(img: &RgbaImage) -> Result<Vec<u8>, String> {
    let dds = dds_from_image(
        img,
        ImageFormat::BC1RgbaUnorm,
        image_dds::Quality::Normal,
        Mipmaps::GeneratedAutomatic
    ).map_err(|e| format!("DDS conversion failed: {}", e))?;
    
    let mut out = Vec::new();
    dds.write(&mut out).map_err(|e| format!("DDS write failed: {}", e))?;
    Ok(out)
}
