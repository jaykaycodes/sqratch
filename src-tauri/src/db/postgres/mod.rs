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
        ColumnDefinition, ColumnInfo, ColumnTypeCategory, ConstraintInfo, ConstraintType,
        EntityInfo, EntityType, ForeignKeyReference, FunctionInfo, IndexInfo, QueryResult, Row,
        TableInfo, ViewInfo,
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

    async fn get_constraints(&self, schema: &str, table: &str) -> DbResult<Vec<ConstraintInfo>> {
        let pool = self.get_pool()?;

        let sql = r#"
            SELECT
                c.conname as name,
                c.contype as type,
                pg_get_constraintdef(c.oid) as definition,
                array_agg(a.attname) as columns,
                CASE c.contype
                    WHEN 'f' THEN (
                        SELECT json_build_object(
                            'referenced_schema', nf.nspname,
                            'referenced_table', tf.relname,
                            'referenced_column', af.attname,
                            'on_update', rc.update_rule,
                            'on_delete', rc.delete_rule
                        )
                        FROM pg_class tf
                        JOIN pg_namespace nf ON tf.relnamespace = nf.oid
                        JOIN pg_attribute af ON af.attrelid = tf.oid
                        LEFT JOIN information_schema.referential_constraints rc
                            ON rc.constraint_name = c.conname
                        WHERE tf.oid = c.confrelid
                        AND af.attnum = ANY(c.confkey)
                        LIMIT 1
                    )
                    ELSE NULL
                END as fk_reference
            FROM pg_constraint c
            JOIN pg_namespace n ON n.oid = c.connamespace
            JOIN pg_class t ON t.oid = c.conrelid
            JOIN pg_attribute a ON a.attrelid = t.oid AND a.attnum = ANY(c.conkey)
            WHERE n.nspname = $1 AND t.relname = $2
            GROUP BY c.oid, c.conname, c.contype
        "#;

        let rows = sqlx::query(sql)
            .bind(schema)
            .bind(table)
            .fetch_all(pool)
            .await?;

        let mut constraints = Vec::new();
        for row in rows {
            let name: String = row.get("name");
            let type_char: String = row.get("type");
            let definition: String = row.get("definition");
            let columns: Vec<String> = row.get("columns");
            let fk_reference: Option<Value> = row.get("fk_reference");

            let constraint_type = match type_char.as_str() {
                "p" => ConstraintType::PrimaryKey,
                "f" => ConstraintType::ForeignKey,
                "u" => ConstraintType::Unique,
                "c" => ConstraintType::Check,
                "x" => ConstraintType::Exclusion,
                _ => continue, // Skip unknown constraint types
            };

            let foreign_key_reference = if let Some(Value::Object(obj)) = fk_reference {
                Some(ForeignKeyReference {
                    referenced_schema: obj["referenced_schema"].as_str().unwrap_or("").to_string(),
                    referenced_table: obj["referenced_table"].as_str().unwrap_or("").to_string(),
                    referenced_column: obj["referenced_column"].as_str().unwrap_or("").to_string(),
                    on_update: obj["on_update"].as_str().map(|s| s.to_string()),
                    on_delete: obj["on_delete"].as_str().map(|s| s.to_string()),
                })
            } else {
                None
            };

            let check_definition = if constraint_type == ConstraintType::Check {
                Some(definition)
            } else {
                None
            };

            constraints.push(ConstraintInfo {
                name,
                constraint_type,
                table_name: table.to_string(),
                schema_name: schema.to_string(),
                column_names: columns,
                foreign_key_reference,
                check_definition,
            });
        }

        Ok(constraints)
    }

    async fn get_indices(&self, schema: &str, table: &str) -> DbResult<Vec<IndexInfo>> {
        let pool = self.get_pool()?;

        let sql = r#"
            SELECT
                i.relname as name,
                am.amname as method,
                ix.indisunique as is_unique,
                ix.indisprimary as is_primary,
                array_agg(a.attname) as column_names
            FROM pg_index ix
            JOIN pg_class i ON i.oid = ix.indexrelid
            JOIN pg_class t ON t.oid = ix.indrelid
            JOIN pg_am am ON am.oid = i.relam
            JOIN pg_attribute a ON a.attrelid = t.oid AND a.attnum = ANY(ix.indkey)
            JOIN pg_namespace n ON n.oid = t.relnamespace
            WHERE n.nspname = $1 AND t.relname = $2
            GROUP BY i.relname, am.amname, ix.indisunique, ix.indisprimary
        "#;

        let rows = sqlx::query(sql)
            .bind(schema)
            .bind(table)
            .fetch_all(pool)
            .await?;

        let mut indices = Vec::new();
        for row in rows {
            let name: String = row.get("name");
            let method: String = row.get("method");
            let is_unique: bool = row.get("is_unique");
            let is_primary: bool = row.get("is_primary");
            let column_names: Vec<String> = row.get("column_names");

            indices.push(IndexInfo {
                name,
                schema: schema.to_string(),
                table: table.to_string(),
                is_unique,
                is_primary,
                column_names,
                method: Some(method),
            });
        }

        Ok(indices)
    }

    async fn get_tables_info(&self) -> DbResult<Vec<TableInfo>> {
        let pool = self.get_pool()?;

        let sql = r#"
            SELECT
                n.nspname as schema_name,
                c.relname as table_name,
                a.attname as column_name,
                t.typname as data_type,
                a.attnotnull as not_null,
                a.attnum as ordinal_position,
                pg_get_expr(d.adbin, d.adrelid) as default_value,
                col_description(c.oid, a.attnum) as description,
                format_type(a.atttypid, a.atttypmod) as full_data_type,
                EXISTS (
                    SELECT 1 FROM pg_index i
                    WHERE i.indrelid = c.oid
                    AND i.indisprimary
                    AND a.attnum = ANY(i.indkey)
                ) as is_primary,
                EXISTS (
                    SELECT 1 FROM pg_index i
                    WHERE i.indrelid = c.oid
                    AND a.attnum = ANY(i.indkey)
                ) as is_indexed,
                EXISTS (
                    SELECT 1 FROM pg_index i
                    WHERE i.indrelid = c.oid
                    AND i.indisunique
                    AND a.attnum = ANY(i.indkey)
                ) as is_unique,
                pg_get_serial_sequence(c.relname::text, a.attname::text) IS NOT NULL as is_serial
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            JOIN pg_attribute a ON a.attrelid = c.oid
            JOIN pg_type t ON t.oid = a.atttypid
            LEFT JOIN pg_attrdef d ON d.adrelid = c.oid AND d.adnum = a.attnum
            WHERE c.relkind = 'r'
            AND n.nspname NOT IN ('pg_catalog', 'information_schema')
            AND a.attnum > 0
            AND NOT a.attisdropped
            ORDER BY n.nspname, c.relname, a.attnum
        "#;

        let rows = sqlx::query(sql).fetch_all(pool).await?;

        let mut tables = Vec::new();
        let mut current_schema = String::new();
        let mut current_table = String::new();
        let mut current_columns: Vec<ColumnInfo> = Vec::new();

        for row in rows {
            let schema_name: String = row.get("schema_name");
            let table_name: String = row.get("table_name");

            if current_schema != schema_name || current_table != table_name {
                if !current_table.is_empty() {
                    let constraints = self
                        .get_constraints(&current_schema, &current_table)
                        .await?;
                    let indices = self.get_indices(&current_schema, &current_table).await?;

                    let primary_key_columns = current_columns
                        .iter()
                        .filter(|c| c.primary_key)
                        .map(|c| c.name.clone())
                        .collect();

                    tables.push(TableInfo {
                        name: current_table,
                        schema: current_schema,
                        columns: current_columns,
                        constraints,
                        indices,
                        primary_key_columns,
                        row_count_estimate: None,
                        size_bytes: None,
                        comment: None,
                        last_modified: None,
                    });
                }
                current_schema = schema_name;
                current_table = table_name;
                current_columns = Vec::new();
            }

            let column_name: String = row.get("column_name");
            let data_type: String = row.get("data_type");
            let not_null: bool = row.get("not_null");
            let ordinal_position: i32 = row.get("ordinal_position");
            let default_value: Option<String> = row.get("default_value");
            let description: Option<String> = row.get("description");
            let full_data_type: String = row.get("full_data_type");
            let is_primary: bool = row.get("is_primary");
            let is_indexed: bool = row.get("is_indexed");
            let is_unique: bool = row.get("is_unique");
            let is_serial: bool = row.get("is_serial");

            current_columns.push(ColumnInfo {
                name: column_name,
                data_type: full_data_type.clone(),
                type_category: Self::map_pg_type_to_category(&data_type),
                nullable: !not_null,
                primary_key: is_primary,
                auto_increment: is_serial,
                indexed: is_indexed,
                unique: is_unique,
                char_max_length: None,
                numeric_precision: None,
                numeric_scale: None,
                default_value,
                comment: description,
                position: Some(ordinal_position as u32),
                foreign_key: None,
                display_hint: None,
            });
        }

        // Add the last table
        if !current_table.is_empty() {
            let constraints = self
                .get_constraints(&current_schema, &current_table)
                .await?;
            let indices = self.get_indices(&current_schema, &current_table).await?;

            let primary_key_columns = current_columns
                .iter()
                .filter(|c| c.primary_key)
                .map(|c| c.name.clone())
                .collect();

            tables.push(TableInfo {
                name: current_table,
                schema: current_schema,
                columns: current_columns,
                constraints,
                indices,
                primary_key_columns,
                row_count_estimate: None,
                size_bytes: None,
                comment: None,
                last_modified: None,
            });
        }

        Ok(tables)
    }

    async fn get_views_info(&self) -> DbResult<Vec<ViewInfo>> {
        let pool = self.get_pool()?;

        let sql = r#"
            SELECT
                c.relname as name,
                n.nspname as schema,
                pg_get_viewdef(c.oid) as definition,
                obj_description(c.oid, 'pg_class') as description,
                c.relkind as view_type,
                c.reltuples as row_estimate
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE c.relkind IN ('v', 'm')
            AND n.nspname NOT IN ('pg_catalog', 'information_schema')
        "#;

        let rows = sqlx::query(sql).fetch_all(pool).await?;

        let mut views = Vec::new();
        for row in rows {
            let name: String = row.get("name");
            let schema: String = row.get("schema");
            let definition: String = row.get("definition");
            let description: Option<String> = row.get("description");
            let view_type: String = row.get("view_type");
            let row_estimate: f64 = row.get("row_estimate");

            // Get view columns
            let columns_sql = r#"
                SELECT
                    a.attname as column_name,
                    t.typname as data_type,
                    a.attnotnull as not_null,
                    format_type(a.atttypid, a.atttypmod) as full_data_type,
                    col_description(a.attrelid, a.attnum) as description,
                    a.attnum as ordinal_position
                FROM pg_attribute a
                JOIN pg_class c ON a.attrelid = c.oid
                JOIN pg_namespace n ON c.relnamespace = n.oid
                JOIN pg_type t ON a.atttypid = t.oid
                WHERE n.nspname = $1
                AND c.relname = $2
                AND a.attnum > 0
                AND NOT a.attisdropped
                ORDER BY a.attnum
            "#;

            let column_rows = sqlx::query(columns_sql)
                .bind(&schema)
                .bind(&name)
                .fetch_all(pool)
                .await?;

            let mut columns = Vec::new();
            for col_row in column_rows {
                let column_name: String = col_row.get("column_name");
                let data_type: String = col_row.get("data_type");
                let not_null: bool = col_row.get("not_null");
                let full_data_type: String = col_row.get("full_data_type");
                let column_description: Option<String> = col_row.get("description");
                let position: i32 = col_row.get("ordinal_position");

                columns.push(ColumnInfo {
                    name: column_name,
                    data_type: full_data_type,
                    type_category: Self::map_pg_type_to_category(&data_type),
                    nullable: !not_null,
                    primary_key: false, // Views don't have primary keys
                    auto_increment: false,
                    indexed: false,
                    unique: false,
                    char_max_length: None,
                    numeric_precision: None,
                    numeric_scale: None,
                    default_value: None,
                    comment: column_description,
                    position: Some(position as u32),
                    foreign_key: None,
                    display_hint: None,
                });
            }

            views.push(ViewInfo {
                name,
                schema,
                columns,
                definition: Some(definition),
            });
        }

        Ok(views)
    }

    async fn get_functions_info(&self) -> DbResult<Vec<FunctionInfo>> {
        let pool = self.get_pool()?;

        let sql = r#"
            SELECT
                p.proname as name,
                n.nspname as schema,
                pg_get_function_arguments(p.oid) as arguments,
                pg_get_function_result(p.oid) as return_type,
                pg_get_functiondef(p.oid) as definition,
                obj_description(p.oid, 'pg_proc') as description,
                p.prokind as function_type
            FROM pg_proc p
            JOIN pg_namespace n ON p.pronamespace = n.oid
            WHERE n.nspname NOT IN ('pg_catalog', 'information_schema')
            ORDER BY n.nspname, p.proname
        "#;

        let rows = sqlx::query(sql).fetch_all(pool).await?;

        let mut functions = Vec::new();
        for row in rows {
            let name: String = row.get("name");
            let schema: String = row.get("schema");
            let arguments: String = row.get("arguments");
            let return_type: String = row.get("return_type");
            let definition: String = row.get("definition");
            let description: Option<String> = row.get("description");

            functions.push(FunctionInfo {
                name,
                schema,
                arguments: arguments.split(',').map(|s| s.trim().to_string()).collect(),
                return_type: Some(return_type),
                definition: Some(definition),
            });
        }

        Ok(functions)
    }

    fn map_pg_type_to_category(type_name: &str) -> ColumnTypeCategory {
        match type_name {
            // Numeric types
            "smallint" | "integer" | "bigint" | "decimal" | "numeric" | "real"
            | "double precision" => ColumnTypeCategory::Numeric,
            // Text types
            "character varying" | "varchar" | "character" | "char" | "text" => {
                ColumnTypeCategory::Text
            }
            // Boolean type
            "boolean" => ColumnTypeCategory::Boolean,
            // Date/time types
            "date" => ColumnTypeCategory::Date,
            "time" | "timetz" => ColumnTypeCategory::Time,
            "timestamp" | "timestamptz" | "interval" => ColumnTypeCategory::DateTime,
            // Binary types
            "bytea" => ColumnTypeCategory::Binary,
            // JSON types
            "json" | "jsonb" => ColumnTypeCategory::Json,
            // Array types
            t if t.ends_with("[]") => ColumnTypeCategory::Array,
            // UUID type
            "uuid" => ColumnTypeCategory::UUID,
            // Network types
            "inet" | "cidr" | "macaddr" => ColumnTypeCategory::Network,
            // Geometry types
            t if t.starts_with("geometry") || t.starts_with("geography") => {
                ColumnTypeCategory::Geometry
            }
            // Enum types
            _ => ColumnTypeCategory::Other,
        }
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

    async fn get_entities(&self) -> DbResult<Vec<EntityInfo>> {
        let pool = self.get_pool()?;
        let mut entities = Vec::new();

        // Get information about schemas
        let schemas_sql = r#"
            SELECT
                nspname as name,
                obj_description(oid, 'pg_namespace') as description
            FROM pg_namespace
            WHERE nspname NOT IN ('pg_catalog', 'information_schema')
            ORDER BY nspname
        "#;

        let schemas = sqlx::query(schemas_sql).fetch_all(pool).await?;

        for schema in schemas {
            let name: String = schema.get("name");
            let description: Option<String> = schema.get("description");

            entities.push(EntityInfo {
                name: name.clone(),
                schema: "".to_string(), // No parent schema for schemas
                entity_type: EntityType::Schema,
                comment: description,
                last_modified: None,
                size_bytes: None,
                is_system: false,
                table_info: None,
                view_info: None,
                function_info: None,
            });
        }

        // Get information about tables
        let tables = self.get_tables_info().await?;

        // Get additional metadata for tables
        let table_metadata_sql = r#"
            SELECT
                n.nspname as schema_name,
                c.relname as table_name,
                c.reltuples as row_estimate,
                pg_total_relation_size(c.oid) as total_bytes,
                obj_description(c.oid, 'pg_class') as description,
                EXTRACT(EPOCH FROM age(now(), greatest(pg_stat_get_last_analyze_time(c.oid), pg_stat_get_last_autoanalyze_time(c.oid)))) as last_analyzed
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE c.relkind = 'r'
            AND n.nspname NOT IN ('pg_catalog', 'information_schema')
        "#;

        let metadata_rows = sqlx::query(table_metadata_sql).fetch_all(pool).await?;
        let mut metadata_map = HashMap::new();

        for row in metadata_rows {
            let schema: String = row.get("schema_name");
            let name: String = row.get("table_name");
            let key = format!("{}.{}", schema, name);

            let row_estimate: f64 = row.get("row_estimate");
            let total_bytes: i64 = row.get("total_bytes");
            let description: Option<String> = row.get("description");
            let last_analyzed: Option<f64> = row.get("last_analyzed");

            metadata_map.insert(key, (row_estimate, total_bytes, description, last_analyzed));
        }

        for table in tables {
            let key = format!("{}.{}", table.schema, table.name);
            let (row_estimate, size_bytes, description, last_analyzed) = metadata_map
                .get(&key)
                .map_or((0.0, 0, None, None), |(r, s, d, l)| (*r, *s, d.clone(), *l));

            entities.push(EntityInfo {
                name: table.name.clone(),
                schema: table.schema.clone(),
                entity_type: EntityType::Table,
                comment: description.clone(),
                last_modified: last_analyzed.map(|t| t as u64),
                size_bytes: Some(size_bytes as u64),
                is_system: false,
                table_info: Some(TableInfo {
                    name: table.name,
                    schema: table.schema,
                    columns: table.columns,
                    constraints: table.constraints,
                    indices: table.indices,
                    primary_key_columns: table.primary_key_columns,
                    row_count_estimate: Some(row_estimate as u64),
                    size_bytes: Some(size_bytes as u64),
                    comment: description.clone(),
                    last_modified: last_analyzed.map(|t| t as u64),
                }),
                view_info: None,
                function_info: None,
            });
        }

        // Get information about views
        let views = self.get_views_info().await?;

        for view in views {
            let entity_type = if view
                .definition
                .as_ref()
                .unwrap()
                .contains("MATERIALIZED VIEW")
            {
                EntityType::MaterializedView
            } else {
                EntityType::View
            };

            entities.push(EntityInfo {
                name: view.name.clone(),
                schema: view.schema.clone(),
                entity_type,
                comment: None,
                last_modified: None,
                size_bytes: None,
                is_system: false,
                table_info: None,
                view_info: Some(ViewInfo {
                    name: view.name,
                    schema: view.schema,
                    columns: view.columns,
                    definition: view.definition,
                }),
                function_info: None,
            });
        }

        // Get information about functions
        let functions = self.get_functions_info().await?;

        for function in functions {
            // Determine if this is a function or procedure based on the definition
            let is_procedure = function
                .definition
                .as_ref()
                .map_or(false, |def| def.contains("PROCEDURE"));

            let entity_type = if is_procedure {
                EntityType::Procedure
            } else {
                EntityType::Function
            };

            entities.push(EntityInfo {
                name: function.name.clone(),
                schema: function.schema.clone(),
                entity_type,
                comment: None,
                last_modified: None,
                size_bytes: None,
                is_system: false,
                table_info: None,
                view_info: None,
                function_info: Some(FunctionInfo {
                    name: function.name,
                    schema: function.schema,
                    arguments: function.arguments,
                    return_type: function.return_type,
                    definition: function.definition,
                }),
            });
        }

        Ok(entities)
    }
}
