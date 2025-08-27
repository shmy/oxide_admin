use bon::Builder;
use serde::Serialize;
use sqlx::prelude::FromRow;

use crate::iam::{
    error::IamError,
    value_object::{permission_code::PermissionCode, role_id::RoleId},
};

#[derive(Debug, Clone, Builder, Serialize, FromRow)]
#[readonly::make]
pub struct Role {
    pub id: RoleId,
    pub name: String,
    pub privileged: bool,
    pub permission_ids: Vec<PermissionCode>,
    pub enabled: bool,
}

impl Role {
    pub fn update_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn update_permission_ids(&mut self, permission_ids: Vec<PermissionCode>) {
        self.permission_ids = permission_ids;
    }

    pub fn update_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn assert_activated(&self) -> Result<(), IamError> {
        if !self.enabled {
            return Err(IamError::RoleDisabled);
        }
        Ok(())
    }
}
