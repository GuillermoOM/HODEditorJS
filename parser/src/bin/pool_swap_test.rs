/// Creates hybrid HOD files that swap individual POOL streams between HODOR and generated.
/// This isolates which compressed stream causes the in-game spikiness.
/// 
/// Usage: pool_swap_test <hodor_hod> <generated_hod> <output_dir>
/// Creates 3 files:
///   hybrid_tex_from_hodor.hod  - texture pool from HODOR, mesh/face from generated
///   hybrid_mesh_from_hodor.hod - mesh pool from HODOR, texture/face from generated
///   hybrid_face_from_hodor.hod - face pool from HODOR, texture/mesh from generated

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use hwr_hod_parser::iff::IffChunk;
use std::io::{Cursor, Seek, SeekFrom};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: pool_swap_test <hodor_hod> <generated_hod> <output_dir>");
        std::process::exit(1);
    }

    let hodor_bytes = std::fs::read(&args[1])?;
    let gen_bytes = std::fs::read(&args[2])?;
    let output_dir = &args[3];

    let hodor_pool = extract_pool(&hodor_bytes)?;
    let gen_pool = extract_pool(&gen_bytes)?;

    println!("HODOR POOL: tex={} comp/{} decomp, mesh={} comp/{} decomp, face={} comp/{} decomp",
        hodor_pool.tex_comp.len(), hodor_pool.tex_decomp.len(),
        hodor_pool.mesh_comp.len(), hodor_pool.mesh_decomp.len(),
        hodor_pool.face_comp.len(), hodor_pool.face_decomp.len());
    println!("Generated POOL: tex={} comp/{} decomp, mesh={} comp/{} decomp, face={} comp/{} decomp",
        gen_pool.tex_comp.len(), gen_pool.tex_decomp.len(),
        gen_pool.mesh_comp.len(), gen_pool.mesh_decomp.len(),
        gen_pool.face_comp.len(), gen_pool.face_decomp.len());

    // Create hybrid: swap each pool individually
    let hybrids = vec![
        ("hybrid_tex_from_hodor", &hodor_pool.tex_comp, hodor_pool.tex_decomp.len(), &gen_pool.mesh_comp, gen_pool.mesh_decomp.len(), &gen_pool.face_comp, gen_pool.face_decomp.len()),
        ("hybrid_mesh_from_hodor", &gen_pool.tex_comp, gen_pool.tex_decomp.len(), &hodor_pool.mesh_comp, hodor_pool.mesh_decomp.len(), &gen_pool.face_comp, gen_pool.face_decomp.len()),
        ("hybrid_face_from_hodor", &gen_pool.tex_comp, gen_pool.tex_decomp.len(), &gen_pool.mesh_comp, gen_pool.mesh_decomp.len(), &hodor_pool.face_comp, hodor_pool.face_decomp.len()),
        ("hybrid_all_from_hodor", &hodor_pool.tex_comp, hodor_pool.tex_decomp.len(), &hodor_pool.mesh_comp, hodor_pool.mesh_decomp.len(), &hodor_pool.face_comp, hodor_pool.face_decomp.len()),
        ("hybrid_mesh_uncompressed", &gen_pool.tex_comp, gen_pool.tex_decomp.len(), &gen_pool.mesh_decomp, gen_pool.mesh_decomp.len(), &gen_pool.face_comp, gen_pool.face_decomp.len()),
    ];

    for (name, tex_comp, tex_decomp_len, mesh_comp, mesh_decomp_len, face_comp, face_decomp_len) in hybrids {
        let mut pool_data = Vec::new();
        pool_data.write_u32::<LittleEndian>(3518)?; // pool_type
        pool_data.write_u32::<LittleEndian>(tex_comp.len() as u32)?;
        pool_data.write_u32::<LittleEndian>(tex_decomp_len as u32)?;
        pool_data.extend_from_slice(tex_comp);
        pool_data.write_u32::<LittleEndian>(mesh_comp.len() as u32)?;
        pool_data.write_u32::<LittleEndian>(mesh_decomp_len as u32)?;
        pool_data.extend_from_slice(mesh_comp);
        pool_data.write_u32::<LittleEndian>(face_comp.len() as u32)?;
        pool_data.write_u32::<LittleEndian>(face_decomp_len as u32)?;
        pool_data.extend_from_slice(face_comp);

        // Build the HOD with the new pool data
        let output = replace_pool(&gen_bytes, &pool_data)?;
        let output_path = format!("{}/{}.hod", output_dir, name);
        std::fs::write(&output_path, &output)?;
        println!("Created: {} ({} bytes)", output_path, output.len());
    }

    Ok(())
}

