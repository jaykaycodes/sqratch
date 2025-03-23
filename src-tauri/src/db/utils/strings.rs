use url::Url;
use std::collections::HashMap;

use crate::db::errors::{DbError, DbResult};
use crate::db::types::{ConnectionInfo, DatabaseType};

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
        return Err(DbError::Parsing("Unclosed string literal in SQL statement".to_string()));
    }
    if in_identifier {
        return Err(DbError::Parsing("Unclosed quoted identifier in SQL statement".to_string()));
    }
    if in_block_comment {
        return Err(DbError::Parsing("Unclosed block comment in SQL statement".to_string()));
    }

    Ok(statements)
}

/// Parse a connection string to extract database type and connection info
pub fn parse_connection_string(connection_string: &str) -> DbResult<ConnectionInfo> {
    let url = Url::parse(connection_string)?;

    let scheme = url.scheme();
    let db_type = match scheme {
        "postgres" | "postgresql" => DatabaseType::Postgres,
        _ => return Err(DbError::Config(format!("Unsupported database type: {}", scheme))),
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
    let mut connection = ConnectionInfo::new(name, db_type);
    connection.connection_string = Some(connection_string.to_string());
    connection.host = Some(host);
    connection.port = Some(port);
    connection.database = Some(database);
    connection.username = Some(username);
    connection.password = Some(password);

    // Parse query parameters as options
    let mut options = HashMap::new();
    for (key, value) in url.query_pairs() {
        options.insert(key.to_string(), value.to_string());
    }

    if !options.is_empty() {
        connection.options = Some(options);
    }

    Ok(connection)
}
