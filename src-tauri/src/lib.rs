use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use chrono::Local;
use hwr_hod_parser::hod::{HODModel, HODTexture};
use tauri::Manager;

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
                .unwrap_or_else(|| {
                    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
                });
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
        
        let location = info.location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".to_string());
            
        write_log("PANIC", &format!("Rust panicked at {}: {}", location, message));
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
            write_log("INFO", &format!("User selected HOD file: {}", path.to_string_lossy()));
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
            write_log("INFO", &format!("User selected keeper file: {}", path.to_string_lossy()));
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
            write_log("INFO", &format!("User selected DAE file: {}", path.to_string_lossy()));
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
    write_log("INFO", &format!("Attempting to load HOD from path: {} with keeper_path: {:?}", file_path, keeper_path));
    
    // Read raw HOD bytes from file system
    let bytes = fs::read(&file_path)
        .map_err(|e| {
            let err_msg = format!("Failed to read file: {}", e);
            write_log("ERROR", &err_msg);
            err_msg
        })?;

    write_log("INFO", &format!("Successfully read {} bytes from disk. Parsing HOD structures...", bytes.len()));

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


    write_log("INFO", &format!(
        "Successfully parsed HOD Model: '{}' | Meshes: {} | Joints: {} | Markers: {} | Materials: {} | Textures: {}",
        model.name, model.meshes.len(), model.joints.len(), model.markers.len(), model.materials.len(), model.textures.len()
    ));

    for joint in &model.joints {
        if joint.name.to_lowercase().contains("nozzle") || joint.name.to_lowercase().contains("engine") {
            write_log("INFO", &format!(
                "Joint DBG: Name='{}' | Parent='{:?}' | Translation=[{:.3}, {:.3}, {:.3}]",
                joint.name,
                joint.parent_name,
                joint.local_transform.m[3][0],
                joint.local_transform.m[3][1],
                joint.local_transform.m[3][2],
            ));
        }
    }

    Ok(model)
}

#[tauri::command]
fn save_hod(file_path: String, mut model: HODModel) -> Result<(), String> {
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

    write_log("INFO", &format!("Original HOD size: {} bytes. Patching chunks...", original_bytes.len()));

    // generate_collision_mesh(&mut model); // Removed per user request

    // 2. Patch HIER and MRKR chunks, and MULT chunks using save_edits
    // Always use save_edits to preserve the exact original chunk structure, including
    // skipping CFHodEd-specific NRML wrappers that generate_v2_from_model produces.
    let patched_bytes = hwr_hod_parser::hod::save_edits(&original_bytes, &model)
        .map_err(|e| {
            let err_msg = format!("Failed to serialize HOD edits: {}", e);
            write_log("ERROR", &err_msg);
            err_msg
        })?;

    write_log("INFO", &format!("Successfully serialized edited HOD stream ({} bytes). Writing back to disk...", patched_bytes.len()));

    // 3. Write back the updated HOD file to disk
    fs::write(&file_path, &patched_bytes)
        .map_err(|e| {
            let err_msg = format!("Failed to write patched HOD file: {}", e);
            write_log("ERROR", &err_msg);
            err_msg
        })?;

    // 4. Save companion .mad file if applicable
    if !model.animations.is_empty() {
        let hod_path = std::path::Path::new(&file_path);
        let mad_path = hod_path.with_extension("mad");
        write_log("INFO", &format!("Writing companion .mad file: {:?}", mad_path));
        match hwr_hod_parser::hod::serialize_mad_companion(&model) {
            Ok(mad_bytes) => {
                if let Err(e) = fs::write(&mad_path, &mad_bytes) {
                    write_log("ERROR", &format!("Failed to write .mad file: {}", e));
                } else {
                    write_log("INFO", "Companion .mad file successfully saved!");
                }
            }
            Err(e) => {
                write_log("ERROR", &format!("Failed to serialize companion .mad: {}", e));
            }
        }
    }

    write_log("INFO", "HOD file successfully patched and saved!");

    Ok(())
}

