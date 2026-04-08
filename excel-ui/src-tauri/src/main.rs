// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod aggregator;
mod app;
mod cache;
mod config;
mod domain;
mod parser;
mod pipeline;
mod scanner;
mod writer;

use crate::pipeline::supervisor::Supervisor;
use crate::cache::sqlite::CacheManager;
use tauri::Manager;

#[tauri::command]
async fn clear_cache(app: tauri::AppHandle) -> Result<(), String> {
    let cache_dir = app.path().app_cache_dir().map_err(|e| e.to_string())?;
    let db_path = cache_dir.join("cache.db");
    if db_path.exists() {
        let cache = CacheManager::new(&db_path).map_err(|e| e.to_string())?;
        cache.clear_all().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn run_consolidation(
    app: tauri::AppHandle,
    window: tauri::Window,
    input_dir: String,
    output_file: String,
    template_path: Option<String>,
    skip_rows: usize,
    overrides: std::collections::HashMap<String, crate::domain::models::ColumnMapping>,
) -> Result<String, String> {
    let cache_dir = app.path().app_cache_dir().map_err(|e| e.to_string())?;
    if !cache_dir.exists() {
        std::fs::create_dir_all(&cache_dir).map_err(|e| e.to_string())?;
    }

    let db_path = cache_dir.join("cache.db");
    let result_path = cache_dir.join(&output_file);
    
    let supervisor = Supervisor::new(db_path.to_str().ok_or("Invalid path")?).map_err(|e| e.to_string())?;

    supervisor
        .run(
            &window,
            &input_dir,
            result_path.to_str().ok_or("Invalid path")?,
            template_path,
            skip_rows,
            overrides,
        )
        .map_err(|e| e.to_string())?;

    Ok(format!("Successfully generated {}", output_file))
}

#[tauri::command]
async fn get_template_headers(path: String, skip_rows: usize) -> Result<Vec<String>, String> {
    crate::parser::ExcelParser::get_headers(path, skip_rows).map_err(|e| e.to_string())
}

#[tauri::command]
async fn open_file(app: tauri::AppHandle, path: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    app.opener().open_path(&path, None::<&str>).map_err(|e| e.to_string())
}

#[tauri::command]
async fn open_folder(app: tauri::AppHandle, path: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    let folder = std::path::Path::new(&path).parent().unwrap_or(std::path::Path::new(&path));
    app.opener().open_path(folder.to_string_lossy(), None::<&str>).map_err(|e| e.to_string())
}

#[tauri::command]
async fn open_result_folder(app: tauri::AppHandle, path: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;

    let path_buf = std::path::PathBuf::from(&path);

    // Attempt to open the file directly first (most intuitive for "Export" button)
    if path_buf.exists() {
        if let Err(_) = app.opener().open_path(&path, None::<&str>) {
            // Fallback to opening the parent folder if file open fails
            if let Some(parent) = path_buf.parent() {
                app.opener()
                    .open_path(parent.to_string_lossy(), None::<&str>)
                    .map_err(|e| e.to_string())?;
            }
        }
    } else {
        // If file doesn't exist, try opening the parent folder anyway
        if let Some(parent) = path_buf.parent() {
            app.opener()
                .open_path(parent.to_string_lossy(), None::<&str>)
                .map_err(|e| e.to_string())?;
        } else {
            return Err("File does not exist and no parent folder found.".into());
        }
    }

    Ok(())
}

#[tauri::command]
async fn finalize_export(app: tauri::AppHandle, source: String, destination: String) -> Result<(), String> {
    use tauri::Manager;
    let cache_dir = app.path().app_cache_dir().map_err(|e| e.to_string())?;
    let source_path = cache_dir.join(&source);

    if !source_path.exists() {
        return Err(format!("Source file {} does not exist in cache.", source)).into();
    }
    std::fs::copy(&source_path, &destination).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn get_file_data(path: String, skip_rows: usize) -> Result<Vec<crate::domain::models::ExcelRecord>, String> {
    let (records, _) = crate::parser::ExcelParser::parse(path, None, skip_rows).map_err(|e| e.to_string())?;
    Ok(records)
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            run_consolidation,
            get_template_headers,
            open_result_folder,
            open_file,
            open_folder,
            finalize_export,
            get_file_data,
            clear_cache
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
