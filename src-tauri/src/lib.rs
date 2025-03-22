// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri_plugin_cli::{self, CliExt};
use tauri_plugin_fs::FsExt;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_cli::init())
        .setup(|app| {
            // Configure file system permissions
            let fs_scope = app.fs_scope();
            if let Err(e) = fs_scope.allow_directory("**/.sqratch", true) {
                eprintln!("Failed to set fs permissions: {}", e);
            }

            // Print CLI arguments for debugging
            if let Ok(matches) = app.cli().matches() {
                if let Some(project_path) = matches.args.get("project-path") {
                    println!("Project path: {}", project_path.value.as_str().unwrap_or(""));
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
