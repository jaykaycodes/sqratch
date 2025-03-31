use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Represents a project identifier that can be a database URL, directory path, or file path
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(tag = "type", content = "value")]
pub enum ProjectId {
    /// Direct database URL
    Url(String),

    /// Directory containing a .env file
    Directory(PathBuf),

    /// Direct path to a .env file
    File(PathBuf),
}

impl ProjectId {
    /// Convert the ProjectId to a unique window label
    pub fn to_window_label(&self) -> String {
        match self {
            ProjectId::Url(url) => format!("project_url_{}", hash_string(url)),
            ProjectId::Directory(path) => {
                format!("project_dir_{}", hash_string(&path.to_string_lossy()))
            }
            ProjectId::File(path) => {
                format!("project_file_{}", hash_string(&path.to_string_lossy()))
            }
        }
    }

    /// Get a user-friendly display name for the project
    pub fn display_name(&self) -> String {
        match self {
            ProjectId::Url(url) => {
                // Create a more readable version of the URL
                if url.len() > 50 {
                    format!("{}...", &url[..47])
                } else {
                    url.clone()
                }
            }
            ProjectId::Directory(path) => {
                // Use the directory name
                path.file_name().map_or_else(
                    || path.to_string_lossy().into_owned(),
                    |name| name.to_string_lossy().into_owned(),
                )
            }
            ProjectId::File(path) => {
                // Use the parent directory + filename
                let file_name = path.file_name().map_or_else(
                    || path.to_string_lossy().into_owned(),
                    |name| name.to_string_lossy().into_owned(),
                );

                path.parent()
                    .and_then(|parent| parent.file_name())
                    .map_or_else(
                        || file_name.clone(),
                        |parent_name| format!("{}/{}", parent_name.to_string_lossy(), file_name),
                    )
            }
        }
    }
}

/// Loads the database connection string from the project identifier
pub fn load_connection_string(project_id: &ProjectId) -> Result<String, String> {
    match project_id {
        ProjectId::Url(url) => Ok(url.clone()),
        ProjectId::Directory(path) => {
            // Look for .env file in the directory
            let env_path = path.join(".env");
            if !env_path.exists() {
                return Err(format!("Env file not found: {}", path.to_string_lossy()));
            }
            extract_db_url_from_env_file(&env_path)
        }
        ProjectId::File(path) => {
            // Use the file directly as a .env file
            extract_db_url_from_env_file(path)
        }
    }
}

/// Extracts DATABASE_URL from a .env file
fn extract_db_url_from_env_file(path: &Path) -> Result<String, String> {
    // Make sure the file exists
    if !path.exists() {
        return Err(format!("Path not found: {}", path.to_string_lossy()));
    }

    // Check that it's a file
    if !path.is_file() {
        return Err(format!("Not a file: {}", path.to_string_lossy()));
    }

    // Read the file content
    let content = fs::read_to_string(path)
        .map_err(|_| format!("Failed to read file: {}", path.to_string_lossy()))?;

    // Parse manually to find DATABASE_URL
    for line in content.lines() {
        let line = line.trim();
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Split by first equals sign
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            if key == "DATABASE_URL" {
                // Clean the value (remove quotes if present)
                let url = value.trim().trim_matches('"').trim_matches('\'');
                if url.is_empty() {
                    return Err(format!("Empty connection string"));
                }
                return Ok(url.to_string());
            }
        }
    }

    Err(format!(
        "Database URL not found in file: {}",
        path.to_string_lossy()
    ))
}

/// Parse a project argument from the command line
pub fn parse_project_arg(arg: &str, cwd: &str) -> Result<ProjectId, String> {
    // Check if it's a URL (has a scheme)
    if arg.contains("://") {
        return Ok(ProjectId::Url(arg.to_string()));
    }

    // Otherwise, treat as a path (relative to cwd if not absolute)
    let path = if Path::new(arg).is_absolute() {
        Path::new(arg).to_path_buf()
    } else {
        Path::new(cwd).join(arg)
    };

    // Canonicalize to resolve .. and symlinks
    let path = fs::canonicalize(&path)
        .map_err(|_| format!("Failed to resolve path: {}", path.to_string_lossy()))?;

    // Check if the path exists
    if !path.exists() {
        return Err(format!("Path not found: {}", path.to_string_lossy()));
    }

    // Determine if it's a file or directory
    if path.is_dir() {
        Ok(ProjectId::Directory(path))
    } else {
        // Check if it's a .env file
        if path.file_name().map_or(false, |name| name == ".env") {
            Ok(ProjectId::File(path))
        } else {
            // Check if it's another type of file (could be a database file itself)
            Ok(ProjectId::File(path))
        }
    }
}

// Simple hash function to create an identifier from a string
fn hash_string(s: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}
