use crate::database::{Field, TableInfoTrait};
use anyhow::{Result, bail};
use sqlx::pool;
pub struct Sqlite {
    pool: sqlx::pool::Pool<sqlx::Sqlite>,
}

impl Sqlite {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = pool::Pool::connect(database_url).await?;
        Ok(Self { pool })
    }
}

impl TableInfoTrait for Sqlite {
    async fn table_info(&self, table: &str) -> Result<Vec<Field>> {
        let infos: Vec<SqliteTableInfo> = sqlx::query_as(&format!("PRAGMA table_info({})", table))
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
            "TEXT" => "String",
            "BOOLEAN" => "bool",
            "JSONB" => "sqlx::types::Json",
            "DATE" => "chrono::NaiveDate",
            "INTEGER" => "i64",
            "DATETIME" => "chrono::NaiveDateTime",
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
struct SqliteTableInfo {
    // cid: u32,
    name: String,
    r#type: String,
    notnull: bool,
    // pk: bool,
}
