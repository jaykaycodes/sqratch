use async_trait::async_trait;
use std::sync::Arc;

use crate::db::errors::DbResult;
use crate::db::types::{
    ConnectionInfo, DatabaseType, FunctionInfo, QueryResult,
    SchemaInfo, TableInfo, ViewInfo
};

/// Common interface for all database clients
#[async_trait]
pub trait DatabaseClient: Send + Sync + 'static {
    /// Returns the database type this client handles
    fn db_type(&self) -> DatabaseType;

    /// Returns the connection info for this client
    fn connection_info(&self) -> &ConnectionInfo;

    /// Checks if the client is connected
    async fn is_connected(&self) -> bool;

    /// Attempts to connect to the database
    async fn connect(&self) -> DbResult<()>;

    /// Disconnects from the database
    async fn disconnect(&self) -> DbResult<()>;

    /// Executes a query and returns the result
    async fn execute_query(&self, sql: &str) -> DbResult<QueryResult>;

    /// Executes multiple queries and returns the results
    async fn execute_queries(&self, sql: &str) -> DbResult<Vec<QueryResult>>;

    /// Tests the connection by executing a simple query
    async fn test_connection(&self) -> DbResult<()>;

    /// Gets information about the database schema
    async fn get_schema_info(&self) -> DbResult<SchemaInfo>;

    /// Gets a list of tables
    async fn get_tables(&self) -> DbResult<Vec<TableInfo>>;

    /// Gets a list of views
    async fn get_views(&self) -> DbResult<Vec<ViewInfo>>;

    /// Gets a list of functions
    async fn get_functions(&self) -> DbResult<Vec<FunctionInfo>>;

    /// Gets a specific table's information
    async fn get_table_info(&self, table_name: &str, schema: Option<&str>) -> DbResult<TableInfo>;

    /// Begins a transaction
    async fn begin_transaction(&self) -> DbResult<Arc<dyn Transaction>>;
}

/// Interface for database transactions
#[async_trait]
pub trait Transaction: Send + Sync + 'static {
    /// Executes a query within the transaction
    async fn execute_query(&self, sql: &str) -> DbResult<QueryResult>;

    /// Commits the transaction
    async fn commit(&self) -> DbResult<()>;

    /// Rolls back the transaction
    async fn rollback(&self) -> DbResult<()>;
}
