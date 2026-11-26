//! The COLD collision mesh lives in the POOL (TRIS holds only counts). It must be read
//! on load and written back on save, with the KDOP regenerated from the real mesh.
use hwr_hod_parser::hod::HODModel;
use hwr_hod_parser::iff::IffChunk;
use std::io::Cursor;
use std::path::PathBuf;

fn vanilla(name: &str) -> Option<PathBuf> {
    let env = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.omo/local-environment.md");
    let text = std::fs::read_to_string(env).ok()?;
    let line = text.lines().find(|l| l.contains("HOD 2.0 (vanilla game)"))?;
    let p = PathBuf::from(line.split('`').nth(1)?).join(format!("{name}/{name}.hod"));
    p.exists().then_some(p)
}

fn dtrm_child(bytes: &[u8], id: &str) -> Option<IffChunk> {
    let mut c = Cursor::new(bytes);
    while (c.position() as usize) < bytes.len() {
        let ch = IffChunk::read_chunk(&mut c).ok()?;
        if ch.id == "DTRM" {
            return ch.children.into_iter().find(|k| k.id == id);
        }
    }
    None
}

#[test]
fn cold_mesh_is_read_from_pool_and_round_trips() {
    let Some(path) = vanilla("hgn_assaultfrigate") else {
        eprintln!("SKIP: vanilla fixture not available");
        return;
    };
    let bytes = std::fs::read(&path).unwrap();
    let model = HODModel::parse(&bytes).unwrap();
    let cm = &model.collision_meshes[0];
    let part = &cm.mesh.parts[0];
    let tris = dtrm_child(&bytes, "COLD").unwrap().children.into_iter().find(|c| c.id == "TRIS").unwrap();
    let rd = |o: usize| i32::from_le_bytes(tris.data[o..o + 4].try_into().unwrap()) as usize;
    assert_eq!((part.vertices.len(), part.indices.len()), (rd(0), rd(4)), "real COLD mesh, not the KDOP hull");

    let ui: HODModel = serde_json::from_str(&serde_json::to_string(&model).unwrap()).unwrap();
    let saved = hwr_hod_parser::hod::generate_v2_from_model(&bytes, &ui).unwrap();
    let re = HODModel::parse(&saved).unwrap();
    let p2 = &re.collision_meshes[0].mesh.parts[0];
    assert_eq!(p2.vertices.len(), part.vertices.len());
    assert_eq!(p2.indices, part.indices);
    for (a, b) in p2.vertices.iter().zip(&part.vertices) {
        assert_eq!((a.position.x, a.position.y, a.position.z), (b.position.x, b.position.y, b.position.z));
    }
    let k0 = dtrm_child(&bytes, "KDOP").unwrap().data;
    let k1 = dtrm_child(&saved, "KDOP").unwrap().data;
    // The engine only reads the KDOP min/max (joint OBB): they must equal the vanilla ones,
    // i.e. the bbox of the visible LOD0 meshes (not of the collision mesh).
    let f = |d: &[u8], i: usize| f32::from_le_bytes(d[i * 4..i * 4 + 4].try_into().unwrap());
    for i in 1..7 {
        let (a, b) = (f(&k0, i), f(&k1, i));
        assert!((a - b).abs() <= 1e-4 * (1.0 + a.abs()), "KDOP min/max word {i}: {a} vs {b}");
    }
    println!("KDOP original {} B, regenerated {} B, identical: {}", k0.len(), k1.len(), k0 == k1);
}
