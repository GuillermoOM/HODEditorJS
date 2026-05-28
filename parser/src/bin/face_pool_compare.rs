use std::fs;
use std::io::{Cursor, Read};

use byteorder::{LittleEndian, ReadBytesExt};
use hwr_hod_parser::iff::IffChunk;
use hwr_hod_parser::xpress;

fn decompressed_face_pool(path: &str) -> Vec<u8> {
    let bytes = fs::read(path).unwrap();
    let mut cursor = Cursor::new(bytes);
    while cursor.position() < cursor.get_ref().len() as u64 {
        let chunk = IffChunk::read_chunk(&mut cursor).unwrap();
        if chunk.id != "POOL" {
            continue;
        }

        let mut pool = Cursor::new(chunk.data);
        let _pool_type = pool.read_u32::<LittleEndian>().unwrap();
        let comp_tex_len = pool.read_u32::<LittleEndian>().unwrap();
        let _decomp_tex_len = pool.read_u32::<LittleEndian>().unwrap();
        pool.set_position(pool.position() + comp_tex_len as u64);

        let comp_mesh_len = pool.read_u32::<LittleEndian>().unwrap();
        let _decomp_mesh_len = pool.read_u32::<LittleEndian>().unwrap();
        pool.set_position(pool.position() + comp_mesh_len as u64);

        let comp_face_len = pool.read_u32::<LittleEndian>().unwrap();
        let decomp_face_len = pool.read_u32::<LittleEndian>().unwrap();
        let mut comp_face = vec![0; comp_face_len as usize];
        pool.read_exact(&mut comp_face).unwrap();
        if comp_face_len == decomp_face_len {
            return comp_face;
        }
        return xpress::decompress(&comp_face, decomp_face_len as usize).unwrap();
    }
    panic!("POOL not found in {path}");
}

fn dump(path: &str, label: &str) {
    let face = decompressed_face_pool(path);
    println!("\n=== {label} ===");
    println!("bytes={} u16s={}", face.len(), face.len() / 2);

    for row in 0..8 {
        let start = row * 32;
        if start >= face.len() {
            break;
        }
        let end = (start + 32).min(face.len());
        println!("{:04x}: {:02x?}", start, &face[start..end]);
    }

    let le: Vec<u16> = face
        .chunks_exact(2)
        .take(80)
        .map(|b| u16::from_le_bytes([b[0], b[1]]))
        .collect();
    let be: Vec<u16> = face
        .chunks_exact(2)
        .take(80)
        .map(|b| u16::from_be_bytes([b[0], b[1]]))
        .collect();
    println!("LE first 80: {le:?}");
    println!("BE first 80: {be:?}");
}

fn main() {
    let base = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld/HWRM_FSFC/source/pebble/pebble_0";
    dump(&format!("{base}/pebble_0_original.hod"), "original");
    dump(&format!("{base}/pebble_0.hod"), "generated");
}
