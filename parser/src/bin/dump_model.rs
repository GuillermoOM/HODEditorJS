use hwr_hod_parser::hod::HODModel;
use std::fs::File;
use std::io::{Read, Write};

fn main() {
    let path = std::env::args().nth(1).expect("usage: dump_model <file.hod>");
    let mut file = File::open(&path).expect("Failed to open HOD");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("Failed to read");

    let model = HODModel::parse_with_external(&buffer, Some(&path), None).expect("parse failed");

    println!("\n==== RESULT ====");
    println!("version: {}", model.version);
    println!("textures ({}):", model.textures.len());
    for (i, t) in model.textures.iter().enumerate() {
        println!("  [{}] name='{}' format='{}' {}x{} png_data_len={}",
            i, t.name, t.format, t.width, t.height,
            t.png_data.as_ref().map(|d| d.len()).unwrap_or(0));
    }
    println!("materials ({}):", model.materials.len());
    for (i, m) in model.materials.iter().enumerate() {
        println!("  [{}] name='{}' shader='{}' texture_maps={:?} params_len={}",
            i, m.name, m.shader_name, m.texture_maps, m.parameters.len());
    }

    let out_dir = "/tmp/opencode/hod_tex";
    std::fs::create_dir_all(out_dir).ok();
    for t in &model.textures {
        if let Some(ref png_b64) = t.png_data {
            let fname = format!("{}/{}.b64", out_dir, t.name.replace('/', "_").replace('\\', "_"));
            let mut f = File::create(&fname).unwrap();
            f.write_all(png_b64.as_bytes()).unwrap();
            println!("wrote {} ({} b64 chars)", fname, png_b64.len());
        }
    }
}

fn base64_decode(s: &str) -> Vec<u8> {
    fn val(c: u8) -> u32 {
        match c {
            b'A'..=b'Z' => (c - b'A') as u32,
            b'a'..=b'z' => (c - b'a') as u32 + 26,
            b'0'..=b'9' => (c - b'0') as u32 + 52,
            b'+' => 62,
            b'/' => 63,
            _ => 0,
        }
    }
    let mut out = Vec::new();
    let bytes: Vec<u8> = s.bytes().filter(|&c| c != b'=' && c != b'\n' && c != b'\r' && c != b' ').collect();
    let mut i = 0;
    while i + 4 <= bytes.len() {
        let n = (val(bytes[i]) << 18) | (val(bytes[i + 1]) << 12) | (val(bytes[i + 2]) << 6) | val(bytes[i + 3]);
        out.push((n >> 16) as u8);
        out.push((n >> 8) as u8);
        out.push(n as u8);
        i += 4;
    }
    if bytes.len() - i == 2 {
        let n = (val(bytes[i]) << 18) | (val(bytes[i + 1]) << 12);
        out.push((n >> 16) as u8);
    } else if bytes.len() - i == 3 {
        let n = (val(bytes[i]) << 18) | (val(bytes[i + 1]) << 12) | (val(bytes[i + 2]) << 6);
        out.push((n >> 16) as u8);
        out.push((n >> 8) as u8);
    }
    out
}
