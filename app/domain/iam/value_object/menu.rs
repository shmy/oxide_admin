use std::ops::Deref;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(transparent)]
pub struct Menu(i32);

impl Menu {
    pub const fn new(code: i32) -> Self {
        Self(code)
    }
}

impl Deref for Menu {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

macro_rules! define_menus {
    ( $( $name:ident = $value:expr ),* $(,)? ) => {
        $(
            pub const $name: Menu = Menu::new($value);
        )*

        pub const ALL_MENUS: &[Menu] = &[
            $( $name ),*
        ];
    };

}

define_menus! {
    NONE = 0,
    SYSTEM = 1,
    SYSTEM_USER = 2,
    SYSTEM_ROLE = 3,
    SYSTEM_STAT = 4,
    SYSTEM_EXAMPLE = 5,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let key = Menu::new(100);
        assert_eq!(key.0, 100);
        assert_eq!(*key, 100);
    }
}
