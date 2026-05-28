use hwr_hod_parser::hod::{HODModel, HODJoint, Matrix4};

fn main() {
    let mut model = HODModel::new();
    model.name = "New_Model".to_string();
    model.is_v2 = true;
    model.joints.push(HODJoint {
        name: "Root".to_string(),
        parent_name: Some("Root".to_string()),
        position: None,
        rotation: None,
        scale: None,
        local_transform: Matrix4 { m: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ] },
    });
    
    let original_bytes = Vec::new();
    let patched_bytes = hwr_hod_parser::hod::save_edits(&original_bytes, &model).unwrap();
    
    match hwr_hod_parser::hod::HODModel::parse_with_external(&patched_bytes, None, None) {
        Ok(parsed_model) => {
            println!("Joints parsed: {}", parsed_model.joints.len());
            for j in &parsed_model.joints {
                println!("- {}", j.name);
            }
        },
        Err(e) => println!("Error: {}", e),
    }
}
