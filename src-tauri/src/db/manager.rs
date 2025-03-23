use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use url::Url;

use crate::db::client::{create_client, DatabaseClient};
use crate::db::types::DatabaseType;
use crate::db::errors::{DbError, DbResult};
use crate::db::types::{ConnectionInfo, QueryResult};
use crate::db::utils::strings;
use crate::project;

pub struct ConnectionManager {
    connections: RwLock<HashMap<String, ConnectionInfo>>,
    clients: RwLock<HashMap<String, Arc<Box<dyn DatabaseClient>>>>,
}

impl ConnectionManager {
    /// Creates a new database manager
    pub fn new() -> Self {
        Self {
            connections: RwLock::new(HashMap::new()),
            clients: RwLock::new(HashMap::new()),
        }
    }

    /// Adds a connection without connecting
    pub async fn add_connection(&self, connection: ConnectionInfo) -> DbResult<String> {
        let id = connection.id.clone();
        let mut connections = self.connections.write().await;
        connections.insert(id.clone(), connection);
        Ok(id)
    }

    /// Gets a connection by ID
    pub async fn get_connection(&self, id: &str) -> DbResult<ConnectionInfo> {
        let connections = self.connections.read().await;
        connections
            .get(id)
            .cloned()
            .ok_or_else(|| DbError::NotFound(format!("Connection not found: {}", id)))
    }

    /// Updates a connection
    pub async fn update_connection(&self, id: &str, connection: ConnectionInfo) -> DbResult<()> {
        // First check if connection exists
        let mut connections = self.connections.write().await;
        if !connections.contains_key(id) {
            return Err(DbError::NotFound(format!("Connection not found: {}", id)));
        }

        // If the connection is active, disconnect it first
        let mut clients = self.clients.write().await;
        if clients.remove(id).is_some() {
            // Connection will be closed when Arc is dropped
        }

        // Update the connection
        connections.insert(id.to_string(), connection);
        Ok(())
    }

    /// Removes a connection
    pub async fn remove_connection(&self, id: &str) -> DbResult<()> {
        // Remove the client (which will disconnect)
        let mut clients = self.clients.write().await;
        clients.remove(id);

        // Now remove the connection
        let mut connections = self.connections.write().await;
        if connections.remove(id).is_none() {
            return Err(DbError::NotFound(format!("Connection not found: {}", id)));
        }

        Ok(())
    }

    /// Connects to a database by ID
    pub async fn connect(&self, id: &str) -> DbResult<()> {
        // Check if already connected
        {
            let clients = self.clients.read().await;
            if clients.contains_key(id) {
                return Ok(());
            }
        }

        // Get connection info
        let connection = self.get_connection(id).await?;

        // Create a client
        let client = create_client(connection).await?;

        // Store the client
        let mut clients = self.clients.write().await;
        clients.insert(id.to_string(), Arc::new(client));

        Ok(())
    }

    /// Disconnects from a database
    pub async fn disconnect(&self, id: &str) -> DbResult<()> {
        let mut clients = self.clients.write().await;
        if clients.remove(id).is_none() {
            return Err(DbError::NotFound(format!("Not connected to database: {}", id)));
        }
        Ok(())
    }

    /// Checks if connected to a database
    pub async fn is_connected(&self, id: &str) -> bool {
        let clients = self.clients.read().await;
        clients.contains_key(id)
    }

    /// Gets an active client by ID
    async fn get_client(&self, id: &str) -> DbResult<Arc<Box<dyn DatabaseClient>>> {
        let clients = self.clients.read().await;
        clients
            .get(id)
            .cloned()
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
        let connection = self.get_connection(id).await?;

        // Create a client and test connection
        let client = create_client(connection).await?;
        client.test_connection().await
    }

    /// Lists all connections
    pub async fn list_connections(&self) -> Vec<ConnectionInfo> {
        let connections = self.connections.read().await;
        connections.values().cloned().collect()
    }

