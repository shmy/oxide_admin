use std::collections::HashMap;
use std::fmt::Display;

use crate::iam::value_object::permission_group::PermissionGroup;
use crate::iam::value_object::user_id::UserId;

pub type PermissionMap = HashMap<UserId, PermissionGroup>;

pub trait PermissionResolver: Clone {
    type Error: Display;
    fn resolve(&self, id: &UserId) -> impl Future<Output = PermissionGroup>;
    fn refresh(&self) -> impl Future<Output = Result<(), Self::Error>> + Send;
}
