use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};
use url::Url;

use crate::errors::AppError;
use crate::utils;

use super::config;

/// Resolves a database URL from either:
/// - A direct connection string
/// - A path to an .env file with optional environment variable name (e.g. "../.env|DB_URL")
pub fn resolve_db_url(db_url: &str, cwd: &Path) -> Result<Url, AppError> {
    // Try direct URL first
    if let Ok(url) = Url::parse(db_url) {
        return Ok(url);
    }

    // Parse env file path and optional var name
    let (file_path, var_name) = match db_url.split_once('|') {
        Some((path, var)) => (path.trim(), var.trim()),
        None => (db_url.trim(), "DATABASE_URL"),
    };

    // Resolve absolute path
    let abs_path = if Path::new(file_path).is_relative() {
        cwd.join(file_path)
    } else {
        PathBuf::from(file_path)
    };

    // Read and parse .env file
    let env_content =
        fs::read_to_string(&abs_path).map_err(|e| AppError::Config(config::ConfigError::Io(e)))?;

    // Find the env var, handling comments and empty lines
    let db_url = env_content
        .lines()
        .find_map(|line| {
            let line = line.trim();
            match line.split_once('=') {
                Some((key, value)) if !line.starts_with('#') && key.trim() == var_name => {
                    Some(value.trim().trim_matches(|c| c == '"' || c == '\''))
                }
                _ => None,
            }
        })
        .ok_or_else(|| {
            AppError::Config(config::ConfigError::Other(format!(
                "No {} found in .env file",
                var_name
            )))
        })?;

    // Parse and validate the URL
    Url::parse(db_url).map_err(|e| {
        AppError::Config(config::ConfigError::Other(format!(
            "Invalid database URL in .env file: {}",
            e
        )))
    })
}

/// Infer a project name based on location:
/// - For app data projects (from connection strings): use database name from the connection string
/// - For local directory projects: use parent directory name
pub fn infer_project_name(path: &Path, db_url: &Url) -> Result<String, AppError> {
    let projects_dir = utils::paths::global_projects_dir()?;

    // Check if we're in the app data directory
    if path.starts_with(&projects_dir) {
        // For app data projects (likely from connection strings), extract DB name
        if let Some(segments) = db_url.path_segments() {
            if let Some(db_name) = segments.last() {
                if !db_name.is_empty() {
                    return Ok(db_name.to_string());
                }
            }
        }

        // Fallback: Try to use the host as part of the name
        if let Some(host) = db_url.host_str() {
            return Ok(format!("DB on {}", host));
        }

        // Last resort for app data projects
        return Ok("Unnamed Database".to_string());
    } else {
        // For local directory projects, use parent directory name
        let parent_dir = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|name| name.to_str())
            .ok_or_else(|| "Unable to determine parent directory name".to_string())?;

        Ok(parent_dir.to_string())
    }
}

/// We default to a hash of the DB connection string as the dirname for global projects
pub fn hash_str(conn_str: &str) -> String {
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
