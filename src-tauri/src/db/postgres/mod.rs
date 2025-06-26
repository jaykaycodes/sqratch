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
    types::{ColumnDefinition, DbEntity, QueryResult, Row, SchemaEntity, SchemaLevelEntity},
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
            .max_connections(10)
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

    async fn get_all_entities(&self) -> DbResult<HashMap<String, DbEntity>> {
        let pool = self.get_pool()?;
        let mut entities = HashMap::new();
        let mut schema_children_map: HashMap<String, Vec<String>> = HashMap::new();

        // Query 1: Get all schemas
        let schema_query = r#"
            SELECT
                n.oid::TEXT AS schema_id,
                n.nspname AS schema_name,
                CASE
                    WHEN n.nspname IN ('pg_catalog', 'information_schema', 'pg_toast')
                         OR n.nspname LIKE 'pg_%'
                    THEN true
                    ELSE false
                END AS is_system,
                e.extname AS extension_name
            FROM pg_namespace n
            LEFT JOIN pg_depend d ON d.objid = n.oid AND d.deptype = 'e'
            LEFT JOIN pg_extension e ON e.oid = d.refobjid
            ORDER BY n.nspname
        "#;

        let schema_rows = sqlx::query(schema_query).fetch_all(pool).await?;
        for row in schema_rows {
            let id: String = row.get("schema_id");
            let name: String = row.get("schema_name");
            let is_system: bool = row.get("is_system");
            let extension_name: Option<String> = row.get("extension_name");

            schema_children_map.insert(id.clone(), Vec::new());
            entities.insert(
                id.clone(),
                DbEntity::Schema(SchemaEntity {
                    id,
                    name,
                    is_system,
                    extension_name,
                    children: Vec::new(),
                }),
            );
        }

        // Query 2: Get tables, views, materialized views, foreign tables
        let class_query = r#"
            SELECT
                c.oid::TEXT AS id,
                c.relname AS name,
                c.relkind::TEXT AS kind,
                n.oid::TEXT AS schema_id,
                CASE
                    WHEN n.nspname IN ('pg_catalog', 'information_schema', 'pg_toast')
                         OR n.nspname LIKE 'pg_%'
                    THEN true
                    ELSE false
                END AS is_system,
                e.extname AS extension_name
            FROM pg_class c
            JOIN pg_namespace n ON c.relnamespace = n.oid
            LEFT JOIN pg_depend d ON d.objid = n.oid AND d.deptype = 'e'
            LEFT JOIN pg_extension e ON e.oid = d.refobjid
            WHERE c.relkind IN ('r', 'v', 'm', 'f', 'S')
            ORDER BY n.nspname, c.relname
        "#;

        let class_rows = sqlx::query(class_query).fetch_all(pool).await?;
        for row in class_rows {
            let id: String = row.get("id");
            let name: String = row.get("name");
            let kind: String = row.get("kind");
            let schema_id: String = row.get("schema_id");
            let is_system: bool = row.get("is_system");
            let extension_name: Option<String> = row.get("extension_name");

            if let Some(children) = schema_children_map.get_mut(&schema_id) {
                children.push(id.clone());
            }

            let schema_level = SchemaLevelEntity {
                id: id.clone(),
                name,
                is_system,
                schema_id,
                extension_name,
            };

            let entity = match kind.as_str() {
                "r" => DbEntity::Table(schema_level),
                "v" => DbEntity::View(schema_level),
                "m" => DbEntity::MaterializedView(schema_level),
                "f" => DbEntity::ForeignTable(schema_level),
                // "S" => DbEntity::Sequence(schema_level),
                _ => continue,
            };

            entities.insert(id, entity);
        }

        // Query 3: Get functions and procedures
        // let proc_query = r#"
        //     SELECT
        //         p.oid::TEXT AS id,
        //         p.proname AS name,
        //         n.oid::TEXT AS schema_id,
        //         CASE
        //             WHEN n.nspname IN ('pg_catalog', 'information_schema', 'pg_toast')
        //                  OR n.nspname LIKE 'pg_%'
        //             THEN true
        //             ELSE false
        //         END AS is_system,
        //         e.extname AS extension_name
        //     FROM pg_proc p
        //     JOIN pg_namespace n ON p.pronamespace = n.oid
        //     LEFT JOIN pg_depend d ON d.objid = n.oid AND d.deptype = 'e'
        //     LEFT JOIN pg_extension e ON e.oid = d.refobjid
        //     ORDER BY n.nspname, p.proname
        // "#;

        // let proc_rows = sqlx::query(proc_query).fetch_all(pool).await?;
        // for row in proc_rows {
        //     let id: String = row.get("id");
        //     let name: String = row.get("name");
        //     let schema_id: String = row.get("schema_id");
        //     let is_system: bool = row.get("is_system");
        //     let extension_name: Option<String> = row.get("extension_name");

        //     if let Some(children) = schema_children_map.get_mut(&schema_id) {
        //         children.push(id.clone());
        //     }

        //     entities.insert(
        //         id.clone(),
        //         DbEntity::Function(SchemaLevelEntity {
        //             id,
        //             name,
        //             is_system,
        //             schema_id,
        //             extension_name,
        //             children: Vec::new(),
        //         }),
        //     );
        // }

        // // Query 4: Get custom types
        // let type_query = r#"
        //     SELECT
        //         t.oid::TEXT AS id,
        //         t.typname AS name,
        //         n.oid::TEXT AS schema_id,
        //         CASE
        //             WHEN n.nspname IN ('pg_catalog', 'information_schema', 'pg_toast')
        //                  OR n.nspname LIKE 'pg_%'
        //             THEN true
        //             ELSE false
        //         END AS is_system,
        //         e.extname AS extension_name
        //     FROM pg_type t
        //     JOIN pg_namespace n ON t.typnamespace = n.oid
        //     LEFT JOIN pg_depend d ON d.objid = n.oid AND d.deptype = 'e'
        //     LEFT JOIN pg_extension e ON e.oid = d.refobjid
        //     WHERE t.typtype NOT IN ('b', 'p')  -- Exclude built-in and pseudo types
        //     ORDER BY n.nspname, t.typname
        // "#;

        // let type_rows = sqlx::query(type_query).fetch_all(pool).await?;
        // for row in type_rows {
        //     let id: String = row.get("id");
        //     let name: String = row.get("name");
        //     let schema_id: String = row.get("schema_id");
        //     let is_system: bool = row.get("is_system");
        //     let extension_name: Option<String> = row.get("extension_name");

        //     if let Some(children) = schema_children_map.get_mut(&schema_id) {
        //         children.push(id.clone());
        //     }

        //     entities.insert(
        //         id.clone(),
        //         DbEntity::CustomType(SchemaLevelEntity {
        //             id,
        //             name,
        //             is_system,
        //             schema_id,
        //             extension_name,
        //             children: Vec::new(),
        //         }),
        //     );
        // }

        // // Query 5: Get indexes
        // let index_query = r#"
        //     SELECT
        //         i.indexrelid::TEXT AS id,
        //         ic.relname AS name,
        //         i.indrelid::TEXT AS table_id,
        //         CASE
        //             WHEN n.nspname IN ('pg_catalog', 'information_schema', 'pg_toast')
        //                  OR n.nspname LIKE 'pg_%'
        //             THEN true
        //             ELSE false
        //         END AS is_system
        //     FROM pg_index i
        //     JOIN pg_class ic ON ic.oid = i.indexrelid
        //     JOIN pg_class tc ON tc.oid = i.indrelid
        //     JOIN pg_namespace n ON tc.relnamespace = n.oid
        //     ORDER BY ic.relname
        // "#;

        // let index_rows = sqlx::query(index_query).fetch_all(pool).await?;
        // for row in index_rows {
        //     let id: String = row.get("id");
        //     let name: String = row.get("name");
        //     let table_id: String = row.get("table_id");
        //     let is_system: bool = row.get("is_system");

        //     entities.insert(
        //         id.clone(),
        //         DbEntity::Index(TableLevelEntity {
        //             id,
        //             name,
        //             is_system,
        //             table_id,
        //         }),
        //     );
        // }

        // // Query 6: Get triggers
        // let trigger_query = r#"
        //     SELECT
        //         t.oid::TEXT AS id,
        //         t.tgname AS name,
        //         t.tgrelid::TEXT AS table_id,
        //         CASE
        //             WHEN n.nspname IN ('pg_catalog', 'information_schema', 'pg_toast')
        //                  OR n.nspname LIKE 'pg_%'
        //             THEN true
        //             ELSE false
        //         END AS is_system
        //     FROM pg_trigger t
        //     JOIN pg_class c ON c.oid = t.tgrelid
        //     JOIN pg_namespace n ON c.relnamespace = n.oid
        //     WHERE NOT t.tgisinternal  -- Exclude internal triggers
        //     ORDER BY t.tgname
        // "#;

        // let trigger_rows = sqlx::query(trigger_query).fetch_all(pool).await?;
        // for row in trigger_rows {
        //     let id: String = row.get("id");
        //     let name: String = row.get("name");
        //     let table_id: String = row.get("table_id");
        //     let is_system: bool = row.get("is_system");

        //     entities.insert(
        //         id.clone(),
        //         DbEntity::Trigger(TableLevelEntity {
        //             id,
        //             name,
        //             is_system,
        //             table_id,
        //         }),
        //     );
        // }

        // Update schema entities with their children
        for (schema_id, children) in schema_children_map {
            if let Some(DbEntity::Schema(schema)) = entities.get_mut(&schema_id) {
                schema.children = children;
            }
        }

        Ok(entities)
    }
}
