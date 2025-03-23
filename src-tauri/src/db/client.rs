use async_trait::async_trait;
use crate::db::errors::DbResult;
use crate::db::types::{ConnectionInfo, QueryResult, SchemaInfo, TableInfo};

/// Core database client interface for all database operations
#[async_trait]
pub trait DatabaseClient: Send + Sync {
    /// Test the database connection
    async fn test_connection(&self) -> DbResult<()>;

    /// Execute a raw SQL query
    async fn execute_query(&self, sql: &str) -> DbResult<QueryResult>;

    /// Get information about all tables
    async fn get_tables(&self) -> DbResult<Vec<TableInfo>>;

    /// Get schema information including tables and their structure
    async fn get_schema_info(&self) -> DbResult<SchemaInfo>;

    /// Get connection info
    fn get_connection_info(&self) -> &ConnectionInfo;
}

/// Creates a database client based on connection info
pub async fn create_client(info: ConnectionInfo) -> DbResult<Box<dyn DatabaseClient>> {
    use crate::db::postgres::PostgresClient;
    use crate::db::types::DatabaseType;

    match info.db_type {
        DatabaseType::Postgres => {
            let client = PostgresClient::new(info).await?;
            Ok(Box::new(client))
        }
    }
}
