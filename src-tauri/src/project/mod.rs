mod config;
mod handle;
mod helpers;

use url::Url;

use crate::errors::AppError;

pub use self::config::{ConfigError, ProjectConfig};
pub use self::handle::ProjectHandle;
use self::helpers::{infer_project_name, resolve_db_url};

/// Runtime reference to a project
#[taurpc::ipc_type]
#[derive(Debug)]
pub struct Project {
    /// Path to the project directory (it might not exist yet)
    #[serde(flatten)]
    pub handle: ProjectHandle,
    /// Name of the project (inferred if not provided in config)
    pub name: String,
    /// Database connection string
    #[specta(type = String)]
    pub db_url: Url,
}

impl Project {
    /// Loads a project config from disk if it exists, otherwise returns a new temporary project.
    pub fn load(handle: &ProjectHandle) -> Result<Self, AppError> {
        // Try to load config from the directory
        let config = ProjectConfig::load(handle)?;

        let db_url = resolve_db_url(&config.db, &handle.path)?;

        // Determine the project name if not provided in the config
        let name = match config.name {
            Some(name) => name,
            None => infer_project_name(&handle.path, &db_url)?,
        };

        Ok(Project {
            name,
            handle: handle.clone(),
            db_url,
        })
    }

    pub fn window_label(&self) -> String {
        self.handle.to_window_label()
    }
}
