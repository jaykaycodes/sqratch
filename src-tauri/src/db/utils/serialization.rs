use sqlx::Row;
use std::collections::HashMap;
use chrono::{DateTime, NaiveDateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

use crate::db::errors::{DbError, DbResult};
use crate::db::types::{ColumnDefinition, QueryResult, Row as ResultRow};

/// Convert a SQLx row to a serializable row
pub fn sqlx_row_to_row<R: sqlx::Row>(
    row: R,
    column_definitions: &[ColumnDefinition],
) -> DbResult<ResultRow> {
    let mut values = HashMap::new();

    for (i, column) in column_definitions.iter().enumerate() {
        let value = if row.try_get_raw(i).map_or(true, |r| r.is_null()) {
            Value::Null
        } else {
            let column_name = &column.name;
            match column.data_type.to_lowercase().as_str() {
                // Integer types
                "int" | "integer" | "int2" | "int4" | "int8" | "smallint" | "bigint" => {
                    if let Ok(val) = row.try_get::<i64, _>(i) {
                        Value::Number(serde_json::Number::from(val))
                    } else if let Ok(val) = row.try_get::<i32, _>(i) {
                        Value::Number(serde_json::Number::from(val as i64))
                    } else if let Ok(val) = row.try_get::<i16, _>(i) {
                        Value::Number(serde_json::Number::from(val as i64))
                    } else {
                        Value::Null
                    }
                },
                // Floating point types
                "float" | "double" | "real" | "float4" | "float8" => {
                    if let Ok(val) = row.try_get::<f64, _>(i) {
                        match serde_json::Number::from_f64(val) {
                            Some(num) => Value::Number(num),
                            None => Value::Null,
                        }
                    } else {
                        Value::Null
                    }
                },
                // Boolean types
                "bool" | "boolean" => {
                    if let Ok(val) = row.try_get::<bool, _>(i) {
                        Value::Bool(val)
                    } else {
                        Value::Null
                    }
                },
                // String types
                "char" | "varchar" | "text" | "name" | "citext" => {
                    if let Ok(val) = row.try_get::<String, _>(i) {
                        Value::String(val)
                    } else {
                        Value::Null
                    }
                },
                // Date/time types
                "timestamp" | "timestamptz" | "date" | "time" | "timetz" => {
                    if let Ok(val) = row.try_get::<DateTime<Utc>, _>(i) {
                        Value::String(val.to_rfc3339())
                    } else if let Ok(val) = row.try_get::<NaiveDateTime, _>(i) {
                        Value::String(val.to_string())
                    } else {
                        Value::Null
                    }
                },
                // JSON types
                "json" | "jsonb" => {
                    if let Ok(val) = row.try_get::<serde_json::Value, _>(i) {
                        val
                    } else {
                        Value::Null
                    }
                },
                // UUID type
                "uuid" => {
                    if let Ok(val) = row.try_get::<Uuid, _>(i) {
                        Value::String(val.to_string())
                    } else {
                        Value::Null
                    }
                },
                // Binary types
                "bytea" | "blob" | "binary" => {
                    if let Ok(val) = row.try_get::<Vec<u8>, _>(i) {
                        Value::String(format!("<binary data: {} bytes>", val.len()))
                    } else {
                        Value::Null
                    }
                },
                // Default to string representation
                _ => {
                    if let Ok(val) = row.try_get::<String, _>(i) {
                        Value::String(val)
                    } else {
                        Value::Null
                    }
                },
            }
        };

        values.insert(column.name.clone(), value);
    }

    Ok(ResultRow { values })
}

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
