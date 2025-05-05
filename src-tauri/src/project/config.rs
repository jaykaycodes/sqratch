use super::ProjectPath;

/// Represents the user-defined configuration for a project
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ProjectConfig {
    /// Optional project name; if not set, it will be inferred from the directory or database name.
    pub name: Option<String>,
    /// Database connection string or path to a .env file with a DATABASE_URL variable.
    /// Format for .env path can include an environment name, e.g., "../.env|ENV_NAME".
    pub db: String,
}

impl ProjectConfig {
    /// Loads a project configuration from a specified path, supporting both direct file paths
    /// and directory paths containing a `config.json` file.
    ///
    /// # Arguments
    /// * `path` - A string slice representing the path to the configuration file or directory.
    ///
    /// # Errors
    /// Returns an error if the path is invalid, the file cannot be read, or the JSON parsing fails.
    pub fn load(path: &ProjectPath) -> Result<Self, ConfigError> {
        let config_content = match (path.is_file(), path.is_dir()) {
            (true, _) => std::fs::read_to_string(path),
            (false, true) => std::fs::read_to_string(path.join("config.json")),
            _ => {
                return Err(ConfigError::InvalidPath(
                    path.to_string_lossy().into_owned(),
                ))
            }
        }?;

        serde_json::from_str(&config_content).map_err(|e| ConfigError::Parse(e.to_string()))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Other error: {0}")]
    Other(String),
}

impl From<serde_json::Error> for ConfigError {
    fn from(error: serde_json::Error) -> Self {
        ConfigError::Parse(error.to_string())
    }
}

impl From<&str> for ConfigError {
    fn from(error: &str) -> Self {
        ConfigError::Other(error.to_string())
    }
}
