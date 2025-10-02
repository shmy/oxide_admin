use std::collections::HashMap;
use std::fmt::Display;

use crate::organization::value_object::user_id::UserId;
use crate::system::value_object::menu_group::MenuGroup;

pub type MenunMap = HashMap<UserId, MenuGroup>;

pub trait MenuResolver: Clone {
    type Error: Display;
    fn resolve(&self, id: &UserId) -> impl Future<Output = MenuGroup>;
    fn refresh(&self) -> impl Future<Output = Result<(), Self::Error>> + Send;
}
