use bon::Builder;

use crate::iam::{
    error::IamError,
    value_object::{permission_code::PermissionCode, role_id::RoleId},
};

#[derive(Debug, Clone, Builder)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::iam::value_object::permission_code::PermissionCode;

    #[test]
    fn test_update_name() {
        let mut role = Role::builder()
            .id(RoleId::generate())
            .name("test".to_string())
            .privileged(true)
            .permission_ids(vec![])
            .enabled(true)
            .build();
        role.update_name("test2".to_string());
        assert_eq!(role.name, "test2");
    }

    #[test]
    fn test_update_permission_ids() {
        let mut role = Role::builder()
            .id(RoleId::generate())
            .name("test".to_string())
            .privileged(true)
            .permission_ids(vec![])
            .enabled(true)
            .build();
        role.update_permission_ids(vec![PermissionCode::new(123)]);
        assert_eq!(role.permission_ids.len(), 1);
    }

    #[test]
    fn test_update_enabled() {
        let mut role = Role::builder()
            .id(RoleId::generate())
            .name("test".to_string())
            .privileged(true)
            .permission_ids(vec![])
            .enabled(true)
            .build();
        role.update_enabled(false);
        assert_eq!(role.enabled, false);
    }

    #[test]
    fn should_assert_activated_return_err() {
        let role = Role::builder()
            .id(RoleId::generate())
            .name("test".to_string())
            .privileged(true)
            .permission_ids(vec![])
            .enabled(false)
            .build();
        assert!(role.assert_activated().is_err());
    }

    #[test]
    fn should_assert_activated_return_ok() {
        let role = Role::builder()
            .id(RoleId::generate())
            .name("test".to_string())
            .privileged(true)
            .permission_ids(vec![])
            .enabled(true)
            .build();
        assert!(role.assert_activated().is_ok());
    }
}
