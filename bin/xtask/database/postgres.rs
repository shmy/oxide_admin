use crate::database::{Field, TableInfoTrait};
use anyhow::{Result, bail};
use sqlx::pool;
pub struct Postgres {
    pool: sqlx::PgPool,
}

impl Postgres {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = pool::Pool::connect(database_url).await?;
        Ok(Self { pool })
    }
}

impl TableInfoTrait for Postgres {
    async fn table_info(&self, table: &str) -> Result<Vec<Field>> {
        let infos: Vec<PostgresTableInfo> = sqlx::query_as(
            r#"
            SELECT column_name AS name,
                    data_type AS type,
                    is_nullable = 'NO' AS notnull
            FROM information_schema.columns
            WHERE table_name = $1
            ORDER BY ordinal_position
            "#,
        )
        .bind(table)
        .fetch_all(&self.pool)
        .await?;
        if infos.is_empty() {
            bail!("table not found: {}", table);
        }
        Ok(infos
            .into_iter()
            .map(|info| Field {
                name: info.name,
                r#type: type_mapping(&info.r#type, info.notnull),
            })
            .collect())
    }
}

fn type_mapping(r#type: &str, notnull: bool) -> String {
    let ty = match r#type.to_lowercase().as_str() {
        "text" | "character" | "character varying" => "String".to_string(),

        "boolean" => "bool".to_string(),

        "smallint" | "int2" => "i16".to_string(),
        "integer" | "int4" => "i32".to_string(),
        "bigint" | "int8" => "i64".to_string(),

        "real" | "float4" => "f32".to_string(),
        "double precision" | "float8" => "f64".to_string(),

        "numeric" | "decimal" => "rust_decimal::Decimal".to_string(),

        "date" => "chrono::NaiveDate".to_string(),
        "time without time zone" => "chrono::NaiveTime".to_string(),
        "timestamp without time zone" => "chrono::NaiveDateTime".to_string(),
        "timestamp with time zone" => "chrono::DateTime<chrono::FixedOffset>".to_string(),

        "array" | "json" | "jsonb" => "serde_json::Value".to_string(),

        "uuid" => "uuid::Uuid".to_string(),

        "bytea" => "Vec<u8>".to_string(),

        t if t.ends_with("[]") => "serde_json::Value".to_string(),

        other => other.to_string(),
    };

    if notnull {
        ty
    } else {
        format!("Option<{}>", ty)
    }
}

#[derive(Debug, sqlx::FromRow)]
struct PostgresTableInfo {
    // cid: u32,
    name: String,
    r#type: String,
    notnull: bool,
    // pk: bool,
}
