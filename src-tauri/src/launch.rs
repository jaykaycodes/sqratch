use clap::Parser;
use log;
use std::env;
use std::ffi::OsString;
use std::fmt::Debug;
use tauri::{AppHandle, Manager, WebviewWindow, Window};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

use crate::constants::{APP_NAME, LAUNCHER_LABEL};
use crate::errors::AppError;
use crate::projects::{parse_project_arg, ProjectId};
use crate::state::{cleanup_window_state, init_state_for_project};
use crate::windows::{build_launcher_window, build_project_window};

#[derive(Parser, Debug, Default)]
#[command(version, about)]
struct SqratchArgs {
    /// Project to open (URL, directory, or file path)
    project: Option<String>,
}

fn launch_window<T>(app: &AppHandle, args: Vec<T>, cwd: &str) -> Result<(), AppError>
where
    T: Into<OsString> + Clone + Debug,
{
    #[cfg(all(windows, not(dev)))]
    attach_console();

    log::debug!("Received args: {:?} from cwd: {}", args, cwd);
    let args = SqratchArgs::try_parse_from(args)?;
    log::debug!("Parsed args: {:?}", args);

    if let Some(project_id) = args.project {
        let project_id = parse_project_arg(&project_id, &cwd)?;
        open_project_window(app, &project_id)?;
    } else {
        // No project specified, open launcher
        open_launcher_window(app)?;
    }

    #[cfg(all(windows, not(dev)))]
    free_console();

    Ok(())
}

pub fn launch_instance(app: &AppHandle, args: Vec<String>, cwd: &str) {
    launch_window(app, args, cwd).unwrap_or_else(|e| {
        app.dialog()
            .message(format!("Failed to open project: {}", e))
            .kind(MessageDialogKind::Error)
            .title(APP_NAME)
            .buttons(MessageDialogButtons::Ok)
            .show(|_| {});
    })
}

pub fn launch_app(app: &AppHandle) {
    let args = app.env().args_os;
    let dir = env::current_dir().unwrap_or_default();
    let cwd = dir.to_string_lossy();

    launch_window(app, args, &cwd).unwrap_or_else(|e| {
        app.dialog()
            .message(format!("Failed to open: {}", e))
            .kind(MessageDialogKind::Error)
            .title(APP_NAME)
            .buttons(MessageDialogButtons::Ok)
            .blocking_show();
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
    log::debug!(
        "Opening window: {} ({})",
        project_id.display_name(),
        project_id.to_window_label()
    );

    // If window already exists, just focus it
    if let Some(_) = ensure_window_visible(app, &project_id.to_window_label()) {
        return Ok(());
    }

    init_state_for_project(app, project_id)?;

    build_project_window(app, project_id.to_window_label(), project_id.display_name())
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

    build_launcher_window(app).map_err(|e| AppError::Other(e.to_string()))?;

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
