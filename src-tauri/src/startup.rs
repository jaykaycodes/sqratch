use std::sync::Arc;
use tauri_plugin_fs::FsExt;
use tauri_plugin_cli::CliExt;
use crate::db::{DatabaseManager, parse_connection_config};
use crate::AppState;

pub fn get_project_path(app: &tauri::App) -> Option<String> {
    // Get CLI arguments
    let mut project_path: Option<String> = None;

    if let Ok(matches) = app.cli().matches() {
        if let Some(path_arg) = matches.args.get("project-path") {
            project_path = path_arg.value.as_str().map(|s| s.to_string());
            println!("Project path argument: {}", path_arg.value.as_str().unwrap_or(""));
        }
    }

    // Also check environment variable
    if project_path.is_none() {
        project_path = std::env::var("SQRATCH_PROJECT_PATH").ok();
        if let Some(ref path) = project_path {
            println!("Project path from env: {}", path);
        }
    }

    project_path
}

pub async fn handle_project_path_async(path: String, db_manager: Arc<DatabaseManager>) {
    println!("Attempting to find database connections for project at: {}", path);

    // Try to load connection config from project path
    let connection_result = parse_connection_config(Some(&path), None).await;
    match connection_result {
        Ok(Some(connection)) => {
            println!("Found connection configuration in project path");

            // Add connection to manager
            match db_manager.add_connection(connection).await {
                Ok(id) => {
                    println!("Added connection with ID: {}", id);

                    // Try to connect
                    if let Err(e) = db_manager.connect(&id).await {
                        eprintln!("Failed to connect to database: {}", e);
                    } else {
                        println!("Successfully connected to database");
                    }
                },
                Err(e) => {
                    eprintln!("Failed to add connection: {}", e);
                }
            }
        },
        Ok(None) => {
            // Try loading connections from the project directory
            match db_manager.load_from_project(&path).await {
                Ok(connection_ids) => {
                    println!("Loaded {} connections from project", connection_ids.len());

                    // Auto-connect to the first connection if available
                    if let Some(first_id) = connection_ids.first() {
                        if let Err(e) = db_manager.connect(first_id).await {
                            eprintln!("Failed to auto-connect to database: {}", e);
                        } else {
                            println!("Auto-connected to database {}", first_id);
                        }
                    }
                },
                Err(e) => {
                    eprintln!("Failed to load connections from project: {}", e);
                }
            }
        },
        Err(e) => {
            eprintln!("Error parsing connection configuration: {}", e);
        }
    }
}

pub fn configure_fs_permissions(app: &tauri::App) {
    // Configure file system permissions
    let fs_scope = app.fs_scope();
    if let Err(e) = fs_scope.allow_directory("**/.sqratch", true) {
        eprintln!("Failed to set fs permissions: {}", e);
    }
}
