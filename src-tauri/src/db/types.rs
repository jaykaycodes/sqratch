use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use uuid::Uuid;

/// Supported database types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DatabaseType {
    Postgres,
}

impl fmt::Display for DatabaseType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseType::Postgres => write!(f, "PostgreSQL"),
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
        }
    }

    /// Get default port for the database type
    pub fn default_port(&self) -> u16 {
        match self.db_type {
            DatabaseType::Postgres => 5432,
        }
    }

    /// Build a connection string from parts
    pub fn to_connection_string(&self) -> Result<String, String> {
        if let Some(ref conn_str) = self.connection_string {
            return Ok(conn_str.clone());
        }

        match self.db_type {
            DatabaseType::Postgres => {
                let host = self
                    .host
                    .as_ref()
                    .ok_or_else(|| "Host is required".to_string())?;
                let port = self.port.unwrap_or_else(|| self.default_port());
                let database = self
                    .database
                    .as_ref()
                    .ok_or_else(|| "Database name is required".to_string())?;
                let username = self
                    .username
                    .as_ref()
                    .ok_or_else(|| "Username is required".to_string())?;

                // Create a longer-lived password value
                let password = match self.password.as_ref() {
                    Some(p) => p,
                    None => "",
                };

                let prefix = match self.db_type {
                    DatabaseType::Postgres => "postgres",
                };

                Ok(format!(
                    "{}://{}:{}@{}:{}/{}",
                    prefix, username, password, host, port, database
                ))
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
    /// Schema name
    pub name: String,
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
    pub schema: String,
    /// Table columns
    pub columns: Vec<ColumnInfo>,
    /// Table comment/description
    pub comment: Option<String>,
}

/// Column information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
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
    /// Column comment/description
    pub comment: Option<String>,
    /// Column position in the table (optional)
    pub position: Option<u32>,
}

/// View information
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Result of a paginated rows query
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
