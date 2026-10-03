use byteorder::{LittleEndian, ReadBytesExt};
use hwr_hod_parser::iff::IffChunk;
use std::fs::File;
use std::io::{Cursor, Read};

fn read_len_string(r: &mut Cursor<&Vec<u8>>) -> Result<String, String> {
    let len = r.read_u32::<LittleEndian>().map_err(|e| e.to_string())? as usize;
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf).map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&buf).to_string())
}

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "testing/hgn_hft_hw2.hod".to_string());
    let mut file = File::open(&path).expect("Failed to open HOD");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("Failed to read");

    let mut cursor = Cursor::new(&buffer);
    let mut chunks = Vec::new();
    while (cursor.position() as usize) < buffer.len() {
        match IffChunk::read_chunk(&mut cursor) {
            Ok(c) => chunks.push(c),
            Err(e) => {
                println!("Error reading chunk: {}", e);
                break;
            }
        }
    }

    println!("Top-level chunks: {:?}", chunks.iter().map(|c| c.id.clone()).collect::<Vec<_>>());

    for chunk in &chunks {
        if chunk.id == "HVMD" {
            println!("\nHVMD children ({}):", chunk.children.len());
            for (i, sub) in chunk.children.iter().enumerate() {
                println!(
                    "  [{}] id='{}' type={:?} data_len={} children={}",
                    i,
                    sub.id,
                    sub.chunk_type,
                    sub.data.len(),
                    sub.children.len()
                );
            }
        }
    }

    // Dump raw STAT / MATT / TEXM / LMIP chunks
    for chunk in &chunks {
        if chunk.id != "HVMD" {
            continue;
        }
        for sub in &chunk.children {
            let id = sub.id.trim();
            if id == "STAT" || id == "MATT" {
                println!("\n===== {} ({} bytes) =====", id, sub.data.len());
                let data = &sub.data;
                // hex dump
                for (row, slice) in data.chunks(16).enumerate() {
                    let hex: Vec<String> = slice.iter().map(|b| format!("{:02x}", b)).collect();
                    let ascii: String = slice
                        .iter()
                        .map(|&b| if b >= 32 && b <= 126 { b as char } else { '.' })
                        .collect();
                    println!("  {:04x}: {:<47} {}", row * 16, hex.join(" "), ascii);
                }

                let mut r = Cursor::new(data);
                let mat_name = read_len_string(&mut r).unwrap_or_default();
                let shader_name = read_len_string(&mut r).unwrap_or_default();
                let param_count = r.read_u32::<LittleEndian>().unwrap_or(0);
                println!(
                    "  PARSE: mat_name='{}' shader_name='{}' param_count={}",
                    mat_name, shader_name, param_count
                );
                println!("  cursor after param_count at offset: {}", r.position());
            } else if id == "TEXM" || id == "LMIP" {
                println!("\n===== {} ({} bytes) =====", id, sub.data.len());
                let data = &sub.data;
                let mut r = Cursor::new(data);
                let tex_name = read_len_string(&mut r).unwrap_or_default();
                println!("  PARSE: tex_name='{}' (offset after name: {})", tex_name, r.position());
                // dump remaining header bytes
                let pos = r.position() as usize;
                let rest = &data[pos..std::cmp::min(pos + 48, data.len())];
                let hex: Vec<String> = rest.iter().map(|b| format!("{:02x}", b)).collect();
                println!("  header bytes: {}", hex.join(" "));
            }
        }
    }
}
