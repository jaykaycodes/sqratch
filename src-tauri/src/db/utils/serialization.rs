use sqlx::{postgres::PgRow, mysql::MySqlRow, sqlite::SqliteRow, Row};
use std::collections::HashMap;
use serde_json::Value as JsonValue;

use crate::db::errors::DbResult;
use crate::db::types::{ColumnDefinition, QueryResult, Row as ResultRow};

/// Trait for converting database rows to our format
pub trait RowConverter {
    fn to_result_row(&self, column_definitions: &[ColumnDefinition]) -> DbResult<ResultRow>;
}

/// Implementation for PostgreSQL rows
impl RowConverter for PgRow {
    fn to_result_row(&self, column_definitions: &[ColumnDefinition]) -> DbResult<ResultRow> {
        let mut values = HashMap::new();

        for column in column_definitions {
            let json_value = if let Ok(val) = self.try_get::<Option<String>, _>(column.name.as_str()) {
                match val {
                    Some(val) => parse_string_to_json(val),
                    None => JsonValue::Null,
                }
            } else {
                JsonValue::String(format!("<{}:value>", column.data_type))
            };

            values.insert(column.name.clone(), json_value);
        }

        Ok(ResultRow { values })
    }
}

/// Implementation for MySQL rows
impl RowConverter for MySqlRow {
    fn to_result_row(&self, column_definitions: &[ColumnDefinition]) -> DbResult<ResultRow> {
        let mut values = HashMap::new();

        for column in column_definitions {
            let json_value = if let Ok(val) = self.try_get::<Option<String>, _>(column.name.as_str()) {
                match val {
                    Some(val) => parse_string_to_json(val),
                    None => JsonValue::Null,
                }
            } else {
                JsonValue::String(format!("<{}:value>", column.data_type))
            };

            values.insert(column.name.clone(), json_value);
        }

        Ok(ResultRow { values })
    }
}

/// Implementation for SQLite rows
impl RowConverter for SqliteRow {
    fn to_result_row(&self, column_definitions: &[ColumnDefinition]) -> DbResult<ResultRow> {
        let mut values = HashMap::new();

        for column in column_definitions {
            let json_value = if let Ok(val) = self.try_get::<Option<String>, _>(column.name.as_str()) {
                match val {
                    Some(val) => parse_string_to_json(val),
                    None => JsonValue::Null,
                }
            } else {
                JsonValue::String(format!("<{}:value>", column.data_type))
            };

            values.insert(column.name.clone(), json_value);
        }

        Ok(ResultRow { values })
    }
}

// Helper function to parse a string to a JSON value
fn parse_string_to_json(val: String) -> JsonValue {
    if let Ok(num) = val.parse::<i64>() {
        JsonValue::Number(num.into())
    } else if let Ok(num) = val.parse::<f64>() {
        match serde_json::Number::from_f64(num) {
            Some(n) => JsonValue::Number(n),
            None => JsonValue::String(val),
        }
    } else if val == "true" {
        JsonValue::Bool(true)
    } else if val == "false" {
        JsonValue::Bool(false)
    } else {
        JsonValue::String(val)
    }
}

// For compatibility with existing code
pub fn pg_row_to_row(row: &PgRow, column_definitions: &[ColumnDefinition]) -> DbResult<ResultRow> {
    row.to_result_row(column_definitions)
}

// pub fn mysql_row_to_row(row: &MySqlRow, column_definitions: &[ColumnDefinition]) -> DbResult<ResultRow> {
//     row.to_result_row(column_definitions)
// }

// pub fn sqlite_row_to_row(row: &SqliteRow, column_definitions: &[ColumnDefinition]) -> DbResult<ResultRow> {
//     row.to_result_row(column_definitions)
// }


/// Helper to create a new QueryResult
pub fn create_query_result(
    query: String,
    columns: Vec<ColumnDefinition>,
    rows: Vec<ResultRow>,
    rows_affected: Option<u64>,
    execution_time_ms: u64,
    result_index: usize,
) -> QueryResult {
    QueryResult {
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        query,
        rows_affected,
        execution_time_ms,
        columns,
        rows,
        warnings: Vec::new(),
        result_index,
    }
}
