use std::sync::Arc;
use tauri_plugin_fs::FsExt;
use crate::DatabaseManager;
use crate::db::manager::parse_connection_config;

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

    // Allow the .sqratch directory anywhere - needed for loading projects
    if let Err(e) = fs_scope.allow_directory("**/.sqratch", true) {
        eprintln!("Failed to set fs permissions for .sqratch: {}", e);
    }

    // Allow broader directories for CLI usage
    // This allows the CLI to access typical project directories
    if let Err(e) = fs_scope.allow_directory("**", false) {
        eprintln!("Failed to set broader fs permissions: {}", e);
    }

    // For common user directories
    if let Ok(home) = std::env::var("HOME") {
        if let Err(e) = fs_scope.allow_directory(&home, true) {
            eprintln!("Failed to set permissions for home directory: {}", e);
        }
    }

    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        if let Err(e) = fs_scope.allow_directory(&userprofile, true) {
            eprintln!("Failed to set permissions for user profile: {}", e);
        }
    }
}
