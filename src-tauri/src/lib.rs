use chrono::Local;
use hwr_hod_parser::hod::{HODModel, HODTexture};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;

#[derive(Serialize, Deserialize, Default)]
struct ShaderConfig {
    shader_directories: Vec<String>,
}

static LOG_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);

pub fn write_log(level: &str, message: &str) {
    let log_path = match LOG_PATH.lock() {
        Ok(g) => g.clone(),
        Err(_) => None,
    };

    let path = match log_path {
        Some(p) => p,
        None => {
            let mut p = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|parent| parent.to_path_buf()))
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
            p.push("hwr_hod_editor.log");
            p
        }
    };

    let now = Local::now().format("%Y-%m-%d %H:%M:%S");
    let log_line = format!("[{}] [{}] {}\n", now, level, message);

    if level == "ERROR" || level == "PANIC" {
        eprint!("{}", log_line);
    } else {
        print!("{}", log_line);
    }

    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&path)
    {
        let _ = file.write_all(log_line.as_bytes());
    }
}

fn setup_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let payload = info.payload();
        let message = if let Some(s) = payload.downcast_ref::<&str>() {
            *s
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.as_str()
        } else {
            "Unknown panic payload"
        };

        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".to_string());

        write_log(
            "PANIC",
            &format!("Rust panicked at {}: {}", location, message),
        );
    }));
}

