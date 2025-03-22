use serde::{Deserialize, Serialize};

/// Project configuration from .sqratch/config.jsonc
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Environment variable name for the database connection string
    #[serde(default = "default_connection_variable")]
    pub connection_variable: String,

    /// Individual connection parameters as environment variables
    pub connection_params: Option<ConnectionParams>,

    /// Project-specific settings
    #[serde(default)]
    pub settings: ProjectSettings,
}

/// Connection parameters as environment variables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionParams {
    /// Host environment variable
    pub host: String,

    /// Port environment variable
    pub port: String,

    /// Database name environment variable
    pub database: String,

    /// Username environment variable
    pub user: String,

    /// Password environment variable
    pub password: String,
}

/// Project-specific settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectSettings {
    /// Project name for display
    pub project_name: Option<String>,

    /// Whether to save queries
    #[serde(default = "default_save_queries")]
    pub save_queries: bool,
}

/// Default value for connection_variable
fn default_connection_variable() -> String {
    "DATABASE_URL".to_string()
}

/// Default value for save_queries
fn default_save_queries() -> bool {
    true
}