    /// Loads connections from a project directory
    pub async fn load_from_project(&self, project_path: &str) -> DbResult<Vec<String>> {
        // Try to get a connection from the project configuration
        if let Some(connection) = project::parse_project_config(project_path).await? {
            let id = connection.id.clone();
            self.add_connection(connection).await?;
            return Ok(vec![id]);
        }

        // If no connection from config, try to load from connections directory
        let connection_dir = Path::new(project_path).join(".sqratch").join("connections");
        if !connection_dir.exists() {
            return Ok(vec![]);
        }

        let mut loaded_ids = Vec::new();

        // Read all JSON files in the connections directory
        if let Ok(entries) = std::fs::read_dir(&connection_dir) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                    match connection::load_connection_from_file(&path) {
                        Ok(connection) => {
                            let id = connection.id.clone();
                            self.add_connection(connection).await?;
                            loaded_ids.push(id);
                        },
                        Err(e) => {
                            eprintln!("Failed to load connection file {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }

        Ok(loaded_ids)
    }

    /// Gets project configuration
    pub fn get_project_config(&self, project_path: &str) -> DbResult<project::ProjectConfig> {
        project::ProjectManager::parse_config(project_path)
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
                sqlx::query("SELECT 1").execute(&pool).await
                    .map_err(|e| DbError::Connection(e.to_string()))?;

                // Close the pool
                pool.close().await;
            }
        }

        Ok(())
    }

    /// Parses a connection string to determine the database type and info
    pub fn parse_connection_string(connection_string: &str) -> DbResult<ConnectionInfo> {
        let url = Url::parse(connection_string)?;

        let scheme = url.scheme();
        let db_type = match scheme {
            "postgres" | "postgresql" => DatabaseType::Postgres,
            _ => return Err(DbError::Config(format!("Unsupported database type: {}", scheme))),
        };

        let host = url.host_str().unwrap_or("localhost").to_string();
        let port = url.port().unwrap_or(match db_type {
            DatabaseType::Postgres => 5432,
        });

        let database = match db_type {
            _ => url.path().trim_start_matches('/').to_string(),
        };

        let username = url.username().to_string();
        let password = url.password().unwrap_or("").to_string();

        // Create a default name based on host and database
        let name = match db_type {
            _ => format!("{} on {}", database, host),
        };

        // Create a new connection info
        let mut connection = ConnectionInfo::new(name, db_type);
        connection.connection_string = Some(connection_string.to_string());
        connection.host = Some(host);
        connection.port = Some(port);
        connection.database = Some(database);
        connection.username = Some(username);
        connection.password = Some(password);

        // Parse query parameters as options
        let mut options = HashMap::new();
        for (key, value) in url.query_pairs() {
            options.insert(key.to_string(), value.to_string());
        }

        if !options.is_empty() {
            connection.options = Some(options);
        }

        Ok(connection)
    }
}

// Helper function to create a database manager
pub fn create_db_manager() -> ConnectionManager {
    ConnectionManager::new()
}

/// Helper function to parse connection information from project path or env var
pub async fn parse_connection_config(
    project_path: Option<&str>,
    env_var: Option<&str>,
) -> DbResult<Option<ConnectionInfo>> {
    // If project path is provided, use the project config
    if let Some(path) = project_path {
        // Try to get connection from project configuration
        if let Some(connection) = project::parse_project_config(path).await? {
            return Ok(Some(connection));
        }
    }

    // If env_var is specified, try that
    if let Some(env_name) = env_var {
        if let Ok(conn_str) = std::env::var(env_name) {
            println!("Found connection string in environment variable {}", env_name);
            let connection = connection::parse_connection_string(&conn_str)?;
            return Ok(Some(connection));
        }
    }

    // Check for SQRATCH_PROJECT_PATH env var
    if let Ok(conn_str) = std::env::var("SQRATCH_PROJECT_PATH") {
        if conn_str.starts_with("postgres://") {
            println!("Found connection string in SQRATCH_PROJECT_PATH");
            let connection = connection::parse_connection_string(&conn_str)?;
            return Ok(Some(connection));
        }
    }

    // If project path is provided, try traditional method as fallback
    if let Some(path) = project_path {
        // Look for .sqratch/connections/current.json in the project path
        let connection_file = Path::new(path).join(".sqratch").join("connections").join("current.json");
        if connection_file.exists() {
            println!("Found connection file at: {}", connection_file.display());

            // Read and parse the connection file
            match connection::load_connection_from_file(&connection_file) {
                Ok(connection) => {
                    println!("Using connection for project: {}", connection.name);
                    return Ok(Some(connection));
                },
                Err(e) => {
                    return Err(e);
                }
            }
        } else {
            println!("No connection file found at: {}", connection_file.display());
        }
    }

    // If we get here, we couldn't find a connection
    Ok(None)
}
