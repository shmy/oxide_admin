use std::ops::Deref;

use serde::{Deserialize, Serialize};

use crate::shared::id_generator::IdGenerator;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct RoleId(String);

impl RoleId {
    pub fn generate() -> Self {
        Self(IdGenerator::primary_id())
    }

    pub fn new_unchecked(id: String) -> Self {
        Self(id)
    }
}

impl Deref for RoleId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
