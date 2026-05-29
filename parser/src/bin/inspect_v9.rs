use byteorder::{LittleEndian, ReadBytesExt};
use hwr_hod_parser::xpress;
use std::fs;
use std::io::{Cursor, Read};

fn main() -> Result<(), String> {
    let hodor_path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/mod-tools/HODEditorJS/testing/ter_centaur/ter_centaur_hodor.hod";
    let hodor_data = fs::read(hodor_path).unwrap();
    let mut cursor = Cursor::new(&hodor_data);

    let mut hodor_mesh = Vec::new();
    loop {
        if cursor.position() >= hodor_data.len() as u64 {
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
            hodor_mesh = xpress::decompress(&comp_mesh, decomp_mesh_len).unwrap();
            break;
        } else {
            let mut skip = vec![0u8; size as usize];
            cursor.read_exact(&mut skip).unwrap();
        }
    }

    let gen_path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/mod-tools/HODEditorJS/testing/ter_centaur/ter_centaur_generated.hod";
    let gen_data = fs::read(gen_path).unwrap();
    let mut cursor2 = Cursor::new(&gen_data);

    let mut gen_mesh = Vec::new();
    loop {
        if cursor2.position() >= gen_data.len() as u64 {
            break;
        }
        let mut id_bytes = [0u8; 4];
        cursor2.read_exact(&mut id_bytes).unwrap();
        let id = String::from_utf8_lossy(&id_bytes).to_string();
        let size = cursor2.read_u32::<byteorder::BigEndian>().unwrap();

        if id == "POOL" {
            let _ = cursor2.read_u32::<LittleEndian>().unwrap();
            let comp_tex_len = cursor2.read_u32::<LittleEndian>().unwrap() as usize;
            let decomp_tex_len = cursor2.read_u32::<LittleEndian>().unwrap() as usize;
            let mut comp_tex = vec![0u8; comp_tex_len];
            cursor2.read_exact(&mut comp_tex).unwrap();

            let comp_mesh_len = cursor2.read_u32::<LittleEndian>().unwrap() as usize;
            let decomp_mesh_len = cursor2.read_u32::<LittleEndian>().unwrap() as usize;
            let mut comp_mesh = vec![0u8; comp_mesh_len];
            cursor2.read_exact(&mut comp_mesh).unwrap();
            gen_mesh = xpress::decompress(&comp_mesh, decomp_mesh_len).unwrap();
            break;
        } else {
            let mut skip = vec![0u8; size as usize];
            cursor2.read_exact(&mut skip).unwrap();
        }
    }

    let idx = 9;
    let h_off = idx * 64;
    let g_off = idx * 64;

    let h_pos = [
        f32::from_le_bytes([
            hodor_mesh[h_off],
            hodor_mesh[h_off + 1],
            hodor_mesh[h_off + 2],
            hodor_mesh[h_off + 3],
        ]),
        f32::from_le_bytes([
            hodor_mesh[h_off + 4],
            hodor_mesh[h_off + 5],
            hodor_mesh[h_off + 6],
            hodor_mesh[h_off + 7],
        ]),
        f32::from_le_bytes([
            hodor_mesh[h_off + 8],
            hodor_mesh[h_off + 9],
            hodor_mesh[h_off + 10],
            hodor_mesh[h_off + 11],
        ]),
    ];

    let h_nor = [
        f32::from_le_bytes([
            hodor_mesh[h_off + 16],
            hodor_mesh[h_off + 17],
            hodor_mesh[h_off + 18],
            hodor_mesh[h_off + 19],
        ]),
        f32::from_le_bytes([
            hodor_mesh[h_off + 20],
            hodor_mesh[h_off + 21],
            hodor_mesh[h_off + 22],
            hodor_mesh[h_off + 23],
        ]),
        f32::from_le_bytes([
            hodor_mesh[h_off + 24],
            hodor_mesh[h_off + 25],
            hodor_mesh[h_off + 26],
            hodor_mesh[h_off + 27],
        ]),
    ];

    let h_tan = [
        f32::from_le_bytes([
            hodor_mesh[h_off + 40],
            hodor_mesh[h_off + 41],
            hodor_mesh[h_off + 42],
            hodor_mesh[h_off + 43],
        ]),
        f32::from_le_bytes([
            hodor_mesh[h_off + 44],
            hodor_mesh[h_off + 45],
            hodor_mesh[h_off + 46],
            hodor_mesh[h_off + 47],
        ]),
        f32::from_le_bytes([
            hodor_mesh[h_off + 48],
            hodor_mesh[h_off + 49],
            hodor_mesh[h_off + 50],
            hodor_mesh[h_off + 51],
        ]),
    ];

    let h_bin = [
        f32::from_le_bytes([
            hodor_mesh[h_off + 52],
            hodor_mesh[h_off + 53],
            hodor_mesh[h_off + 54],
            hodor_mesh[h_off + 55],
        ]),
        f32::from_le_bytes([
            hodor_mesh[h_off + 56],
            hodor_mesh[h_off + 57],
            hodor_mesh[h_off + 58],
            hodor_mesh[h_off + 59],
        ]),
        f32::from_le_bytes([
            hodor_mesh[h_off + 60],
            hodor_mesh[h_off + 61],
            hodor_mesh[h_off + 62],
            hodor_mesh[h_off + 63],
        ]),
    ];

    let g_bin = [
        f32::from_le_bytes([
            gen_mesh[g_off + 52],
            gen_mesh[g_off + 53],
            gen_mesh[g_off + 54],
            gen_mesh[g_off + 55],
        ]),
        f32::from_le_bytes([
            gen_mesh[g_off + 56],
            gen_mesh[g_off + 57],
            gen_mesh[g_off + 58],
            gen_mesh[g_off + 59],
        ]),
        f32::from_le_bytes([
            gen_mesh[g_off + 60],
            gen_mesh[g_off + 61],
            gen_mesh[g_off + 62],
            gen_mesh[g_off + 63],
        ]),
    ];

    println!("Vertex 9 details:");
    println!("  Position: {:?}", h_pos);
    println!("  Normal:   {:?}", h_nor);
    println!("  Tangent:  {:?}", h_tan);
    println!("  HODOR Binormal:     {:?}", h_bin);
    println!("  Generated Binormal: {:?}", g_bin);

    // Check cross product of normal and tangent
    let cross = [
        h_nor[1] * h_tan[2] - h_nor[2] * h_tan[1],
        h_nor[2] * h_tan[0] - h_nor[0] * h_tan[2],
        h_nor[0] * h_tan[1] - h_nor[1] * h_tan[0],
    ];
    println!("  Normal x Tangent cross product: {:?}", cross);

    Ok(())
}
