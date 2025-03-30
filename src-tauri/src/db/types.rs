use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Database query result
#[taurpc::ipc_type]
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

/// Result of a paginated rows query
#[taurpc::ipc_type]
#[derive(Debug)]
pub struct PaginatedRowsResult {
    /// The rows returned from the query
    pub rows: Vec<Row>,
    /// The total number of rows in the table
    pub total_rows: u64,
    /// The current page index
    pub page_index: u16,
    /// The number of rows per page
    pub page_size: u32,
    /// The total number of pages
    pub total_pages: u32,
}

/// Database column data type category for UI display
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, PartialEq, Eq)]
pub enum ColumnTypeCategory {
    Text,
    Numeric,
    Boolean,
    Date,
    DateTime,
    Time,
    Binary,
    Json,
    Array,
    Enum,
    Geometry,
    Network,
    UUID,
    Other,
}

/// Constraint type
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, PartialEq, Eq)]
pub enum ConstraintType {
    PrimaryKey,
    ForeignKey,
    Unique,
    Check,
    Exclusion,
}

/// Foreign key reference
#[taurpc::ipc_type]
#[derive(Debug)]
pub struct ForeignKeyReference {
    /// Referenced schema
    pub referenced_schema: String,
    /// Referenced table
    pub referenced_table: String,
    /// Referenced column
    pub referenced_column: String,
    /// On update action
    pub on_update: Option<String>,
    /// On delete action
    pub on_delete: Option<String>,
}

/// Database constraint
#[taurpc::ipc_type]
#[derive(Debug)]
pub struct ConstraintInfo {
    /// Constraint name
    pub name: String,
    /// Constraint type
    pub constraint_type: ConstraintType,
    /// Table name
    pub table_name: String,
    /// Schema name
    pub schema_name: String,
    /// Column names involved in the constraint
    pub column_names: Vec<String>,
    /// Foreign key reference (only for foreign keys)
    pub foreign_key_reference: Option<ForeignKeyReference>,
    /// Check constraint definition (only for check constraints)
    pub check_definition: Option<String>,
}

/// Index information
#[taurpc::ipc_type]
#[derive(Debug)]
pub struct IndexInfo {
    /// Index name
    pub name: String,
    /// Schema name
    pub schema: String,
    /// Table name
    pub table: String,
    /// Is it a unique index
    pub is_unique: bool,
    /// Is it a primary key index
    pub is_primary: bool,
    /// Column names in the index
    pub column_names: Vec<String>,
    /// Index method (btree, hash, etc.)
    pub method: Option<String>,
}

/// Column information with detailed metadata for UI display
#[taurpc::ipc_type]
#[derive(Debug)]
pub struct ColumnInfo {
    /// Column name
    pub name: String,
    /// Database-specific data type string
    pub data_type: String,
    /// Standardized column type category for UI
    pub type_category: ColumnTypeCategory,
    /// Whether the column can be null
    pub nullable: bool,
    /// Whether the column is a primary key
    pub primary_key: bool,
    /// Whether the column auto-increments
    pub auto_increment: bool,
    /// Whether the column is indexed
    pub indexed: bool,
    /// Whether the column has a unique constraint
    pub unique: bool,
    /// Character maximum length for text types
    pub char_max_length: Option<u32>,
    /// Numeric precision for numeric types
    pub numeric_precision: Option<u32>,
    /// Numeric scale for numeric types
    pub numeric_scale: Option<u32>,
    /// Default value for the column
    pub default_value: Option<String>,
    /// Column comment/description
    pub comment: Option<String>,
    /// Column position in the table
    pub position: Option<u32>,
    /// Column foreign key relationship
    pub foreign_key: Option<ForeignKeyReference>,
    /// UI display format hint
    pub display_hint: Option<String>,
}

/// Table information with detailed metadata
#[taurpc::ipc_type]
#[derive(Debug)]
pub struct TableInfo {
    /// Table name
    pub name: String,
    /// Schema name
    pub schema: String,
    /// Table columns
    pub columns: Vec<ColumnInfo>,
    /// Table constraints
    pub constraints: Vec<ConstraintInfo>,
    /// Table indices
    pub indices: Vec<IndexInfo>,
    /// Primary key column names
    pub primary_key_columns: Vec<String>,
    /// Table row count estimate
    pub row_count_estimate: Option<u64>,
    /// Table size estimate in bytes
    pub size_bytes: Option<u64>,
    /// Table comment/description
    pub comment: Option<String>,
    /// Last modified timestamp
    pub last_modified: Option<u64>,
}

/// View information
#[taurpc::ipc_type]
#[derive(Debug)]
pub struct ViewInfo {
    /// View name
    pub name: String,
    /// Schema name
    pub schema: String,
    /// View columns
    pub columns: Vec<ColumnInfo>,
    /// View definition
    pub definition: Option<String>,
}

/// Function information
#[taurpc::ipc_type]
#[derive(Debug)]
pub struct FunctionInfo {
    /// Function name
    pub name: String,
    /// Schema name
    pub schema: String,
    /// Function arguments
    pub arguments: Vec<String>,
    /// Function return type
    pub return_type: Option<String>,
    /// Function definition
    pub definition: Option<String>,
}

/// Schema information with detailed metadata
#[taurpc::ipc_type]
#[derive(Debug)]
pub struct SchemaInfo {
    /// Schema name
    pub name: String,
    /// Tables in this schema
    pub tables: Vec<TableInfo>,
    /// Views in this schema
    pub views: Vec<ViewInfo>,
    /// Functions in this schema
    pub functions: Vec<FunctionInfo>,
    /// Schema constraints
    pub constraints: Vec<ConstraintInfo>,
}
