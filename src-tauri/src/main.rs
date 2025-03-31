// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod launcher;
mod projects;
mod state;
mod utils;

use std::env;

use crate::commands::db::{DbApi, DbApiImpl};
use crate::launcher::{close_window, launch_window, SqratchArgs};
use crate::state::AppState;
use clap::Parser;
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

    let logging = utils::logging::setup_logging_plugin();

    let builder = tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(logging.build())
        .plugin(tauri_plugin_single_instance::init(|app, args, cwd| {
            let args = SqratchArgs::try_parse_from(args).unwrap_or_default();

            let _ = launch_window(app, args, cwd).map_err(|e| {
                log::error!("Failed to open window: {}", e);
            });
        }))
        .invoke_handler(router.into_handler())
        .setup(|app| {
            let app_handle = app.app_handle();
            let args = app_handle.env().args_os;

            if let Err(e) = launch_window(
                &app_handle,
                SqratchArgs::try_parse_from(args).unwrap_or_default(),
                env::current_dir().unwrap().to_string_lossy().to_string(),
            ) {
                log::error!("Failed to open window: {}", e);
                app_handle.exit(1);
            }

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
