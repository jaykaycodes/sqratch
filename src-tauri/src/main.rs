// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::lib;
use crate::state::AppState;
use tauri::Manager;

// Our main entry point for the application
fn main() {
    tauri::Builder::default()
        .manage(db::ConnectionManager::new())
        .manage(AppState::default())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_cli::init())
        .plugin(tauri_plugin_single_instance::init(lib::open_project))
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                let app_handle = window.app_handle();
                let window_label = window.label();
                lib::close_project(&app_handle, &window_label);
            }
        })
        .run(tauri::generate_context!())
        .expect("Error while running tauri application");
}
