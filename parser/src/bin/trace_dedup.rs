use hwr_hod_parser::dae::parse_dae;
use hwr_hod_parser::hod::HODVertex;
use std::fs;

fn main() {
    let xml = fs::read_to_string("../testing/ter_centaur/ter_centaur.DAE").unwrap();
    let model = parse_dae(&xml).unwrap();

    // Find the centaur LOD0 mesh
    for mesh in &model.meshes {
        if !mesh.name.contains("Root_mesh") || mesh.lod != 0 { continue; }
        println!("Mesh '{}' LOD {} parts: {}", mesh.name, mesh.lod, mesh.parts.len());
        for (pi, part) in mesh.parts.iter().enumerate() {
            println!("  Part {}: verts={} indices={} material={} mask=0x{:04x}",
                pi, part.vertices.len(), part.indices.len(), part.material_index, part.vertex_mask);
            
            if pi == 0 {
                // Print first 30 vertices of part 0 (centaur)
                println!("  First 30 part 0 vertices:");
                for i in 0..30.min(part.vertices.len()) {
                    let v = &part.vertices[i];
                    println!("    [{}]: pos=({:.6},{:.6},{:.6}) norm=({:.6},{:.6},{:.6}) uv=({:.6},{:.6}) tan=({:.1},{:.1},{:.1}) bin=({:.1},{:.1},{:.1}) idx={}",
                        i,
                        v.position.x, v.position.y, v.position.z,
                        v.normal.as_ref().map_or(0.0, |n| n.x),
                        v.normal.as_ref().map_or(0.0, |n| n.y),
                        v.normal.as_ref().map_or(0.0, |n| n.z),
                        v.uv.as_ref().map_or(0.0, |u| u.u),
                        v.uv.as_ref().map_or(0.0, |u| u.v),
                        v.tangent.as_ref().map_or(0.0, |t| t.x),
                        v.tangent.as_ref().map_or(0.0, |t| t.y),
                        v.tangent.as_ref().map_or(0.0, |t| t.z),
                        v.binormal.as_ref().map_or(0.0, |b| b.x),
                        v.binormal.as_ref().map_or(0.0, |b| b.y),
                        v.binormal.as_ref().map_or(0.0, |b| b.z),
                        part.indices[i]);
                }
                
                // Check: vertex 0 position
                let v0 = &part.vertices[0];
                println!("\n  DAE vertex 0: pos=({:.6},{:.6},{:.6})", v0.position.x, v0.position.y, v0.position.z);
                
                // Check: what position does index 21 reference?
                let idx21 = part.indices[21] as usize;
                let v21 = &part.vertices[idx21];
                println!("  DAE index 21 -> vertex {}: pos=({:.6},{:.6},{:.6})", idx21, v21.position.x, v21.position.y, v21.position.z);
                
                // Check: what position does index 9 reference?
                let idx9 = part.indices[9] as usize;
                let v9 = &part.vertices[idx9];
                println!("  DAE index 9 -> vertex {}: pos=({:.6},{:.6},{:.6})", idx9, v9.position.x, v9.position.y, v9.position.z);
                
                // Count unique positions
                let mut pos_set: std::collections::HashSet<[u32; 3]> = std::collections::HashSet::new();
                for v in &part.vertices {
                    pos_set.insert([v.position.x.to_bits(), v.position.y.to_bits(), v.position.z.to_bits()]);
                }
                println!("\n  Unique DAE positions: {}", pos_set.len());
            }
        }
    }
}
