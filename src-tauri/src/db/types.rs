use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use uuid::Uuid;

/// Supported database types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DatabaseType {
    Postgres,
    MySQL,
    SQLite,
}

impl fmt::Display for DatabaseType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseType::Postgres => write!(f, "PostgreSQL"),
            DatabaseType::MySQL => write!(f, "MySQL"),
            DatabaseType::SQLite => write!(f, "SQLite"),
        }
    }
}

/// Connection information for a database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    /// Unique identifier for this connection
    pub id: String,
    /// User-friendly name for this connection
    pub name: String,
    /// Database type
    pub db_type: DatabaseType,
    /// Full connection string (if provided)
    pub connection_string: Option<String>,
    /// Connection host or path (for SQLite)
    pub host: Option<String>,
    /// Connection port
    pub port: Option<u16>,
    /// Database name
    pub database: Option<String>,
    /// Username for connection
    pub username: Option<String>,
    /// Password for connection
    pub password: Option<String>,
    /// Connection options
    pub options: Option<HashMap<String, String>>,
    /// SSL configuration
    pub ssl_config: Option<SslConfig>,
    /// Creation timestamp
    pub created_at: u64,
    /// Last accessed timestamp
    pub last_used: Option<u64>,
    /// Associated project ID (if any)
    pub project_id: Option<String>,
}

impl ConnectionInfo {
    /// Create a new connection info with default values
    pub fn new(name: String, db_type: DatabaseType) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            db_type,
            connection_string: None,
            host: None,
            port: None,
            database: None,
            username: None,
            password: None,
            options: None,
            ssl_config: None,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            last_used: None,
            project_id: None,
        }
    }

    /// Get default port for the database type
    pub fn default_port(&self) -> u16 {
        match self.db_type {
            DatabaseType::Postgres => 5432,
            DatabaseType::MySQL => 3306,
            DatabaseType::SQLite => 0,
        }
    }

    /// Build a connection string from parts
    pub fn to_connection_string(&self) -> Result<String, String> {
        if let Some(ref conn_str) = self.connection_string {
            return Ok(conn_str.clone());
        }

        match self.db_type {
            DatabaseType::SQLite => {
                let path = self.host.as_ref().or(self.database.as_ref())
                    .ok_or_else(|| "SQLite database path is required".to_string())?;
                Ok(format!("sqlite:{}", path))
            },
            DatabaseType::Postgres | DatabaseType::MySQL => {
                let host = self.host.as_ref().ok_or_else(|| "Host is required".to_string())?;
                let port = self.port.unwrap_or_else(|| self.default_port());
                let database = self.database.as_ref().ok_or_else(|| "Database name is required".to_string())?;
                let username = self.username.as_ref().ok_or_else(|| "Username is required".to_string())?;
                let password = self.password.as_ref().unwrap_or(&"".to_string());

                let prefix = match self.db_type {
                    DatabaseType::Postgres => "postgres",
                    DatabaseType::MySQL => "mysql",
                    _ => unreachable!(),
                };

                Ok(format!("{}://{}:{}@{}:{}/{}", prefix, username, password, host, port, database))
            }
        }
    }
}

/// SSL configuration for database connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslConfig {
    /// Whether to use SSL
    pub enabled: bool,
    /// Whether to reject unauthorized connections
    pub reject_unauthorized: bool,
    /// Path to CA certificate
    pub ca_cert_path: Option<String>,
    /// Path to client certificate
    pub client_cert_path: Option<String>,
    /// Path to client key
    pub client_key_path: Option<String>,
}

impl Default for SslConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            reject_unauthorized: true,
            ca_cert_path: None,
            client_cert_path: None,
            client_key_path: None,
        }
    }
}

/// Database query result
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Row {
    /// Values indexed by column name
    pub values: HashMap<String, serde_json::Value>,
}

/// Database schema information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaInfo {
    /// Database name
    pub database: String,
    /// Schema name (if applicable)
    pub schema: Option<String>,
    /// Tables in this schema
    pub tables: Vec<TableInfo>,
    /// Views in this schema
    pub views: Vec<ViewInfo>,
    /// Functions in this schema
    pub functions: Vec<FunctionInfo>,
}

/// Table information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    /// Table name
    pub name: String,
    /// Schema name
    pub schema: Option<String>,
    /// Table columns
    pub columns: Vec<ColumnInfo>,
    /// Approximate row count (if available)
    pub row_count: Option<u64>,
    /// Table size in bytes (if available)
    pub size_bytes: Option<u64>,
    /// Table comment/description
    pub comment: Option<String>,
}

/// Column information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    /// Column name
    pub name: String,
    /// Column position in the table
    pub position: i32,
    /// Column data type
    pub data_type: String,
    /// Column character maximum length (if applicable)
    pub char_max_length: Option<i32>,
    /// Whether the column can be null
    pub nullable: bool,
    /// Default value for the column
    pub default_value: Option<String>,
    /// Column comment/description
    pub comment: Option<String>,
    /// Whether the column is part of the primary key
    pub is_primary_key: bool,
    /// Foreign key reference (if applicable)
    pub foreign_key_ref: Option<ForeignKeyRef>,
}

/// Foreign key reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKeyRef {
    /// Referenced schema
    pub ref_schema: Option<String>,
    /// Referenced table
    pub ref_table: String,
    /// Referenced column
    pub ref_column: String,
    /// On delete action
    pub on_delete: Option<String>,
    /// On update action
    pub on_update: Option<String>,
}

/// View information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewInfo {
    /// View name
    pub name: String,
    /// Schema name
    pub schema: Option<String>,
    /// View definition
    pub definition: Option<String>,
    /// View columns
    pub columns: Vec<ColumnInfo>,
    /// View comment/description
    pub comment: Option<String>,
}

/// Function information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    /// Function name
    pub name: String,
    /// Schema name
    pub schema: Option<String>,
    /// Function language
    pub language: Option<String>,
    /// Function definition
    pub definition: Option<String>,
    /// Function arguments
    pub arguments: Vec<FunctionArgument>,
    /// Function return type
    pub return_type: Option<String>,
    /// Function comment/description
    pub comment: Option<String>,
}

/// Function argument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionArgument {
    /// Argument name
    pub name: Option<String>,
    /// Argument data type
    pub data_type: String,
    /// Argument mode (IN, OUT, INOUT)
    pub mode: Option<String>,
    /// Default value
    pub default_value: Option<String>,
}
