use std::collections::HashSet;
use std::ops::Deref;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct PermissionCode(i32);

impl PermissionCode {
    pub const fn new(s: i32) -> Self {
        Self(s)
    }

    pub fn empty() -> HashSet<Self> {
        HashSet::with_capacity(0)
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
