use domain::iam::value_object::permission_code::PermissionCode;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::chrono::NaiveDateTime};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct RoleDto {
    pub id: String,
    pub name: String,
    pub permission_ids: Vec<PermissionCode>,
    pub privileged: bool,
    pub enabled: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
