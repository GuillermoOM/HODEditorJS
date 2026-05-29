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

        // Extract POSITION float array
        let mut pos_data = Vec::new();
        if let Some(source) = mesh_node
            .children()
            .find(|node| node.attribute("id") == Some("MULT[Root_mesh]_LOD[0]-POSITION"))
        {
            if let Some(float_array) = source
                .children()
                .find(|node| node.has_tag_name("float_array"))
            {
                if let Some(text) = float_array.text() {
                    pos_data = text
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
                if input.attribute("semantic") == Some("VERTEX") {
                    position_offset = Some(offset);
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

                    if tri_idx == (54 / 3) {
                        let p0_idx = chunk[position_offset.unwrap_or(0)];
                        let p1_idx = chunk[stride + position_offset.unwrap_or(0)];
                        let p2_idx = chunk[stride * 2 + position_offset.unwrap_or(0)];

                        let x0 = pos_data[p0_idx * 3];
                        let y0 = pos_data[p0_idx * 3 + 1];
                        let z0 = pos_data[p0_idx * 3 + 2];
                        let x1 = pos_data[p1_idx * 3];
                        let y1 = pos_data[p1_idx * 3 + 1];
                        let z1 = pos_data[p1_idx * 3 + 2];
                        let x2 = pos_data[p2_idx * 3];
                        let y2 = pos_data[p2_idx * 3 + 1];
                        let z2 = pos_data[p2_idx * 3 + 2];

                        println!("Triangle 18 (pos 54, 55, 56):");
                        println!("  P0: ({:.6}, {:.6}, {:.6})", x0, y0, z0);
                        println!("  P1: ({:.6}, {:.6}, {:.6})", x1, y1, z1);
                        println!("  P2: ({:.6}, {:.6}, {:.6})", x2, y2, z2);
                    }
                    tri_idx += 1;
                }
            }
        }
    }

    Ok(())
}
