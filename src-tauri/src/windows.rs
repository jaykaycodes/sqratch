use tauri::{AppHandle, WebviewUrl, WebviewWindowBuilder};

use crate::constants::{APP_NAME, LAUNCHER_LABEL, LAUNCHER_URL, PROJECT_URL};

const MIN_WINDOW_WIDTH: f64 = 400.0;
const MIN_WINDOW_HEIGHT: f64 = 300.0;
const LAUNCHER_WIDTH: f64 = 800.0;
const LAUNCHER_HEIGHT: f64 = 600.0;
const DEFAULT_WIDTH: f64 = 800.0;
const DEFAULT_HEIGHT: f64 = 600.0;

/// Creates a new launcher window configuration
pub fn build_launcher_window(app: &AppHandle) -> Result<(), tauri::Error> {
    WebviewWindowBuilder::new(app, LAUNCHER_LABEL, WebviewUrl::App(LAUNCHER_URL.into()))
        .title(APP_NAME)
        .inner_size(LAUNCHER_WIDTH, LAUNCHER_HEIGHT)
        .resizable(false)
        .title_bar_style(tauri::TitleBarStyle::Overlay)
        .build()?;

    Ok(())
}

/// Creates a new project window configuration
pub fn build_project_window(
    app: &AppHandle,
    window_label: String,
    title: String,
) -> Result<(), tauri::Error> {
    WebviewWindowBuilder::new(app, window_label, WebviewUrl::App(PROJECT_URL.into()))
        .title(title)
        .min_inner_size(MIN_WINDOW_WIDTH, MIN_WINDOW_HEIGHT)
        .inner_size(DEFAULT_WIDTH, DEFAULT_HEIGHT)
        .title_bar_style(tauri::TitleBarStyle::Overlay)
        .build()?;

    Ok(())
}
