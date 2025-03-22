// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri_plugin_cli;
use tauri_plugin_fs;
use std::sync::Arc;

// Module declarations
mod db;
mod commands;
mod startup;

// Re-export modules with public interface
use db::{
    ConnectionInfo, DatabaseType, DatabaseManager,
    create_db_manager, parse_connection_config, DbResult,
};

// State for database manager
pub struct AppState {
    db_manager: Arc<DatabaseManager>,
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
            startup::configure_fs_permissions(app);

            // Get project path from CLI or environment
            if let Some(path) = startup::get_project_path(app) {
                // Handle project path and database connection
                let db_manager_clone = db_manager.clone();
                let path_clone = path.clone();
                tauri::async_runtime::spawn(async move {
                    startup::handle_project_path_async(path_clone, db_manager_clone).await;
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::list_connections,
            commands::add_connection,
            commands::connect_to_database,
            commands::disconnect_from_database,
            commands::is_connected,
            commands::execute_query,
            commands::execute_queries,
            commands::test_connection,
            commands::load_connections_from_project,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
