use async_trait::async_trait;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    Column, Pool, Postgres, Row,
};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

use crate::db::clients::common::{DatabaseClient, Transaction};
use crate::db::errors::{DbError, DbResult};
use crate::db::types::{
    ColumnDefinition, ConnectionInfo, DatabaseType, FunctionInfo,
    QueryResult, SchemaInfo, TableInfo, ViewInfo
};
use crate::db::utils::{
    parsing::split_sql_statements,
    serialization::{create_query_result, pg_row_to_row},
};

/// PostgreSQL database client
pub struct PostgresClient {
    /// Connection info
    connection_info: ConnectionInfo,
    /// Connection pool
    pool: Mutex<Option<Pool<Postgres>>>,
}

impl PostgresClient {
    /// Creates a new PostgreSQL client
    pub fn new(connection_info: ConnectionInfo) -> Self {
        Self {
            connection_info,
            pool: Mutex::new(None),
        }
    }

    /// Gets the connection pool
    async fn get_pool(&self) -> DbResult<Pool<Postgres>> {
        let pool = self.pool.lock().await;

        if let Some(ref pool) = *pool {
            Ok(pool.clone())
        } else {
            Err(DbError::Connection("Not connected to database".to_string()))
        }
    }
}

#[async_trait]
impl DatabaseClient for PostgresClient {
    fn db_type(&self) -> DatabaseType {
        DatabaseType::Postgres
    }

    fn connection_info(&self) -> &ConnectionInfo {
        &self.connection_info
    }

    async fn is_connected(&self) -> bool {
        let pool = self.pool.lock().await;
        pool.is_some()
    }

    async fn connect(&self) -> DbResult<()> {
        let mut pool_guard = self.pool.lock().await;

        // Already connected
        if pool_guard.is_some() {
            return Ok(());
        }

        // Get connection options
        let options = if let Some(ref conn_str) = self.connection_info.connection_string {
            PgConnectOptions::from_str(conn_str)
                .map_err(|e| DbError::Connection(e.to_string()))?
        } else {
            // Build from components
            let host = self.connection_info.host.as_deref()
                .ok_or_else(|| DbError::Config("Host is required".to_string()))?;
            let port = self.connection_info.port.unwrap_or(5432);
            let database = self.connection_info.database.as_deref()
                .ok_or_else(|| DbError::Config("Database name is required".to_string()))?;
            let username = self.connection_info.username.as_deref()
                .ok_or_else(|| DbError::Config("Username is required".to_string()))?;
            let password = self.connection_info.password.as_deref().unwrap_or("");

            let mut options = PgConnectOptions::new()
                .host(host)
                .port(port)
                .database(database)
                .username(username)
                .password(password);

            // Add SSL options if configured
            if let Some(ref ssl_config) = self.connection_info.ssl_config {
                if ssl_config.enabled {
                    options = options.ssl_mode(sqlx::postgres::PgSslMode::Require);

                    // TODO: Add more SSL options when needed
                }
            }

            options
        };

        // Connect to the database
        let pool = PgPoolOptions::new()
            .max_connections(10) // Reasonable default
            .connect_with(options)
            .await
            .map_err(|e| DbError::Connection(e.to_string()))?;

        *pool_guard = Some(pool);
        Ok(())
    }

    async fn disconnect(&self) -> DbResult<()> {
        let mut pool_guard = self.pool.lock().await;

        if let Some(pool) = pool_guard.take() {
            pool.close().await;
        }

        Ok(())
    }

    async fn execute_query(&self, sql: &str) -> DbResult<QueryResult> {
        let pool = self.get_pool().await?;
        let start = Instant::now();

        let result = if sql.trim_start().to_uppercase().starts_with("SELECT") {
            // For SELECT queries, we want to return rows
            let rows = sqlx::query(sql)
                .fetch_all(&pool)
                .await
                .map_err(|e| DbError::Query(e.to_string()))?;

            // If we have rows, extract column information from the first row
            let columns = if !rows.is_empty() {
                let first_row = &rows[0];
                (0..first_row.len())
                    .map(|i| {
                        let column = first_row.column(i);
                        ColumnDefinition {
                            name: column.name().to_string(),
                            data_type: column.type_info().to_string(),
                            nullable: false, // We don't know this from the query result, setting to false for now
                            primary_key: false, // We don't know this from the query result
                            default_value: None, // We don't know this from the query result
                        }
                    })
                    .collect()
            } else {
                // If no rows, we don't have column info
                Vec::new()
            };

            // Convert rows to our format
            let result_rows = rows
                .iter()
                .map(|row| pg_row_to_row(row, &columns))
                .collect::<Result<Vec<_>, _>>()?;

            create_query_result(
                sql.to_string(),
                columns,
                result_rows,
                None,
                start.elapsed().as_millis() as u64,
                0,
            )
        } else {
            // For non-SELECT queries, execute and get rows affected
            let result = sqlx::query(sql)
                .execute(&pool)
                .await
                .map_err(|e| DbError::Query(e.to_string()))?;

            create_query_result(
                sql.to_string(),
                Vec::new(),
                Vec::new(),
                Some(result.rows_affected()),
                start.elapsed().as_millis() as u64,
                0,
            )
        };

        Ok(result)
    }

