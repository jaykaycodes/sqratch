use serde::{Deserialize, Serialize};

const SCHEMA_URL: &str = "https://sqratch.dev/schema.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct ProjectConfig {
    #[serde(rename = "$schema")]
    pub schema: Option<String>,
    pub project_name: Option<String>,
    /// Path to .env file
    pub env_file: Option<String>,
    pub db_url: Option<String>,
    pub host: Option<String>,
    pub port: Option<String>,
    pub database: Option<String>,
    pub user: Option<String>,
    pub password: Option<String>
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            schema: Some(SCHEMA_URL.to_string()),
            project_name: None,
            env_file: Some("../.env".to_string()),
            db_url: Some("DATABASE_URL".to_string()),
            host: None,
            port: None,
            database: None,
            user: None,
            password: None,
        }
    }
}
