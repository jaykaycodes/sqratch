// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri_plugin_cli::{self, CliExt};
use tauri_plugin_fs::FsExt;
use std::sync::Arc;

// Database module
mod db;
use db::{
    ConnectionInfo, DatabaseType, DatabaseManager,
    create_db_manager, parse_connection_config, DbResult,
};

// State for database manager
struct AppState {
    db_manager: Arc<DatabaseManager>,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Database related commands
#[tauri::command]
async fn list_connections(state: tauri::State<'_, AppState>) -> Result<Vec<ConnectionInfo>, String> {
    Ok(state.db_manager.list_connections().await)
}

#[tauri::command]
async fn add_connection(
    conn_info: ConnectionInfo,
    state: tauri::State<'_, AppState>
) -> Result<String, String> {
    state.db_manager.add_connection(conn_info).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn connect_to_database(
    connection_id: String,
    state: tauri::State<'_, AppState>
) -> Result<(), String> {
    state.db_manager.connect(&connection_id).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn disconnect_from_database(
    connection_id: String,
    state: tauri::State<'_, AppState>
) -> Result<(), String> {
    state.db_manager.disconnect(&connection_id).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn is_connected(
    connection_id: String,
    state: tauri::State<'_, AppState>
) -> Result<bool, String> {
    Ok(state.db_manager.is_connected(&connection_id).await)
}

#[tauri::command]
async fn execute_query(
    connection_id: String,
    query: String,
    state: tauri::State<'_, AppState>
) -> Result<db::QueryResult, String> {
    state.db_manager.execute_query(&connection_id, &query).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn execute_queries(
    connection_id: String,
    queries: String,
    state: tauri::State<'_, AppState>
) -> Result<Vec<db::QueryResult>, String> {
    state.db_manager.execute_queries(&connection_id, &queries).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn test_connection(
    connection_id: String,
    state: tauri::State<'_, AppState>
) -> Result<(), String> {
    state.db_manager.test_connection(&connection_id).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn load_connections_from_project(
    project_path: String,
    state: tauri::State<'_, AppState>
) -> Result<Vec<String>, String> {
    state.db_manager.load_from_project(&project_path).await
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Create a database manager
    let db_manager = Arc::new(create_db_manager());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_cli::init())
        .manage(AppState {
            db_manager: db_manager.clone(),
        })
        .setup(|app| {
            // Configure file system permissions
            let fs_scope = app.fs_scope();
            if let Err(e) = fs_scope.allow_directory("**/.sqratch", true) {
                eprintln!("Failed to set fs permissions: {}", e);
            }

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

            // If we have a project path, try to set up database connection
            if let Some(path) = project_path {
                println!("Attempting to find database connections for project at: {}", path);

                // Create a task to load connections in the background
                let db_manager_clone = db_manager.clone();
                let path_clone = path.clone();
                tauri::async_runtime::spawn(async move {
                    // Try to load connection config from project path
                    let connection_result = parse_connection_config(Some(&path_clone), None).await;
                    match connection_result {
                        Ok(Some(connection)) => {
                            println!("Found connection configuration in project path");

                            // Add connection to manager
                            match db_manager_clone.add_connection(connection).await {
                                Ok(id) => {
                                    println!("Added connection with ID: {}", id);

                                    // Try to connect
                                    if let Err(e) = db_manager_clone.connect(&id).await {
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
                            match db_manager_clone.load_from_project(&path_clone).await {
                                Ok(connection_ids) => {
                                    println!("Loaded {} connections from project", connection_ids.len());

                                    // Auto-connect to the first connection if available
                                    if let Some(first_id) = connection_ids.first() {
                                        if let Err(e) = db_manager_clone.connect(first_id).await {
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
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            list_connections,
            add_connection,
            connect_to_database,
            disconnect_from_database,
            is_connected,
            execute_query,
            execute_queries,
            test_connection,
            load_connections_from_project,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
