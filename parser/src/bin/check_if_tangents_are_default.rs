use byteorder::{LittleEndian, ReadBytesExt};
use hwr_hod_parser::xpress;
use std::fs;
use std::io::{Cursor, Read};

fn main() -> Result<(), String> {
    let hodor_path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/mod-tools/HODEditorJS/testing/ter_centaur/ter_centaur_hodor.hod";
    let data = fs::read(hodor_path).unwrap();
    let mut cursor = Cursor::new(&data);

    let mut decomp_mesh = Vec::new();
    loop {
        if cursor.position() >= data.len() as u64 {
            break;
        }
        let mut id_bytes = [0u8; 4];
        cursor.read_exact(&mut id_bytes).unwrap();
        let id = String::from_utf8_lossy(&id_bytes).to_string();
        let size = cursor.read_u32::<byteorder::BigEndian>().unwrap();

        if id == "POOL" {
            let _ = cursor.read_u32::<LittleEndian>().unwrap();
            let comp_tex_len = cursor.read_u32::<LittleEndian>().unwrap() as usize;
            let decomp_tex_len = cursor.read_u32::<LittleEndian>().unwrap() as usize;
            let mut comp_tex = vec![0u8; comp_tex_len];
            cursor.read_exact(&mut comp_tex).unwrap();

            let comp_mesh_len = cursor.read_u32::<LittleEndian>().unwrap() as usize;
            let decomp_mesh_len = cursor.read_u32::<LittleEndian>().unwrap() as usize;
            let mut comp_mesh = vec![0u8; comp_mesh_len];
            cursor.read_exact(&mut comp_mesh).unwrap();
            decomp_mesh = xpress::decompress(&comp_mesh, decomp_mesh_len).unwrap();
            break;
        } else {
            let mut skip = vec![0u8; size as usize];
            cursor.read_exact(&mut skip).unwrap();
        }
    }

    let mut default_count = 0;
    let mut non_default_count = 0;
    let total_vertices = decomp_mesh.len() / 64;

    for v in 0..total_vertices {
        let off = v * 64;
        let tan_x = f32::from_le_bytes([
            decomp_mesh[off + 40],
            decomp_mesh[off + 41],
            decomp_mesh[off + 42],
            decomp_mesh[off + 43],
        ]);
        let tan_y = f32::from_le_bytes([
            decomp_mesh[off + 44],
            decomp_mesh[off + 45],
            decomp_mesh[off + 46],
            decomp_mesh[off + 47],
        ]);
        let tan_z = f32::from_le_bytes([
            decomp_mesh[off + 48],
            decomp_mesh[off + 49],
            decomp_mesh[off + 50],
            decomp_mesh[off + 51],
        ]);

        let bin_x = f32::from_le_bytes([
            decomp_mesh[off + 52],
            decomp_mesh[off + 53],
            decomp_mesh[off + 54],
            decomp_mesh[off + 55],
        ]);
        let bin_y = f32::from_le_bytes([
            decomp_mesh[off + 56],
            decomp_mesh[off + 57],
            decomp_mesh[off + 58],
            decomp_mesh[off + 59],
        ]);
        let bin_z = f32::from_le_bytes([
            decomp_mesh[off + 60],
            decomp_mesh[off + 61],
            decomp_mesh[off + 62],
            decomp_mesh[off + 63],
        ]);

        let is_tan_default =
            (tan_x - 1.0).abs() < 0.0001 && tan_y.abs() < 0.0001 && tan_z.abs() < 0.0001;
        let is_bin_default =
            bin_x.abs() < 0.0001 && bin_y.abs() < 0.0001 && (bin_z - 1.0).abs() < 0.0001;

        if is_tan_default && is_bin_default {
            default_count += 1;
        } else {
            non_default_count += 1;
        }
    }

    println!("Total vertices in HODOR mesh pool: {}", total_vertices);
    println!("  Default tangent/binormal count:   {}", default_count);
    println!(
        "  Non-default tangent/binormal count: {}",
        non_default_count
    );

    Ok(())
}
