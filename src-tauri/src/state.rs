use std::collections::HashMap;
use std::sync::RwLock;

use tauri::{AppHandle, Manager, Runtime, Window};

use crate::db::client::{create_client, DatabaseClientRef};
use crate::errors::AppError;
use crate::projects::{load_connection_string, ProjectId};

/// A map of window labels to database clients
pub struct AppState {
    clients: RwLock<HashMap<String, DatabaseClientRef>>,
}

impl AppState {}

impl Default for AppState {
    fn default() -> Self {
        Self {
            clients: RwLock::new(HashMap::new()),
        }
    }
}

pub fn get_window_client<R: Runtime>(window: &Window<R>) -> Result<DatabaseClientRef, AppError> {
    let app = window.app_handle();
    let state = app.state::<AppState>();
    let clients = state.clients.read().unwrap();

    let client = clients
        .get(window.label())
        .ok_or(AppError::Other("Client not found".to_string()))?;

    return Ok(client.clone());
}

pub fn init_state_for_project(app: &AppHandle, project_id: &ProjectId) -> Result<(), AppError> {
    let state = app.state::<AppState>();

    let connection_string = load_connection_string(project_id)?;
    let client = create_client(&connection_string)?;

    state
        .clients
        .write()
        .unwrap()
        .insert(project_id.to_window_label(), client);

    Ok(())
}

pub fn cleanup_window_state<R: Runtime>(window: &Window<R>) {
    let app = window.app_handle();
    let state = app.state::<AppState>();
    state.clients.write().unwrap().remove(window.label());
}
