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
    let ty = match r#type.to_lowercase().as_str() {
        // 文本
        "text" | "character" | "character varying" => "String".to_string(),

        // 布尔
        "boolean" => "bool".to_string(),

        // 整数
        "smallint" | "int2" => "i16".to_string(),
        "integer" | "int4" => "i32".to_string(),
        "bigint" | "int8" => "i64".to_string(),

        // 浮点
        "real" | "float4" => "f32".to_string(),
        "double precision" | "float8" => "f64".to_string(),

        // 数值/小数
        "numeric" | "decimal" => "rust_decimal::Decimal".to_string(),

        // 时间日期
        "date" => "chrono::NaiveDate".to_string(),
        "time without time zone" => "chrono::NaiveTime".to_string(),
        "timestamp without time zone" => "chrono::NaiveDateTime".to_string(),
        "timestamp with time zone" => "chrono::DateTime<chrono::FixedOffset>".to_string(),

        // JSON
        "array" | "json" | "jsonb" => "serde_json::Value".to_string(),

        // UUID
        "uuid" => "uuid::Uuid".to_string(),

        // 字节流
        "bytea" => "Vec<u8>".to_string(),

        // 数组类型，先粗暴映射成 JSON
        t if t.ends_with("[]") => "serde_json::Value".to_string(),

        // 其他（保留原样，可能需要手动处理）
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
