// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod constants;
mod db;
mod errors;
mod launch;
mod plugins;
mod projects;
mod state;
mod windows;

use std::env;

use crate::commands::db::{DbApi, DbApiImpl};
use crate::constants::LAUNCHER_LABEL;
use crate::launch::{close_window, launch_app, launch_instance};
use crate::plugins::setup_logging;
use crate::state::AppState;
use taurpc::Router;

// Our main entry point for the application
#[tokio::main]
async fn main() {
    let router = Router::new()
        .export_config(
            specta_typescript::Typescript::default()
                .formatter(specta_typescript::formatter::biome)
                .bigint(specta_typescript::BigIntExportBehavior::String),
        )
        .merge(DbApiImpl::default().into_handler());

    let builder = tauri::Builder::default()
        // NOTE: single instance should always come first
        .plugin(tauri_plugin_single_instance::init(|app, args, cwd| {
            launch_instance(app, args, &cwd);
        }))
        .plugin(
            tauri_plugin_window_state::Builder::new()
                .with_denylist(&[LAUNCHER_LABEL])
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState::default())
        .invoke_handler(router.into_handler())
        .setup(|app| {
            setup_logging(app.handle())?;
            launch_app(app.handle());
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