#[tauri::command]
fn greet(name: &str) -> String {
    write_log("INFO", &format!("Greeted {}", name));
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn log_event(level: String, message: String) {
    write_log(&level, &message);
}

#[tauri::command]
fn select_hod_file() -> Result<Option<String>, String> {
    write_log("INFO", "Opening native file dialog...");
    let file = rfd::FileDialog::new()
        .add_filter("Homeworld HOD Files", &["hod"])
        .pick_file();

    match &file {
        Some(path) => {
            write_log(
                "INFO",
                &format!("User selected HOD file: {}", path.to_string_lossy()),
            );
            Ok(Some(path.to_string_lossy().to_string()))
        }
        None => {
            write_log("INFO", "User canceled file selection dialog");
            Ok(None)
        }
    }
}

#[tauri::command]
fn select_keeper_file() -> Result<Option<String>, String> {
    write_log("INFO", "Opening native file dialog for keeper.txt...");
    let file = rfd::FileDialog::new()
        .add_filter("Keeper Text file", &["txt"])
        .set_file_name("keeper.txt")
        .pick_file();

    match &file {
        Some(path) => {
            write_log(
                "INFO",
                &format!("User selected keeper file: {}", path.to_string_lossy()),
            );
            Ok(Some(path.to_string_lossy().to_string()))
        }
        None => {
            write_log("INFO", "User canceled keeper file selection dialog");
            Ok(None)
        }
    }
}

#[tauri::command]
fn select_dae_file() -> Result<Option<String>, String> {
    write_log("INFO", "Opening native file dialog for DAE file...");
    let file = rfd::FileDialog::new()
        .add_filter("Collada DAE Files", &["dae", "DAE"])
        .pick_file();

    match &file {
        Some(path) => {
            write_log(
                "INFO",
                &format!("User selected DAE file: {}", path.to_string_lossy()),
            );
            Ok(Some(path.to_string_lossy().to_string()))
        }
        None => {
            write_log("INFO", "User canceled DAE file selection dialog");
            Ok(None)
        }
    }
}

#[tauri::command]
fn load_hod(file_path: String, keeper_path: Option<String>) -> Result<HODModel, String> {
    write_log(
        "INFO",
        &format!(
            "Attempting to load HOD from path: {} with keeper_path: {:?}",
            file_path, keeper_path
        ),
    );

    // Read raw HOD bytes from file system
    let bytes = fs::read(&file_path).map_err(|e| {
        let err_msg = format!("Failed to read file: {}", e);
        write_log("ERROR", &err_msg);
        err_msg
    })?;

    write_log(
        "INFO",
        &format!(
            "Successfully read {} bytes from disk. Parsing HOD structures...",
            bytes.len()
        ),
    );

    // Parse the HOD structure using our native pure-Rust parser engine with external TGA texture support
    let mut model = HODModel::parse_with_external(&bytes, Some(&file_path), keeper_path.as_deref())
        .map_err(|e| {
            let err_msg = format!("Failed to parse HOD file: {}", e);
            write_log("ERROR", &err_msg);
            err_msg
        })?;

    // Auto-transform legacy HOD 1.0 models on open
    if !model.is_v2 {
        write_log("INFO", "HOD 1.0 detected on open. Automatically applying backward compatibility transformations (e.g., Engine Burns extraction)...");
        hwr_hod_parser::hod::synthesize_engine_nozzles_v1(&mut model);
        hwr_hod_parser::hod::validate_marker_parents(&mut model);
        model.is_v2 = true;
        model.version = 512;
    }

    model.auto_assign_and_resize_textures();

    write_log("INFO", &format!(
        "Successfully parsed HOD Model: '{}' | Meshes: {} | Joints: {} | Markers: {} | Materials: {} | Textures: {}",
        model.name, model.meshes.len(), model.joints.len(), model.markers.len(), model.materials.len(), model.textures.len()
    ));

    for joint in &model.joints {
        if joint.name.to_lowercase().contains("nozzle")
            || joint.name.to_lowercase().contains("engine")
        {
            write_log(
                "INFO",
                &format!(
                    "Joint DBG: Name='{}' | Parent='{:?}' | Translation=[{:.3}, {:.3}, {:.3}]",
                    joint.name,
                    joint.parent_name,
                    joint.local_transform.m[3][0],
                    joint.local_transform.m[3][1],
                    joint.local_transform.m[3][2],
                ),
            );
        }
    }

    Ok(model)
}

#[tauri::command]
fn save_hod(file_path: String, model: HODModel) -> Result<(), String> {
    write_log("INFO", &format!("Saving HOD edits to: {}", file_path));

    // 1. Read the original HOD file bytes (or use empty if new file)
    let original_bytes = if file_path.is_empty() || !std::path::Path::new(&file_path).exists() {
        Vec::new()
    } else {
        fs::read(&file_path).map_err(|e| {
            let err_msg = format!("Failed to read original HOD file: {}", e);
            write_log("ERROR", &err_msg);
            err_msg
        })?
    };

    write_log(
        "INFO",
        &format!(
            "Original HOD size: {} bytes. Patching chunks...",
            original_bytes.len()
        ),
    );

    // 2. Generate a fresh HOD 2.0 file from the in-memory model.
    // As per the architecture rules, we never patch the original file using save_edits.
    let patched_bytes = hwr_hod_parser::hod::generate_v2_from_model(&original_bytes, &model).map_err(|e| {
        let err_msg = format!("Failed to serialize HOD edits: {}", e);
        write_log("ERROR", &err_msg);
        err_msg
    })?;

    write_log(
        "INFO",
        &format!(
            "Successfully serialized edited HOD stream ({} bytes). Writing back to disk...",
            patched_bytes.len()
        ),
    );

    // 3. Write back the updated HOD file to disk
    fs::write(&file_path, &patched_bytes).map_err(|e| {
        let err_msg = format!("Failed to write patched HOD file: {}", e);
        write_log("ERROR", &err_msg);
        err_msg
    })?;

    // 4. Save companion .mad file if applicable
    if !model.animations.is_empty() {
        let hod_path = std::path::Path::new(&file_path);
        let mad_path = hod_path.with_extension("mad");
        write_log(
            "INFO",
            &format!("Writing companion .mad file: {:?}", mad_path),
        );
        match hwr_hod_parser::hod::serialize_mad_companion(&model) {
            Ok(mad_bytes) => {
                if let Err(e) = fs::write(&mad_path, &mad_bytes) {
                    write_log("ERROR", &format!("Failed to write .mad file: {}", e));
                } else {
                    write_log("INFO", "Companion .mad file successfully saved!");
                }
            }
            Err(e) => {
                write_log(
                    "ERROR",
                    &format!("Failed to serialize companion .mad: {}", e),
                );
            }
        }
    }

    write_log("INFO", "HOD file successfully patched and saved!");

    Ok(())
}

#[tauri::command]
fn select_save_hod_file(default_name: Option<String>) -> Result<Option<String>, String> {
    write_log("INFO", "Opening native save HOD file dialog...");
    let mut dialog = rfd::FileDialog::new().add_filter("Homeworld HOD Files", &["hod"]);

    if let Some(name) = default_name {
        dialog = dialog.set_file_name(&name);
    }

    let file = dialog.save_file();

    match &file {
        Some(path) => {
            write_log(
                "INFO",
                &format!("User selected save HOD path: {}", path.to_string_lossy()),
            );
            Ok(Some(path.to_string_lossy().to_string()))
        }
        None => {
            write_log("INFO", "User canceled save HOD file dialog");
            Ok(None)
        }
    }
}

#[tauri::command]
fn save_hod_as(
    source_path: String,
    target_path: String,
    model: HODModel,
) -> Result<(), String> {
    write_log(
        "INFO",
        &format!("Saving HOD edits as: {} -> {}", source_path, target_path),
    );

    let original_bytes = if source_path.is_empty() || !std::path::Path::new(&source_path).exists() {
        Vec::new()
    } else {
        fs::read(&source_path).map_err(|e| {
            let err_msg = format!("Failed to read original HOD file from source: {}", e);
            write_log("ERROR", &err_msg);
            err_msg
        })?
    };

    let patched_bytes = hwr_hod_parser::hod::generate_v2_from_model(&original_bytes, &model).map_err(|e| {
        let err_msg = format!("Failed to serialize HOD edits: {}", e);
        write_log("ERROR", &err_msg);
        err_msg
    })?;

    fs::write(&target_path, &patched_bytes).map_err(|e| {
        let err_msg = format!("Failed to write patched HOD file to target: {}", e);
        write_log("ERROR", &err_msg);
        err_msg
    })?;

    // Save companion .mad file if applicable
    if !model.animations.is_empty() {
        let hod_path = std::path::Path::new(&target_path);
        let mad_path = hod_path.with_extension("mad");
        write_log(
            "INFO",
            &format!("Writing companion .mad file for Save As: {:?}", mad_path),
        );
        match hwr_hod_parser::hod::serialize_mad_companion(&model) {
            Ok(mad_bytes) => {
                if let Err(e) = fs::write(&mad_path, &mad_bytes) {
                    write_log(
                        "ERROR",
                        &format!("Failed to write companion .mad file: {}", e),
                    );
                } else {
                    write_log("INFO", "Companion .mad file successfully saved!");
                }
            }
            Err(e) => {
                write_log(
                    "ERROR",
                    &format!("Failed to serialize companion .mad: {}", e),
                );
            }
        }
    }

    write_log(
        "INFO",
        "HOD file successfully patched and saved as new file!",
    );
    Ok(())
}

fn get_config_path() -> PathBuf {
    let exe_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    let exe_dir = exe_path.parent().unwrap_or_else(|| std::path::Path::new("."));
    exe_dir.join("hod_editor_config.json")
}

#[tauri::command]
fn load_shader_config() -> Result<ShaderConfig, String> {
    let config_path = get_config_path();
    if config_path.exists() {
        let data = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config: {}", e))?;
        serde_json::from_str(&data)
            .map_err(|e| format!("Failed to parse config: {}", e))
    } else {
        Ok(ShaderConfig::default())
    }
}

#[tauri::command]
fn save_shader_config(config: ShaderConfig) -> Result<(), String> {
    let config_path = get_config_path();
    let data = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    fs::write(&config_path, data)
        .map_err(|e| format!("Failed to write config: {}", e))?;
    write_log("INFO", &format!("Saved shader config to {:?}", config_path));
    Ok(())
}

#[tauri::command]
fn get_shader_pipelines(keeper_paths: Vec<String>) -> Result<Vec<String>, String> {
    write_log(
        "INFO",
        &format!(
            "Scanning shader pipelines from {} keeper path(s)",
            keeper_paths.len()
        ),
    );
    let mut pipelines: Vec<String> = Vec::new();

    for path_str in &keeper_paths {
        let keeper_dir = std::path::Path::new(path_str);
        let shaders_dir = keeper_dir.join("shaders").join("gl_prog");
        write_log("INFO", &format!("Looking for shaders in: {:?}", shaders_dir));
        if shaders_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&shaders_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file()
                        && path.extension().and_then(|s| s.to_str()) == Some("prog")
                    {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            if !pipelines.contains(&stem.to_string()) {
                                pipelines.push(stem.to_string());
                            }
                            // If it starts with "sob_", also add the stripped version (e.g. "sob_ship" -> "ship")
                            if let Some(stripped) = stem.strip_prefix("sob_") {
                                let simplified = stripped.to_string();
                                if !pipelines.contains(&simplified) {
                                    pipelines.push(simplified);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pipelines.sort();
    pipelines.dedup();
    Ok(pipelines)
}

#[tauri::command]
fn export_textures_tga(folder_path: String, textures: Vec<HODTexture>) -> Result<(), String> {
    write_log(
        "INFO",
        &format!(
            "Exporting {} textures as TGA files to: {}",
            textures.len(),
            folder_path
        ),
    );

    let path = PathBuf::from(folder_path);
    if !path.exists() {
        fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    }

    for tex in textures {
        if let Some(b64_data) = tex.png_data.or(tex.png_preview) {
            let clean_b64 = if b64_data.contains("base64,") {
                b64_data.split("base64,").collect::<Vec<&str>>()[1]
            } else {
                &b64_data
            };

            use base64::prelude::*;
            let png_bytes = BASE64_STANDARD
                .decode(clean_b64)
                .map_err(|e| format!("Base64 decode error: {}", e))?;

            let img = image::load_from_memory_with_format(&png_bytes, image::ImageFormat::Png)
                .map_err(|e| format!("Failed to parse PNG bytes: {}", e))?;

            let tga_name = if tex.name.to_lowercase().ends_with(".tga") {
                tex.name.clone()
            } else {
                format!("{}.tga", tex.name)
            };

            let tga_path = path.join(&tga_name);

            img.save_with_format(&tga_path, image::ImageFormat::Tga)
                .map_err(|e| format!("Failed to save TGA file: {}", e))?;

            write_log(
                "INFO",
                &format!("Successfully exported TGA: {:?}", tga_path),
            );
        }
    }

    Ok(())
}

#[tauri::command]
fn save_text_file(
    default_name: String,
    filters: Vec<String>,
    contents: String,
) -> Result<Option<String>, String> {
    write_log(
        "INFO",
        &format!(
            "Opening native save dialog for default name: {}",
            default_name
        ),
    );
    let mut dialog = rfd::FileDialog::new().set_file_name(&default_name);
    if !filters.is_empty() {
        let filter_slices: Vec<&str> = filters.iter().map(|s| s.as_str()).collect();
        dialog = dialog.add_filter("Export File", &filter_slices);
    }

    if let Some(path) = dialog.save_file() {
        fs::write(&path, contents).map_err(|e| e.to_string())?;
        write_log(
            "INFO",
            &format!("Successfully saved file: {}", path.to_string_lossy()),
        );
        Ok(Some(path.to_string_lossy().to_string()))
    } else {
        write_log("INFO", "User canceled save file selection dialog");
        Ok(None)
    }
}

#[tauri::command]
fn load_text_file(filters: Vec<String>) -> Result<Option<String>, String> {
    write_log("INFO", "Opening native pick file dialog...");
    let mut dialog = rfd::FileDialog::new();
    if !filters.is_empty() {
        let filter_slices: Vec<&str> = filters.iter().map(|s| s.as_str()).collect();
        dialog = dialog.add_filter("Import File", &filter_slices);
    }

    if let Some(path) = dialog.pick_file() {
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        write_log(
            "INFO",
            &format!("Successfully loaded file: {}", path.to_string_lossy()),
        );
        Ok(Some(content))
    } else {
        write_log("INFO", "User canceled pick file selection dialog");
        Ok(None)
    }
}

#[tauri::command]
fn import_tga_textures() -> Result<Vec<HODTexture>, String> {
    write_log(
        "INFO",
        "Opening native pick file dialog for importing TGA textures...",
    );
    let files = rfd::FileDialog::new()
        .add_filter("TGA Image", &["tga"])
        .pick_files();

    match files {
        Some(src_paths) => {
            let mut textures = Vec::new();
            for src_path in src_paths {
                write_log(
                    "INFO",
                    &format!(
                        "User selected TGA to import: {}",
                        src_path.to_string_lossy()
                    ),
                );

                let file_name = src_path
                    .file_name()
                    .ok_or_else(|| "Invalid file name".to_string())?;

                let img_bytes = fs::read(&src_path)
                    .map_err(|e| format!("Failed to read source TGA: {}", e))?;

                let img = image::load_from_memory_with_format(&img_bytes, image::ImageFormat::Tga)
                    .map_err(|e| format!("Failed to decode TGA file: {}", e))?;

                let width = img.width();
                let height = img.height();
                let format = if img.to_rgba8().pixels().any(|pixel| pixel[3] < 250) {
                    "DXT5"
                } else {
                    "DXT1"
                };

                let mut png_bytes: Vec<u8> = Vec::new();
                let mut cursor = std::io::Cursor::new(&mut png_bytes);
                img.write_to(&mut cursor, image::ImageFormat::Png)
                    .map_err(|e| format!("Failed to encode preview as PNG: {}", e))?;

                use base64::prelude::*;
                let b64_preview = BASE64_STANDARD.encode(&png_bytes);

                let tex_name = file_name.to_string_lossy().to_string();
                let final_tex_name = if tex_name.to_lowercase().ends_with(".tga") {
                    tex_name[..tex_name.len() - 4].to_string()
                } else {
                    tex_name
                };

                textures.push(HODTexture {
                    name: final_tex_name,
                    width: width,
                    height: height,
                    format: format.to_string(),
                    png_preview: Some(format!("data:image/png;base64,{}", b64_preview)),
                    png_data: Some(b64_preview),
                    source_path: Some(src_path.to_string_lossy().to_string()),
                    legacy_storage_y_flipped: false,
                });
            }
            Ok(textures)
        }
        None => {
            write_log("INFO", "User canceled TGA import file dialog");
            Ok(Vec::new())
        }
    }
}

#[tauri::command]
fn import_dae_file(path: String) -> Result<hwr_hod_parser::hod::HODModel, String> {
    write_log("INFO", &format!("Importing DAE file from: {}", path));

    let xml_str = std::fs::read_to_string(&path).map_err(|e| {
        let err_msg = format!("Failed to read DAE file: {}", e);
        write_log("ERROR", &err_msg);
        err_msg
    })?;

    let mut model = hwr_hod_parser::dae::parse_dae(&xml_str).map_err(|e| {
        let err_msg = format!("Failed to parse DAE XML: {}", e);
        write_log("ERROR", &err_msg);
        err_msg
    })?;
    // Clean up hierarchy and resolve name collisions
    model.auto_repair_assembly_names();
    model.clean_hierarchy();
    model.deduplicate_names();
    model.auto_assign_and_resize_textures();

    write_log(
        "INFO",
        &format!(
            "Successfully imported DAE as HOD 2.0 ({} meshes, {} joints)",
            model.meshes.len(),
            model.joints.len()
        ),
    );
    Ok(model)
}

#[tauri::command]
fn convert_weapon_to_turret(
    mut model: hwr_hod_parser::hod::HODModel,
    base_name: String,
) -> Result<hwr_hod_parser::hod::HODModel, String> {
    write_log(
        "INFO",
        &format!("Converting Weapon to Turret for assembly: {}", base_name),
    );
    model.convert_weapon_to_turret(&base_name)?;
    Ok(model)
}

#[tauri::command]
fn auto_generate_collision_from_mesh(mut model: HODModel, collision_mesh_name: String, source_mesh_name: String) -> Result<HODModel, String> {
    write_log("INFO", &format!("Auto-generating collision mesh '{}' from visible mesh '{}'...", collision_mesh_name, source_mesh_name));
    
    // Attempt to generate the mesh using convex hull
    let new_col_mesh = hwr_hod_parser::collision::generate_collision_from_visible_mesh(&model.meshes, Some(&source_mesh_name))
        .ok_or_else(|| "Failed to generate collision mesh: no valid vertices found in source mesh".to_string())?;

    // Update the model with the newly generated mesh parts
    if let Some(col) = model.collision_meshes.iter_mut().find(|c| c.name == collision_mesh_name) {
        col.mesh.parts = new_col_mesh.mesh.parts;
    } else {
        return Err(format!("Collision mesh '{}' not found in model", collision_mesh_name));
    }
    
    // Auto-calculate bounds based on the new mesh
    hwr_hod_parser::hod::generate_collision_mesh(&mut model);
    
    write_log("INFO", "Collision mesh generation successful.");
    Ok(model)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Get directory next to the application executable
            let mut log_dir = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|parent| parent.to_path_buf()))
                .unwrap_or_else(|| {
                    app.path().app_log_dir().unwrap_or_else(|_| {
                        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
                    })
                });
            let _ = fs::create_dir_all(&log_dir);
            log_dir.push("hwr_hod_editor.log");

            if let Ok(mut g) = LOG_PATH.lock() {
                *g = Some(log_dir.clone());
            }

            setup_panic_hook();

            write_log("INFO", "--------------------------------------------------");
            write_log(
                "INFO",
                &format!(
                    "HOD Remastered Editor started (v{})",
                    env!("CARGO_PKG_VERSION")
                ),
            );
            write_log(
                "INFO",
                &format!("Log file initialized at: {}", log_dir.to_string_lossy()),
            );
            write_log("INFO", "--------------------------------------------------");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            load_hod,
            save_hod,
            select_hod_file,
            select_keeper_file,
            log_event,
            get_shader_pipelines,
            load_shader_config,
            save_shader_config,
            save_text_file,
            load_text_file,
            export_textures_tga,
            import_tga_textures,
            select_save_hod_file,
            save_hod_as,
            import_dae_file,
            select_dae_file,
            convert_weapon_to_turret,
            auto_generate_collision_from_mesh
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
