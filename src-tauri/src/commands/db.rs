use tauri::{Runtime, Window};
use taurpc;

use crate::db::types::{ConnectionStatus, Entity, QueryResult};
use crate::errors::AppError;
use crate::state::get_window_client;

#[taurpc::procedures(path = "db", export_to = "../src/lib/taurpc.ts", event_trigger = DbEventTrigger)]
pub trait DbApi {
    async fn get_connection_string(window: Window<impl Runtime>) -> Result<String, AppError>;

    // Test connection with raw connection string
    // TODO: Implement this w/o a current connection
    // async fn test_connection_string(conn_string: String) -> Result<String, AppError>;

    // Checks if the current client is connected
    async fn is_connected(window: Window<impl Runtime>) -> Result<bool, AppError>;

    // Connect to database for current window
    async fn connect(window: Window<impl Runtime>) -> Result<(), AppError>;

    // Disconnect from database
    async fn disconnect(window: Window<impl Runtime>) -> Result<(), AppError>;

    // Execute a single query
    async fn execute_query(
        window: Window<impl Runtime>,
        query: String,
    ) -> Result<QueryResult, AppError>;

    // Get all entities including schemas as a flat list
    async fn get_all_entities(window: Window<impl Runtime>) -> Result<Vec<Entity>, AppError>;

    // EVENTS

    #[taurpc(event)]
    async fn subscribe_connection_status(
        window: Window<impl Runtime>,
        status: ConnectionStatus,
    ) -> Result<(), AppError>;
}

#[derive(Clone)]
pub struct DbApiImpl;

impl Default for DbApiImpl {
    fn default() -> Self {
        Self {}
    }
}

#[taurpc::resolvers]
impl DbApi for DbApiImpl {
    async fn get_connection_string(self, window: Window<impl Runtime>) -> Result<String, AppError> {
        let client = get_window_client(&window)?;
        let guard = client.lock().await;
        Ok(guard.get_connection_string())
    }

    async fn is_connected(self, window: Window<impl Runtime>) -> Result<bool, AppError> {
        let client = get_window_client(&window)?;
        let guard = client.lock().await;
        Ok(guard.is_connected().await?)
    }

    async fn connect(self, window: Window<impl Runtime>) -> Result<(), AppError> {
        let client = get_window_client(&window)?;
        let mut guard = client.lock().await;
        Ok(guard.connect().await?)
    }

    async fn disconnect(self, window: Window<impl Runtime>) -> Result<(), AppError> {
        let client = get_window_client(&window)?;
        let mut guard = client.lock().await;
        Ok(guard.disconnect().await?)
    }

    async fn execute_query(
        self,
        window: Window<impl Runtime>,
        query: String,
    ) -> Result<QueryResult, AppError> {
        let client = get_window_client(&window)?;
        let mut guard = client.lock().await;

        if !guard.is_connected().await? {
            guard.connect().await?;
        }

        Ok(guard.execute_query(&query).await?)
    }

    async fn get_all_entities(self, window: Window<impl Runtime>) -> Result<Vec<Entity>, AppError> {
        let client = get_window_client(&window)?;
        let mut guard = client.lock().await;

        if !guard.is_connected().await? {
            guard.connect().await?;
        }

        Ok(guard.get_all_entities().await?)
    }
}
