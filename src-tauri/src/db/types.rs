use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Database query result
#[taurpc::ipc_type]
#[serde(rename_all = "camelCase")]
#[derive(Debug)]
pub struct QueryResult {
    /// Execution timestamp
    pub timestamp: u64,
    /// Query that was executed
    pub query: String,
    /// Rows affected (for DML statements)
    pub rows_affected: Option<u64>,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Column definitions
    pub columns: Vec<ColumnDefinition>,
    /// Result rows (for SELECT statements)
    pub rows: Vec<Row>,
    /// Any warning messages
    pub warnings: Vec<String>,
    /// Sequential result number when multiple statements are executed
    pub result_index: usize,
}

/// Column definition in a query result
#[taurpc::ipc_type]
#[serde(rename_all = "camelCase")]
#[derive(Debug)]
pub struct ColumnDefinition {
    /// Column name
    pub name: String,
    /// Column data type
    pub data_type: String,
    /// Whether the column can be null
    pub nullable: bool,
    /// Whether the column is a primary key
    pub primary_key: bool,
    /// Default value for the column
    pub default_value: Option<String>,
}

/// A single row in a query result
#[taurpc::ipc_type]
#[derive(Debug)]
pub struct Row {
    /// Values indexed by column name
    pub values: HashMap<String, String>,
}

impl From<HashMap<String, serde_json::Value>> for Row {
    fn from(values: HashMap<String, serde_json::Value>) -> Self {
        Self {
            values: values
                .into_iter()
                .map(|(k, v)| (k, v.to_string()))
                .collect(),
        }
    }
}

impl From<Row> for HashMap<String, serde_json::Value> {
    fn from(row: Row) -> Self {
        row.values
            .into_iter()
            .map(|(k, v)| (k, serde_json::Value::String(v)))
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct SchemaEntity {
    pub id: String,
    pub name: String,
    pub is_system: bool,
    pub extension_name: Option<String>,
    pub children: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct SchemaLevelEntity {
    pub id: String,
    pub name: String,
    pub is_system: bool,
    pub schema_id: String,
    pub extension_name: Option<String>,
}

// #[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
// #[serde(rename_all = "camelCase")]
// pub struct TableLevelEntity {
//     pub id: String,
//     pub name: String,
//     pub is_system: bool,
//     pub table_id: String,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
// #[serde(rename_all = "camelCase")]
// pub struct DbExtension {
//     pub id: String,
//     pub name: String,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
// #[serde(rename_all = "camelCase")]
// pub struct GlobalTrigger {
//     pub id: String,
//     pub name: String,
//     pub is_system: bool,
//     pub extension_name: Option<String>,
// }

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(tag = "kind")]
pub enum DbEntity {
    Schema(SchemaEntity),
    Table(SchemaLevelEntity),
    View(SchemaLevelEntity),
    MaterializedView(SchemaLevelEntity),
    ForeignTable(SchemaLevelEntity),
    // Procedure(SchemaLevelEntity),
    // CustomType(SchemaLevelEntity),
    // Function(SchemaLevelEntity),
    // Sequence(SchemaLevelEntity),
    // Trigger(TableLevelEntity),
    // Index(TableLevelEntity),
    // Extension(DbExtension),
    // GlobalTrigger(GlobalTrigger),
}
