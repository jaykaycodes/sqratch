use std::path::PathBuf;

use url::Url;

use crate::errors::AppError;
use crate::utils;

use super::helpers::hash_str;

#[taurpc::ipc_type]
#[derive(Debug)]
pub struct ProjectHandle {
    /// Unique identifier for the project, hashed from the path
    pub id: String,
    /// Path to the project (.sqratch) directory (it might not exist yet)
    pub path: PathBuf,
    /// Whether the project is temporary (not saved to the file system)
    pub is_temp: bool,
    /// If arg was a connection string, this will be Some
    #[serde(skip)]
    pub url: Option<Url>,
}

impl ProjectHandle {
    pub fn to_window_label(&self) -> String {
        format!("prj-{}", self.id)
    }

    /// Resolves a project path from CLI input to a sqratch project directory
    /// It's possible that this dir does not yet exist or does not contain a config yet.
    /// We don't load the config here yet so we can open existing project windows first.
    ///
    /// The path may be:
    /// - A connection string (converted to a hashed directory in app data)
    /// - An absolute path
    /// - A relative path (resolved against current working directory)
    ///
    /// If it's a path that doesn't end in .sqratch, we append .sqratch to it.
    pub fn from_cli_input(input: &str, cwd: &str) -> Result<Self, AppError> {
        let app_data_dir = utils::paths::app_data_dir()?;

        // First, check if the input is a valid URL (connection string)
        if let Ok(url) = Url::parse(input) {
            let id = hash_str(input);
            let path = app_data_dir.join("projects").join(&id);
            let is_temp = !path.exists();
            return Ok(Self {
                id,
                path,
                is_temp,
                url: Some(url),
            });
        }

        // Handle file paths, resolving .. and . segments
        let input_path = PathBuf::from(input);
        let mut resolved_path = if input_path.is_absolute() {
            input_path
        } else {
            PathBuf::from(cwd).join(input_path)
        };
        resolved_path = resolved_path
            .canonicalize()
            .or_else(|_| Ok::<PathBuf, AppError>(resolved_path.clone()))
            .unwrap_or(resolved_path);

        if !resolved_path.exists() {
            return Err(AppError::Other(format!(
                "Path does not exist: {}",
                resolved_path.display()
            )));
        }

        // Determine project directory
        let project_path = match (resolved_path.is_dir(), resolved_path.ends_with(".sqratch")) {
            // Case 1: It's a directory ending with .sqratch
            (true, true) => resolved_path,
            // Case 2: It's a directory not ending with .sqratch, check for .sqratch subdir
            (true, false) => {
                let sqratch_dir = resolved_path.join(".sqratch");
                if sqratch_dir.exists() {
                    sqratch_dir
                } else {
                    return Err(AppError::Other(format!(
                        "No .sqratch directory found in: {}",
                        resolved_path.display()
                    )));
                }
            }
            // Case 3: It's not a directory (likely a file), use parent directory
            (false, _) => resolved_path
                .parent()
                .map(|p| p.to_path_buf())
                .ok_or_else(|| {
                    AppError::Other(format!(
                        "No parent directory for: {}",
                        resolved_path.display()
                    ))
                })?,
        };

        if !project_path.join("config.json").exists() {
            return Err(AppError::Other(format!(
                "No config.json found in: {}",
                project_path.display()
            )));
        }

        let id = hash_str(&project_path.to_string_lossy());

        Ok(Self {
            id,
            path: project_path,
            is_temp: false,
            url: None,
        })
    }
}
