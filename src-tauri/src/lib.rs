// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri_plugin_cli;
use tauri_plugin_fs;
use std::sync::Arc;

// Module declarations
mod db;
mod commands;
mod startup;
mod cli;

// Import and re-export key types for the frontend
pub use db::types::{ConnectionInfo, DatabaseType};
pub use db::manager::{DatabaseManager, create_db_manager, parse_connection_config};

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
        .setup(move |app| {
            // Configure file system permissions
            startup::configure_fs_permissions(app);

            // Process CLI arguments from initial launch
            let db_manager_clone = db_manager.clone();
            if let Err(e) = cli::process_cli_args(app, db_manager_clone) {
                eprintln!("Failed to process CLI arguments: {}", e);
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
