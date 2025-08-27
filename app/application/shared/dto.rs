use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct OptionDto<T = String> {
    pub label: String,
    pub value: T,
}
