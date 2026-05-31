use std::fs;
use hwr_hod_parser::hod::HODModel;
use image::GenericImageView;

fn main() {
    let hod_bytes = fs::read("../testing/ter_centaur/ter_centaur_hodor.hod").unwrap();
    let hod_model = HODModel::parse(&hod_bytes).unwrap();
    let centaur_tex = hod_model.textures.iter().find(|t| t.name.to_lowercase().contains("centaur")).unwrap();
    
    // 2. Decode the HOD texture preview PNG to see what py=0 is
    use base64::prelude::*;
    let png_bytes = BASE64_STANDARD.decode(centaur_tex.png_preview.as_ref().unwrap()).unwrap();
    let img = image::load_from_memory(&png_bytes).unwrap();
    let hod_rgba = img.to_rgba8();
    
    println!("HOD Raw RGBA (py=0, px=0..3):");
    for x in 0..4 {
        let p = hod_rgba.get_pixel(x as u32, 0);
        println!("  {}, {}, {}, {}", p[0], p[1], p[2], p[3]);
    }
    
    let py = img.height() - 1;
    println!("HOD Raw RGBA (py={}, px=0..3):", py);
    for x in 0..4 {
        let p = hod_rgba.get_pixel(x as u32, py);
        println!("  {}, {}, {}, {}", p[0], p[1], p[2], p[3]);
    }

    let tga_bytes = fs::read("../testing/ter_centaur/centaur.tga").unwrap();
    let tga_img = image::load_from_memory_with_format(&tga_bytes, image::ImageFormat::Tga).unwrap();
    let tga_rgba = tga_img.to_rgba8();

    println!("TGA Raw (py=0, px=0..3):");
    for x in 0..4 {
        let p = tga_rgba.get_pixel(x as u32, 0);
        println!("  {}, {}, {}, {}", p[0], p[1], p[2], p[3]);
    }
    
    let tga_py = tga_img.height() - 1;
    println!("TGA Raw (py={}, px=0..3):", tga_py);
    for x in 0..4 {
        let p = tga_rgba.get_pixel(x as u32, tga_py);
        println!("  {}, {}, {}, {}", p[0], p[1], p[2], p[3]);
    }
}
