// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod errors;
mod launch;
mod project;
mod state;
mod utils;

use std::env;

use tauri::Manager;

use crate::commands::db::{DbApi, DbApiImpl};
use crate::commands::projects::{ProjectsApi, ProjectsApiImpl};
use crate::state::AppState;
use crate::utils::paths;
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
        .merge(DbApiImpl {}.into_handler())
        .merge(ProjectsApiImpl {}.into_handler());

    let builder = tauri::Builder::default()
        // NOTE: single instance should always come first
        .plugin(tauri_plugin_single_instance::init(|app, args, cwd| {
            launch::launch_instance(app, args, &cwd);
        }))
        .plugin(
            tauri_plugin_window_state::Builder::new()
                .with_denylist(&[launch::LAUNCHER_LABEL])
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(router.into_handler())
        .setup(|app| {
            paths::init_paths(app.handle());

            app.manage(AppState::new());

            utils::plugins::setup_logging(app.handle())?;
            launch::launch_app(app.handle());
            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::Destroyed => {
                launch::close_window(window);
            }
            _ => {}
        });

    builder
        .run(tauri::generate_context!())
        .expect("Error while running tauri application");
}
