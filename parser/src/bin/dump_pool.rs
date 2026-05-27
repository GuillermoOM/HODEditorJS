use hwr_hod_parser::hod::{HODModel, Vector3, Vector2};
use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: dump_pool <file.hod>");
        return;
    }
    let path = &args[1];
    let bytes = fs::read(path).unwrap();
    
    use hwr_hod_parser::iff::IffChunk;
    use std::io::Cursor;
    
    let mut reader = Cursor::new(&bytes);
    let mut pool_chunk = None;
    while let Ok(chunk) = IffChunk::read_chunk(&mut reader) {
        if chunk.id == "POOL" {
            pool_chunk = Some(chunk);
            break;
        }
    }
    
    if let Some(chunk) = pool_chunk {
        let mut pool_cursor = Cursor::new(&chunk.data);
        use byteorder::{LittleEndian, ReadBytesExt};
        let pool_type = pool_cursor.read_u32::<LittleEndian>().unwrap_or(0);
        let comp_tex_len = pool_cursor.read_u32::<LittleEndian>().unwrap_or(0);
        let decomp_tex_len = pool_cursor.read_u32::<LittleEndian>().unwrap_or(0);
        let mut comp_tex = vec![0u8; comp_tex_len as usize];
        std::io::Read::read_exact(&mut pool_cursor, &mut comp_tex).unwrap();
        
        let comp_mesh_len = pool_cursor.read_u32::<LittleEndian>().unwrap_or(0);
        let decomp_mesh_len = pool_cursor.read_u32::<LittleEndian>().unwrap_or(0);
        let mut comp_mesh = vec![0u8; comp_mesh_len as usize];
        std::io::Read::read_exact(&mut pool_cursor, &mut comp_mesh).unwrap();
        
        let comp_face_len = pool_cursor.read_u32::<LittleEndian>().unwrap_or(0);
        let decomp_face_len = pool_cursor.read_u32::<LittleEndian>().unwrap_or(0);
        
        println!("POOL type: {}", pool_type);
        println!("Tex pool: comp={}, decomp={}", comp_tex_len, decomp_tex_len);
        println!("Mesh pool: comp={}, decomp={}", comp_mesh_len, decomp_mesh_len);
        println!("Face pool: comp={}, decomp={}", comp_face_len, decomp_face_len);
    } else {
        println!("No POOL chunk found.");
    }
}