    async fn execute_queries(&self, sql: &str) -> DbResult<Vec<QueryResult>> {
        let statements = split_sql_statements(sql)?;
        let mut results = Vec::with_capacity(statements.len());

        for statement in statements {
            results.push(self.execute_query(&statement).await?);
        }

        Ok(results)
    }

    async fn test_connection(&self) -> DbResult<()> {
        let pool = self.get_pool().await?;

        sqlx::query("SELECT 1")
            .execute(&pool)
            .await
            .map_err(|e| DbError::Connection(e.to_string()))?;

        Ok(())
    }

    async fn get_schema_info(&self) -> DbResult<SchemaInfo> {
        let pool = self.get_pool().await?;

        // Get list of schemas
        let schema_query = r#"
            SELECT
                schema_name AS name
            FROM
                information_schema.schemata
            WHERE
                schema_name NOT IN ('information_schema', 'pg_catalog', 'pg_toast')
                AND schema_name NOT LIKE 'pg_%'
            ORDER BY
                schema_name
        "#;

        let schemas = sqlx::query(schema_query)
            .fetch_all(&pool)
            .await
            .map_err(|e| DbError::Query(e.to_string()))?;

        // Default to public schema if none found
        if schemas.is_empty() {
            let tables = self.get_tables().await?;
            let views = self.get_views().await?;
            let functions = self.get_functions().await?;

            return Ok(SchemaInfo {
                name: "public".to_string(),
                tables,
                views,
                functions,
            });
        }

        // Just use the first schema for now
        // In the future, we might want to return multiple schemas
        let schema_name: String = schemas[0].get("name");

        let tables = self.get_tables().await?;
        let views = self.get_views().await?;
        let functions = self.get_functions().await?;

        Ok(SchemaInfo {
            name: schema_name,
            tables,
            views,
            functions,
        })
    }

    async fn get_tables(&self) -> DbResult<Vec<TableInfo>> {
        let pool = self.get_pool().await?;

        let query = r#"
            SELECT
                t.table_name AS name,
                t.table_schema AS schema,
                obj_description(format('%s.%s', t.table_schema, t.table_name)::regclass::oid) AS comment
            FROM
                information_schema.tables t
            WHERE
                t.table_type = 'BASE TABLE'
                AND t.table_schema NOT IN ('information_schema', 'pg_catalog')
                AND t.table_schema NOT LIKE 'pg_%'
            ORDER BY
                t.table_schema, t.table_name
        "#;

        let rows = sqlx::query(query)
            .fetch_all(&pool)
            .await
            .map_err(|e| DbError::Query(e.to_string()))?;

        let mut tables = Vec::with_capacity(rows.len());

        for row in rows {
            let name: String = row.get("name");
            let schema: String = row.get("schema");
            let comment: Option<String> = row.get("comment");

            // We'll populate columns later when needed
            tables.push(TableInfo {
                name,
                schema,
                columns: vec![],
                comment,
            });
        }

        Ok(tables)
    }

    async fn get_views(&self) -> DbResult<Vec<ViewInfo>> {
        let pool = self.get_pool().await?;

        let query = r#"
            SELECT
                v.table_name AS name,
                v.table_schema AS schema,
                v.view_definition AS definition
            FROM
                information_schema.views v
            WHERE
                v.table_schema NOT IN ('information_schema', 'pg_catalog')
                AND v.table_schema NOT LIKE 'pg_%'
            ORDER BY
                v.table_schema, v.table_name
        "#;

        let rows = sqlx::query(query)
            .fetch_all(&pool)
            .await
            .map_err(|e| DbError::Query(e.to_string()))?;

        let mut views = Vec::with_capacity(rows.len());

        for row in rows {
            let name: String = row.get("name");
            let schema: String = row.get("schema");
            let definition: Option<String> = row.get("definition");

            // We'll populate columns later when needed
            views.push(ViewInfo {
                name,
                schema,
                columns: vec![],
                definition,
            });
        }

        Ok(views)
    }

