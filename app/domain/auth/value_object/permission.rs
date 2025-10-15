use std::ops::Deref;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(transparent)]
pub struct Permission(i32);

impl Permission {
    pub const fn new(code: i32) -> Self {
        Self(code)
    }
}

impl Deref for Permission {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PermissionTree {
    pub label: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Permission>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(no_recursion)]
    pub children: Option<&'static [PermissionTree]>,
}

include!(concat!(env!("OUT_DIR"), "/permissions.rs"));
