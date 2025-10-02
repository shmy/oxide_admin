use std::collections::HashMap;
use std::fmt::Display;

use crate::organization::value_object::user_id::UserId;
use crate::system::value_object::permission_group::PermissionGroup;

pub type PermissionMap = HashMap<UserId, PermissionGroup>;

pub trait PermissionResolver: Clone {
    type Error: Display;
    fn resolve(&self, id: &UserId) -> impl Future<Output = PermissionGroup>;
    fn refresh(&self) -> impl Future<Output = Result<(), Self::Error>> + Send;
}
