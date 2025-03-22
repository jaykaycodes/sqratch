use async_trait::async_trait;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions, PgRow},
    Pool, Postgres, Row,
};
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
    serialization::{create_query_result, sqlx_row_to_row},
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

        // Create the connection pool
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .map_err(|e| DbError::Connection(e.to_string()))?;

        // Store the pool
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
                            data_type: column.type_info().name().to_string(),
                            nullable: column.type_info().is_nullable(),
                            primary_key: false, // We don't know this from the query result
                            default_value: None, // We don't know this from the query result
                        }
                    })
                    .collect()
            } else {
                // If no rows, try to get column info from the query
                let statement = sqlx::query(sql)
                    .describe(&pool)
                    .await
                    .map_err(|e| DbError::Query(e.to_string()))?;

                statement.columns()
                    .iter()
                    .map(|column| {
                        ColumnDefinition {
                            name: column.name().to_string(),
                            data_type: column.type_info().name().to_string(),
                            nullable: column.type_info().is_nullable(),
                            primary_key: false,
                            default_value: None,
                        }
                    })
                    .collect()
            };

            // Convert rows to our format
            let result_rows = rows
                .iter()
                .map(|row| sqlx_row_to_row(row, &columns))
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
        let statements = split_sql_statements(sql);
        let mut results = Vec::with_capacity(statements.len());

        for (i, statement) in statements.iter().enumerate() {
            if statement.trim().is_empty() {
                continue;
            }

            let mut result = self.execute_query(statement).await?;
            result.result_index = i;
            results.push(result);
        }

        Ok(results)
    }

    async fn test_connection(&self) -> DbResult<()> {
        // Create a temporary pool for testing
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

            PgConnectOptions::new()
                .host(host)
                .port(port)
                .database(database)
                .username(username)
                .password(password)
        };

        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .map_err(|e| DbError::Connection(e.to_string()))?;

        // Execute a simple query to verify the connection
        sqlx::query("SELECT 1")
            .execute(&pool)
            .await
            .map_err(|e| DbError::Query(e.to_string()))?;

        // Close the pool
        pool.close().await;

        Ok(())
    }

    async fn get_schema_info(&self) -> DbResult<SchemaInfo> {
        let pool = self.get_pool().await?;

        // Get database name
        let database_row = sqlx::query("SELECT current_database()")
            .fetch_one(&pool)
            .await
            .map_err(|e| DbError::Query(e.to_string()))?;

        let database = database_row.get::<String, _>(0);

        // Get tables
        let tables = self.get_tables().await?;

        // Get views
        let views = self.get_views().await?;

        // Get functions
        let functions = self.get_functions().await?;

        Ok(SchemaInfo {
            database,
            schema: Some("public".to_string()), // Default to public schema
            tables,
            views,
            functions,
        })
    }

    async fn get_tables(&self) -> DbResult<Vec<TableInfo>> {
        let pool = self.get_pool().await?;

        // Query to get tables
        let rows = sqlx::query(r#"
            SELECT
                t.table_name,
                t.table_schema,
                obj_description(pgc.oid, 'pg_class') as comment,
                (SELECT COUNT(*) FROM information_schema.columns
                 WHERE table_name = t.table_name AND table_schema = t.table_schema) as column_count,
                pg_total_relation_size(pgc.oid) as size_bytes,
                (SELECT reltuples::bigint FROM pg_class WHERE oid = pgc.oid) as row_count
            FROM information_schema.tables t
            JOIN pg_catalog.pg_class pgc ON pgc.relname = t.table_name
            JOIN pg_catalog.pg_namespace n ON pgc.relnamespace = n.oid AND n.nspname = t.table_schema
            WHERE t.table_type = 'BASE TABLE'
            AND t.table_schema NOT IN ('pg_catalog', 'information_schema')
            ORDER BY t.table_schema, t.table_name
        "#)
            .fetch_all(&pool)
            .await
            .map_err(|e| DbError::Query(e.to_string()))?;

        let mut tables = Vec::with_capacity(rows.len());

        for row in rows {
            let name: String = row.get("table_name");
            let schema: String = row.get("table_schema");
            let comment: Option<String> = row.get("comment");
            let size_bytes: Option<i64> = row.get("size_bytes");
            let row_count: Option<i64> = row.get("row_count");

            // Get columns for this table
            let columns = self.get_table_columns(&name, Some(&schema)).await?;

            tables.push(TableInfo {
                name,
                schema: Some(schema),
                columns,
                row_count: row_count.map(|c| c as u64),
                size_bytes: size_bytes.map(|s| s as u64),
                comment,
            });
        }

        Ok(tables)
    }

    async fn get_views(&self) -> DbResult<Vec<ViewInfo>> {
        let pool = self.get_pool().await?;

        // Query to get views
        let rows = sqlx::query(r#"
            SELECT
                v.table_name,
                v.table_schema,
                v.view_definition,
                obj_description(pgc.oid, 'pg_class') as comment
            FROM information_schema.views v
            JOIN pg_catalog.pg_class pgc ON pgc.relname = v.table_name
            JOIN pg_catalog.pg_namespace n ON pgc.relnamespace = n.oid AND n.nspname = v.table_schema
            WHERE v.table_schema NOT IN ('pg_catalog', 'information_schema')
            ORDER BY v.table_schema, v.table_name
        "#)
            .fetch_all(&pool)
            .await
            .map_err(|e| DbError::Query(e.to_string()))?;

        let mut views = Vec::with_capacity(rows.len());

        for row in rows {
            let name: String = row.get("table_name");
            let schema: String = row.get("table_schema");
            let definition: Option<String> = row.get("view_definition");
            let comment: Option<String> = row.get("comment");

            // Get columns for this view
            let columns = self.get_table_columns(&name, Some(&schema)).await?;

            views.push(ViewInfo {
                name,
                schema: Some(schema),
                definition,
                columns,
                comment,
            });
        }

        Ok(views)
    }

    async fn get_functions(&self) -> DbResult<Vec<FunctionInfo>> {
        let pool = self.get_pool().await?;

        // Query to get functions
        let rows = sqlx::query(r#"
            SELECT
                p.proname as name,
                n.nspname as schema,
                pg_get_functiondef(p.oid) as definition,
                l.lanname as language,
                pg_get_function_result(p.oid) as return_type,
                obj_description(p.oid, 'pg_proc') as comment
            FROM pg_proc p
            JOIN pg_namespace n ON p.pronamespace = n.oid
            JOIN pg_language l ON p.prolang = l.oid
            WHERE n.nspname NOT IN ('pg_catalog', 'information_schema')
            ORDER BY n.nspname, p.proname
        "#)
            .fetch_all(&pool)
            .await
            .map_err(|e| DbError::Query(e.to_string()))?;

        let mut functions = Vec::with_capacity(rows.len());

        for row in rows {
            let name: String = row.get("name");
            let schema: String = row.get("schema");
            let definition: Option<String> = row.get("definition");
            let language: Option<String> = row.get("language");
            let return_type: Option<String> = row.get("return_type");
            let comment: Option<String> = row.get("comment");

            // TODO: Add function arguments

            functions.push(FunctionInfo {
                name,
                schema: Some(schema),
                language,
                definition,
                arguments: Vec::new(),
                return_type,
                comment,
            });
        }

        Ok(functions)
    }

    async fn get_table_info(&self, table_name: &str, schema: Option<&str>) -> DbResult<TableInfo> {
        let pool = self.get_pool().await?;
        let schema = schema.unwrap_or("public");

        // Query to get table info
        let row = sqlx::query(r#"
            SELECT
                t.table_name,
                t.table_schema,
                obj_description(pgc.oid, 'pg_class') as comment,
                pg_total_relation_size(pgc.oid) as size_bytes,
                (SELECT reltuples::bigint FROM pg_class WHERE oid = pgc.oid) as row_count
            FROM information_schema.tables t
            JOIN pg_catalog.pg_class pgc ON pgc.relname = t.table_name
            JOIN pg_catalog.pg_namespace n ON pgc.relnamespace = n.oid AND n.nspname = t.table_schema
            WHERE t.table_type = 'BASE TABLE'
            AND t.table_schema = $1
            AND t.table_name = $2
        "#)
            .bind(schema)
            .bind(table_name)
            .fetch_one(&pool)
            .await
            .map_err(|e| DbError::Query(e.to_string()))?;

        let name: String = row.get("table_name");
        let schema: String = row.get("table_schema");
        let comment: Option<String> = row.get("comment");
        let size_bytes: Option<i64> = row.get("size_bytes");
        let row_count: Option<i64> = row.get("row_count");

        // Get columns for this table
        let columns = self.get_table_columns(&name, Some(&schema)).await?;

        Ok(TableInfo {
            name,
            schema: Some(schema),
            columns,
            row_count: row_count.map(|c| c as u64),
            size_bytes: size_bytes.map(|s| s as u64),
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
            FROM information_schema.columns c
            JOIN pg_catalog.pg_class pgc ON pgc.relname = c.table_name
            JOIN pg_catalog.pg_namespace n ON pgc.relnamespace = n.oid AND n.nspname = c.table_schema
            WHERE c.table_schema = $1
            AND c.table_name = $2
            ORDER BY c.ordinal_position
        "#)
            .bind(schema)
            .bind(table_name)
            .fetch_all(&pool)
            .await
            .map_err(|e| DbError::Query(e.to_string()))?;

        let mut columns = Vec::with_capacity(rows.len());

        for row in rows {
            let name: String = row.get("column_name");
            let position: i32 = row.get("ordinal_position");
            let data_type: String = row.get("data_type");
            let char_max_length: Option<i32> = row.get("character_maximum_length");
            let nullable: bool = row.get("is_nullable");
            let default_value: Option<String> = row.get("column_default");
            let comment: Option<String> = row.get("comment");
            let is_primary_key: bool = row.get("is_primary_key");

            // TODO: Add foreign key reference

            columns.push(crate::db::types::ColumnInfo {
                name,
                position,
                data_type,
                char_max_length,
                nullable,
                default_value,
                comment,
                is_primary_key,
                foreign_key_ref: None,
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
                            data_type: column.type_info().name().to_string(),
                            nullable: column.type_info().is_nullable(),
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
                .map(|row| sqlx_row_to_row(row, &columns))
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
