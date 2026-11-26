//! Regenerates SCAR for a vanilla HOD and compares it with the original chunk and
//! pool tails. Needs a vanilla ship with a SCAR chunk (default `hgn_destroyer`); set
//! `HODE_SCAR_FIXTURE` or provide the `HOD 2.0 (vanilla game)` path in the local
//! environment file. Skips (passes) when the fixture is unavailable.
use byteorder::{LittleEndian, ReadBytesExt};
use hwr_hod_parser::hod::HODModel;
use hwr_hod_parser::iff::IffChunk;
use hwr_hod_parser::scar::{generate_scar, generate_scar_with_dims};
use hwr_hod_parser::xpress;
use std::io::{Cursor, Read};
use std::path::PathBuf;

fn fixture() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("HODE_SCAR_FIXTURE") {
        return Some(PathBuf::from(p));
    }
    let env = std::env::var_os("HODE_LOCAL_ENV")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.omo/local-environment.md"));
    let text = std::fs::read_to_string(env).ok()?;
    let line = text.lines().find(|l| l.contains("HOD 2.0 (vanilla game)"))?;
    let dir = line.split('`').nth(1)?;
    let p = PathBuf::from(dir).join("hgn_destroyer/hgn_destroyer.hod");
    p.exists().then_some(p)
}

fn pools(bytes: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let mut c = Cursor::new(bytes);
    while (c.position() as usize) < bytes.len() {
        let ch = IffChunk::read_chunk(&mut c).unwrap();
        if ch.id == "POOL" {
            let mut p = Cursor::new(&ch.data);
            let _ = p.read_u32::<LittleEndian>().unwrap();
            let mut out = Vec::new();
            for i in 0..3 {
                let cl = p.read_u32::<LittleEndian>().unwrap() as usize;
                let dl = p.read_u32::<LittleEndian>().unwrap() as usize;
                let mut b = vec![0u8; cl];
                p.read_exact(&mut b).unwrap();
                let d = if cl == dl { b } else { xpress::decompress(&b, dl).unwrap() };
                if i > 0 {
                    out.push(d);
                }
            }
            return (out.remove(0), out.remove(0));
        }
    }
    panic!("no POOL");
}

struct Layout {
    names_end: usize,
    counts: usize,
    nv: usize,
    nn: usize,
    nt: usize,
    nl: usize,
    grid: usize,
    dims: [usize; 3],
}

fn layout(d: &[u8]) -> Layout {
    let rd = |o: usize| u32::from_le_bytes(d[o..o + 4].try_into().unwrap()) as usize;
    let name_len = rd(0);
    let par_len = rd(4 + name_len);
    let names_end = 8 + name_len + par_len;
    // flag(4) + min(12) + max(12) + center(12) + radius(4)
    let counts = names_end + 44;
    let (nv, nn, nt, nl) = (rd(counts), rd(counts + 4), rd(counts + 8), rd(counts + 12));
    let grid = counts + 16 + nl * 2;
    Layout { names_end, counts, nv, nn, nt, nl, grid, dims: [rd(grid), rd(grid + 4), rd(grid + 8)] }
}

#[test]
fn regenerated_scar_matches_vanilla() {
    let Some(path) = fixture() else {
        eprintln!("SKIP: vanilla SCAR fixture not available");
        return;
    };
    let bytes = std::fs::read(&path).unwrap();
    let model = HODModel::parse(&bytes).unwrap();
    assert!(!model.scar_meshes.is_empty(), "fixture must contain a SCAR chunk");
    let orig = model.preserved_chunks.iter().find(|c| c.id == "SCAR").unwrap().data.clone();
    let (omesh, oface) = pools(&bytes);
    let lo = layout(&orig);

    // Auto-chosen grid: same structure, any valid resolution.
    let auto = generate_scar(&model).expect("scar generated");
    let la = layout(&auto.chunk);
    println!("dims original {:?} auto {:?}; nl {} vs {}", lo.dims, la.dims, lo.nl, la.nl);
    assert_eq!((la.nv, la.nn, la.nt), (lo.nv, lo.nn, lo.nt));

    // Force the vanilla grid resolution so cell lists are comparable byte for byte.
    let g = generate_scar_with_dims(&model, Some(lo.dims.map(|v| v as u32))).expect("scar generated");
    let lg = layout(&g.chunk);
    assert_eq!(&orig[..lo.names_end], &g.chunk[..lg.names_end], "mesh/parent names");
    // flag + bbox min/max (center, radius and trailing id are free-form).
    assert_eq!(&orig[lo.names_end..lo.names_end + 28], &g.chunk[lg.names_end..lg.names_end + 28], "flag + bbox");
    assert_eq!(&orig[lo.counts..lo.counts + 12], &g.chunk[lg.counts..lg.counts + 12], "nv/nn/nt");
    let mut mismatched_cells = 0;
    let cells = lo.dims[0] * lo.dims[1] * lo.dims[2];
    let rd16 = |d: &[u8], o: usize| u16::from_le_bytes(d[o..o + 2].try_into().unwrap());
    let (mut oo, mut go) = (lo.counts + 16, lg.counts + 16);
    for c in 0..cells {
        let (oc, gc) = (rd16(&orig, lo.grid + 12 + c * 2) as usize, rd16(&g.chunk, lg.grid + 12 + c * 2) as usize);
        let mut a: Vec<u16> = (0..oc).map(|i| rd16(&orig, oo + i * 2)).collect();
        let mut b: Vec<u16> = (0..gc).map(|i| rd16(&g.chunk, go + i * 2)).collect();
        if a != b {
            a.sort();
            b.sort();
            assert_eq!(a, b, "cell {c} content");
            println!("cell {c}: same set, different order");
        }
        if oc != gc {
            mismatched_cells += 1;
        }
        oo += oc * 2;
        go += gc * 2;
    }
    assert_eq!(mismatched_cells, 0);
    assert_eq!(lo.nl, lg.nl, "list length");
    // Pool arrays: positions, normals, triangle records.
    assert_eq!(&omesh[omesh.len() - (lo.nv + lo.nn) * 12..], &g.mesh_pool[..], "positions + normals");
    assert_eq!(&oface[oface.len() - lo.nt * 12..], &g.face_pool[..], "triangle records");
}

