use serde::Serialize;
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema, FromRow)]
pub struct OptionStringDto {
    pub label: String,
    pub value: String,
}