#[tauri::command]
fn select_save_hod_file(default_name: Option<String>) -> Result<Option<String>, String> {
    write_log("INFO", "Opening native save HOD file dialog...");
    let mut dialog = rfd::FileDialog::new()
        .add_filter("Homeworld HOD Files", &["hod"]);
        
    if let Some(name) = default_name {
        dialog = dialog.set_file_name(&name);
    }

    let file = dialog.save_file();

    match &file {
        Some(path) => {
            write_log("INFO", &format!("User selected save HOD path: {}", path.to_string_lossy()));
            Ok(Some(path.to_string_lossy().to_string()))
        }
        None => {
            write_log("INFO", "User canceled save HOD file dialog");
            Ok(None)
        }
    }
}

#[tauri::command]
fn save_hod_as(source_path: String, target_path: String, mut model: HODModel) -> Result<(), String> {
    write_log("INFO", &format!("Saving HOD edits as: {} -> {}", source_path, target_path));
    
    let original_bytes = if source_path.is_empty() || !std::path::Path::new(&source_path).exists() {
        Vec::new()
    } else {
        fs::read(&source_path).map_err(|e| {
            let err_msg = format!("Failed to read original HOD file from source: {}", e);
            write_log("ERROR", &err_msg);
            err_msg
        })?
    };

    // generate_collision_mesh(&mut model); // Removed per user request

    let patched_bytes = hwr_hod_parser::hod::save_edits(&original_bytes, &model)
        .map_err(|e| {
            let err_msg = format!("Failed to serialize HOD edits: {}", e);
            write_log("ERROR", &err_msg);
            err_msg
        })?;

    fs::write(&target_path, &patched_bytes)
        .map_err(|e| {
            let err_msg = format!("Failed to write patched HOD file to target: {}", e);
            write_log("ERROR", &err_msg);
            err_msg
        })?;

    // Save companion .mad file if applicable
    if !model.animations.is_empty() {
        let hod_path = std::path::Path::new(&target_path);
        let mad_path = hod_path.with_extension("mad");
        write_log("INFO", &format!("Writing companion .mad file for Save As: {:?}", mad_path));
        match hwr_hod_parser::hod::serialize_mad_companion(&model) {
            Ok(mad_bytes) => {
                if let Err(e) = fs::write(&mad_path, &mad_bytes) {
                    write_log("ERROR", &format!("Failed to write companion .mad file: {}", e));
                } else {
                    write_log("INFO", "Companion .mad file successfully saved!");
                }
            }
            Err(e) => {
                write_log("ERROR", &format!("Failed to serialize companion .mad: {}", e));
            }
        }
    }

    write_log("INFO", "HOD file successfully patched and saved as new file!");
    Ok(())
}

