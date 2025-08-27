use std::ops::Deref;

use serde::{Deserialize, Serialize};

use crate::shared::id_generator::IdGenerator;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct UserId(String);

impl UserId {
    pub fn generate() -> Self {
        Self(IdGenerator::primary_id())
    }

    pub fn new_unchecked(id: String) -> Self {
        Self(id)
    }
}

impl Deref for UserId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
