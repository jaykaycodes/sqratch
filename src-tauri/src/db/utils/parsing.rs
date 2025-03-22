use crate::db::errors::{DbError, DbResult};

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