#[tauri::command]
fn get_shader_pipelines(keeper_path: Option<String>) -> Result<Vec<String>, String> {
    write_log("INFO", &format!("Scanning shader pipelines from keeper_path: {:?}", keeper_path));
    let mut pipelines = vec![
        "ship".to_string(),
        "badge".to_string(),
        "bay".to_string(),
        "thruster".to_string(),
    ]; // base defaults

    if let Some(path_str) = keeper_path {
        let keeper_file = std::path::Path::new(&path_str);
        if let Some(uncompressed_root) = keeper_file.parent() {
            let shaders_dir = uncompressed_root.join("shaders").join("gl_prog");
            if shaders_dir.is_dir() {
                if let Ok(entries) = fs::read_dir(&shaders_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("prog") {
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
    }

    pipelines.sort();
    pipelines.dedup();
    Ok(pipelines)
}

#[tauri::command]
fn export_textures_tga(folder_path: String, textures: Vec<HODTexture>) -> Result<(), String> {
    write_log("INFO", &format!("Exporting {} textures as TGA files to: {}", textures.len(), folder_path));
    
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
            let png_bytes = BASE64_STANDARD.decode(clean_b64).map_err(|e| format!("Base64 decode error: {}", e))?;
            
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
                
            write_log("INFO", &format!("Successfully exported TGA: {:?}", tga_path));
        }
    }
    
    Ok(())
}

#[tauri::command]
fn save_text_file(default_name: String, filters: Vec<String>, contents: String) -> Result<Option<String>, String> {
    write_log("INFO", &format!("Opening native save dialog for default name: {}", default_name));
    let mut dialog = rfd::FileDialog::new().set_file_name(&default_name);
    if !filters.is_empty() {
        let filter_slices: Vec<&str> = filters.iter().map(|s| s.as_str()).collect();
        dialog = dialog.add_filter("Export File", &filter_slices);
    }
    
    if let Some(path) = dialog.save_file() {
        fs::write(&path, contents).map_err(|e| e.to_string())?;
        write_log("INFO", &format!("Successfully saved file: {}", path.to_string_lossy()));
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
        write_log("INFO", &format!("Successfully loaded file: {}", path.to_string_lossy()));
        Ok(Some(content))
    } else {
        write_log("INFO", "User canceled pick file selection dialog");
        Ok(None)
    }
}

#[tauri::command]
fn import_tga_texture(hod_file_path: String) -> Result<Option<HODTexture>, String> {
    write_log("INFO", "Opening native pick file dialog for importing TGA texture...");
    let file = rfd::FileDialog::new()
        .add_filter("TGA Image", &["tga"])
        .pick_file();

    match file {
        Some(src_path) => {
            write_log("INFO", &format!("User selected TGA to import: {}", src_path.to_string_lossy()));
            
            let hod_path = PathBuf::from(&hod_file_path);
            let target_dir = hod_path.parent().ok_or_else(|| "Failed to get HOD folder path".to_string())?;
            
            let file_name = src_path.file_name().ok_or_else(|| "Invalid file name".to_string())?;
            let dest_path = target_dir.join(file_name);
            
            let img_bytes = fs::read(&src_path)
                .map_err(|e| format!("Failed to read source TGA: {}", e))?;
                
            fs::write(&dest_path, &img_bytes)
                .map_err(|e| format!("Failed to copy TGA to HOD folder: {}", e))?;
                
            write_log("INFO", &format!("Copied TGA file to: {}", dest_path.to_string_lossy()));
            
            let img = image::load_from_memory_with_format(&img_bytes, image::ImageFormat::Tga)
                .map_err(|e| format!("Failed to decode TGA file: {}", e))?;
                
            let width = img.width();
            let height = img.height();
            
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

            Ok(Some(HODTexture {
                name: final_tex_name,
                width: width,
                height: height,
                format: "TGA".to_string(),
                png_preview: Some(format!("data:image/png;base64,{}", b64_preview)),
                png_data: Some(b64_preview),
            }))
        }
        None => {
            write_log("INFO", "User canceled TGA import file dialog");
            Ok(None)
        }
    }
}

#[tauri::command]
fn import_dae_file(path: String) -> Result<hwr_hod_parser::hod::HODModel, String> {
    write_log("INFO", &format!("Importing DAE file from: {}", path));
    
    let xml_str = std::fs::read_to_string(&path)
        .map_err(|e| {
            let err_msg = format!("Failed to read DAE file: {}", e);
            write_log("ERROR", &err_msg);
            err_msg
        })?;
        
    let mut model = hwr_hod_parser::dae::parse_dae(&xml_str)
        .map_err(|e| {
            let err_msg = format!("Failed to parse DAE XML: {}", e);
            write_log("ERROR", &err_msg);
            err_msg
        })?;
    // Clean up hierarchy and resolve name collisions
    model.clean_hierarchy();
    model.deduplicate_names();
        
    // hwr_hod_parser::hod::generate_collision_mesh(&mut model); // Removed per user request
    
    write_log("INFO", &format!("Successfully imported DAE as HOD 2.0 ({} meshes, {} joints)", model.meshes.len(), model.joints.len()));
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
            write_log("INFO", &format!("HOD Remastered Editor started (v{})", env!("CARGO_PKG_VERSION")));
            write_log("INFO", &format!("Log file initialized at: {}", log_dir.to_string_lossy()));
            write_log("INFO", "--------------------------------------------------");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet, load_hod, save_hod, select_hod_file, select_keeper_file, log_event, get_shader_pipelines, save_text_file, load_text_file, export_textures_tga, import_tga_texture, select_save_hod_file, save_hod_as, import_dae_file, select_dae_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
