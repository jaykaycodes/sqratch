use log;
use tauri::{AppHandle, Manager};
use tauri_plugin_cli::CliExt;

mod commands;
mod db;
mod projects;
mod state;
mod utils;

use crate::db::ConnectionManagerSafe;
use crate::projects::ProjectId;
use crate::state::AppStateSafe;

pub fn open_project(app: &AppHandle, cwd: String) {
    let project_arg = match app.cli().matches() {
        Ok(m) => match m.args.get("project") {
            Some(project) => Some(project.value.to_string()),
            None => None,
        },
        Err(_) => {
            log::error!("Failed to parse CLI arguments");
            return;
        }
    };

    if let Some(project_arg) = project_arg {
        // Open specific project
        match projects::parse_project_arg(&project_arg, &cwd) {
            Ok(project_id) => {
                if let Err(e) = open_project_window(app, &project_id) {
                    log::error!("Failed to create project window: {}", e);
                }
            }
            Err(e) => {
                log::error!("Failed to parse project argument: {}", e);
            }
        }
    } else {
        // No project specified, open launcher
        if let Err(e) = open_launcher_window(app) {
            log::error!("Failed to create launcher window: {}", e);
        }
    }
}

pub fn close_project(app: &AppHandle, window_label: &str) {
    // Create runtime for async cleanup
    let rt = tokio::runtime::Runtime::new().unwrap();

    if window_label != "launcher" {
        // Project window closing - clean up just this window
        let state = app.state::<AppStateSafe>();
        if let Err(e) = rt.block_on(state.cleanup_window(&app, window_label)) {
            log::warn!("Error cleaning up window {}: {}", window_label, e);
        }
    }

    rt.shutdown_background();
}

/// Opens (or focuses) a project window using a unique window ID
fn open_project_window(app: &AppHandle, project_id: &ProjectId) -> Result<(), String> {
    // Get or create the project and get a window ID
    let app_state = app.state::<AppStateSafe>();
    let (project, window_id) = app_state.get_or_create_project(project_id)?;

    // Check if window already exists
    if let Some(existing_window) = app.get_webview_window(&window_id) {
        // Just focus the existing window instead of creating a new one
        let _ = existing_window.show();
        let _ = existing_window.set_focus();
        return Ok(());
    }

    // Create connection with same ID as window
    let db_manager = app.state::<ConnectionManagerSafe>();
    let mut connection_info = project.connection_info.clone();

    // Use window ID as connection ID
    connection_info.id = window_id.clone();
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(db_manager.add_connection(connection_info))
        .map_err(|e| e.to_string())?;

    // Create window using Tauri v2 API
    let window_config = tauri::WebviewWindowBuilder::new(
        app,
        window_id,
        tauri::WebviewUrl::App("index.html".into()),
    )
    .title(format!("Sqratch - {}", project.name));

    let _window = window_config.build().map_err(|e| e.to_string())?;

    Ok(())
}

/// Opens (or focuses) the launcher window when no project is specified
fn open_launcher_window(app: &AppHandle) -> Result<(), String> {
    const LAUNCHER_LABEL: &str = "launcher";

    // Check if launcher already exists
    if let Some(existing_window) = app.get_webview_window(LAUNCHER_LABEL) {
        // Just focus the existing launcher instead of creating a new one
        let _ = existing_window.show();
        let _ = existing_window.set_focus();
        return Ok(());
    }

    // Create the launcher window using Tauri v2 API
    let window_config = tauri::WebviewWindowBuilder::new(
        app,
        LAUNCHER_LABEL,
        tauri::WebviewUrl::App("index.html".into()),
    )
    .title("Sqratch - Project Launcher");

    let _launcher = window_config.build().map_err(|e| e.to_string())?;

    Ok(())
}
