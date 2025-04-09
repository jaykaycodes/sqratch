use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;
use url::Url;

use crate::db::errors::{DbError, DbResult};
use crate::db::types::{EntityInfo, EntityType, QueryResult};

/// Core database client interface for all database operations
#[async_trait]
pub trait DatabaseClient: Send + Sync {
    fn get_connection_string(&self) -> String;

    /// Check if the database is connected
    async fn is_connected(&self) -> DbResult<bool>;

    /// Test the database connection
    async fn test_connection(&self) -> DbResult<()>;

    /// Connect to the database
    async fn connect(&mut self) -> DbResult<()>;

    /// Disconnect from the database
    async fn disconnect(&mut self) -> DbResult<()>;

    /// Reconnect to the database
    async fn reconnect(&mut self) -> DbResult<()>;

    /// Update the connection string & attempt to reconnect
    async fn reconnect_with_string(&mut self, connection_string: &str) -> DbResult<()>;

    /// Execute a raw SQL query
    async fn execute_query(&self, sql: &str) -> DbResult<QueryResult>;

    /// Get all database entities (tables, views, functions, etc.) with their metadata
    async fn get_entities(&self) -> DbResult<Vec<EntityInfo>>;
}
pub type DatabaseClientRef = Arc<Mutex<dyn DatabaseClient>>;

/// Creates a database client based on connection info without establishing a connection
pub fn create_client(connection_string: &str) -> DbResult<DatabaseClientRef> {
    use crate::db::postgres::PostgresClient;

    let url = Url::parse(connection_string)
        .map_err(|e| DbError::Parsing(format!("Invalid connection URL: {}", e)))?;

    match url.scheme() {
        "postgres" | "postgresql" => {
            let client = PostgresClient::new(connection_string)?;
            Ok(Arc::new(Mutex::new(client)))
        }
        _ => Err(DbError::Unsupported(format!(
            "Unsupported database type: {}",
            url.scheme()
        ))),
    }
}
