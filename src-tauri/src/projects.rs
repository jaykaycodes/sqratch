use std::path::Path;

use crate::db::ConnectionInfo;
use crate::utils;

#[derive(Debug, Clone)]
pub struct Project {
    pub id: ProjectId,
    pub name: String, // User-friendly name for the project
    pub connection_info: ConnectionInfo,
}

impl Project {
    pub fn new(id: ProjectId, name: String, connection_info: ConnectionInfo) -> Self {
        Self {
            id,
            name,
            connection_info,
        }
    }
}

/// Represents a project identifier that can either be a database URL or directory path
#[derive(Debug, Clone)]
pub enum ProjectId {
    Url(String),
    Directory(String),
}

impl ProjectId {
    /// Convert the project ID to a string representation
    pub fn to_string(&self) -> String {
        match self {
            ProjectId::Url(url) => url.clone(),
            ProjectId::Directory(path) => path.clone(),
        }
    }

    /// Derive a user-friendly name from the ProjectId
    pub fn derive_name(&self) -> String {
        match self {
            ProjectId::Url(url) => {
                // For simplicity now, just use the full URL, truncating if too long
                if url.len() > 50 {
                    format!("{}...", &url[..47])
                } else {
                    url.clone()
                }
            }
            ProjectId::Directory(path_str) => {
                // Use the final component of the path (directory name)
                Path::new(path_str).file_name().map_or_else(
                    || path_str.clone(),
                    |os_str| os_str.to_string_lossy().into_owned(),
                )
            }
        }
    }
}

/// Parse the "project" arg into either a database URL or absolute directory path
pub fn parse_project_arg(project_arg: &str, cwd: &str) -> Result<ProjectId, String> {
    // Check if it's a URL (has :// pattern)
    if project_arg.contains("://") {
        return Ok(ProjectId::Url(project_arg.to_string()));
    }

    // Otherwise treat as a path and ensure it's absolute
    let path = if Path::new(project_arg).is_absolute() {
        Path::new(project_arg).to_path_buf()
    } else {
        Path::new(cwd).join(project_arg)
    };

    // Ensure path exists and is a directory before creating ProjectId::Directory
    match std::fs::metadata(&path) {
        Ok(metadata) => {
            if metadata.is_dir() {
                Ok(ProjectId::Directory(path.to_string_lossy().into_owned()))
            } else {
                Err(format!(
                    "Project path exists but is not a directory: {}",
                    path.display()
                ))
            }
        }
        Err(e) => Err(format!("Invalid project path: {}: {}", path.display(), e)),
    }
}

/// Load connection info from either a database URL or directory containing .env
pub fn load_connection_info(project_id: &ProjectId) -> Result<ConnectionInfo, String> {
    match project_id {
        ProjectId::Url(url) => utils::strings::parse_connection_string(url),
        ProjectId::Directory(path) => {
            let env_path = Path::new(path).join(".env");
            if !env_path.exists() {
                return Err(format!("No .env file found in directory: {}", path));
            }

            // Read .env file content
            let content = std::fs::read_to_string(&env_path)
                .map_err(|e| format!("Failed to read .env file: {}", e))?;

            // Parse the file content to find DATABASE_URL
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                if let Some((key, value)) = line.split_once('=') {
                    if key.trim() == "DATABASE_URL" {
                        let url = value.trim().trim_matches('"').trim_matches('\'');
                        return utils::strings::parse_connection_string(url);
                    }
                }
            }

            Err("DATABASE_URL not found in .env file".to_string())
        }
    }
}
