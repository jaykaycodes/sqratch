// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod launcher;
mod projects;
mod state;
mod utils;

use crate::commands::db::{DbApi, DbApiImpl};
use crate::launcher::{close_window, open_window};
use crate::state::AppState;
use taurpc::Router;

// Our main entry point for the application
#[tokio::main]
async fn main() {
    let router = Router::new()
        .export_config(
            specta_typescript::Typescript::default()
                .bigint(specta_typescript::BigIntExportBehavior::String),
        )
        .merge(DbApiImpl::default().into_handler());

    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_cli::init())
        .plugin(tauri_plugin_single_instance::init(|app, _, cwd| {
            open_window(app, cwd);
        }))
        .invoke_handler(router.into_handler())
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::Destroyed => {
                close_window(window);
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("Error while running tauri application");
}
