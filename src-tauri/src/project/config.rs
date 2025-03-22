use std::fs;
use std::path::Path;
use dotenv::dotenv;

use crate::db::errors::{DbError, DbResult};
use crate::db::manager::DatabaseManager;
use crate::db::types::{ConnectionInfo, DatabaseType};
use super::types::{ProjectConfig, ProjectSettings};

/// Manages project configuration
pub struct ProjectManager;

impl ProjectManager {
    /// Parse the project configuration file
    pub fn parse_config(project_path: &str) -> DbResult<ProjectConfig> {
        let config_path = Path::new(project_path).join(".sqratch").join("config.jsonc");

        if !config_path.exists() {
            // Return default config if no config file exists
            return Ok(ProjectConfig {
                connection_variable: "DATABASE_URL".to_string(),
                connection_params: None,
                settings: ProjectSettings::default(),
            });
        }

        // Read the config file
        let config_content = fs::read_to_string(&config_path)
            .map_err(|e| DbError::Config(format!("Failed to read config file: {}", e)))?;

        // Strip JSONC comments
        let json_str = strip_jsonc_comments(&config_content);

        // Parse the JSON
        let config: ProjectConfig = serde_json::from_str(&json_str)
            .map_err(|e| DbError::Config(format!("Failed to parse config file: {}", e)))?;

        Ok(config)
    }

    /// Create a connection from the project configuration
    pub async fn create_connection_from_config(
        project_path: &str,
        config: &ProjectConfig,
    ) -> DbResult<Option<ConnectionInfo>> {
        // Load environment variables from .env file
        let env_path = Path::new(project_path).join(".env");
        if env_path.exists() {
            // Use dotenv to load environment variables
            let env_path_str = env_path.to_string_lossy().to_string();
            dotenv::from_path(&env_path)
                .map_err(|e| DbError::Config(format!("Failed to load .env file: {}", e)))?;

            println!("Loaded environment variables from: {}", env_path_str);
        } else {
            // Try to load from .env in the current directory
            if dotenv().ok().is_some() {
                println!("Loaded environment variables from default .env");
            }
        }

        // First try with connection_variable
        if let Ok(conn_str) = std::env::var(&config.connection_variable) {
            println!("Found connection string in environment variable: {}", config.connection_variable);
            let mut connection = DatabaseManager::parse_connection_string(&conn_str)?;

            // Add project info
            if let Some(project_name) = &config.settings.project_name {
                connection.name = format!("{} - {}", project_name, connection.name);
            }
            connection.project_id = Some(project_path.to_string());

            return Ok(Some(connection));
        }

        // If connection_variable didn't work, try connection_params
        if let Some(params) = &config.connection_params {
            // Get all the parameters from environment variables
            let host = std::env::var(&params.host).ok();
            let port_str = std::env::var(&params.port).ok();
            let port = port_str.and_then(|p| p.parse::<u16>().ok());
            let database = std::env::var(&params.database).ok();
            let username = std::env::var(&params.user).ok();
            let password = std::env::var(&params.password).ok();

            // Check if we have enough information to build a connection
            if let (Some(host_val), Some(database_val)) = (host.clone(), database.clone()) {
                println!("Building connection from individual parameters");

                // Determine database type
                let db_type = if host_val.contains("postgres") || port == Some(5432) {
                    DatabaseType::Postgres
                } else if host_val.contains("mysql") || port == Some(3306) {
                    DatabaseType::MySQL
                } else if database_val.ends_with(".db") || database_val.ends_with(".sqlite") {
                    DatabaseType::SQLite
                } else {
                    // Default to PostgreSQL
                    DatabaseType::Postgres
                };

                // Build a connection info object
                let name = if let Some(project_name) = &config.settings.project_name {
                    format!("{} - {}", project_name, database_val)
                } else {
                    database_val.clone()
                };

                let mut connection = ConnectionInfo::new(name, db_type);
                connection.host = host;
                connection.port = port;
                connection.database = database;
                connection.username = username;
                connection.password = password;
                connection.project_id = Some(project_path.to_string());

                return Ok(Some(connection));
            }
        }

        // No connection information found
        Ok(None)
    }
}

/// Helper function to parse connection information from project path
pub async fn parse_project_config(
    project_path: &str,
) -> DbResult<Option<ConnectionInfo>> {
    // Parse project configuration
    let project_config = ProjectManager::parse_config(project_path)?;

    // Create connection from config
    ProjectManager::create_connection_from_config(project_path, &project_config).await
}

/// Helper function to strip JSONC comments
fn strip_jsonc_comments(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut in_string = false;
    let mut in_single_line_comment = false;
    let mut in_multi_line_comment = false;
    let mut prev_char: Option<char> = None;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if in_single_line_comment {
            // Single line comment ends at the end of line
            if c == '\n' {
                in_single_line_comment = false;
                result.push(c);
            }
        } else if in_multi_line_comment {
            // Multi-line comment ends with */
            if c == '*' && chars.peek() == Some(&'/') {
                chars.next(); // consume the '/'
                in_multi_line_comment = false;
            }
        } else if in_string {
            // In a string, append character, but check for end of string
            result.push(c);
            if c == '"' && prev_char != Some('\\') {
                in_string = false;
            }
        } else {
            // Normal code
            if c == '"' {
                in_string = true;
                result.push(c);
            } else if c == '/' && chars.peek() == Some(&'/') {
                chars.next(); // consume the second '/'
                in_single_line_comment = true;
            } else if c == '/' && chars.peek() == Some(&'*') {
                chars.next(); // consume the '*'
                in_multi_line_comment = true;
            } else {
                result.push(c);
            }
        }

        prev_char = Some(c);
    }

    result
}
