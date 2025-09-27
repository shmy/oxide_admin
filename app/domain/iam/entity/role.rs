use bon::Builder;

use crate::iam::{
    error::IamError,
    value_object::{menu::Menu, permission::Permission, role_id::RoleId},
};

#[derive(Debug, Clone, Builder)]
#[readonly::make]
pub struct Role {
    pub id: RoleId,
    pub name: String,
    pub privileged: bool,
    pub menus: Vec<Menu>,
    pub permissions: Vec<Permission>,
    pub enabled: bool,
}

impl Role {
    pub fn update_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn update_menus(&mut self, menus: Vec<Menu>) {
        self.menus = menus;
    }

    pub fn update_permissions(&mut self, permissions: Vec<Permission>) {
        self.permissions = permissions;
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
    use crate::iam::value_object::permission::Permission;

    #[test]
    fn test_update_name() {
        let mut role = Role::builder()
            .id(RoleId::generate())
            .name("test".to_string())
            .privileged(true)
            .menus(vec![])
            .permissions(vec![])
            .enabled(true)
            .build();
        role.update_name("test2".to_string());
        assert_eq!(role.name, "test2");
    }

    #[test]
    fn test_update_permissions() {
        let mut role = Role::builder()
            .id(RoleId::generate())
            .name("test".to_string())
            .privileged(true)
            .menus(vec![])
            .permissions(vec![])
            .enabled(true)
            .build();
        role.update_permissions(vec![Permission::new(123)]);
        assert_eq!(role.permissions.len(), 1);
    }

    #[test]
    fn test_update_enabled() {
        let mut role = Role::builder()
            .id(RoleId::generate())
            .name("test".to_string())
            .privileged(true)
            .menus(vec![])
            .permissions(vec![])
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
            .menus(vec![])
            .permissions(vec![])
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
            .menus(vec![])
            .permissions(vec![])
            .enabled(true)
            .build();
        assert!(role.assert_activated().is_ok());
    }
}