    async fn get_functions(&self) -> DbResult<Vec<FunctionInfo>> {
        let pool = self.get_pool().await?;

        let query = r#"
            SELECT
                p.proname AS name,
                n.nspname AS schema,
                pg_get_function_result(p.oid) AS return_type,
                pg_get_function_arguments(p.oid) AS arguments,
                pg_get_functiondef(p.oid) AS definition
            FROM
                pg_proc p
                INNER JOIN pg_namespace n ON p.pronamespace = n.oid
            WHERE
                n.nspname NOT IN ('information_schema', 'pg_catalog')
                AND n.nspname NOT LIKE 'pg_%'
                AND p.prokind = 'f'
            ORDER BY
                n.nspname, p.proname
        "#;

        let rows = sqlx::query(query)
            .fetch_all(&pool)
            .await
            .map_err(|e| DbError::Query(e.to_string()))?;

        let mut functions = Vec::with_capacity(rows.len());

        for row in rows {
            let name: String = row.get("name");
            let schema: String = row.get("schema");
            let return_type: Option<String> = row.get("return_type");
            let arguments_str: Option<String> = row.get("arguments");
            let definition: Option<String> = row.get("definition");

            // Parse arguments
            let arguments = arguments_str.map_or_else(Vec::new, |args| {
                args.split(',')
                    .map(|arg| arg.trim().to_string())
                    .collect()
            });

            functions.push(FunctionInfo {
                name,
                schema,
                arguments,
                return_type,
                definition,
            });
        }

        Ok(functions)
    }

    async fn get_table_info(&self, table_name: &str, schema: Option<&str>) -> DbResult<TableInfo> {
        let pool = self.get_pool().await?;
        let schema = schema.unwrap_or("public");

        // Check if table exists
        let exists: Option<i32> = sqlx::query_scalar(
            "SELECT 1 FROM information_schema.tables
             WHERE table_schema = $1 AND table_name = $2
             AND table_type = 'BASE TABLE'"
        )
        .bind(schema)
        .bind(table_name)
        .fetch_optional(&pool)
        .await
        .map_err(|e| DbError::Query(e.to_string()))?;

        if exists.is_none() {
            return Err(DbError::NotFound(format!("Table '{}.{}' not found", schema, table_name)));
        }

        // Get table comment
        let comment: Option<String> = sqlx::query_scalar(
            "SELECT obj_description(format('%s.%s', $1, $2)::regclass::oid)"
        )
        .bind(schema)
        .bind(table_name)
        .fetch_optional(&pool)
        .await
        .map_err(|e| DbError::Query(e.to_string()))?;

        // Get column information
        let columns = self.get_table_columns(table_name, Some(schema)).await?;

        Ok(TableInfo {
            name: table_name.to_string(),
            schema: schema.to_string(),
            columns,
            comment,
        })
    }

    async fn begin_transaction(&self) -> DbResult<Arc<dyn Transaction>> {
        let pool = self.get_pool().await?;

        let tx = pool.begin()
            .await
            .map_err(|e| DbError::Transaction(e.to_string()))?;

        Ok(Arc::new(PostgresTransaction { tx: Mutex::new(Some(tx)) }))
    }
}

