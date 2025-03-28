use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tauri::State;
use tokio::sync::RwLock;

use crate::db::client::{create_client, DatabaseClient};
use crate::db::errors::{DbError, DbResult};
use crate::db::types::DatabaseType;
use crate::db::types::{ConnectionInfo, QueryResult};
use crate::utils::strings;

pub struct ConnectionManager {
    connections: RwLock<HashMap<String, (ConnectionInfo, Option<Arc<Box<dyn DatabaseClient>>>)>>,
}
pub type ConnectionManagerSafe = State<'static, ConnectionManager>;

impl ConnectionManager {
    /// Creates a new database manager
    pub fn new() -> Self {
        Self {
            connections: RwLock::new(HashMap::new()),
        }
    }

    /// Adds a connection without connecting
    pub async fn add_connection(&self, connection: ConnectionInfo) -> DbResult<String> {
        let id = connection.id.clone();
        let mut connections = self.connections.write().await;
        connections.insert(id.clone(), (connection, None));
        Ok(id)
    }

    /// Gets a connection by ID
    pub async fn get_connection(&self, id: &str) -> DbResult<ConnectionInfo> {
        let connections = self.connections.read().await;
        connections
            .get(id)
            .map(|(info, _)| info.clone())
            .ok_or_else(|| DbError::NotFound(format!("Connection not found: {}", id)))
    }

    /// Updates a connection
    pub async fn update_connection(&self, id: &str, connection: ConnectionInfo) -> DbResult<()> {
        let mut connections = self.connections.write().await;

        if !connections.contains_key(id) {
            return Err(DbError::NotFound(format!("Connection not found: {}", id)));
        }

        // Update connection, removing any existing client
        connections.insert(id.to_string(), (connection, None));
        Ok(())
    }

    /// Removes a connection
    pub async fn remove_connection(&self, id: &str) -> DbResult<()> {
        let mut connections = self.connections.write().await;

        if connections.remove(id).is_none() {
            return Err(DbError::NotFound(format!("Connection not found: {}", id)));
        }

        Ok(())
    }

    /// Connects to a database by ID
    pub async fn connect(&self, id: &str) -> DbResult<()> {
        let mut connections = self.connections.write().await;

        let entry = connections
            .get_mut(id)
            .ok_or_else(|| DbError::NotFound(format!("Connection not found: {}", id)))?;

        // If already connected, nothing to do
        if entry.1.is_some() {
            return Ok(());
        }

        // Create and store client
        let client = create_client(entry.0.clone()).await?;
        entry.1 = Some(Arc::new(client));

        Ok(())
    }

    /// Disconnects from a database
    pub async fn disconnect(&self, id: &str) -> DbResult<()> {
        let mut connections = self.connections.write().await;

        let entry = connections
            .get_mut(id)
            .ok_or_else(|| DbError::NotFound(format!("Connection not found: {}", id)))?;

        if entry.1.is_none() {
            return Err(DbError::NotFound(format!(
                "Not connected to database: {}",
                id
            )));
        }

        // Remove the client (which will disconnect)
        entry.1 = None;
        Ok(())
    }

    /// Checks if connected to a database
    pub async fn is_connected(&self, id: &str) -> bool {
        let connections = self.connections.read().await;
        connections
            .get(id)
            .map(|(_, client)| client.is_some())
            .unwrap_or(false)
    }

    /// Gets an active client by ID
    async fn get_client(&self, id: &str) -> DbResult<Arc<Box<dyn DatabaseClient>>> {
        let connections = self.connections.read().await;

        let (_, client_opt) = connections
            .get(id)
            .ok_or_else(|| DbError::NotFound(format!("Connection not found: {}", id)))?;

        client_opt
            .clone()
            .ok_or_else(|| DbError::NotFound(format!("Not connected to database: {}", id)))
    }

    /// Executes a query on a database
    pub async fn execute_query(&self, id: &str, sql: &str) -> DbResult<QueryResult> {
        let client = self.get_client(id).await?;
        client.execute_query(sql).await
    }

    /// Executes multiple queries on a database
    pub async fn execute_queries(&self, id: &str, sql: &str) -> DbResult<Vec<QueryResult>> {
        let client = self.get_client(id).await?;
        let statements = strings::split_sql_statements(sql)?;

        let mut results = Vec::with_capacity(statements.len());
        for statement in statements {
            results.push(client.execute_query(&statement).await?);
        }

        Ok(results)
    }

    /// Tests a connection by ID
    pub async fn test_connection(&self, id: &str) -> DbResult<()> {
        // Get connection info
        let connections = self.connections.read().await;
        let (connection_info, _) = connections
            .get(id)
            .ok_or_else(|| DbError::NotFound(format!("Connection not found: {}", id)))?;

        // Create a client and test connection
        let client = create_client(connection_info.clone()).await?;
        client.test_connection().await
    }

    /// Lists all connections
    pub async fn list_connections(&self) -> Vec<ConnectionInfo> {
        let connections = self.connections.read().await;
        connections.values().map(|(info, _)| info.clone()).collect()
    }

    /// Establishes a connection with timeout and testing
    pub async fn establish_connection(conn_string: &str, db_type: &DatabaseType) -> DbResult<()> {
        match db_type {
            DatabaseType::Postgres => {
                use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
                use std::str::FromStr;

                let options = PgConnectOptions::from_str(conn_string)
                    .map_err(|e| DbError::Connection(e.to_string()))?;

                let pool = PgPoolOptions::new()
                    .max_connections(1)
                    .acquire_timeout(Duration::from_secs(5))
                    .test_before_acquire(true)
                    .connect_with(options)
                    .await
                    .map_err(|e| DbError::Connection(e.to_string()))?;

                // Test the connection
                sqlx::query("SELECT 1")
                    .execute(&pool)
                    .await
                    .map_err(|e| DbError::Connection(e.to_string()))?;

                // Close the pool
                pool.close().await;
            }
        }

        Ok(())
    }

    /// Closes all connections
    pub async fn close_all_connections(&self) {
        let mut connections = self.connections.write().await;

        // Clear all connections by replacing with empty HashMap
        let entries: Vec<(
            String,
            (ConnectionInfo, Option<Arc<Box<dyn DatabaseClient>>>),
        )> = connections.drain().collect();

        // Re-insert the connections without clients
        for (id, (info, _)) in entries {
            connections.insert(id, (info, None));
        }
    }
}
