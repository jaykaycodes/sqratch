#![allow(dead_code)]

use async_trait::async_trait;
use url::Url;

use crate::db::errors::{DbError, DbResult};
use crate::db::types::{Entity, QueryResult};

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

    /// Get a flat list of all entities including schemas
    async fn get_all_entities(&self) -> DbResult<Vec<Entity>>;
}

/// Creates a database client based on connection info without establishing a connection
pub fn create_client(url: &Url) -> DbResult<impl DatabaseClient> {
    use crate::db::postgres::PostgresClient;

    match url.scheme() {
        "postgres" | "postgresql" => {
            let client = PostgresClient::new(url.to_string().as_str())?;
            Ok(client)
        }
        _ => Err(DbError::Unsupported(format!(
            "Unsupported database type: {}",
            url.scheme()
        ))),
    }
}