// Helper methods for PostgresClient
impl PostgresClient {
    /// Gets columns for a table or view
    async fn get_table_columns(&self, table_name: &str, schema: Option<&str>) -> DbResult<Vec<crate::db::types::ColumnInfo>> {
        let pool = self.get_pool().await?;
        let schema = schema.unwrap_or("public");

        // Query to get columns
        let rows = sqlx::query(r#"
            SELECT
                c.column_name,
                c.ordinal_position,
                c.data_type,
                c.character_maximum_length,
                c.is_nullable = 'YES' as is_nullable,
                c.column_default,
                col_description(pgc.oid, c.ordinal_position) as comment,
                (
                    SELECT COUNT(*)
                    FROM information_schema.table_constraints tc
                    JOIN information_schema.key_column_usage kcu
                    ON tc.constraint_name = kcu.constraint_name
                    AND tc.table_schema = kcu.table_schema
                    AND tc.table_name = kcu.table_name
                    WHERE tc.constraint_type = 'PRIMARY KEY'
                    AND tc.table_schema = c.table_schema
                    AND tc.table_name = c.table_name
                    AND kcu.column_name = c.column_name
                ) > 0 as is_primary_key
            FROM
                information_schema.columns c
                JOIN pg_class pgc ON c.table_name = pgc.relname
                JOIN pg_namespace pgn ON pgc.relnamespace = pgn.oid AND c.table_schema = pgn.nspname
            WHERE
                c.table_schema = $1
                AND c.table_name = $2
            ORDER BY
                c.ordinal_position
        "#)
        .bind(schema)
        .bind(table_name)
        .fetch_all(&pool)
        .await
        .map_err(|e| DbError::Query(e.to_string()))?;

        let mut columns = Vec::with_capacity(rows.len());

        for row in rows {
            let column_name: String = row.get("column_name");
            let position: i32 = row.get("ordinal_position");
            let data_type: String = row.get("data_type");
            let is_nullable: bool = row.get("is_nullable");
            let column_default: Option<String> = row.get("column_default");
            let comment: Option<String> = row.get("comment");
            let is_primary_key: bool = row.get("is_primary_key");

            columns.push(crate::db::types::ColumnInfo {
                name: column_name,
                data_type,
                nullable: is_nullable,
                primary_key: is_primary_key,
                default_value: column_default,
                comment,
                position: Some(position as u32),
            });
        }

        Ok(columns)
    }
}

/// PostgreSQL transaction
pub struct PostgresTransaction {
    tx: Mutex<Option<sqlx::Transaction<'static, Postgres>>>,
}

#[async_trait]
impl Transaction for PostgresTransaction {
    async fn execute_query(&self, sql: &str) -> DbResult<QueryResult> {
        let mut tx_guard = self.tx.lock().await;
        let tx = tx_guard.as_mut()
            .ok_or_else(|| DbError::Transaction("Transaction already committed or rolled back".to_string()))?;

        let start = Instant::now();

        let result = if sql.trim_start().to_uppercase().starts_with("SELECT") {
            // For SELECT queries, we want to return rows
            let rows = sqlx::query(sql)
                .fetch_all(&mut **tx)
                .await
                .map_err(|e| DbError::Query(e.to_string()))?;

            // If we have rows, extract column information from the first row
            let columns = if !rows.is_empty() {
                let first_row = &rows[0];
                (0..first_row.len())
                    .map(|i| {
                        let column = first_row.column(i);
                        ColumnDefinition {
                            name: column.name().to_string(),
                            data_type: column.type_info().to_string(),
                            nullable: false, // We don't know this from the query result, setting to false for now
                            primary_key: false, // We don't know this from the query result
                            default_value: None, // We don't know this from the query result
                        }
                    })
                    .collect()
            } else {
                // If no rows, we don't have column info
                Vec::new()
            };

            // Convert rows to our format
            let result_rows = rows
                .iter()
                .map(|row| pg_row_to_row(row, &columns))
                .collect::<Result<Vec<_>, _>>()?;

            create_query_result(
                sql.to_string(),
                columns,
                result_rows,
                None,
                start.elapsed().as_millis() as u64,
                0,
            )
        } else {
            // For non-SELECT queries, execute and get rows affected
            let result = sqlx::query(sql)
                .execute(&mut **tx)
                .await
                .map_err(|e| DbError::Query(e.to_string()))?;

            create_query_result(
                sql.to_string(),
                Vec::new(),
                Vec::new(),
                Some(result.rows_affected()),
                start.elapsed().as_millis() as u64,
                0,
            )
        };

        Ok(result)
    }

    async fn commit(&self) -> DbResult<()> {
        let mut tx_guard = self.tx.lock().await;
        let tx = tx_guard.take()
            .ok_or_else(|| DbError::Transaction("Transaction already committed or rolled back".to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DbError::Transaction(e.to_string()))?;

        Ok(())
    }

    async fn rollback(&self) -> DbResult<()> {
        let mut tx_guard = self.tx.lock().await;
        let tx = tx_guard.take()
            .ok_or_else(|| DbError::Transaction("Transaction already committed or rolled back".to_string()))?;

        tx.rollback()
            .await
            .map_err(|e| DbError::Transaction(e.to_string()))?;

        Ok(())
    }
}
