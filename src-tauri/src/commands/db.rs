use crate::db::{ConnectionInfo, ConnectionManager, DatabaseType, QueryResult};
use crate::utils::errors::Error;
use serde_json::Value;
use tauri::AppHandle;
use taurpc;

#[taurpc::procedures(path = "db")]
trait DbApi {
    async fn test_connection_string(
        conn_string: String,
        db_type: DatabaseType,
    ) -> Result<String, Error>;
    async fn list_connections() -> Result<Vec<ConnectionInfo>, Error>;
    async fn add_connection(conn_info: ConnectionInfo) -> Result<String, Error>;
    async fn connect_to_database(connection_id: String) -> Result<(), Error>;
    async fn disconnect_from_database(connection_id: String) -> Result<(), Error>;
    async fn is_connected(connection_id: String) -> Result<bool, Error>;
    async fn execute_query(connection_id: String, query: String) -> Result<QueryResult, Error>;
    async fn execute_queries(
        connection_id: String,
        queries: String,
    ) -> Result<Vec<QueryResult>, Error>;
    async fn test_connection(connection_id: String) -> Result<(), Error>;
    async fn load_connections_from_project(project_path: String) -> Result<Vec<String>, Error>;
    async fn get_schema_info(connection_id: String) -> Result<Value, Error>;
    async fn get_tables(connection_id: String) -> Result<Value, Error>;
    async fn get_paginated_rows(
        connection_id: String,
        table_name: String,
        page_index: u16,
        page_size: u32,
    ) -> Result<QueryResult, Error>;
}

#[derive(Clone)]
struct DbApiImpl {
    db_manager: ConnectionManager,
    app_handle: AppHandle,
}

#[taurpc::resolvers]
impl DbApi for DbApiImpl {
    async fn test_connection_string(
        self,
        conn_string: String,
        db_type: DatabaseType,
    ) -> Result<String, Error> {
        DatabaseManager::establish_connection(&conn_string, &db_type)
            .await
            .map(|_| "Connection successful".to_string())
            .map_err(Error::Db)
    }

    async fn list_connections(self) -> Result<Vec<ConnectionInfo>, Error> {
        Ok(self.db_manager.list_connections().await)
    }

    async fn add_connection(self, conn_info: ConnectionInfo) -> Result<String, Error> {
        let result = self
            .db_manager
            .add_connection(conn_info)
            .await
            .map_err(Error::Db)?;

        // Emit an event to notify UI
        #[derive(Clone, serde::Serialize)]
        pub struct ConnectionAddedEvent {
            id: String,
        }

        let event = ConnectionAddedEvent { id: result.clone() };
        self.app_handle
            .emit_all("connection-added", event)
            .map_err(|e| Error::Other(e.to_string()))?;

        Ok(result)
    }

    async fn connect_to_database(self, connection_id: String) -> Result<(), Error> {
        self.db_manager
            .connect(&connection_id)
            .await
            .map(|_| ()) // Ignore the handler return value
            .map_err(Error::Db)?;

        // Emit an event to notify UI
        #[derive(Clone, serde::Serialize)]
        pub struct ConnectionEstablishedEvent {
            id: String,
        }

        let event = ConnectionEstablishedEvent { id: connection_id };
        self.app_handle
            .emit_all("connection-established", event)
            .map_err(|e| Error::Other(e.to_string()))?;

        Ok(())
    }

    async fn disconnect_from_database(self, connection_id: String) -> Result<(), Error> {
        self.db_manager
            .disconnect(&connection_id)
            .await
            .map_err(Error::Db)?;

        // Emit an event to notify UI
        #[derive(Clone, serde::Serialize)]
        pub struct ConnectionClosedEvent {
            id: String,
        }

        let event = ConnectionClosedEvent { id: connection_id };
        self.app_handle
            .emit_all("connection-closed", event)
            .map_err(|e| Error::Other(e.to_string()))?;

        Ok(())
    }

    async fn is_connected(self, connection_id: String) -> Result<bool, Error> {
        Ok(self.db_manager.is_connected(&connection_id).await)
    }

    async fn execute_query(
        self,
        connection_id: String,
        query: String,
    ) -> Result<QueryResult, Error> {
        self.db_manager
            .execute_query(&connection_id, &query)
            .await
            .map_err(Error::Db)
    }

    async fn execute_queries(
        self,
        connection_id: String,
        queries: String,
    ) -> Result<Vec<QueryResult>, Error> {
        self.db_manager
            .execute_queries(&connection_id, &queries)
            .await
            .map_err(Error::Db)
    }

    async fn test_connection(self, connection_id: String) -> Result<(), Error> {
        self.db_manager
            .test_connection(&connection_id)
            .await
            .map_err(Error::Db)
    }

    async fn load_connections_from_project(
        self,
        project_path: String,
    ) -> Result<Vec<String>, Error> {
        let result = self
            .db_manager
            .load_from_project(&project_path)
            .await
            .map_err(Error::Db)?;

        // Emit an event to notify UI if connections were loaded
        if !result.is_empty() {
            #[derive(Clone, serde::Serialize)]
            pub struct ConnectionsLoadedEvent {
                count: usize,
                ids: Vec<String>,
            }

            let event = ConnectionsLoadedEvent {
                count: result.len(),
                ids: result.clone(),
            };
            self.app_handle
                .emit_all("connections-loaded", event)
                .map_err(|e| Error::Other(e.to_string()))?;
        }

        Ok(result)
    }

    async fn get_schema_info(self, connection_id: String) -> Result<Value, Error> {
        let handler = self
            .db_manager
            .get_handler(&connection_id)
            .await
            .map_err(Error::Db)?;

        let schema = handler.get_schema_info().await.map_err(Error::Db)?;

        serde_json::to_value(schema)
            .map_err(|e| Error::Other(format!("Failed to serialize schema: {}", e)))
    }

    async fn get_tables(self, connection_id: String) -> Result<Value, Error> {
        let handler = self
            .db_manager
            .get_handler(&connection_id)
            .await
            .map_err(Error::Db)?;

        let tables = handler.get_tables().await.map_err(Error::Db)?;

        serde_json::to_value(tables)
            .map_err(|e| Error::Other(format!("Failed to serialize tables: {}", e)))
    }

    async fn get_paginated_rows(
        self,
        connection_id: String,
        table_name: String,
        page_index: u16,
        page_size: u32,
    ) -> Result<QueryResult, Error> {
        let handler = self
            .db_manager
            .get_handler(&connection_id)
            .await
            .map_err(Error::Db)?;

        handler
            .get_paginated_rows(&table_name, page_index, page_size)
            .await
            .map_err(Error::Db)
    }
}
