use crate::db::errors::{DbError, DbResult};
use crate::db::types::{ConnectionInfo, PaginatedRowsResult, QueryResult, SchemaInfo, TableInfo};
use async_trait::async_trait;

/// Core database client interface for all database operations
#[async_trait]
pub trait DatabaseClient: Send + Sync {
    /// Get connection info
    fn get_connection_info(&self) -> &ConnectionInfo;

    /// Test the database connection
    async fn test_connection(&self) -> DbResult<()>;

    /// Execute a raw SQL query
    async fn execute_query(&self, sql: &str) -> DbResult<QueryResult>;

    /// Get information about all tables
    async fn get_tables(&self) -> DbResult<Vec<TableInfo>>;

    /// Get schema information including tables and their structure
    async fn get_schema_info(&self) -> DbResult<SchemaInfo>;

    /// Get paginated rows from a table
    async fn get_paginated_rows(
        &self,
        table_name: &str,
        page_index: u16,
        page_size: u32,
    ) -> DbResult<PaginatedRowsResult>;

    /// Delete rows from a table using primary key
    async fn delete_rows(
        &self,
        pk_col_name: &str,
        table_name: &str,
        params: &str,
    ) -> DbResult<String>;

    /// Create a new row in a table
    async fn create_row(&self, table_name: &str, columns: &str, values: &str) -> DbResult<String>;

    /// Update a row in a table using primary key
    async fn update_row(
        &self,
        table_name: &str,
        set_condition: &str,
        pk_col_name: &str,
        pk_col_value: &str,
    ) -> DbResult<String>;
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
        #[allow(unreachable_patterns)]
        _ => Err(DbError::Unsupported(format!(
            "Unsupported database type: {}",
            info.db_type
        ))),
    }
}
