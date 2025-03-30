use log;
use tauri::{AppHandle, Manager, Window};
use tauri_plugin_cli::CliExt;

use crate::projects::{parse_project_arg, ProjectId};
use crate::state::{cleanup_window_state, init_state_for_project};
use crate::utils::errors::AppError;

pub const LAUNCHER_LABEL: &str = "launcher";

pub fn open_window(app: &AppHandle, cwd: String) {
    let project_arg = match app.cli().matches() {
        Ok(m) => match m.args.get("project") {
            Some(arg) => Some(arg.value.to_string()),
            None => None,
        },
        Err(_) => {
            log::error!("Failed to parse CLI arguments");
            return;
        }
    };

    if let Some(project_arg) = project_arg {
        // Open specific project
        match parse_project_arg(&project_arg, &cwd) {
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

pub fn close_window(window: &Window) {
    if window.label() != LAUNCHER_LABEL {
        cleanup_window_state(window);
    }
}

/// Opens (or focuses) a project window using a unique window ID
fn open_project_window(app: &AppHandle, project_id: &ProjectId) -> Result<(), AppError> {
    // If window already exists, just focus it
    if let Some(existing_window) = app.get_webview_window(&project_id.to_window_label()) {
        let _ = existing_window.show();
        let _ = existing_window.set_focus();
        return Ok(());
    }

    init_state_for_project(app, project_id)?;

    // Create window using Tauri v2 API
    let window_config = tauri::WebviewWindowBuilder::new(
        app,
        project_id.to_window_label(),
        tauri::WebviewUrl::App("project.html".into()),
    )
    .title(project_id.display_name());

    let _window = window_config.build().map_err(|e| e.to_string())?;

    Ok(())
}

/// Opens (or focuses) the launcher window when no project is specified
fn open_launcher_window(app: &AppHandle) -> Result<(), String> {
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
        tauri::WebviewUrl::App("launcher.html".into()),
    )
    .title("Projects");

    let _launcher = window_config.build().map_err(|e| e.to_string())?;

    Ok(())
}
