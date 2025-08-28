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
        let infos: Vec<PostgresTableInfo> = sqlx::query_as!(
            PostgresTableInfo,
            r#"
            SELECT column_name AS "name!",
                    data_type AS "type!",
                    is_nullable = 'NO' AS "notnull!"
            FROM information_schema.columns
            WHERE table_name = $1
            ORDER BY ordinal_position
            "#,
            table,
        )
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
    let ty = if r#type.starts_with("VARCHAR") {
        "String".to_string()
    } else {
        match r#type {
            "text" => "String",
            "character" => "String",
            "character varying" => "String",
            "boolean" => "bool",
            "JSONB" | "ARRAY" => "serde_json::Value",
            "timestamp without time zone" => "chrono::NaiveDateTime",
            _ => r#type,
        }
        .to_string()
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
