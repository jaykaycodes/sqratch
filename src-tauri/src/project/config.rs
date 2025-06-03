use super::ProjectHandle;

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
    /// Loads a project configuration given a ProjectRef.
    /// Attempts to read a config.json file from the project directory.
    /// If the file doesn't exist or can't be read, returns a default configuration.
    pub fn load(project_ref: &ProjectHandle) -> Result<Self, ConfigError> {
        // If the project is temporary, return a default configuration
        if project_ref.is_temp {
            let url = project_ref.url.clone().unwrap();
            let name = url
                .path_segments()
                .and_then(|segments| segments.last())
                .unwrap_or("Untitled")
                .to_string();

            return Ok(ProjectConfig {
                name: Some(name),
                db: url.to_string(),
            });
        }

        // For non-temporary projects, attempt to read the config file
        let config_path = project_ref.path.join("config.json");
        match std::fs::read_to_string(&config_path) {
            Ok(content) => {
                serde_json::from_str(&content).map_err(|e| ConfigError::Parse(e.to_string()))
            }
            Err(e) => Err(ConfigError::Io(e)),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

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
