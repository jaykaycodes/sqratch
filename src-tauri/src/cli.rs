use std::sync::Arc;
use std::path::Path;
use tauri::{App, AppHandle, Manager};
use tauri_plugin_cli::CliExt;
use crate::DatabaseManager;
use crate::startup;

/// Process CLI arguments from the initial launch
pub fn process_cli_args(app: &App, db_manager: Arc<DatabaseManager>) -> Result<(), Box<dyn std::error::Error>> {
    // Get project path from arguments if available
    if let Some(project_path) = get_project_path(app) {
        // Check if the path exists and is a directory
        let path = Path::new(&project_path);
        if path.exists() && path.is_dir() {
            println!("Opening project directory: {}", project_path);

            // Process the project path asynchronously
            let db_manager_clone = db_manager.clone();
            let path_string = project_path.clone();
            tauri::async_runtime::spawn(async move {
                startup::handle_project_path_async(path_string, db_manager_clone).await;
            });
        } else {
            eprintln!("Project path does not exist or is not a directory: {}", project_path);
        }
    }

    // Make sure app is visible
    ensure_app_visible(&app.handle());

    Ok(())
}

/// Get the project path from CLI arguments or environment
pub fn get_project_path(app: &App) -> Option<String> {
    // Get CLI arguments
    let mut project_path: Option<String> = None;

    // Try to get project path from CLI arguments
    if let Ok(matches) = app.cli().matches() {
        // Extract the first positional argument if it exists
        project_path = matches.args.get("")
            .and_then(|args| args.value.as_array())
            .and_then(|raw_args| raw_args.first())
            .and_then(|path_value| path_value.as_str())
            .map(|path_str| {
                println!("Project path from positional argument: {}", path_str);
                path_str.to_string()
            });
    }

    // Also check environment variable as a fallback
    if project_path.is_none() {
        project_path = std::env::var("SQRATCH_PROJECT_PATH").ok();
        if let Some(ref path) = project_path {
            println!("Project path from env: {}", path);
        }
    }

    project_path
}

/// Ensure the app window is visible
fn ensure_app_visible(app_handle: &AppHandle) {
    if let Some(window) = app_handle.get_webview_window("main") {
        // Show and focus the window
        let _ = window.show();
        let _ = window.set_focus();
    }
}
