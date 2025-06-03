use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder, Window};

use log;
use std::env;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

use crate::errors::AppError;
use crate::project::{Project, ProjectHandle};
use crate::state::{cleanup_window_state, init_project_window};

// Window labels
pub const LAUNCHER_LABEL: &str = "launcher";

// Window URLs
const LAUNCHER_URL: &str = "/";
const PROJECT_URL: &str = "/project";

// Window sizes
const MIN_WINDOW_WIDTH: f64 = 400.0;
const MIN_WINDOW_HEIGHT: f64 = 300.0;
const LAUNCHER_WIDTH: f64 = 800.0;
const LAUNCHER_HEIGHT: f64 = 600.0;
const DEFAULT_WIDTH: f64 = 800.0;
const DEFAULT_HEIGHT: f64 = 600.0;

pub fn launch_app(app: &AppHandle) {
    let args = app
        .env()
        .args_os
        .iter()
        .map(|s| s.to_string_lossy().to_string())
        .collect();
    let dir = env::current_dir().unwrap_or_default();
    let cwd = dir.to_string_lossy();

    launch_window(app, args, &cwd).unwrap_or_else(|e| {
        app.dialog()
            .message(format!("Failed to open: {}", e))
            .kind(MessageDialogKind::Error)
            .buttons(MessageDialogButtons::Ok)
            .blocking_show();
        app.exit(1);
    });
}

pub fn launch_instance(app: &AppHandle, args: Vec<String>, cwd: &str) {
    launch_window(app, args, cwd).unwrap_or_else(|e| {
        app.dialog()
            .message(format!("Failed to open project: {}", e))
            .kind(MessageDialogKind::Error)
            .buttons(MessageDialogButtons::Ok)
            .show(|_| {});
    })
}

pub fn close_window(window: &Window) {
    if window.label() != LAUNCHER_LABEL {
        cleanup_window_state(window);
    }
}

fn launch_window(app: &AppHandle, args: Vec<String>, cwd: &str) -> Result<(), AppError> {
    #[cfg(all(windows, not(dev)))]
    attach_console();

    log::debug!("Received args: {:?} from cwd: {}", args, cwd);
    if let Some(arg) = args.get(1) {
        let handle = ProjectHandle::from_cli_input(arg, cwd)?;
        open_project_window(app, &handle)?;
    } else {
        // No project specified, open launcher
        open_launcher_window(app)?;
    }

    #[cfg(all(windows, not(dev)))]
    free_console();

    Ok(())
}

/// Only on windows.
///
/// Attaches the console so the user can see output in the terminal.
#[cfg(all(windows, not(dev)))]
fn attach_console() {
    use windows::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};
    let _ = unsafe { AttachConsole(ATTACH_PARENT_PROCESS) };
}

/// Only on windows.
///
/// Frees the console so the user won't see weird println's.
#[cfg(all(windows, not(dev)))]
fn free_console() {
    use windows::Win32::System::Console::FreeConsole;
    let _ = unsafe { FreeConsole() };
}

/// Opens (or focuses) a project window using a unique window ID
fn open_project_window(app: &AppHandle, handle: &ProjectHandle) -> Result<(), AppError> {
    let window_label = handle.to_window_label();

    log::debug!("Opening window: {}", window_label);

    // If window already exists, just focus it
    if let Some(_) = ensure_window_visible(app, &window_label) {
        return Ok(());
    }

    // Otherwise, load the project and open a window for it
    let project = Project::load(handle)?;

    let title = project.name.clone();
    init_project_window(app, project)?;

    WebviewWindowBuilder::new(app, window_label, WebviewUrl::App(PROJECT_URL.into()))
        .title(title)
        .min_inner_size(MIN_WINDOW_WIDTH, MIN_WINDOW_HEIGHT)
        .inner_size(DEFAULT_WIDTH, DEFAULT_HEIGHT)
        .title_bar_style(tauri::TitleBarStyle::Overlay)
        .build()
        .map_err(|e| AppError::Other(e.to_string()))?;
    Ok(())
}

/// Opens (or focuses) the launcher window when no project is specified
fn open_launcher_window(app: &AppHandle) -> Result<(), AppError> {
    log::debug!("Opening launcher window");

    // Check if launcher already exists
    if let Some(_) = ensure_window_visible(app, LAUNCHER_LABEL) {
        return Ok(());
    }

    WebviewWindowBuilder::new(app, LAUNCHER_LABEL, WebviewUrl::App(LAUNCHER_URL.into()))
        .title("Sqratch")
        .inner_size(LAUNCHER_WIDTH, LAUNCHER_HEIGHT)
        .resizable(false)
        .title_bar_style(tauri::TitleBarStyle::Overlay)
        .build()
        .map_err(|e| AppError::Other(e.to_string()))?;

    Ok(())
}

fn ensure_window_visible(app: &AppHandle, label: &str) -> Option<WebviewWindow> {
    if let Some(window) = app.get_webview_window(label) {
        let _ = window.show();
        let _ = window.set_focus();
        return Some(window);
    }

    None
}
