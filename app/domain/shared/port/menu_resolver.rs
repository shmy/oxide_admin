use std::collections::HashMap;
use std::fmt::Display;

use crate::system::value_object::menu_group::MenuGroup;
use crate::system::value_object::user_id::UserId;

pub type MenunMap = HashMap<UserId, MenuGroup>;

pub trait MenuResolver: Clone {
    type Error: Display;
    fn resolve(&self, id: &UserId) -> impl Future<Output = MenuGroup>;
    fn refresh(&self) -> impl Future<Output = Result<(), Self::Error>> + Send;
}
