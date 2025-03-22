use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use url::Url;

use crate::db::clients::common::DatabaseClient;
use crate::db::errors::{DbError, DbResult};
use crate::db::types::{ConnectionInfo, DatabaseType, QueryResult};

/// Manages multiple database connections
pub struct DatabaseManager {
    /// Stored connection configurations
    connections: RwLock<HashMap<String, ConnectionInfo>>,
    /// Active database clients
    clients: RwLock<HashMap<String, Arc<dyn DatabaseClient>>>,
}

impl DatabaseManager {
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
        if let Some(client) = clients.remove(id) {
            // Drop the lock to avoid deadlock
            drop(connections);
            drop(clients);

            // Disconnect the client
            client.disconnect().await?;

            // Reacquire the lock and update the connection
            connections = self.connections.write().await;
        }

        // Update the connection
        connections.insert(id.to_string(), connection);
        Ok(())
    }

    /// Removes a connection
    pub async fn remove_connection(&self, id: &str) -> DbResult<()> {
        // First disconnect if connected
        let mut clients = self.clients.write().await;
        if let Some(client) = clients.remove(id) {
            // Drop the lock to avoid deadlock
            drop(clients);

            // Disconnect the client
            client.disconnect().await?;

            // Reacquire the lock
            clients = self.clients.write().await;
        }

        // Now remove the connection
        let mut connections = self.connections.write().await;
        if connections.remove(id).is_none() {
            return Err(DbError::NotFound(format!("Connection not found: {}", id)));
        }

        Ok(())
    }

    /// Connects to a database by ID
    pub async fn connect(&self, id: &str) -> DbResult<Arc<dyn DatabaseClient>> {
        // Check if already connected
        {
            let clients = self.clients.read().await;
            if let Some(client) = clients.get(id) {
                return Ok(client.clone());
            }
        }

        // Get connection info
        let connection = self.get_connection(id).await?;

        // Create a client based on the database type
        let client = self.create_client(connection).await?;

        // Attempt to connect
        client.connect().await?;

        // Store the client
        let mut clients = self.clients.write().await;
        clients.insert(id.to_string(), client.clone());

        Ok(client)
    }

    /// Disconnects from a database
    pub async fn disconnect(&self, id: &str) -> DbResult<()> {
        let mut clients = self.clients.write().await;
        if let Some(client) = clients.remove(id) {
            // Drop the lock to avoid deadlock
            drop(clients);

            // Disconnect the client
            client.disconnect().await?;

            Ok(())
        } else {
            Err(DbError::NotFound(format!("Not connected to database: {}", id)))
        }
    }

    /// Checks if connected to a database
    pub async fn is_connected(&self, id: &str) -> bool {
        let clients = self.clients.read().await;
        clients.contains_key(id)
    }

    /// Gets an active client by ID
    pub async fn get_client(&self, id: &str) -> DbResult<Arc<dyn DatabaseClient>> {
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
        client.execute_queries(sql).await
    }

    /// Tests a connection by ID
    pub async fn test_connection(&self, id: &str) -> DbResult<()> {
        // Get connection info
        let connection = self.get_connection(id).await?;

        // Create a client based on the database type
        let client = self.create_client(connection).await?;

        // Test the connection
        client.test_connection().await
    }

    /// Lists all connections
    pub async fn list_connections(&self) -> Vec<ConnectionInfo> {
        let connections = self.connections.read().await;
        connections.values().cloned().collect()
    }

    /// Creates a database client based on connection info
    async fn create_client(&self, connection: ConnectionInfo) -> DbResult<Arc<dyn DatabaseClient>> {
        // Import client implementations here to avoid circular dependencies
        use crate::db::clients::postgres::PostgresClient;
        use crate::db::clients::mysql::MySqlClient;
        use crate::db::clients::sqlite::SqliteClient;

        match connection.db_type {
            DatabaseType::Postgres => Ok(Arc::new(PostgresClient::new(connection))),
            DatabaseType::MySQL => Ok(Arc::new(MySqlClient::new(connection))),
            DatabaseType::SQLite => Ok(Arc::new(SqliteClient::new(connection))),
        }
    }

    /// Loads connections from a project directory
    pub async fn load_from_project(&self, project_path: &str) -> DbResult<Vec<String>> {
        let connection_dir = Path::new(project_path).join(".sqratch").join("connections");
        if !connection_dir.exists() {
            return Ok(vec![]);
        }

        let mut loaded_ids = Vec::new();

        // Read all JSON files in the connections directory
        if let Ok(entries) = fs::read_dir(&connection_dir) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        match serde_json::from_str::<ConnectionInfo>(&content) {
                            Ok(connection) => {
                                let id = connection.id.clone();
                                self.add_connection(connection).await?;
                                loaded_ids.push(id);
                            },
                            Err(e) => {
                                eprintln!("Failed to parse connection file {}: {}", path.display(), e);
                            }
                        }
                    }
                }
            }
        }

        Ok(loaded_ids)
    }

    /// Parses a connection string to determine the database type and info
    pub fn parse_connection_string(connection_string: &str) -> DbResult<ConnectionInfo> {
        let url = Url::parse(connection_string)?;

        let scheme = url.scheme();
        let db_type = match scheme {
            "postgres" | "postgresql" => DatabaseType::Postgres,
            "mysql" | "mariadb" => DatabaseType::MySQL,
            "sqlite" => DatabaseType::SQLite,
            _ => return Err(DbError::Config(format!("Unsupported database type: {}", scheme))),
        };

        let host = url.host_str().unwrap_or("localhost").to_string();
        let port = url.port().unwrap_or(match db_type {
            DatabaseType::Postgres => 5432,
            DatabaseType::MySQL => 3306,
            DatabaseType::SQLite => 0,
        });

        let database = match db_type {
            DatabaseType::SQLite => url.path().to_string(),
            _ => url.path().trim_start_matches('/').to_string(),
        };

        let username = url.username().to_string();
        let password = if url.password().is_some() {
            url.password().unwrap().to_string()
        } else {
            "".to_string()
        };

        // Create a default name based on host and database
        let name = match db_type {
            DatabaseType::SQLite => format!("SQLite: {}", database),
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
pub fn create_db_manager() -> DatabaseManager {
    DatabaseManager::new()
}

/// Helper function to parse connection information from project path or env var
pub fn parse_connection_config(
    project_path: Option<&str>,
    env_var: Option<&str>,
) -> DbResult<Option<ConnectionInfo>> {
    // First check env var if specified
    if let Some(env_name) = env_var {
        if let Ok(conn_str) = std::env::var(env_name) {
            println!("Found connection string in environment variable {}", env_name);
            let connection = DatabaseManager::parse_connection_string(&conn_str)?;
            return Ok(Some(connection));
        }
    }

    // Check for SQRATCH_PROJECT_PATH env var
    if let Ok(conn_str) = std::env::var("SQRATCH_PROJECT_PATH") {
        if conn_str.starts_with("postgres://") ||
           conn_str.starts_with("mysql://") ||
           conn_str.starts_with("sqlite:") {
            println!("Found connection string in SQRATCH_PROJECT_PATH");
            let connection = DatabaseManager::parse_connection_string(&conn_str)?;
            return Ok(Some(connection));
        }
    }

    // Then check project path if provided
    if let Some(path) = project_path {
        // Look for .sqratch/connections/current.json in the project path
        let connection_file = Path::new(path).join(".sqratch").join("connections").join("current.json");
        if connection_file.exists() {
            println!("Found connection file at: {}", connection_file.display());

            // Read and parse the JSON file
            match fs::read_to_string(&connection_file) {
                Ok(json_str) => {
                    match serde_json::from_str::<ConnectionInfo>(&json_str) {
                        Ok(connection) => {
                            println!("Using connection for project: {}", connection.name);
                            return Ok(Some(connection));
                        },
                        Err(e) => {
                            return Err(DbError::Config(format!("Failed to parse connection file: {}", e)));
                        }
                    }
                },
                Err(e) => {
                    return Err(DbError::Config(format!("Failed to read connection file: {}", e)));
                }
            }
        } else {
            println!("No connection file found at: {}", connection_file.display());
        }
    }

    // If we get here, we couldn't find a connection
    Ok(None)
}
