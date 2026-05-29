use roxmltree::Document;
use std::fs;

fn main() -> Result<(), String> {
    let path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/mod-tools/HODEditorJS/testing/ter_centaur/ter_centaur.DAE";
    let xml = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let doc = Document::parse(&xml).map_err(|e| e.to_string())?;

    for geometry in doc
        .descendants()
        .filter(|node| node.has_tag_name("geometry"))
    {
        let geom_id = geometry.attribute("id").unwrap_or_default();
        if geom_id != "MULT[Root_mesh]_LOD[0]-lib" {
            continue;
        }

        let Some(mesh_node) = geometry.children().find(|node| node.has_tag_name("mesh")) else {
            continue;
        };

        // Extract UV0 float array
        let mut uv_data = Vec::new();
        if let Some(source) = mesh_node
            .children()
            .find(|node| node.attribute("id") == Some("MULT[Root_mesh]_LOD[0]-UV0"))
        {
            if let Some(float_array) = source
                .children()
                .find(|node| node.has_tag_name("float_array"))
            {
                if let Some(text) = float_array.text() {
                    uv_data = text
                        .split_whitespace()
                        .map(|s| s.parse::<f32>().unwrap_or(0.0))
                        .collect();
                }
            }
        }

        for triangles in mesh_node
            .children()
            .filter(|node| node.has_tag_name("triangles"))
        {
            let material = triangles.attribute("material").unwrap_or("none");
            if !material.starts_with("MAT[centaur]") {
                continue;
            }

            let mut position_offset = None;
            let mut normal_offset = None;
            let mut uv_offset = None;
            let mut max_offset = 0usize;
            for input in triangles
                .children()
                .filter(|node| node.has_tag_name("input"))
            {
                let offset = input
                    .attribute("offset")
                    .unwrap_or("0")
                    .parse::<usize>()
                    .unwrap_or(0);
                max_offset = max_offset.max(offset);
                match input.attribute("semantic") {
                    Some("VERTEX") => position_offset = Some(offset),
                    Some("NORMAL") => normal_offset = Some(offset),
                    Some("TEXCOORD") => uv_offset = Some(offset),
                    _ => {}
                }
            }
            let stride = max_offset + 1;

            for p in triangles.children().filter(|node| node.has_tag_name("p")) {
                let Some(text) = p.text() else {
                    continue;
                };
                let indices: Vec<usize> = text
                    .split_whitespace()
                    .map(|value| value.parse::<usize>().unwrap_or(0))
                    .collect();

                let mut tri_idx = 0;
                for chunk in indices.chunks(stride * 3) {
                    if chunk.len() < stride * 3 {
                        continue;
                    }
                    let uv0_idx = chunk[uv_offset.unwrap_or(0)];
                    let uv1_idx = chunk[stride + uv_offset.unwrap_or(0)];
                    let uv2_idx = chunk[stride * 2 + uv_offset.unwrap_or(0)];

                    let u0 = uv_data[uv0_idx * 2];
                    let v0 = uv_data[uv0_idx * 2 + 1];
                    let u1 = uv_data[uv1_idx * 2];
                    let v1 = uv_data[uv1_idx * 2 + 1];
                    let u2 = uv_data[uv2_idx * 2];
                    let v2 = uv_data[uv2_idx * 2 + 1];

                    let du1 = u1 - u0;
                    let dv1 = v1 - v0;
                    let du2 = u2 - u0;
                    let dv2 = v2 - v0;
                    let denom = du1 * dv2 - du2 * dv1;

                    let is_target =
                        tri_idx == (54 / 3) || tri_idx == (57 / 3) || tri_idx == (9 / 3);
                    if is_target {
                        println!("Triangle {}:", tri_idx);
                        println!("  UV0: ({:.6}, {:.6})", u0, v0);
                        println!("  UV1: ({:.6}, {:.6})", u1, v1);
                        println!("  UV2: ({:.6}, {:.6})", u2, v2);
                        println!("  Determinant (UV Area): {:.9}", denom);
                    }
                    tri_idx += 1;
                }
            }
        }
    }

    Ok(())
}
