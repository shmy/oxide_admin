use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct DepartmentDto {
    pub id: String,
    pub name: String,
    pub code: String,
    pub parent_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DepartmentWithChildren {
    pub id: String,
    pub label: String,
    pub value: String,
    #[schema(no_recursion)]
    pub children: Vec<DepartmentWithChildren>,
}
