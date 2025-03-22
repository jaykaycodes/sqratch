use std::collections::HashSet;

/// Split a SQL script into individual statements
pub fn split_sql_statements(sql: &str) -> Vec<String> {
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

    statements
}

/// Detect the type of SQL statement (SELECT, INSERT, UPDATE, etc.)
pub fn get_statement_type(sql: &str) -> StatementType {
    let trimmed = sql.trim_start();

    // Common SQL keywords to check at the beginning of a statement
    if trimmed.to_uppercase().starts_with("SELECT") {
        StatementType::Select
    } else if trimmed.to_uppercase().starts_with("INSERT") {
        StatementType::Insert
    } else if trimmed.to_uppercase().starts_with("UPDATE") {
        StatementType::Update
    } else if trimmed.to_uppercase().starts_with("DELETE") {
        StatementType::Delete
    } else if trimmed.to_uppercase().starts_with("CREATE TABLE") {
        StatementType::CreateTable
    } else if trimmed.to_uppercase().starts_with("ALTER TABLE") {
        StatementType::AlterTable
    } else if trimmed.to_uppercase().starts_with("DROP TABLE") {
        StatementType::DropTable
    } else if trimmed.to_uppercase().starts_with("CREATE VIEW") {
        StatementType::CreateView
    } else if trimmed.to_uppercase().starts_with("CREATE FUNCTION") ||
              trimmed.to_uppercase().starts_with("CREATE PROCEDURE") {
        StatementType::CreateFunction
    } else if trimmed.to_uppercase().starts_with("BEGIN") ||
              trimmed.to_uppercase().starts_with("START TRANSACTION") {
        StatementType::BeginTransaction
    } else if trimmed.to_uppercase().starts_with("COMMIT") {
        StatementType::CommitTransaction
    } else if trimmed.to_uppercase().starts_with("ROLLBACK") {
        StatementType::RollbackTransaction
    } else {
        StatementType::Other
    }
}

/// SQL statement types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatementType {
    Select,
    Insert,
    Update,
    Delete,
    CreateTable,
    AlterTable,
    DropTable,
    CreateView,
    CreateFunction,
    BeginTransaction,
    CommitTransaction,
    RollbackTransaction,
    Other,
}

impl StatementType {
    /// Returns true if this statement is expected to return rows
    pub fn returns_rows(&self) -> bool {
        matches!(self, StatementType::Select)
    }

    /// Returns true if this statement is a DML statement
    pub fn is_dml(&self) -> bool {
        matches!(
            self,
            StatementType::Insert | StatementType::Update | StatementType::Delete
        )
    }

    /// Returns true if this statement is a DDL statement
    pub fn is_ddl(&self) -> bool {
        matches!(
            self,
            StatementType::CreateTable
            | StatementType::AlterTable
            | StatementType::DropTable
            | StatementType::CreateView
            | StatementType::CreateFunction
        )
    }

    /// Returns true if this statement is a transaction control statement
    pub fn is_transaction_control(&self) -> bool {
        matches!(
            self,
            StatementType::BeginTransaction
            | StatementType::CommitTransaction
            | StatementType::RollbackTransaction
        )
    }
}

/// Extract table names from a SQL query (simple implementation)
pub fn extract_table_names(sql: &str) -> HashSet<String> {
    let mut tables = HashSet::new();
    let lowercase_sql = sql.to_lowercase();

    // Look for FROM, JOIN, UPDATE, and INSERT INTO clauses
    let from_index = lowercase_sql.find(" from ");
    let join_indices: Vec<_> = ["inner join", "left join", "right join", "full join", "join"]
        .iter()
        .flat_map(|join_type| {
            let mut indices = Vec::new();
            let mut start = 0;
            while let Some(pos) = lowercase_sql[start..].find(join_type) {
                indices.push(start + pos);
                start += pos + join_type.len();
            }
            indices
        })
        .collect();

    let update_index = lowercase_sql.find("update ");
    let insert_index = lowercase_sql.find("insert into ");

    if let Some(index) = from_index {
        // Extract tables after FROM
        let from_sql = &lowercase_sql[index + 6..];
        let end_index = from_sql.find(" where ")
            .or_else(|| from_sql.find(" group by "))
            .or_else(|| from_sql.find(" having "))
            .or_else(|| from_sql.find(" order by "))
            .or_else(|| from_sql.find(" limit "))
            .unwrap_or(from_sql.len());

        let from_clause = &from_sql[..end_index].trim();

        // Split by commas, considering potential alias
        for table_ref in from_clause.split(',') {
            let mut parts = table_ref.trim().split_whitespace();
            if let Some(table_name) = parts.next() {
                // Remove any schema prefix
                let table_name = table_name.split('.').last().unwrap_or(table_name);
                tables.insert(table_name.to_string());
            }
        }
    }

    // Extract tables from JOIN clauses
    for &index in &join_indices {
        let join_sql = &lowercase_sql[index..];
        let space_index = join_sql.find(' ').unwrap_or(0) + 1;
        let join_clause = &join_sql[space_index..];
        let end_index = join_clause.find(" on ")
            .or_else(|| join_clause.find(" using "))
            .unwrap_or(join_clause.len());

        let table_ref = &join_clause[..end_index].trim();
        let mut parts = table_ref.split_whitespace();
        if let Some(table_name) = parts.next() {
            // Remove any schema prefix
            let table_name = table_name.split('.').last().unwrap_or(table_name);
            tables.insert(table_name.to_string());
        }
    }

    // Extract table from UPDATE
    if let Some(index) = update_index {
        let update_sql = &lowercase_sql[index + 7..];
        let end_index = update_sql.find(" set ")
            .unwrap_or(update_sql.len());

        let table_ref = &update_sql[..end_index].trim();
        let table_name = table_ref.split_whitespace().next().unwrap_or(table_ref);
        // Remove any schema prefix
        let table_name = table_name.split('.').last().unwrap_or(table_name);
        tables.insert(table_name.to_string());
    }

    // Extract table from INSERT INTO
    if let Some(index) = insert_index {
        let insert_sql = &lowercase_sql[index + 12..];
        let end_index = insert_sql.find(" values ")
            .or_else(|| insert_sql.find(" select "))
            .or_else(|| insert_sql.find('('))
            .unwrap_or(insert_sql.len());

        let table_ref = &insert_sql[..end_index].trim();
        let table_name = table_ref.split_whitespace().next().unwrap_or(table_ref);
        // Remove any schema prefix
        let table_name = table_name.split('.').last().unwrap_or(table_name);
        tables.insert(table_name.to_string());
    }

    tables
}
