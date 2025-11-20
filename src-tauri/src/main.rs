// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use tauri::Manager;
use vibe_app::commands;
use vibe_app::MODEL_BASE_PATH;
use anyhow::Result;
use tauri::AppHandle;
use tauri::path::BaseDirectory;
use tauri::App;

const MODEL_RELATIVE_PATH: &str = "tts_models/supertonic";

#[tauri::command]
fn greet(name: &str) -> String {
    let res = commands::add_method(1, 2);
    format!("Hello, {}! You've been greeted from Rust!{}", name, res)
}

fn main() {
    tauri::Builder::default().setup(|app| {
            let result = initialize_model_paths(app.handle());
            if let Err(e) = result {
                eprintln!("初始化模型路径失败: {:?}", e);
                // 这里可以选择让程序失败或继续，取决于错误是否致命
            }
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![greet, commands::add_method, commands::generate_voice])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn initialize_model_paths(app_handle: &AppHandle) -> Result<()> {
    
    let model_base_dir = app_handle.path().resolve(MODEL_RELATIVE_PATH, BaseDirectory::Resource)?;

    if !model_base_dir.exists() {
        // 在开发模式下，这可能需要更复杂的处理，但在构建后应始终存在
        anyhow::bail!("模型基目录未找到: {}", model_base_dir.display());
    }

    if MODEL_BASE_PATH.set(model_base_dir).is_err() {
        anyhow::bail!("尝试重复初始化 ONNX_BASE_PATH");
    }

    Ok(())
}