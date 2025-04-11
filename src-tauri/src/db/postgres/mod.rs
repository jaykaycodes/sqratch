use async_trait::async_trait;
use serde_json::Value;
use sqlx::{
    postgres::{PgPoolOptions, PgRow},
    Column, Pool, Postgres, Row as SqlxRow,
};
use std::collections::HashMap;

use crate::db::{
    client::DatabaseClient,
    errors::{DbError, DbResult},
    types::{ColumnDefinition, QueryResult, Row, SchemaEntity, SchemaEntityType, SchemaInfo},
};

pub struct PostgresClient {
    connection_string: String,
    pool: Option<Pool<Postgres>>,
}

impl PostgresClient {
    pub fn new(connection_string: &str) -> DbResult<Self> {
        Ok(Self {
            connection_string: connection_string.to_string(),
            pool: None,
        })
    }

    // This function gets the pool or returns an error if not connected
    fn get_pool(&self) -> DbResult<&Pool<Postgres>> {
        self.pool
            .as_ref()
            .ok_or_else(|| DbError::Connection("Database client is not connected".to_string()))
    }
}

#[async_trait]
impl DatabaseClient for PostgresClient {
    fn get_connection_string(&self) -> String {
        self.connection_string.clone()
    }

    async fn is_connected(&self) -> DbResult<bool> {
        match self.get_pool() {
            Ok(pool) => Ok(!pool.is_closed()),
            Err(_) => Ok(false),
        }
    }

    async fn test_connection(&self) -> DbResult<()> {
        let pool = self.get_pool()?;
        sqlx::query("SELECT 1").execute(pool).await?;
        Ok(())
    }

    async fn connect(&mut self) -> DbResult<()> {
        // Check if already connected
        if let Ok(true) = self.is_connected().await {
            return Ok(());
        }

        // Create a new pool
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&self.connection_string)
            .await?;

        self.pool = Some(pool);
        Ok(())
    }

    async fn disconnect(&mut self) -> DbResult<()> {
        if let Ok(true) = self.is_connected().await {
            if let Some(pool) = self.pool.take() {
                pool.close().await;
            }
        }
        Ok(())
    }

    async fn reconnect(&mut self) -> DbResult<()> {
        self.disconnect().await?;
        self.connect().await
    }

    async fn reconnect_with_string(&mut self, connection_string: &str) -> DbResult<()> {
        self.disconnect().await?;
        self.connection_string = connection_string.to_string();
        self.connect().await
    }

    async fn execute_query(&self, sql: &str) -> DbResult<QueryResult> {
        let pool = self.get_pool()?;
        let rows = sqlx::query(sql).fetch_all(pool).await?;

        if rows.is_empty() {
            return Ok(QueryResult {
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                query: sql.to_string(),
                rows_affected: None,
                execution_time_ms: 0,
                columns: Vec::new(),
                rows: Vec::new(),
                warnings: Vec::new(),
                result_index: 0,
            });
        }

        let pg_row: &PgRow = rows.first().unwrap();
        let columns = pg_row
            .columns()
            .iter()
            .map(|col| ColumnDefinition {
                name: col.name().to_string(),
                data_type: col.type_info().to_string(),
                nullable: true,      // Default to true since we can't easily determine
                primary_key: false,  // Cannot determine from result alone
                default_value: None, // Cannot determine from result alone
            })
            .collect();

        let mut result_rows = Vec::new();
        for row in rows {
            let mut values = HashMap::new();
            for (i, col) in row.columns().iter().enumerate() {
                let value: Option<Value> = row.try_get(i)?;
                values.insert(
                    col.name().to_string(),
                    value.map_or_else(|| "NULL".to_string(), |v| v.to_string()),
                );
            }
            result_rows.push(Row { values });
        }

        Ok(QueryResult {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            query: sql.to_string(),
            rows_affected: None,
            execution_time_ms: 0,
            columns,
            rows: result_rows,
            warnings: Vec::new(),
            result_index: 0,
        })
    }

    async fn get_all_schemas(&self) -> DbResult<Vec<SchemaInfo>> {
        let pool = self.get_pool()?;

        // Query for schemas with their OIDs
        let schema_query = r#"
            SELECT
                n.oid::TEXT AS schema_id,
                n.nspname AS schema_name
            FROM pg_namespace n
            WHERE n.nspname NOT IN ('pg_catalog', 'information_schema')
            ORDER BY n.nspname
        "#;

        // Query for tables, views, and materialized views across all schemas
        let rel_query = r#"
            SELECT
                c.oid::TEXT AS entity_id,
                n.oid::TEXT AS schema_id,
                n.nspname AS schema_name,
                c.relname AS entity_name,
                c.relkind::TEXT AS entity_type
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE c.relkind IN ('r', 'v', 'm')
            AND n.nspname NOT IN ('pg_catalog', 'information_schema')
            ORDER BY n.nspname, c.relname
        "#;

        // Query for functions across all schemas
        let func_query = r#"
            SELECT
                p.oid::TEXT AS entity_id,
                n.oid::TEXT AS schema_id,
                n.nspname AS schema_name,
                p.proname AS entity_name
            FROM pg_proc p
            JOIN pg_namespace n ON n.oid = p.pronamespace
            WHERE n.nspname NOT IN ('pg_catalog', 'information_schema')
            ORDER BY n.nspname, p.proname
        "#;

        // Fetch schemas
        let schema_rows = sqlx::query(schema_query).fetch_all(pool).await?;

        // Initialize schema map with IDs
        let mut schema_map: HashMap<String, (String, Vec<SchemaEntity>)> = HashMap::new();
        for row in schema_rows {
            let schema_name: String = row.get("schema_name");
            let schema_id: String = row.get("schema_id");
            schema_map.insert(schema_name.clone(), (schema_id.to_string(), Vec::new()));
        }

        // Fetch relations (tables, views, materialized views)
        let rel_rows = sqlx::query(rel_query).fetch_all(pool).await?;

        // Fetch functions
        let func_rows = sqlx::query(func_query).fetch_all(pool).await?;

        // Process relations
        for row in rel_rows {
            let schema_name: String = row.get("schema_name");
            let entity_type_str: String = row.get("entity_type");
            let entity_id: String = row.get("entity_id");

            let entity_type = match entity_type_str.as_str() {
                "r" => SchemaEntityType::Table,
                "v" => SchemaEntityType::View,
                "m" => SchemaEntityType::MaterializedView,
                _ => continue,
            };

            let entity = SchemaEntity {
                id: entity_id,
                name: row.get("entity_name"),
                schema: schema_name.clone(),
                entity_type,
            };

            if let Some((_, entities)) = schema_map.get_mut(&schema_name) {
                entities.push(entity);
            }
        }

        // Process functions
        for row in func_rows {
            let schema_name: String = row.get("schema_name");
            let entity_id: String = row.get("entity_id");

            let entity = SchemaEntity {
                id: entity_id,
                name: row.get("entity_name"),
                schema: schema_name.clone(),
                entity_type: SchemaEntityType::Function,
            };

            if let Some((_, entities)) = schema_map.get_mut(&schema_name) {
                entities.push(entity);
            }
        }

        // Convert HashMap to Vec<SchemaInfo>
        let schema_infos: Vec<SchemaInfo> = schema_map
            .into_iter()
            .map(|(schema_name, (schema_id, entities))| SchemaInfo {
                id: schema_id,
                name: schema_name,
                entities,
            })
            .collect();

        Ok(schema_infos)
    }
}
