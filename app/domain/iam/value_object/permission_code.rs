use std::ops::Deref;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(transparent)]
pub struct PermissionCode(i32);

impl PermissionCode {
    pub const fn new(code: i32) -> Self {
        Self(code)
    }
}

impl Deref for PermissionCode {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

macro_rules! define_permissions {
    ( $( $name:ident = $value:expr ),* $(,)? ) => {
        $(
            pub const $name: PermissionCode = PermissionCode::new($value);
        )*

        pub const ALL_PERMISSIONS: &[PermissionCode] = &[
            $( $name ),*
        ];
    };
}

define_permissions! {
    NONE = 0,
    SYSTEM = 100,
    SYSTEM_USER = 101,
    SYSTEM_ROLE = 102,
    SYSTEM_INFO = 103,
    SYSTEM_EXAMPLE = 104,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let permission_code = PermissionCode::new(100);
        assert_eq!(permission_code.0, 100);
        assert_eq!(*permission_code, 100);
    }
}
