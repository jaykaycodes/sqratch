use tauri::{Runtime, Window};
use taurpc;

use crate::db::types::{QueryResult, SchemaInfo};
use crate::errors::AppError;
use crate::state::get_window_client;

#[taurpc::procedures(path = "db", export_to = "../src/lib/taurpc.ts")]
pub trait DbApi {
    async fn get_connection_string<R: Runtime>(window: Window<R>) -> Result<String, AppError>;

    // Test connection with raw connection string
    // TODO: Implement this w/o a current connection
    // async fn test_connection_string(conn_string: String) -> Result<String, AppError>;

    // Checks if the current client is connected
    async fn is_connected<R: Runtime>(window: Window<R>) -> Result<bool, AppError>;

    // Connect to database for current window
    async fn connect<R: Runtime>(window: Window<R>) -> Result<(), AppError>;

    // Disconnect from database
    async fn disconnect<R: Runtime>(window: Window<R>) -> Result<(), AppError>;

    // Execute a single query
    async fn execute_query<R: Runtime>(
        window: Window<R>,
        query: String,
    ) -> Result<QueryResult, AppError>;

    // Get all entities (tables, views, functions, etc.) from the database
    async fn get_all_schemas<R: Runtime>(window: Window<R>) -> Result<Vec<SchemaInfo>, AppError>;
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
    async fn get_connection_string<R: Runtime>(
        self,
        window: Window<R>,
    ) -> Result<String, AppError> {
        let client = get_window_client(&window)?;
        let guard = client.lock().await;
        Ok(guard.get_connection_string())
    }

    async fn is_connected<R: Runtime>(self, window: Window<R>) -> Result<bool, AppError> {
        let client = get_window_client(&window)?;
        let guard = client.lock().await;
        Ok(guard.is_connected().await?)
    }

    async fn connect<R: Runtime>(self, window: Window<R>) -> Result<(), AppError> {
        let client = get_window_client(&window)?;
        let mut guard = client.lock().await;
        Ok(guard.connect().await?)
    }

    async fn disconnect<R: Runtime>(self, window: Window<R>) -> Result<(), AppError> {
        let client = get_window_client(&window)?;
        let mut guard = client.lock().await;
        Ok(guard.disconnect().await?)
    }

    async fn execute_query<R: Runtime>(
        self,
        window: Window<R>,
        query: String,
    ) -> Result<QueryResult, AppError> {
        let client = get_window_client(&window)?;
        let guard = client.lock().await;
        Ok(guard.execute_query(&query).await?)
    }

    async fn get_all_schemas<R: Runtime>(
        self,
        window: Window<R>,
    ) -> Result<Vec<SchemaInfo>, AppError> {
        let client = get_window_client(&window)?;
        let guard = client.lock().await;
        Ok(guard.get_all_schemas().await?)
    }
}
