use std::ops::Deref;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use url::Url;

use crate::errors::AppError;
use crate::utils;

#[derive(Debug, Clone)]
pub struct ProjectPath(PathBuf);

impl ProjectPath {
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }

    pub fn to_window_label(&self) -> String {
        format!("prj-{}", hash_str(&self.0.to_string_lossy()))
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

        // Check if input is a connection string
        if Url::parse(input).is_ok() {
            let dirname = hash_str(input);
            let path = app_data_dir.join("projects").join(dirname);
            return Ok(path.into());
        }

        // Handle file path (absolute or relative)
        let input_path = PathBuf::from(input);
        let resolved_path = if input_path.is_absolute() {
            input_path
        } else {
            PathBuf::from(cwd).join(input_path)
        };

        // If it's a file, assume it's a project config and return the parent dir
        if !resolved_path.is_dir() {
            return Ok(resolved_path.parent().unwrap().into());
        }

        if resolved_path.ends_with(".sqratch") {
            return Ok(resolved_path.into());
        }

        // If it doesn't end in .sqratch, look for a .sqratch subdir
        if !resolved_path.ends_with(".sqratch") {
            let sqratch_dir = resolved_path.join(".sqratch");
            if sqratch_dir.exists() {
                return Ok(sqratch_dir.into());
            }
        }

        Err("No .sqratch dir found".into())
    }
}

impl Deref for ProjectPath {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<Path> for ProjectPath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl From<PathBuf> for ProjectPath {
    fn from(path: PathBuf) -> Self {
        ProjectPath::new(path)
    }
}

impl From<&Path> for ProjectPath {
    fn from(path: &Path) -> Self {
        ProjectPath::new(path.to_path_buf())
    }
}

impl From<ProjectPath> for PathBuf {
    fn from(path: ProjectPath) -> Self {
        path.0
    }
}

/// We default to a hash of the DB connection string as the dirname for global projects
fn hash_str(conn_str: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(conn_str);
    let hash = hasher.finalize();

    // Take first 8 bytes and convert to a more readable format
    format!(
        "{:x}-{:x}-{:x}",
        u32::from_be_bytes(hash[0..4].try_into().unwrap()),
        u16::from_be_bytes(hash[4..6].try_into().unwrap()),
        u16::from_be_bytes(hash[6..8].try_into().unwrap())
    )
}
