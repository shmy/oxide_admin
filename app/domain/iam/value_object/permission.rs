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

macro_rules! define_permissions {
    ( $( $name:ident = $value:expr ),* $(,)? ) => {
        $(
            pub const $name: Permission = Permission::new($value);
        )*

        pub const ALL_PERMISSIONS: &[Permission] = &[
            $( $name ),*
        ];
    };
}

define_permissions! {
    SYSTEM_USER_READ = 100,
    SYSTEM_USER_CREATE = 101,
    SYSTEM_USER_UPDATE = 102,
    SYSTEM_USER_DELETE = 103,
    SYSTEM_USER_ENABLE = 104,
    SYSTEM_USER_DISABLE = 105,
    SYSTEM_USER_UPDATE_PASSWORD = 106,
    SYSTEM_ROLE_READ = 200,
    SYSTEM_ROLE_CREATE = 201,
    SYSTEM_ROLE_UPDATE = 202,
    SYSTEM_ROLE_DELETE = 203,
    SYSTEM_ROLE_ENABLE = 204,
    SYSTEM_ROLE_DISABLE = 205,
    SYSTEM_UPLOAD_FILE = 300,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let permission = Permission::new(100);
        assert_eq!(permission.0, 100);
        assert_eq!(*permission, 100);
    }
}
