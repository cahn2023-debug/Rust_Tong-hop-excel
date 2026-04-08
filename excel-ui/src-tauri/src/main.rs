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

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
async fn run_consolidation(
    window: tauri::Window,
    input_dir: String,
    output_file: String,
    template_path: Option<String>,
    skip_rows: usize,
    overrides: std::collections::HashMap<String, crate::domain::models::ColumnMapping>,
) -> Result<String, String> {
    let db_path = "cache.db";
    let supervisor = Supervisor::new(db_path).map_err(|e| e.to_string())?;

    supervisor
        .run(
            &window,
            &input_dir,
            &output_file,
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
async fn open_file(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        // Use cmd /c start to open with default application
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn open_folder(path: String) -> Result<(), String> {
    let folder = std::path::Path::new(&path).parent().ok_or("Invalid path")?;

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(folder)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
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

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            run_consolidation,
            get_template_headers,
            open_result_folder,
            open_file,
            open_folder
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
