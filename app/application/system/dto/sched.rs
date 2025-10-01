use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::chrono};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct SchedDto {
    pub id: String,
    pub key: String,
    pub name: String,
    pub schedule: String,
    pub succeed: bool,
    pub result: String,
    pub run_at: chrono::NaiveDateTime,
    pub duration_ms: i64,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}
