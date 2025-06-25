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
    types::{
        ColumnDefinition, DbEntity, IndexEntity, QueryResult, Row, SchemaEntity, SchemaLevelEntity,
        TableLikeEntity, TriggerEntity,
    },
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

    async fn get_all_entities(&self) -> DbResult<Vec<DbEntity>> {
        let pool = self.get_pool()?;
        let mut entities = Vec::new();

        // 1. Query for schemas
        let schema_query = r#"
            SELECT
                n.oid::TEXT AS id,
                n.nspname AS name,
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
            let id: String = row.get("id");
            let name: String = row.get("name");
            let is_system: bool = row.get("is_system");
            let extension_name: Option<String> = row.get("extension_name");

            entities.push(DbEntity::Schema(SchemaEntity {
                id,
                name,
                is_system,
                extension_name,
            }));
        }

        // 2. Query for tables, views, materialized views, foreign tables with stats
        let table_like_query = r#"
            SELECT
                c.oid::TEXT AS id,
                c.relname AS name,
                n.oid::TEXT AS schema_id,
                c.relkind::TEXT AS kind,
                CASE
                    WHEN n.nspname IN ('pg_catalog', 'information_schema', 'pg_toast')
                         OR n.nspname LIKE 'pg_%'
                    THEN true
                    ELSE false
                END AS is_system,
                e.extname AS extension_name,
                COALESCE(s.n_tup_ins + s.n_tup_upd + s.n_tup_del + s.n_live_tup + s.n_dead_tup, 0)::BIGINT AS row_count_estimate,
                pg_total_relation_size(c.oid)::BIGINT AS size_bytes_estimate,
                COALESCE((SELECT COUNT(*) FROM pg_attribute WHERE attrelid = c.oid AND attnum > 0 AND NOT attisdropped), 0)::INT AS column_count
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            LEFT JOIN pg_stat_user_tables s ON s.relid = c.oid
            LEFT JOIN pg_depend d ON d.objid = c.oid AND d.deptype = 'e'
            LEFT JOIN pg_extension e ON e.oid = d.refobjid
            WHERE c.relkind IN ('r', 'v', 'm', 'f') -- tables, views, materialized views, foreign tables
            ORDER BY n.nspname, c.relname
        "#;

        let table_rows = sqlx::query(table_like_query).fetch_all(pool).await?;
        for row in table_rows {
            let id: String = row.get("id");
            let name: String = row.get("name");
            let schema_id: String = row.get("schema_id");
            let kind: String = row.get("kind");
            let is_system: bool = row.get("is_system");
            let extension_name: Option<String> = row.get("extension_name");
            let row_count_estimate: i64 = row.get("row_count_estimate");
            let size_bytes_estimate: i64 = row.get("size_bytes_estimate");
            let column_count: i32 = row.get("column_count");

            let table_like = TableLikeEntity {
                id,
                name,
                is_system,
                extension_name,
                schema_id,
                size_bytes_estimate: size_bytes_estimate.max(0) as u64,
                row_count_estimate: row_count_estimate.max(0) as u64,
                column_count: column_count.max(0) as u32,
            };

            let entity = match kind.as_str() {
                "r" => DbEntity::Table(table_like.clone()),
                "v" => DbEntity::View(table_like.clone()),
                "m" => DbEntity::MaterializedView(table_like.clone()),
                "f" => DbEntity::ForeignTable(table_like.clone()),
                _ => continue,
            };

            entities.push(entity);
        }

        // 3. Query for functions and procedures
        let function_query = r#"
            SELECT
                p.oid::TEXT AS id,
                p.proname AS name,
                n.oid::TEXT AS schema_id,
                CASE
                    WHEN n.nspname IN ('pg_catalog', 'information_schema', 'pg_toast')
                         OR n.nspname LIKE 'pg_%'
                    THEN true
                    ELSE false
                END AS is_system,
                e.extname AS extension_name,
                CASE
                    WHEN p.prokind = 'p' THEN 'procedure'
                    ELSE 'function'
                END AS func_type
            FROM pg_proc p
            JOIN pg_namespace n ON n.oid = p.pronamespace
            LEFT JOIN pg_depend d ON d.objid = p.oid AND d.deptype = 'e'
            LEFT JOIN pg_extension e ON e.oid = d.refobjid
            ORDER BY n.nspname, p.proname
        "#;

        let func_rows = sqlx::query(function_query).fetch_all(pool).await?;
        for row in func_rows {
            let id: String = row.get("id");
            let name: String = row.get("name");
            let schema_id: String = row.get("schema_id");
            let is_system: bool = row.get("is_system");
            let extension_name: Option<String> = row.get("extension_name");
            let func_type: String = row.get("func_type");

            let schema_level = SchemaLevelEntity {
                id,
                name,
                is_system,
                extension_name,
                schema_id,
            };

            let entity = match func_type.as_str() {
                "procedure" => DbEntity::Procedure(schema_level),
                _ => DbEntity::Function(schema_level),
            };

            entities.push(entity);
        }

        // 4. Query for sequences
        let sequence_query = r#"
            SELECT
                c.oid::TEXT AS id,
                c.relname AS name,
                n.oid::TEXT AS schema_id,
                CASE
                    WHEN n.nspname IN ('pg_catalog', 'information_schema', 'pg_toast')
                         OR n.nspname LIKE 'pg_%'
                    THEN true
                    ELSE false
                END AS is_system,
                e.extname AS extension_name
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            LEFT JOIN pg_depend d ON d.objid = c.oid AND d.deptype = 'e'
            LEFT JOIN pg_extension e ON e.oid = d.refobjid
            WHERE c.relkind = 'S' -- sequences
            ORDER BY n.nspname, c.relname
        "#;

        let seq_rows = sqlx::query(sequence_query).fetch_all(pool).await?;
        for row in seq_rows {
            let id: String = row.get("id");
            let name: String = row.get("name");
            let schema_id: String = row.get("schema_id");
            let is_system: bool = row.get("is_system");
            let extension_name: Option<String> = row.get("extension_name");

            entities.push(DbEntity::Sequence(SchemaLevelEntity {
                id,
                name,
                is_system,
                extension_name,
                schema_id,
            }));
        }

        // 5. Query for custom types
        let type_query = r#"
            SELECT
                t.oid::TEXT AS id,
                t.typname AS name,
                n.oid::TEXT AS schema_id,
                CASE
                    WHEN n.nspname IN ('pg_catalog', 'information_schema', 'pg_toast')
                         OR n.nspname LIKE 'pg_%'
                    THEN true
                    ELSE false
                END AS is_system,
                e.extname AS extension_name
            FROM pg_type t
            JOIN pg_namespace n ON n.oid = t.typnamespace
            LEFT JOIN pg_depend d ON d.objid = t.oid AND d.deptype = 'e'
            LEFT JOIN pg_extension e ON e.oid = d.refobjid
            WHERE t.typtype IN ('c', 'e', 'd') -- composite, enum, domain types
              AND NOT EXISTS (SELECT 1 FROM pg_class WHERE oid = t.typrelid) -- exclude table types
            ORDER BY n.nspname, t.typname
        "#;

        let type_rows = sqlx::query(type_query).fetch_all(pool).await?;
        for row in type_rows {
            let id: String = row.get("id");
            let name: String = row.get("name");
            let schema_id: String = row.get("schema_id");
            let is_system: bool = row.get("is_system");
            let extension_name: Option<String> = row.get("extension_name");

            entities.push(DbEntity::CustomType(SchemaLevelEntity {
                id,
                name,
                is_system,
                extension_name,
                schema_id,
            }));
        }

        // 6. Query for indexes
        let index_query = r#"
            SELECT
                i.indexrelid::TEXT AS id,
                ic.relname AS name,
                n.oid::TEXT AS schema_id,
                tc.relname AS table_name,
                CASE
                    WHEN n.nspname IN ('pg_catalog', 'information_schema', 'pg_toast')
                         OR n.nspname LIKE 'pg_%'
                    THEN true
                    ELSE false
                END AS is_system,
                pg_total_relation_size(i.indexrelid)::BIGINT AS size_bytes_estimate
            FROM pg_index i
            JOIN pg_class ic ON ic.oid = i.indexrelid
            JOIN pg_class tc ON tc.oid = i.indrelid
            JOIN pg_namespace n ON n.oid = ic.relnamespace
            WHERE ic.relkind = 'i' -- indexes only
            ORDER BY n.nspname, ic.relname
        "#;

        let index_rows = sqlx::query(index_query).fetch_all(pool).await?;
        for row in index_rows {
            let id: String = row.get("id");
            let name: String = row.get("name");
            let schema_id: String = row.get("schema_id");
            let table_name: String = row.get("table_name");
            let is_system: bool = row.get("is_system");
            let size_bytes_estimate: i64 = row.get("size_bytes_estimate");

            entities.push(DbEntity::Index(IndexEntity {
                id,
                name,
                is_system,
                extension_name: None, // Indexes typically don't have direct extension associations
                schema_id,
                table_name,
                size_bytes_estimate: size_bytes_estimate.max(0) as u64,
            }));
        }

        // 7. Query for triggers
        let trigger_query = r#"
            SELECT
                t.oid::TEXT AS id,
                t.tgname AS name,
                n.oid::TEXT AS schema_id,
                c.relname AS table_name,
                CASE
                    WHEN n.nspname IN ('pg_catalog', 'information_schema', 'pg_toast')
                         OR n.nspname LIKE 'pg_%'
                    THEN true
                    ELSE false
                END AS is_system
            FROM pg_trigger t
            JOIN pg_class c ON c.oid = t.tgrelid
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE NOT t.tgisinternal -- exclude internal triggers
            ORDER BY n.nspname, c.relname, t.tgname
        "#;

        let trigger_rows = sqlx::query(trigger_query).fetch_all(pool).await?;
        for row in trigger_rows {
            let id: String = row.get("id");
            let name: String = row.get("name");
            let schema_id: String = row.get("schema_id");
            let table_name: String = row.get("table_name");
            let is_system: bool = row.get("is_system");

            entities.push(DbEntity::Trigger(TriggerEntity {
                id,
                name,
                is_system,
                extension_name: None, // Triggers typically don't have direct extension associations
                schema_id,
                table_name,
            }));
        }

        Ok(entities)
    }
}
