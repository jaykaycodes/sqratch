use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use tauri::{AppHandle, Manager, Runtime, Window};
use tokio::sync::Mutex;

use crate::db::client::{create_client, DatabaseClient};
use crate::errors::AppError;
use crate::project::Project;

pub struct WindowState {
    project: Arc<Project>,
    client: Arc<Mutex<dyn DatabaseClient>>,
}

pub struct AppState {
    /// A map of window labels to their state
    windows: RwLock<HashMap<String, WindowState>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            windows: RwLock::new(HashMap::new()),
        }
    }
}

pub fn get_window_client(
    window: &Window<impl Runtime>,
) -> Result<Arc<Mutex<dyn DatabaseClient>>, AppError> {
    let app = window.app_handle();
    let state = app.state::<AppState>();
    let windows = state.windows.read().unwrap();

    let window_state = windows
        .get(window.label())
        .ok_or(AppError::Other("Window not found".to_string()))?;

    return Ok(window_state.client.clone());
}

pub fn get_window_project(window: &Window<impl Runtime>) -> Result<Arc<Project>, AppError> {
    let app = window.app_handle();
    let state = app.state::<AppState>();
    let windows = state.windows.read().unwrap();

    let window_state = windows
        .get(window.label())
        .ok_or(AppError::Other("Window not found".to_string()))?;

    return Ok(window_state.project.clone());
}

pub fn init_project_window(app: &AppHandle, project: Project) -> Result<(), AppError> {
    let state = app.state::<AppState>();

    let client = create_client(&project.db_url)?;

    let window_label = project.window_label();
    let window_state = WindowState {
        project: Arc::new(project),
        client: Arc::new(Mutex::new(client)),
    };

    state
        .windows
        .write()
        .unwrap()
        .insert(window_label, window_state);

    Ok(())
}

pub fn cleanup_window_state(window: &Window) {
    let app = window.app_handle();
    let state = app.state::<AppState>();
    state.windows.write().unwrap().remove(window.label());
}
