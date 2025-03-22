use std::sync::Arc;
use crate::db::{
    ConnectionInfo, DatabaseManager, DbResult, QueryResult,
};

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Database related commands
#[tauri::command]
pub async fn list_connections(state: tauri::State<'_, super::AppState>) -> Result<Vec<ConnectionInfo>, String> {
    Ok(state.db_manager.list_connections().await)
}

#[tauri::command]
pub async fn add_connection(
    conn_info: ConnectionInfo,
    state: tauri::State<'_, super::AppState>
) -> Result<String, String> {
    state.db_manager.add_connection(conn_info).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn connect_to_database(
    connection_id: String,
    state: tauri::State<'_, super::AppState>
) -> Result<(), String> {
    state.db_manager.connect(&connection_id).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn disconnect_from_database(
    connection_id: String,
    state: tauri::State<'_, super::AppState>
) -> Result<(), String> {
    state.db_manager.disconnect(&connection_id).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn is_connected(
    connection_id: String,
    state: tauri::State<'_, super::AppState>
) -> Result<bool, String> {
    Ok(state.db_manager.is_connected(&connection_id).await)
}

#[tauri::command]
pub async fn execute_query(
    connection_id: String,
    query: String,
    state: tauri::State<'_, super::AppState>
) -> Result<QueryResult, String> {
    state.db_manager.execute_query(&connection_id, &query).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn execute_queries(
    connection_id: String,
    queries: String,
    state: tauri::State<'_, super::AppState>
) -> Result<Vec<QueryResult>, String> {
    state.db_manager.execute_queries(&connection_id, &queries).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn test_connection(
    connection_id: String,
    state: tauri::State<'_, super::AppState>
) -> Result<(), String> {
    state.db_manager.test_connection(&connection_id).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn load_connections_from_project(
    project_path: String,
    state: tauri::State<'_, super::AppState>
) -> Result<Vec<String>, String> {
    state.db_manager.load_from_project(&project_path).await
        .map_err(|e| e.to_string())
}
