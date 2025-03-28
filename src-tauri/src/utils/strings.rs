use std::collections::HashMap;
use std::path::Path;
use url::Url;

use crate::db::*;

/// Split a SQL script into individual statements
pub fn split_sql_statements(sql: &str) -> DbResult<Vec<String>> {
    let mut statements = Vec::new();
    let mut current_statement = String::new();
    let mut in_string = false;
    let mut in_identifier = false;
    let mut in_comment = false;
    let mut in_block_comment = false;
    let mut previous_char = ' ';

    for c in sql.chars() {
        // Handle string literals
        if c == '\'' && !in_comment && !in_block_comment {
            if !in_string || previous_char != '\\' {
                in_string = !in_string;
            }
        }

        // Handle quoted identifiers
        if c == '"' && !in_string && !in_comment && !in_block_comment {
            if !in_identifier || previous_char != '\\' {
                in_identifier = !in_identifier;
            }
        }

        // Handle line comments
        if c == '-' && previous_char == '-' && !in_string && !in_identifier && !in_block_comment {
            in_comment = true;
        }

        // Handle block comments
        if c == '*' && previous_char == '/' && !in_string && !in_identifier && !in_comment {
            in_block_comment = true;
        }

        if c == '/' && previous_char == '*' && in_block_comment {
            in_block_comment = false;
        }

        // End of line resets line comments
        if c == '\n' && in_comment {
            in_comment = false;
        }

        // Add character to current statement
        if !in_comment && !in_block_comment {
            current_statement.push(c);
        }

        // Handle statement terminator
        if c == ';' && !in_string && !in_identifier && !in_comment && !in_block_comment {
            let trimmed = current_statement.trim();
            if !trimmed.is_empty() {
                statements.push(trimmed.to_string());
            }
            current_statement.clear();
        }

        previous_char = c;
    }

    // Add the last statement if it's not empty
    let trimmed = current_statement.trim();
    if !trimmed.is_empty() {
        statements.push(trimmed.to_string());
    }

    // Check for unclosed string or quoted identifier
    if in_string {
        return Err(DbError::Parsing(
            "Unclosed string literal in SQL statement".to_string(),
        ));
    }
    if in_identifier {
        return Err(DbError::Parsing(
            "Unclosed quoted identifier in SQL statement".to_string(),
        ));
    }
    if in_block_comment {
        return Err(DbError::Parsing(
            "Unclosed block comment in SQL statement".to_string(),
        ));
    }

    Ok(statements)
}

/// Parse a connection string to extract database type and connection info
pub fn parse_connection_string(connection_string: &str) -> Result<ConnectionInfo, String> {
    let url =
        Url::parse(connection_string).map_err(|e| format!("Invalid connection URL: {}", e))?;

    let scheme = url.scheme();
    let db_type = match scheme {
        "postgres" | "postgresql" => DatabaseType::Postgres,
        _ => return Err(format!("Unsupported database type: {}", scheme)),
    };

    let host = url.host_str().unwrap_or("localhost").to_string();
    let port = url.port().unwrap_or(match db_type {
        DatabaseType::Postgres => 5432,
    });

    let database = url.path().trim_start_matches('/').to_string();

    let username = url.username().to_string();
    let password = url.password().unwrap_or("").to_string();

    // Create a default name based on host and database
    let name = format!("{} on {}", database, host);

    // Create a new connection info
    let mut conn_info = ConnectionInfo::new(name, db_type);
    conn_info.connection_string = Some(connection_string.to_string());
    conn_info.host = Some(host);
    conn_info.port = Some(port);
    conn_info.database = Some(database);
    conn_info.username = Some(username);
    conn_info.password = Some(password);

    // Parse query parameters as options
    let mut options = HashMap::new();
    for (key, value) in url.query_pairs() {
        options.insert(key.to_string(), value.to_string());
    }

    if !options.is_empty() {
        conn_info.options = Some(options);
    }

    Ok(conn_info)
}

/// Parse the "project" arg into a connection string
///
/// This function handles two cases:
/// 1. If the arg is a database URL, it returns it directly
/// 2. If the arg is a directory path, it looks for a .env file and extracts the DATABASE_URL
pub fn parse_project_arg(project_arg: &str, cwd: &str) -> Result<ConnectionInfo, String> {
    // Check if it's a URL (has :// pattern)
    if project_arg.contains("://") {
        return parse_connection_string(project_arg);
    }

    // Otherwise treat as a path to a directory with .env file
    let path = if Path::new(project_arg).is_absolute() {
        Path::new(project_arg).to_path_buf()
    } else {
        Path::new(cwd).join(project_arg)
    };

    if path.is_dir() {
        let env_path = path.join(".env");
        if env_path.exists() {
            // Read .env file content
            let content = std::fs::read_to_string(&env_path)
                .map_err(|e| format!("Failed to read .env file: {}", e))?;

            // Parse the file content to find DATABASE_URL
            for line in content.lines() {
                let line = line.trim();
                // Skip comments and empty lines
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                // Look for DATABASE_URL=value pattern
                if let Some((key, value)) = line.split_once('=') {
                    if key.trim() == "DATABASE_URL" {
                        // Remove quotes if present
                        let url = value.trim().trim_matches('"').trim_matches('\'');

                        return parse_connection_string(url);
                    }
                }
            }

            Err("DATABASE_URL not found in .env file".to_string())
        } else {
            Err(format!(
                "No .env file found in directory: {}",
                path.display()
            ))
        }
    } else {
        Err(format!("Invalid project path: {}", path.display()))
    }
}
