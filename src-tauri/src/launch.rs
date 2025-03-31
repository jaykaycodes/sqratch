use clap::Parser;
use log;
use std::error::Error;
use std::ffi::OsString;
use tauri::{AppHandle, Manager, Window};

use crate::projects::{parse_project_arg, ProjectId};
use crate::state::{cleanup_window_state, init_state_for_project};
use crate::utils::errors::AppError;

pub const LAUNCHER_LABEL: &str = "launcher";

#[derive(Parser, Debug, Default)]
#[command(version, about)]
struct SqratchArgs {
    /// Project to open (URL, directory, or file path)
    project: Option<String>,
}

pub fn launch_window<I, T>(app: &AppHandle, args: I, cwd: String) -> Result<(), Box<dyn Error>>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    #[cfg(all(windows, not(dev)))]
    attach_console();

    let args = SqratchArgs::try_parse_from(args).unwrap_or_default();

    log::debug!("Launching window with args: {:?}", args);
    if let Some(project_arg) = args.project {
        let project_id = parse_project_arg(&project_arg, &cwd)?;
        open_project_window(app, &project_id)?;
    } else {
        // No project specified, open launcher
        open_launcher_window(app)?;
    }

    #[cfg(all(windows, not(dev)))]
    free_console();

    Ok(())
}

/// Convenience wrapper that derives arguments from the app handle
pub fn launch_app(app: &AppHandle) {
    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    launch_window(app, app.env().args_os, cwd).unwrap_or_else(|e| {
        eprintln!("Failed to launch sqratch: {}", e);
        app.exit(1);
    });
}

pub fn close_window(window: &Window) {
    if window.label() != LAUNCHER_LABEL {
        cleanup_window_state(window);
    }
}

/// Opens (or focuses) a project window using a unique window ID
fn open_project_window(app: &AppHandle, project_id: &ProjectId) -> Result<(), AppError> {
    log::debug!("Opening project window: {}", project_id.to_window_label());

    // If window already exists, just focus it
    if let Some(existing_window) = app.get_webview_window(&project_id.to_window_label()) {
        let _ = existing_window.show();
        let _ = existing_window.set_focus();
        return Ok(());
    }

    init_state_for_project(app, project_id)?;

    // Create window using Tauri v2 API
    let window_config = tauri::WebviewWindowBuilder::new(
        app,
        project_id.to_window_label(),
        tauri::WebviewUrl::App("index.html".into()),
    )
    .title(project_id.display_name());

    let _window = window_config.build().map_err(|e| e.to_string())?;

    Ok(())
}

/// Opens (or focuses) the launcher window when no project is specified
fn open_launcher_window(app: &AppHandle) -> Result<(), String> {
    log::debug!("Opening launcher window");

    // Check if launcher already exists
    if let Some(existing_window) = app.get_webview_window(LAUNCHER_LABEL) {
        // Just focus the existing launcher instead of creating a new one
        let _ = existing_window.show();
        let _ = existing_window.set_focus();
        return Ok(());
    }

    // Create the launcher window using Tauri v2 API
    let window_config = tauri::WebviewWindowBuilder::new(
        app,
        LAUNCHER_LABEL,
        tauri::WebviewUrl::App("launcher.html".into()),
    )
    .title("Projects");

    let _launcher = window_config.build().map_err(|e| e.to_string())?;

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
/// Frees the console so the user won't see weird println's
/// after he is done using the cli.
#[cfg(all(windows, not(dev)))]
fn free_console() {
    use windows::Win32::System::Console::FreeConsole;
    let _ = unsafe { FreeConsole() };
}