/// Simulates the UI save path (model round-tripped through JSON, `preserved_chunks`
/// dropped) and checks the saved HOD carries a complete, correctly placed SCAR.
#[test]
fn ui_save_path_keeps_scar() {
    let Some(path) = fixture() else {
        eprintln!("SKIP: vanilla SCAR fixture not available");
        return;
    };
    let bytes = std::fs::read(&path).unwrap();
    let model = HODModel::parse(&bytes).unwrap();
    let ui_model: HODModel = serde_json::from_str(&serde_json::to_string(&model).unwrap()).unwrap();
    assert!(ui_model.preserved_chunks.is_empty());
    assert!(!ui_model.scar_meshes.is_empty(), "scar_meshes must survive the JSON round trip");

    let saved = hwr_hod_parser::hod::generate_v2_from_model(&bytes, &ui_model).unwrap();
    let reparsed = HODModel::parse(&saved).unwrap();
    assert!(!reparsed.scar_meshes.is_empty(), "saved HOD lost its SCAR chunk");
    let scar = reparsed.preserved_chunks.iter().find(|c| c.id == "SCAR").unwrap();
    let l = layout(&scar.data);
    let (mesh, face) = pools(&saved);
    let want = generate_scar(&ui_model).unwrap();
    assert_eq!(scar.data, want.chunk, "saved SCAR chunk is the regenerated one");
    assert!(mesh.ends_with(&want.mesh_pool), "SCAR positions/normals are the mesh pool tail");
    assert!(face.ends_with(&want.face_pool), "SCAR triangles are the face pool tail");
    assert_eq!(l.nt * 12, want.face_pool.len());
    // SCAR must follow COLD in DTRM (pool streams are consumed in chunk order).
    let mut c = Cursor::new(&saved[..]);
    let mut order = Vec::new();
    while (c.position() as usize) < saved.len() {
        let ch = IffChunk::read_chunk(&mut c).unwrap();
        if ch.id == "DTRM" {
            order = ch.children.iter().map(|k| k.id.clone()).collect();
        }
    }
    let cold = order.iter().position(|i| i == "COLD").expect("COLD present");
    let sc = order.iter().position(|i| i == "SCAR").expect("SCAR present");
    assert!(sc > cold, "DTRM order {:?}", order);
}

/// HODOR-built mods carry one SCAR per mesh (`ter_elysium`: barrel/turret/Root_mesh).
/// Every chunk must survive a UI save, in order, with its arrays stacked in the pools.
#[test]
fn multi_scar_mod_ship_keeps_every_scar() {
    let Some(path) = std::env::var_os("HODE_MULTI_SCAR_FIXTURE").map(PathBuf::from).or_else(|| {
        let env = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.omo/local-environment.md");
        let text = std::fs::read_to_string(env).ok()?;
        let line = text.lines().find(|l| l.contains("`ter_elysium.hod`"))?;
        let p = PathBuf::from(line.split_once("`ter_elysium.hod`: ")?.1.trim().trim_matches('`'));
        p.exists().then_some(p)
    }) else {
        eprintln!("SKIP: multi-SCAR fixture not available");
        return;
    };
    let bytes = std::fs::read(&path).unwrap();
    let model = HODModel::parse(&bytes).unwrap();
    let n = model.preserved_chunks.iter().filter(|c| c.id == "SCAR").count();
    assert!(n > 1, "fixture should have several SCAR chunks");
    assert_eq!(model.scar_meshes.len(), n);

    let ui_model: HODModel = serde_json::from_str(&serde_json::to_string(&model).unwrap()).unwrap();
    let saved = hwr_hod_parser::hod::generate_v2_from_model(&bytes, &ui_model).unwrap();
    let reparsed = HODModel::parse(&saved).unwrap();
    assert_eq!(reparsed.scar_meshes, model.scar_meshes, "same SCARs, same order");

    // Each original chunk must be reproduced (vertex, normal and triangle counts).
    let chunks: Vec<_> = reparsed.preserved_chunks.iter().filter(|c| c.id == "SCAR").collect();
    let origs: Vec<_> = model.preserved_chunks.iter().filter(|c| c.id == "SCAR").collect();
    let (mesh, face) = pools(&saved);
    let mut tail_m = 0;
    let mut tail_f = 0;
    for (new, old) in chunks.iter().zip(&origs) {
        let (ln, lo) = (layout(&new.data), layout(&old.data));
        assert_eq!((ln.nv, ln.nn, ln.nt), (lo.nv, lo.nn, lo.nt)); // nl depends on the (auto) grid dims
        tail_m += (ln.nv + ln.nn) * 12;
        tail_f += ln.nt * 12;
    }
    // Arrays of all SCARs occupy the end of the pools, back to back, in chunk order.
    let wants = hwr_hod_parser::scar::generate_scars(&ui_model);
    assert_eq!(wants.len(), n);
    let all_m: Vec<u8> = wants.iter().flat_map(|w| w.mesh_pool.clone()).collect();
    let all_f: Vec<u8> = wants.iter().flat_map(|w| w.face_pool.clone()).collect();
    assert_eq!((all_m.len(), all_f.len()), (tail_m, tail_f));
    assert!(mesh.ends_with(&all_m) && face.ends_with(&all_f));
}
