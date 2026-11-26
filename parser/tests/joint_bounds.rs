//! HIER sx/sy/sz are bounds/angles, not scale: the joint matrix must be unit scale while the
//! original floats stay in `joint.scale` and survive a save.
use hwr_hod_parser::hod::HODModel;
use std::path::PathBuf;

fn vanilla(name: &str) -> Option<PathBuf> {
    let env = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.omo/local-environment.md");
    let text = std::fs::read_to_string(env).ok()?;
    let line = text.lines().find(|l| l.contains("HOD 2.0 (vanilla game)"))?;
    let p = PathBuf::from(line.split('`').nth(1)?).join(format!("{name}/{name}.hod"));
    p.exists().then_some(p)
}

fn row_len(m: &[[f32; 4]; 4], r: usize) -> f32 {
    (m[r][0] * m[r][0] + m[r][1] * m[r][1] + m[r][2] * m[r][2]).sqrt()
}

#[test]
fn bounds_do_not_scale_the_joint_matrix_and_survive_save() {
    let Some(path) = vanilla("vgr_battlecruiser") else {
        eprintln!("SKIP: vanilla fixture not available");
        return;
    };
    let bytes = std::fs::read(&path).unwrap();
    let model = HODModel::parse(&bytes).unwrap();
    let nozzle = model.joints.iter().find(|j| j.name == "EngineNozzle1").unwrap();
    let b = nozzle.scale.as_ref().unwrap();
    assert!((b.x - std::f32::consts::PI).abs() < 1e-4 && (b.z - std::f32::consts::PI).abs() < 1e-4, "bounds kept: {b:?}");
    for r in 0..3 {
        assert!((row_len(&nozzle.local_transform.m, r) - 1.0).abs() < 1e-4, "row {r} must be unit length");
    }

    let ui: HODModel = serde_json::from_str(&serde_json::to_string(&model).unwrap()).unwrap();
    let saved = hwr_hod_parser::hod::generate_v2_from_model(&bytes, &ui).unwrap();
    let re = HODModel::parse(&saved).unwrap();
    let n2 = re.joints.iter().find(|j| j.name == "EngineNozzle1").unwrap();
    let b2 = n2.scale.as_ref().unwrap();
    assert!((b2.x - b.x).abs() < 1e-6 && (b2.y - b.y).abs() < 1e-6 && (b2.z - b.z).abs() < 1e-6, "bounds written back unchanged");
}
