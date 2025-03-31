// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod launch;
mod plugins;
mod projects;
mod state;
mod utils;

use std::env;

use crate::commands::db::{DbApi, DbApiImpl};
use crate::launch::{close_window, launch_app, launch_window};
use crate::plugins::logging_plugin;
use crate::state::AppState;
use tauri::Manager;
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

    let builder = tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(logging_plugin())
        .plugin(tauri_plugin_single_instance::init(|app, args, cwd| {
            if let Err(e) = launch_window(app, args, cwd) {
                eprintln!("Failed to launch window: {}", e);
            }
        }))
        .invoke_handler(router.into_handler())
        .setup(|app| {
            launch_app(app.app_handle());
            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::Destroyed => {
                close_window(window);
            }
            _ => {}
        });

    builder
        .run(tauri::generate_context!())
        .expect("Error while running tauri application");
}
