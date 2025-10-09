use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::chrono};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct AccessLogDto {
    pub id: String,
    pub user_id: String,
    pub method: String,
    pub uri: String,
    pub user_agent: Option<String>,
    pub ip: Option<String>,
    pub status: i16,
    pub elapsed: i64,
    pub occurred_at: chrono::NaiveDateTime,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}
