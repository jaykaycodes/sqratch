use crate::db::{
    ConnectionInfo, DatabaseType, QueryResult,
};
use crate::AppState;
use serde_json::Value;
use tauri::State;
use tauri_specta::Event;

/// Test a connection string before saving
#[tauri::command]
#[specta::specta]
pub async fn test_connection_string(
    conn_string: String,
    db_type: DatabaseType
) -> Result<String, String> {
    use crate::db::manager::DatabaseManager;

    DatabaseManager::establish_connection(&conn_string, &db_type)
        .await
        .map(|_| "Connection successful".to_string())
        .map_err(|e| e.to_string())
}

/// List saved connections
#[tauri::command]
#[specta::specta]
pub async fn list_connections(
    state: State<'_, AppState>
) -> Result<Vec<ConnectionInfo>, String> {
    Ok(state.db_manager.list_connections().await)
}

/// Add a new connection
#[tauri::command]
#[specta::specta]
pub async fn add_connection(
    conn_info: ConnectionInfo,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let result = state.db_manager.add_connection(conn_info).await
        .map_err(|e| e.to_string())?;

    // Emit an event to notify UI
    #[derive(Clone, serde::Serialize)]
    pub struct ConnectionAddedEvent {
        id: String,
    }

    let event = ConnectionAddedEvent { id: result.clone() };
    let _ = event.emit(&app);

    Ok(result)
}

/// Connect to a database
#[tauri::command]
#[specta::specta]
pub async fn connect_to_database(
    connection_id: String,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let result = state.db_manager.connect(&connection_id).await
        .map(|_| ()) // Ignore the handler return value
        .map_err(|e| e.to_string())?;

    // Emit an event to notify UI
    #[derive(Clone, serde::Serialize)]
    pub struct ConnectionEstablishedEvent {
        id: String,
    }

    let event = ConnectionEstablishedEvent { id: connection_id };
    let _ = event.emit(&app);

    Ok(result)
}

/// Disconnect from a database
#[tauri::command]
#[specta::specta]
pub async fn disconnect_from_database(
    connection_id: String,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let result = state.db_manager.disconnect(&connection_id).await
        .map_err(|e| e.to_string())?;

    // Emit an event to notify UI
    #[derive(Clone, serde::Serialize)]
    pub struct ConnectionClosedEvent {
        id: String,
    }

    let event = ConnectionClosedEvent { id: connection_id };
    let _ = event.emit(&app);

    Ok(result)
}

/// Check if connected to a database
#[tauri::command]
#[specta::specta]
pub async fn is_connected(
    connection_id: String,
    state: State<'_, AppState>
) -> Result<bool, String> {
    Ok(state.db_manager.is_connected(&connection_id).await)
}

/// Execute a query on a database
#[tauri::command]
#[specta::specta]
pub async fn execute_query(
    connection_id: String,
    query: String,
    state: State<'_, AppState>
) -> Result<QueryResult, String> {
    state.db_manager.execute_query(&connection_id, &query).await
        .map_err(|e| e.to_string())
}

/// Execute multiple queries on a database
#[tauri::command]
#[specta::specta]
pub async fn execute_queries(
    connection_id: String,
    queries: String,
    state: State<'_, AppState>
) -> Result<Vec<QueryResult>, String> {
    state.db_manager.execute_queries(&connection_id, &queries).await
        .map_err(|e| e.to_string())
}

/// Test an existing connection
#[tauri::command]
#[specta::specta]
pub async fn test_connection(
    connection_id: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    state.db_manager.test_connection(&connection_id).await
        .map_err(|e| e.to_string())
}

/// Load connections from a project
#[tauri::command]
#[specta::specta]
pub async fn load_connections_from_project(
    project_path: String,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<Vec<String>, String> {
    let result = state.db_manager.load_from_project(&project_path).await
        .map_err(|e| e.to_string())?;

    // Emit an event to notify UI if connections were loaded
    if !result.is_empty() {
        #[derive(Clone, serde::Serialize)]
        pub struct ConnectionsLoadedEvent {
            count: usize,
            ids: Vec<String>,
        }

        let event = ConnectionsLoadedEvent {
            count: result.len(),
            ids: result.clone(),
        };
        let _ = event.emit(&app);
    }

    Ok(result)
}

/// Get schema information for a database
#[tauri::command]
#[specta::specta]
pub async fn get_schema_info(
    connection_id: String,
    state: State<'_, AppState>
) -> Result<Value, String> {
    let handler = state.db_manager.get_handler(&connection_id).await
        .map_err(|e| e.to_string())?;

    let schema = handler.get_schema_info().await
        .map_err(|e| e.to_string())?;

    serde_json::to_value(schema)
        .map_err(|e| format!("Failed to serialize schema: {}", e))
}

/// Get tables for a database
#[tauri::command]
#[specta::specta]
pub async fn get_tables(
    connection_id: String,
    state: State<'_, AppState>
) -> Result<Value, String> {
    let handler = state.db_manager.get_handler(&connection_id).await
        .map_err(|e| e.to_string())?;

    let tables = handler.get_tables().await
        .map_err(|e| e.to_string())?;

    serde_json::to_value(tables)
        .map_err(|e| format!("Failed to serialize tables: {}", e))
}

/// Get paginated rows for a table
#[tauri::command]
#[specta::specta]
pub async fn get_paginated_rows(
    connection_id: String,
    table_name: String,
    page_index: u16,
    page_size: u32,
    state: State<'_, AppState>
) -> Result<QueryResult, String> {
    let handler = state.db_manager.get_handler(&connection_id).await
        .map_err(|e| e.to_string())?;

    handler.get_paginated_rows(&table_name, page_index, page_size).await
        .map_err(|e| e.to_string())
}
