use domain::iam::value_object::role_id::RoleId;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::chrono::NaiveDateTime};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, FromRow)]
pub struct UserDto {
    pub id: String,
    pub account: String,
    pub portrait: Option<String>,
    pub name: String,
    pub role_ids: Vec<RoleId>,
    pub role_names: Vec<String>,
    pub privileged: bool,
    pub enabled: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
