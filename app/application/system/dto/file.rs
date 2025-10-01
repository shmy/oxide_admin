use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::chrono};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct FileDto {
    pub id: String,
    pub name: String,
    pub path: String,
    pub size: i64,
    pub used: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}
