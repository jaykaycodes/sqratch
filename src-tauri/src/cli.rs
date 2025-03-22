use std::sync::Arc;
use std::path::Path;
use tauri::{App, AppHandle, Manager};
use clap::Parser;
use crate::DatabaseManager;
use crate::startup;

/// CLI arguments for Sqratch
#[derive(Parser, Debug)]
#[command(version, about = "Sqratch - A developer-friendly SQL database UI")]
pub struct Args {
    /// Path to a project directory to open
    project_path: Option<String>,
}

/// Process CLI arguments from the initial launch
pub fn process_cli_args(app: &App, db_manager: Arc<DatabaseManager>) -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_cli_args();

    // Process project path if provided
    if let Some(project_path) = args.project_path {
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
    } else {
        // Check environment variable as a fallback
        if let Ok(env_path) = std::env::var("SQRATCH_PROJECT_PATH") {
            println!("Project path from env: {}", env_path);

            let path = Path::new(&env_path);
            if path.exists() && path.is_dir() {
                let db_manager_clone = db_manager.clone();
                tauri::async_runtime::spawn(async move {
                    startup::handle_project_path_async(env_path, db_manager_clone).await;
                });
            } else {
                eprintln!("Project path from env does not exist or is not a directory: {}", env_path);
            }
        }
    }

    // Make sure app is visible
    ensure_app_visible(&app.handle());

    Ok(())
}

/// Parse CLI arguments
pub fn parse_cli_args() -> Args {
    // In a real CLI context, we would use Args::parse()
    // But since Tauri handles the args differently, we need to extract them manually

    // Try to get raw args from the environment
    match std::env::args().skip(1).next() {
        Some(arg) if !arg.starts_with('-') => Args { project_path: Some(arg) },
        _ => Args { project_path: None }
    }
}

/// Ensure the app window is visible
fn ensure_app_visible(app_handle: &AppHandle) {
    if let Some(window) = app_handle.get_webview_window("main") {
        // Show and focus the window
        let _ = window.show();
        let _ = window.set_focus();
    }
}
