use once_cell::sync::OnceCell;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

use crate::errors::AppError;

/// Global app_data_dir accessor
static APP_DATA_DIR: OnceCell<Mutex<Option<PathBuf>>> = OnceCell::new();

/// Initialize the global app data directory
/// This should be called once during app setup
pub fn init_paths<R: tauri::Runtime>(app_handle: &AppHandle<R>) {
    let path = app_handle.path().app_data_dir().ok();
    let _ = APP_DATA_DIR.set(Mutex::new(path));
}

pub fn app_data_dir() -> Result<PathBuf, AppError> {
    let guard = APP_DATA_DIR
        .get()
        .ok_or_else(|| "App data dir not initialized")?
        .lock()
        .map_err(|_| "Failed to acquire lock")?;

    guard
        .as_ref()
        .cloned()
        .ok_or_else(|| "App data dir is None".into())
}

pub fn global_projects_dir() -> Result<PathBuf, AppError> {
    Ok(app_data_dir()?.join("projects"))
}
