use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::chrono};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct SchedDto {
    pub key: String,
    pub name: String,
    pub expr: String,
    pub last_succeed: Option<bool>,
    pub last_result: Option<String>,
    pub last_run_at: Option<chrono::NaiveDateTime>,
    pub next_run_at: Option<chrono::NaiveDateTime>,
    pub last_duration_ms: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct SchedRecordDto {
    pub id: String,
    pub key: String,
    pub name: String,
    pub expr: String,
    pub succeed: Option<bool>,
    pub result: Option<String>,
    pub run_at: Option<chrono::NaiveDateTime>,
    pub duration_ms: Option<i64>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}
