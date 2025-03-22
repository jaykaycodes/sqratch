use std::fmt;
use sqlx::Error as SqlxError;

/// Database error types
#[derive(Debug)]
pub enum DbError {
    /// Error connecting to the database
    Connection(String),
    /// Error executing a query
    Query(String),
    /// Error parsing or preparing a query
    Parse(String),
    /// Error with the configuration
    Config(String),
    /// Resource not found
    NotFound(String),
    /// Authentication error
    Auth(String),
    /// Operation not supported for this database type
    Unsupported(String),
    /// Transaction error
    Transaction(String),
    /// SQL parsing error
    Parsing(String),
    /// Other error
    Other(String),
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbError::Connection(msg) => write!(f, "Database connection error: {}", msg),
            DbError::Query(msg) => write!(f, "Database query error: {}", msg),
            DbError::Parse(msg) => write!(f, "SQL parse error: {}", msg),
            DbError::Config(msg) => write!(f, "Database configuration error: {}", msg),
            DbError::NotFound(msg) => write!(f, "Not found: {}", msg),
            DbError::Auth(msg) => write!(f, "Authentication error: {}", msg),
            DbError::Unsupported(msg) => write!(f, "Operation not supported: {}", msg),
            DbError::Transaction(msg) => write!(f, "Transaction error: {}", msg),
            DbError::Parsing(msg) => write!(f, "SQL parsing error: {}", msg),
            DbError::Other(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

impl std::error::Error for DbError {}

impl From<SqlxError> for DbError {
    fn from(error: SqlxError) -> Self {
        match error {
            SqlxError::Database(e) => DbError::Query(e.to_string()),
            SqlxError::RowNotFound => DbError::NotFound("Row not found".to_string()),
            SqlxError::PoolTimedOut => DbError::Connection("Connection pool timeout".to_string()),
            SqlxError::PoolClosed => DbError::Connection("Connection pool closed".to_string()),
            SqlxError::WorkerCrashed => DbError::Connection("Database worker crashed".to_string()),
            _ => DbError::Other(error.to_string()),
        }
    }
}

impl From<url::ParseError> for DbError {
    fn from(error: url::ParseError) -> Self {
        DbError::Config(format!("Invalid connection URL: {}", error))
    }
}

impl From<std::io::Error> for DbError {
    fn from(error: std::io::Error) -> Self {
        DbError::Other(format!("IO error: {}", error))
    }
}

/// Result type for database operations
pub type DbResult<T> = Result<T, DbError>;