struct PoolStreams {
    tex_comp: Vec<u8>,
    tex_decomp: Vec<u8>,
    mesh_comp: Vec<u8>,
    mesh_decomp: Vec<u8>,
    face_comp: Vec<u8>,
    face_decomp: Vec<u8>,
}

fn extract_pool(bytes: &[u8]) -> Result<PoolStreams, Box<dyn std::error::Error>> {
    let mut cursor = Cursor::new(bytes);
    while cursor.position() < bytes.len() as u64 {
        let chunk = IffChunk::read_chunk(&mut cursor)?;
        if chunk.id == "POOL" {
            let mut pc = Cursor::new(&chunk.data);
            let _pool_type = pc.read_u32::<LittleEndian>()?;
            let comp_tex_len = pc.read_u32::<LittleEndian>()? as usize;
            let decomp_tex_len = pc.read_u32::<LittleEndian>()? as usize;
            let tex_start = pc.position() as usize;
            let tex_comp = chunk.data[tex_start..tex_start + comp_tex_len].to_vec();
            pc.seek(SeekFrom::Start((tex_start + comp_tex_len) as u64))?;

            let comp_mesh_len = pc.read_u32::<LittleEndian>()? as usize;
            let decomp_mesh_len = pc.read_u32::<LittleEndian>()? as usize;
            let mesh_start = pc.position() as usize;
            let mesh_comp = chunk.data[mesh_start..mesh_start + comp_mesh_len].to_vec();
            pc.seek(SeekFrom::Start((mesh_start + comp_mesh_len) as u64))?;

            let comp_face_len = pc.read_u32::<LittleEndian>()? as usize;
            let decomp_face_len = pc.read_u32::<LittleEndian>()? as usize;
            let face_start = pc.position() as usize;
            let face_comp = chunk.data[face_start..face_start + comp_face_len].to_vec();

            let tex_decomp = if comp_tex_len == decomp_tex_len {
                tex_comp.clone()
            } else {
                hwr_hod_parser::xpress::decompress(&tex_comp, decomp_tex_len)?
            };
            let mesh_decomp = if comp_mesh_len == decomp_mesh_len {
                mesh_comp.clone()
            } else {
                hwr_hod_parser::xpress::decompress(&mesh_comp, decomp_mesh_len)?
            };
            let face_decomp = if comp_face_len == decomp_face_len {
                face_comp.clone()
            } else {
                hwr_hod_parser::xpress::decompress(&face_comp, decomp_face_len)?
            };

            return Ok(PoolStreams {
                tex_comp, tex_decomp,
                mesh_comp, mesh_decomp,
                face_comp, face_decomp,
            });
        }
    }
    Err("No POOL chunk found".into())
}

fn replace_pool(original_bytes: &[u8], new_pool_data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut output = Vec::new();
    let mut cursor = Cursor::new(original_bytes);
    
    while cursor.position() < original_bytes.len() as u64 {
        let pos = cursor.position() as usize;
        let chunk = IffChunk::read_chunk(&mut cursor)?;
        let chunk_end = cursor.position() as usize;
        
        if chunk.id == "POOL" {
            // Write new POOL chunk
            output.extend_from_slice(b"POOL");
            output.write_u32::<LittleEndian>(new_pool_data.len() as u32)?;
            output.extend_from_slice(new_pool_data);
        } else {
            // Copy original chunk as-is
            output.extend_from_slice(&original_bytes[pos..chunk_end]);
        }
    }
    
    Ok(output)
}
