use tauri::{AppHandle, Runtime, WebviewUrl, WebviewWindowBuilder};

// App constants
pub const APP_NAME: &str = "Sqratch";

// Window labels
pub const LAUNCHER_LABEL: &str = "launcher";

// Window URLs
pub const LAUNCHER_URL: &str = "/";
pub const PROJECT_URL: &str = "/project";

// Window dimensions
pub const MIN_WINDOW_WIDTH: f64 = 400.0;
pub const MIN_WINDOW_HEIGHT: f64 = 300.0;
pub const LAUNCHER_WIDTH: f64 = 800.0;
pub const LAUNCHER_HEIGHT: f64 = 600.0;
pub const DEFAULT_WIDTH: f64 = 800.0;
pub const DEFAULT_HEIGHT: f64 = 600.0;

/// Creates a new launcher window configuration
pub fn create_launcher_window_config<R: Runtime>(
    app: &AppHandle<R>,
) -> WebviewWindowBuilder<R, AppHandle<R>> {
    WebviewWindowBuilder::new(app, LAUNCHER_LABEL, WebviewUrl::App(LAUNCHER_URL.into()))
        .title(APP_NAME)
        .inner_size(LAUNCHER_WIDTH, LAUNCHER_HEIGHT)
        .resizable(false)
}

/// Creates a new project window configuration
pub fn create_project_window_config<R: Runtime>(
    app: &AppHandle<R>,
    window_label: String,
    title: String,
) -> WebviewWindowBuilder<R, AppHandle<R>> {
    WebviewWindowBuilder::new(app, window_label, WebviewUrl::App(PROJECT_URL.into()))
        .title(title)
        .min_inner_size(MIN_WINDOW_WIDTH, MIN_WINDOW_HEIGHT)
        .inner_size(DEFAULT_WIDTH, DEFAULT_HEIGHT)
}
