use async_trait::async_trait;
use sqlx::{postgres::{PgPoolOptions, PgConnectOptions}, Pool, Postgres, Row, Column};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use crate::db::client::DatabaseClient;
use crate::db::errors::{DbError, DbResult};
use crate::db::types::{
    ColumnDefinition, ConnectionInfo, QueryResult, SchemaInfo, TableInfo
};

pub struct PostgresClient {
    connection_info: ConnectionInfo,
    pool: Arc<Mutex<Option<Pool<Postgres>>>>,
}

impl PostgresClient {
    pub async fn new(connection_info: ConnectionInfo) -> DbResult<Self> {
        let client = Self {
            connection_info,
            pool: Arc::new(Mutex::new(None)),
        };

        // Connect to the database
        client.ensure_connected().await?;

        Ok(client)
    }

    async fn ensure_connected(&self) -> DbResult<()> {
        let mut pool_guard = self.pool.lock().await;

        if pool_guard.is_none() {
            // Build connection string
            let conn_str = self.connection_info.to_connection_string()
                .map_err(|e| DbError::Connection(e))?;

            // Parse connection options
            let options = conn_str.parse::<PgConnectOptions>()
                .map_err(|e| DbError::Connection(e.to_string()))?;

            // Create connection pool
            let pool = PgPoolOptions::new()
                .max_connections(5)
                .acquire_timeout(Duration::from_secs(5))
                .connect_with(options)
                .await
                .map_err(|e| DbError::Connection(e.to_string()))?;

            *pool_guard = Some(pool);
        }

        Ok(())
    }

    async fn get_pool(&self) -> DbResult<Pool<Postgres>> {
        self.ensure_connected().await?;
        let pool_guard = self.pool.lock().await;

        pool_guard.as_ref()
            .ok_or_else(|| DbError::Connection("Database connection not established".to_string()))
            .map(|p| p.clone())
    }
}

#[async_trait]
impl DatabaseClient for PostgresClient {
    async fn test_connection(&self) -> DbResult<()> {
        let pool = self.get_pool().await?;

        // Execute a simple query to test the connection
        sqlx::query("SELECT 1")
            .execute(&pool)
            .await
            .map_err(|e| DbError::Connection(e.to_string()))?;

        Ok(())
    }

    async fn execute_query(&self, sql: &str) -> DbResult<QueryResult> {
        let pool = self.get_pool().await?;
        let start_time = Instant::now();

        // Execute the query
        let result = sqlx::query(sql)
            .fetch_all(&pool)
            .await
            .map_err(|e| DbError::Query(e.to_string()))?;

        let execution_time = start_time.elapsed().as_millis() as u64;

        // Extract column information
        let column_defs = if !result.is_empty() {
            let row = &result[0];
            let columns = row.columns();

            columns.iter().map(|col| {
                ColumnDefinition {
                    name: col.name().to_string(),
                    data_type: col.type_info().to_string(),
                    nullable: true, // Default to true since we can't easily determine
                    primary_key: false, // Cannot determine from result alone
                    default_value: None, // Cannot determine from result alone
                }
            }).collect()
        } else {
            vec![]
        };

        // Convert rows to our format
        let rows = result.iter().map(|row| {
            let mut values = std::collections::HashMap::new();

            for (i, column) in row.columns().iter().enumerate() {
                let name = column.name().to_string();
                let type_info = column.type_info();
                let value = if type_info.to_string() == "INT4" || type_info.to_string() == "INT8" {
                    if let Ok(val) = row.try_get::<i64, _>(i) {
                        serde_json::Value::Number(serde_json::Number::from(val))
                    } else {
                        serde_json::Value::Null
                    }
                } else if type_info.to_string() == "TEXT" || type_info.to_string() == "VARCHAR" {
                    if let Ok(val) = row.try_get::<String, _>(i) {
                        serde_json::Value::String(val)
                    } else {
                        serde_json::Value::Null
                    }
                } else if type_info.to_string() == "BOOL" {
                    if let Ok(val) = row.try_get::<bool, _>(i) {
                        serde_json::Value::Bool(val)
                    } else {
                        serde_json::Value::Null
                    }
                } else if type_info.to_string() == "JSON" || type_info.to_string() == "JSONB" {
                    if let Ok(val) = row.try_get::<serde_json::Value, _>(i) {
                        val
                    } else {
                        serde_json::Value::Null
                    }
                } else {
                    serde_json::Value::Null
                };

                values.insert(name, value);
            }

            crate::db::types::Row { values }
        }).collect();

        // Create the query result
        Ok(QueryResult {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            query: sql.to_string(),
            rows_affected: None, // Cannot determine for SELECT queries
            execution_time_ms: execution_time,
            columns: column_defs,
            rows,
            warnings: vec![],
            result_index: 0,
        })
    }

    async fn get_tables(&self) -> DbResult<Vec<TableInfo>> {
        let query = r#"
            SELECT
                t.table_schema as schema,
                t.table_name as name,
                pg_catalog.obj_description(
                    pg_catalog.pg_class.oid, 'pg_class'
                ) as comment
            FROM
                information_schema.tables t
            JOIN
                pg_catalog.pg_class ON t.table_name = pg_catalog.pg_class.relname
            WHERE
                t.table_schema NOT IN ('pg_catalog', 'information_schema')
                AND t.table_type = 'BASE TABLE'
            ORDER BY
                t.table_schema, t.table_name
        "#;

        let result = self.execute_query(query).await?;

        let mut tables = Vec::new();
        for row in result.rows {
            let schema = row.values.get("schema")
                .and_then(|v| v.as_str())
                .unwrap_or("public")
                .to_string();

            let name = row.values.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let comment = row.values.get("comment")
                .and_then(|v| v.as_str())
                .map(String::from);

            // Get columns for this table
            let columns_query = format!(
                "SELECT column_name, data_type, is_nullable, column_default
                 FROM information_schema.columns
                 WHERE table_schema = '{}' AND table_name = '{}'
                 ORDER BY ordinal_position",
                schema, name
            );

            let columns_result = self.execute_query(&columns_query).await?;

            // TODO: Get primary key information
            // This is a simplified version - a complete version would get PK constraints

            // Convert to column info objects
            let columns = columns_result.rows.iter().map(|row| {
                let column_name = row.values.get("column_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let data_type = row.values.get("data_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let nullable = row.values.get("is_nullable")
                    .and_then(|v| v.as_str())
                    .map(|s| s == "YES")
                    .unwrap_or(true);

                let default_value = row.values.get("column_default")
                    .and_then(|v| v.as_str())
                    .map(String::from);

                crate::db::types::ColumnInfo {
                    name: column_name,
                    data_type,
                    nullable,
                    primary_key: false, // Simplified
                    default_value,
                    comment: None,
                    position: None,
                }
            }).collect();

            tables.push(TableInfo {
                name,
                schema,
                columns,
                comment,
            });
        }

        Ok(tables)
    }

    async fn get_schema_info(&self) -> DbResult<SchemaInfo> {
        let tables = self.get_tables().await?;

        // TODO: Get views and functions
        // Simplified implementation

        Ok(SchemaInfo {
            name: "public".to_string(), // Default schema
            tables,
            views: Vec::new(),
            functions: Vec::new(),
        })
    }

    fn get_connection_info(&self) -> &ConnectionInfo {
        &self.connection_info
    }
}
