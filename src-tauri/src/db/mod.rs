// Database connection module
pub mod clients;
pub mod errors;
pub mod manager;
pub mod types;
pub mod utils;

// Re-export key types for easier access
pub use clients::common::{DatabaseClient, Transaction};
pub use errors::{DbError, DbResult};
pub use manager::{create_db_manager, DatabaseManager, parse_connection_config};
pub use types::{
    ConnectionInfo, DatabaseType, QueryResult, Row,
    SchemaInfo, TableInfo, ViewInfo,
};
